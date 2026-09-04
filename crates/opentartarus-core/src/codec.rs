use crate::constants::IPC_MAX_MESSAGE_BYTES;
use crate::error::ErrorCode;

pub fn encode_frame(payload: &[u8]) -> Result<Vec<u8>, ErrorCode> {
    let len = payload.len();
    if len == 0 || len > IPC_MAX_MESSAGE_BYTES as usize {
        return Err(ErrorCode::InvalidProfile);
    }
    let mut out = Vec::with_capacity(4 + len);
    out.extend_from_slice(&(len as u32).to_le_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

pub fn decode_frame(buf: &[u8]) -> Result<(Vec<u8>, usize), ErrorCode> {
    if buf.len() < 4 {
        return Err(ErrorCode::Io);
    }
    let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if len == 0 || len > IPC_MAX_MESSAGE_BYTES {
        return Err(ErrorCode::InvalidProfile);
    }
    let total = 4 + len as usize;
    if buf.len() < total {
        return Err(ErrorCode::Io);
    }
    Ok((buf[4..total].to_vec(), total))
}

#[cfg(test)]
mod codec_tests {
    use super::{decode_frame, encode_frame};
    use crate::constants::IPC_MAX_MESSAGE_BYTES;
    use crate::error::ErrorCode;

    #[test]
    fn roundtrip_json() {
        let body = br#"{"type":"req","id":"a","method":"GetStatus","params":{}}"#;
        let framed = encode_frame(body).unwrap();
        assert_eq!(&framed[..4], &(body.len() as u32).to_le_bytes());
        let (out, n) = decode_frame(&framed).unwrap();
        assert_eq!(out, body);
        assert_eq!(n, framed.len());
    }

    #[test]
    fn reject_zero_length() {
        assert_eq!(encode_frame(b"").unwrap_err(), ErrorCode::InvalidProfile);
        let buf = vec![0, 0, 0, 0];
        assert_eq!(decode_frame(&buf).unwrap_err(), ErrorCode::InvalidProfile);
    }

    #[test]
    fn reject_oversize() {
        let huge = vec![0u8; (IPC_MAX_MESSAGE_BYTES as usize) + 1];
        assert_eq!(encode_frame(&huge).unwrap_err(), ErrorCode::InvalidProfile);
        let oversize_len = IPC_MAX_MESSAGE_BYTES + 1;
        let mut buf = oversize_len.to_le_bytes().to_vec();
        buf.extend_from_slice(&huge);
        assert_eq!(decode_frame(&buf).unwrap_err(), ErrorCode::InvalidProfile);
    }
}
