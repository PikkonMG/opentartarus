use crate::app::Message;
use crate::theme;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::Subscription;
use opentartarus_core::binpath::resolve_bin_from_env;
use opentartarus_core::codec::{decode_frame, encode_frame};
use opentartarus_core::constants::IPC_MAX_MESSAGE_BYTES;
use opentartarus_core::error::ErrorCode;
use opentartarus_core::ipc::{EventMethod, EventMsg, Method, ReqTag, RequestMsg, ResponseMsg};
use opentartarus_core::paths::Paths;
use serde_json::Value;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::sleep;

pub const DAEMON_BIN: &str = "opentartarus-daemon";
pub const PKEXEC_BIN: &str = "pkexec";
pub const FIX_PERMISSIONS_HELPER: &str = "/usr/libexec/opentartarus-fix-permissions";
pub const CONNECT_RETRY: Duration = Duration::from_millis(100);
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_CHUNK: usize = 4096;

#[derive(Debug, Clone)]
pub struct Outgoing {
    pub method: Method,
    pub params: Value,
}

pub fn subscription() -> Subscription<Message> {
    Subscription::run(ipc_stream)
}

fn ipc_stream() -> impl Stream<Item = Message> {
    iced::stream::channel(64, |output| async move {
        run_client(output).await;
    })
}

async fn run_client(mut output: mpsc::Sender<Message>) {
    let (tx, mut rx) = mpsc::unbounded();
    let _ = output.send(Message::IpcReady(tx)).await;
    let socket = Paths::from_env().socket;
    loop {
        match connect_with_retry(&socket, &mut output).await {
            Some(stream) => {
                let _ = output.send(Message::DaemonConnected).await;
                if pump_connection(stream, &mut rx, &mut output).await {
                    break;
                }
            }
            None => {
                std::future::pending::<()>().await;
            }
        }
    }
}

async fn connect_with_retry(
    socket: &Path,
    output: &mut mpsc::Sender<Message>,
) -> Option<UnixStream> {
    if let Ok(stream) = UnixStream::connect(socket).await {
        return Some(stream);
    }
    if let Err(message) = spawn_daemon() {
        let _ = output.send(Message::DaemonFailed(message)).await;
        return None;
    }
    let _ = output.send(Message::DaemonStarting).await;
    let started = Instant::now();
    loop {
        if let Ok(stream) = UnixStream::connect(socket).await {
            return Some(stream);
        }
        if started.elapsed() >= CONNECT_TIMEOUT {
            let _ = output
                .send(Message::DaemonFailed(connect_timeout_message(socket)))
                .await;
            return None;
        }
        sleep(CONNECT_RETRY).await;
    }
}

fn spawn_daemon() -> Result<(), String> {
    let bin = daemon_bin();
    std::process::Command::new(&bin)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|err| spawn_failure_message(&bin, &err))
}

pub fn daemon_bin() -> PathBuf {
    resolve_bin_from_env(DAEMON_BIN)
}

pub fn spawn_failure_message(bin: &Path, err: &std::io::Error) -> String {
    let reason = match err.kind() {
        ErrorKind::NotFound => format!("Couldn’t find the tray program ({}).", bin.display()),
        ErrorKind::PermissionDenied => {
            format!("No permission to start the tray ({}).", bin.display())
        }
        _ => format!("Couldn’t start the tray ({}): {err}.", bin.display()),
    };
    theme::could_not_start_message(&reason)
}

pub fn connect_timeout_message(socket: &Path) -> String {
    theme::could_not_start_message(&format!(
        "Couldn’t connect to the tray ({}).",
        socket.display()
    ))
}

async fn pump_connection(
    stream: UnixStream,
    rx: &mut mpsc::UnboundedReceiver<Outgoing>,
    output: &mut mpsc::Sender<Message>,
) -> bool {
    let (mut reader, mut writer) = stream.into_split();
    let mut buf = Vec::new();
    let mut next_id: u64 = 1;
    let mut pending: HashMap<String, Method> = HashMap::new();
    loop {
        tokio::select! {
            outgoing = rx.next() => {
                match outgoing {
                    Some(req) => {
                        let id = format!("u{next_id}");
                        next_id += 1;
                        pending.insert(id.clone(), req.method);
                        let msg = RequestMsg {
                            r#type: ReqTag,
                            id,
                            method: req.method,
                            params: req.params,
                        };
                        if write_json(&mut writer, &msg).await.is_err() {
                            return false;
                        }
                    }
                    None => return true,
                }
            }
            frame = read_json(&mut reader, &mut buf) => {
                match frame {
                    Ok(value) => {
                        if dispatch_incoming(value, &mut pending, output).await {
                            return true;
                        }
                    }
                    Err(_) => return false,
                }
            }
        }
    }
}

