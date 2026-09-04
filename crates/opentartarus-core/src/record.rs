use crate::constants::{RECORD_TIMEOUT_MAX_MS, RECORD_TIMEOUT_MS};
use crate::error::ErrorCode;
use crate::types::{Action, KeyId, KeyToken, Modifier, MouseTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordReason {
    Timeout,
    User,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordOutcome {
    Recorded { key_id: KeyId, action: Action },
    Cancelled { reason: RecordReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordSession {
    pub key_id: KeyId,
    pub deadline_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Recorder {
    session: Option<RecordSession>,
}

impl Recorder {
    pub fn start(
        &mut self,
        key_id: KeyId,
        timeout_ms: Option<u64>,
        now_ms: u64,
    ) -> Result<(KeyId, u64), ErrorCode> {
        if self.session.is_some() {
            return Err(ErrorCode::RecordBusy);
        }
        let timeout = timeout_ms
            .unwrap_or(RECORD_TIMEOUT_MS)
            .min(RECORD_TIMEOUT_MAX_MS);
        let deadline_ms = now_ms + timeout;
        self.session = Some(RecordSession {
            key_id,
            deadline_ms,
        });
        Ok((key_id, deadline_ms))
    }

    pub fn stop(&mut self) -> bool {
        self.session.take().is_some()
    }

    pub fn on_timeout(&mut self, now_ms: u64) -> Option<RecordOutcome> {
        let session = self.session?;
        if now_ms < session.deadline_ms {
            return None;
        }
        self.session = None;
        Some(RecordOutcome::Cancelled {
            reason: RecordReason::Timeout,
        })
    }

    pub fn submit_key(
        &mut self,
        key: KeyToken,
        modifiers: Vec<Modifier>,
    ) -> Result<RecordOutcome, ErrorCode> {
        let session = self.session.take().ok_or(ErrorCode::NotFound)?;
        Ok(RecordOutcome::Recorded {
            key_id: session.key_id,
            action: Action::Key { key, modifiers },
        })
    }

    pub fn submit_mouse(&mut self, target: MouseTarget) -> Result<RecordOutcome, ErrorCode> {
        let session = self.session.take().ok_or(ErrorCode::NotFound)?;
        Ok(RecordOutcome::Recorded {
            key_id: session.key_id,
            action: Action::Mouse { target },
        })
    }

    pub fn on_disconnect(&mut self) -> Option<RecordOutcome> {
        self.session.take()?;
        Some(RecordOutcome::Cancelled {
            reason: RecordReason::Disconnected,
        })
    }

    pub fn is_active(&self) -> bool {
        self.session.is_some()
    }

    pub fn session(&self) -> Option<RecordSession> {
        self.session
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{KeyId, KeyToken};

    #[test]
    fn submit_completes() {
        let mut r = Recorder::default();
        r.start(KeyId::Kp01, None, 0).unwrap();
        let out = r
            .submit_key(KeyToken::C, vec![crate::types::Modifier::Ctrl])
            .unwrap();
        match out {
            RecordOutcome::Recorded { key_id, action } => {
                assert_eq!(key_id, KeyId::Kp01);
                assert!(matches!(action, crate::types::Action::Key { .. }));
            }
            _ => panic!("expected recorded"),
        }
        assert!(!r.is_active());
    }

    #[test]
    fn timeout_writes_nothing() {
        let mut r = Recorder::default();
        r.start(KeyId::Kp01, Some(50), 0).unwrap();
        let out = r.on_timeout(50).unwrap();
        assert!(matches!(
            out,
            RecordOutcome::Cancelled {
                reason: RecordReason::Timeout
            }
        ));
        assert!(!r.is_active());
    }

    #[test]
    fn stop_and_second_start() {
        let mut r = Recorder::default();
        r.start(KeyId::Kp01, None, 0).unwrap();
        assert!(r.stop());
        assert!(r.on_timeout(0).is_none());
        r.start(KeyId::Kp02, None, 0).unwrap();
        assert_eq!(
            r.start(KeyId::Kp03, None, 1).unwrap_err(),
            crate::error::ErrorCode::RecordBusy
        );
    }

    #[test]
    fn submit_without_session_is_not_found() {
        let mut r = Recorder::default();
        assert_eq!(
            r.submit_key(KeyToken::A, vec![]).unwrap_err(),
            crate::error::ErrorCode::NotFound
        );
    }
}
