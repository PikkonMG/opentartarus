use crate::app::Message;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::Subscription;
use opentartarus_core::codec::{decode_frame, encode_frame};
use opentartarus_core::error::ErrorCode;
use opentartarus_core::ipc::{
    EventMethod, EventMsg, Method, ReqTag, RequestMsg, ResponseMsg,
};
use opentartarus_core::paths::Paths;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
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
    spawn_daemon();
    let _ = output.send(Message::DaemonStarting).await;
    let started = Instant::now();
    loop {
        if let Ok(stream) = UnixStream::connect(socket).await {
            return Some(stream);
        }
        if started.elapsed() >= CONNECT_TIMEOUT {
            let _ = output.send(Message::DaemonFailed).await;
            return None;
        }
        sleep(CONNECT_RETRY).await;
    }
}

fn spawn_daemon() {
    let mut cmd = daemon_command();
    let _ = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn daemon_command() -> std::process::Command {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join(DAEMON_BIN);
            if sibling.exists() {
                return std::process::Command::new(sibling);
            }
        }
    }
    std::process::Command::new(DAEMON_BIN)
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
    let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if buf.len() < 4 + len {
        return Ok(None);
    }
    let (payload, consumed) = decode_frame(buf)?;
    buf.drain(..consumed);
    serde_json::from_slice(&payload)
        .map(Some)
        .map_err(|_| ErrorCode::InvalidProfile)
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