async fn dispatch_incoming(
    value: Value,
    pending: &mut HashMap<String, Method>,
    output: &mut mpsc::Sender<Message>,
) -> bool {
    if let Ok(res) = serde_json::from_value::<ResponseMsg>(value.clone()) {
        let method = pending.remove(&res.id);
        let _ = output
            .send(Message::IpcResponse {
                method,
                ok: res.ok,
                result: res.result,
                error: res.error,
            })
            .await;
        return false;
    }
    if let Ok(event) = serde_json::from_value::<EventMsg>(value) {
        if event.method == EventMethod::DaemonStopping {
            let _ = output.send(Message::DaemonStopping).await;
            return true;
        }
        let _ = output
            .send(Message::IpcEvent {
                method: event.method,
                params: event.params,
            })
            .await;
    }
    false
}

async fn write_json<T: serde::Serialize>(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    value: &T,
) -> Result<(), ErrorCode> {
    let payload = serde_json::to_vec(value).map_err(|_| ErrorCode::InvalidProfile)?;
    let frame = encode_frame(&payload)?;
    writer.write_all(&frame).await.map_err(|_| ErrorCode::Io)
}

async fn read_json(
    reader: &mut tokio::net::unix::OwnedReadHalf,
    buf: &mut Vec<u8>,
) -> Result<Value, ErrorCode> {
    loop {
        if let Some(value) = try_decode(buf)? {
            return Ok(value);
        }
        let mut chunk = [0u8; READ_CHUNK];
        let n = reader.read(&mut chunk).await.map_err(|_| ErrorCode::Io)?;
        if n == 0 {
            return Err(ErrorCode::Io);
        }
        buf.extend_from_slice(&chunk[..n]);
    }
}

fn try_decode(buf: &mut Vec<u8>) -> Result<Option<Value>, ErrorCode> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if len == 0 || len > IPC_MAX_MESSAGE_BYTES {
        return Err(ErrorCode::InvalidProfile);
    }
    if buf.len() < 4 + len as usize {
        return Ok(None);
    }
    let (payload, consumed) = decode_frame(buf)?;
    buf.drain(..consumed);
    serde_json::from_slice(&payload)
        .map(Some)
        .map_err(|_| ErrorCode::InvalidProfile)
}

#[cfg(test)]
mod tests {
    use super::{connect_timeout_message, spawn_failure_message, try_decode};
    use opentartarus_core::constants::IPC_MAX_MESSAGE_BYTES;
    use opentartarus_core::error::ErrorCode;
    use std::io::ErrorKind;
    use std::path::Path;

    #[test]
    fn try_decode_rejects_oversize_length_before_buffering() {
        let mut buf = (IPC_MAX_MESSAGE_BYTES + 1).to_le_bytes().to_vec();
        assert_eq!(try_decode(&mut buf).unwrap_err(), ErrorCode::InvalidProfile);
    }

    #[test]
    fn try_decode_rejects_zero_length_before_buffering() {
        let mut buf = 0u32.to_le_bytes().to_vec();
        assert_eq!(try_decode(&mut buf).unwrap_err(), ErrorCode::InvalidProfile);
    }

    #[test]
    fn spawn_not_found_names_the_bin() {
        let err = std::io::Error::new(ErrorKind::NotFound, "not found");
        let message =
            spawn_failure_message(Path::new("/repo/target/debug/opentartarus-daemon"), &err);
        assert!(message.starts_with("OpenTartarus couldn’t start."));
        assert!(message.contains("/repo/target/debug/opentartarus-daemon"));
        assert!(message.contains("Couldn’t find the tray program"));
    }

    #[test]
    fn connect_timeout_names_the_socket() {
        let message = connect_timeout_message(Path::new("/run/user/1000/opentartarus/daemon.sock"));
        assert!(message.starts_with("OpenTartarus couldn’t start."));
        assert!(message.contains("/run/user/1000/opentartarus/daemon.sock"));
        assert!(!message.contains("Try opening it again"));
    }
}

pub async fn run_fix_permissions() -> FixPermissionsOutcome {
    match tokio::process::Command::new(PKEXEC_BIN)
        .arg(FIX_PERMISSIONS_HELPER)
        .status()
        .await
    {
        Ok(status) if status.success() => FixPermissionsOutcome::Success,
        Ok(_) => FixPermissionsOutcome::Cancelled,
        Err(_) => FixPermissionsOutcome::Failed,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixPermissionsOutcome {
    Success,
    Cancelled,
    Failed,
}
