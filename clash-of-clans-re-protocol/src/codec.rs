use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::error::{ProtocolError, Result};
use crate::messages::MessageId;

pub trait Encodable {
    fn encode(&self, buf: &mut BytesMut);
}

pub trait Decodable: Sized {
    fn decode(buf: &mut Bytes) -> Result<Self>;
}

pub fn encode_varint(buf: &mut BytesMut, value: i32) {
    let mut v = value as u32;
    while v >= 0x80 {
        buf.put_u8((v & 0x7F) as u8 | 0x80);
        v >>= 7;
    }
    buf.put_u8(v as u8);
}

pub fn decode_varint(buf: &mut Bytes) -> Result<i32> {
    let mut result = 0i32;
    let mut shift = 0;
    
    loop {
        if buf.remaining() == 0 {
            return Err(ProtocolError::BufferTooSmall { expected: 1, actual: 0 });
        }
        let byte = buf.get_u8();
        result |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift >= 35 {
            return Err(ProtocolError::DeserializationError("VarInt too long".into()));
        }
    }
    Ok(result)
}

pub fn encode_vlong(buf: &mut BytesMut, value: i64) {
    let mut v = value as u64;
    while v >= 0x80 {
        buf.put_u8((v & 0x7F) as u8 | 0x80);
        v >>= 7;
    }
    buf.put_u8(v as u8);
}

pub fn decode_vlong(buf: &mut Bytes) -> Result<i64> {
    let mut result = 0i64;
    let mut shift = 0;
    
    loop {
        if buf.remaining() == 0 {
            return Err(ProtocolError::BufferTooSmall { expected: 1, actual: 0 });
        }
        let byte = buf.get_u8();
        result |= ((byte & 0x7F) as i64) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift >= 70 {
            return Err(ProtocolError::DeserializationError("VLong too long".into()));
        }
    }
    Ok(result)
}

pub fn encode_string(buf: &mut BytesMut, s: &str) {
    let bytes = s.as_bytes();
    encode_varint(buf, bytes.len() as i32);
    buf.extend_from_slice(bytes);
}

pub fn decode_string(buf: &mut Bytes) -> Result<String> {
    let len = decode_varint(buf)? as usize;
    if buf.remaining() < len {
        return Err(ProtocolError::BufferTooSmall { expected: len, actual: buf.remaining() });
    }
    let bytes = buf.copy_to_bytes(len);
    String::from_utf8(bytes.to_vec())
        .map_err(|e| ProtocolError::DeserializationError(e.to_string()))
}

pub fn encode_logic_long(buf: &mut BytesMut, high: i32, low: i32) {
    buf.put_i32(high);
    buf.put_i32(low);
}

pub fn decode_logic_long(buf: &mut Bytes) -> Result<(i32, i32)> {
    if buf.remaining() < 8 {
        return Err(ProtocolError::BufferTooSmall { expected: 8, actual: buf.remaining() });
    }
    let high = buf.get_i32();
    let low = buf.get_i32();
    Ok((high, low))
}

pub fn encode_data_reference(buf: &mut BytesMut, table_id: i32, instance_id: i32) {
    encode_varint(buf, table_id);
    encode_varint(buf, instance_id);
}

pub fn decode_data_reference(buf: &mut Bytes) -> Result<(i32, i32)> {
    let table_id = decode_varint(buf)?;
    let instance_id = decode_varint(buf)?;
    Ok((table_id, instance_id))
}

pub fn encode_byte_array(buf: &mut BytesMut, data: &[u8]) {
    encode_varint(buf, data.len() as i32);
    buf.extend_from_slice(data);
}

pub fn decode_byte_array(buf: &mut Bytes) -> Result<Vec<u8>> {
    let len = decode_varint(buf)? as usize;
    if buf.remaining() < len {
        return Err(ProtocolError::BufferTooSmall { expected: len, actual: buf.remaining() });
    }
    Ok(buf.copy_to_bytes(len).to_vec())
}

pub fn encode_boolean(buf: &mut BytesMut, value: bool) {
    buf.put_u8(if value { 1 } else { 0 });
}

pub fn decode_boolean(buf: &mut Bytes) -> Result<bool> {
    if buf.remaining() == 0 {
        return Err(ProtocolError::BufferTooSmall { expected: 1, actual: 0 });
    }
    Ok(buf.get_u8() != 0)
}

pub fn encode_int(buf: &mut BytesMut, value: i32) {
    buf.put_i32(value);
}

pub fn decode_int(buf: &mut Bytes) -> Result<i32> {
    if buf.remaining() < 4 {
        return Err(ProtocolError::BufferTooSmall { expected: 4, actual: buf.remaining() });
    }
    Ok(buf.get_i32())
}

pub fn encode_long(buf: &mut BytesMut, value: i64) {
    buf.put_i64(value);
}

pub fn decode_long(buf: &mut Bytes) -> Result<i64> {
    if buf.remaining() < 8 {
        return Err(ProtocolError::BufferTooSmall { expected: 8, actual: buf.remaining() });
    }
    Ok(buf.get_i64())
}

pub fn encode_message_id(buf: &mut BytesMut, id: MessageId) {
    buf.put_u16(id as u16);
}

pub fn decode_message_id(buf: &mut Bytes) -> Result<MessageId> {
    if buf.remaining() < 2 {
        return Err(ProtocolError::BufferTooSmall { expected: 2, actual: buf.remaining() });
    }
    let id = buf.get_u16();
    MessageId::from_u16(id).ok_or(ProtocolError::InvalidMessageId(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_roundtrip() {
        let test_values = [0, 1, 127, 128, 16383, 16384, 2097151, 2097152, -1, -128];
        
        for val in test_values {
            let mut buf = BytesMut::new();
            encode_varint(&mut buf, val);
            let mut decode_buf = buf.freeze();
            let decoded = decode_varint(&mut decode_buf).unwrap();
            assert_eq!(val, decoded, "Failed for value: {}", val);
        }
    }

    #[test]
    fn test_vlong_roundtrip() {
        let test_values = [0i64, 1, 127, 128, 16383, 16384, 2097151, 2097152, -1, -128, i64::MAX, i64::MIN];
        
        for val in test_values {
            let mut buf = BytesMut::new();
            encode_vlong(&mut buf, val);
            let mut decode_buf = buf.freeze();
            let decoded = decode_vlong(&mut decode_buf).unwrap();
            assert_eq!(val, decoded, "Failed for value: {}", val);
        }
    }

    #[test]
    fn test_string_roundtrip() {
        let long_string = "a".repeat(1000);
        let test_strings = ["", "hello", "🎮🎉", long_string.as_str()];
        
        for s in test_strings {
            let mut buf = BytesMut::new();
            encode_string(&mut buf, s);
            let mut decode_buf = buf.freeze();
            let decoded = decode_string(&mut decode_buf).unwrap();
            assert_eq!(s, decoded);
        }
    }

    #[test]
    fn test_boolean_roundtrip() {
        let mut buf = BytesMut::new();
        encode_boolean(&mut buf, true);
        encode_boolean(&mut buf, false);
        
        let mut decode_buf = buf.freeze();
        assert!(decode_boolean(&mut decode_buf).unwrap());
        assert!(!decode_boolean(&mut decode_buf).unwrap());
    }
}