use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const LOG_MAX_BYTES: u64 = 1_048_576;
pub const LIGHTING_ERROR_TOKEN: &str = "lighting_error";
const ROTATED_SUFFIX: &str = ".1";

static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init(log_file: &Path) {
    if let Some(parent) = log_file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut guard) = LOG_PATH.lock() {
        *guard = Some(log_file.to_path_buf());
    }
}

pub fn write(message: &str) {
    eprintln!("{message}");
    let path = match LOG_PATH.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => None,
    };
    if let Some(path) = path {
        append_rotated(&path, message);
    }
}

pub fn log_lighting_error() {
    write(LIGHTING_ERROR_TOKEN);
}

fn append_rotated(path: &Path, message: &str) {
    let line = format!("{message}\n");
    let line_len = line.len() as u64;
    let current_len = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if current_len > 0 && current_len.saturating_add(line_len) > LOG_MAX_BYTES {
        let rotated = rotated_path(path);
        let _ = fs::rename(path, &rotated);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

fn rotated_path(path: &Path) -> PathBuf {
    let mut rotated = path.as_os_str().to_os_string();
    rotated.push(ROTATED_SUFFIX);
    PathBuf::from(rotated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rotates_keep_log_1_when_over_one_mib() {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("opentartarus-log-{n}"));
        fs::create_dir_all(&dir).unwrap();
        let log_file = dir.join("opentartarus.log");
        fs::write(&log_file, vec![b'x'; LOG_MAX_BYTES as usize]).unwrap();
        init(&log_file);
        write("rotated");
        assert!(log_file.exists());
        let rotated = dir.join("opentartarus.log.1");
        assert!(rotated.exists());
        assert_eq!(fs::read_to_string(&log_file).unwrap(), "rotated\n");
        assert_eq!(fs::metadata(&rotated).unwrap().len(), LOG_MAX_BYTES);
        assert_eq!(LIGHTING_ERROR_TOKEN, "lighting_error");
        if let Ok(mut guard) = LOG_PATH.lock() {
            *guard = None;
        }
    }
}
