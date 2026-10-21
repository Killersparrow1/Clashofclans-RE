use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::error::{ProtocolError, Result};

pub const HEADER_SIZE: usize = 7;
pub const MAX_PAYLOAD_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    pub message_id: u16,
    pub payload_length: u32,
    pub version: u16,
}

impl PacketHeader {
    pub fn new(message_id: u16, payload_length: u32, version: u16) -> Self {
        Self {
            message_id,
            payload_length,
            version,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.message_id);
        buf.put_uint(self.payload_length as u64, 3);
        buf.put_u16(self.version);
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self> {
        if buf.remaining() < HEADER_SIZE {
            return Err(ProtocolError::BufferTooSmall {
                expected: HEADER_SIZE,
                actual: buf.remaining(),
            });
        }

        let message_id = buf.get_u16();
        let payload_length = buf.get_uint(3) as u32;
        let version = buf.get_u16();

        Ok(Self {
            message_id,
            payload_length,
            version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: Bytes,
}

impl Packet {
    pub fn new(message_id: u16, payload: Bytes, version: u16) -> Self {
        let header = PacketHeader::new(message_id, payload.len() as u32, version);
        Self { header, payload }
    }

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(HEADER_SIZE + self.payload.len());
        self.header.encode(&mut buf);
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn decode(mut buf: Bytes) -> Result<Self> {
        let header = PacketHeader::decode(&mut buf)?;
        
        if buf.remaining() < header.payload_length as usize {
            return Err(ProtocolError::BufferTooSmall {
                expected: header.payload_length as usize,
                actual: buf.remaining(),
            });
        }

        let payload = buf.split_to(header.payload_length as usize);
        Ok(Self { header, payload })
    }

    pub fn try_decode(buf: &mut Bytes) -> Result<Option<Self>> {
        if buf.remaining() < HEADER_SIZE {
            return Ok(None);
        }

        let header = PacketHeader::decode(buf)?;
        
        if buf.remaining() < header.payload_length as usize {
            return Ok(None);
        }

        let payload = buf.split_to(header.payload_length as usize);
        Ok(Some(Self { header, payload }))
    }
}

pub struct PacketBuffer {
    buffer: BytesMut,
}

impl PacketBuffer {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    pub fn push(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn try_pop(&mut self) -> Result<Option<Packet>> {
        if self.buffer.len() < HEADER_SIZE {
            return Ok(None);
        }

        let mut temp = self.buffer.clone().freeze();
        match Packet::try_decode(&mut temp)? {
            Some(packet) => {
                self.buffer.advance(HEADER_SIZE + packet.payload.len());
                Ok(Some(packet))
            }
            None => Ok(None),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for PacketBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_header_encode_decode() {
        let header = PacketHeader::new(10101, 100, 0);
        let mut buf = BytesMut::new();
        header.encode(&mut buf);
        
        let mut decode_buf = buf.freeze();
        let decoded = PacketHeader::decode(&mut decode_buf).unwrap();
        
        assert_eq!(header.message_id, decoded.message_id);
        assert_eq!(header.payload_length, decoded.payload_length);
        assert_eq!(header.version, decoded.version);
    }

    #[test]
    fn test_packet_encode_decode() {
        let payload = Bytes::from(vec![1, 2, 3, 4, 5]);
        let packet = Packet::new(10101, payload.clone(), 0);
        let encoded = packet.encode();
        
        let decode_buf = encoded.freeze();
        let decoded = Packet::decode(decode_buf).unwrap();
        
        assert_eq!(packet.header.message_id, decoded.header.message_id);
        assert_eq!(packet.payload, decoded.payload);
    }

    #[test]
    fn test_packet_buffer() {
        let mut buffer = PacketBuffer::new();
        let payload = Bytes::from(vec![1, 2, 3, 4, 5]);
        let packet = Packet::new(10101, payload, 0);
        let encoded = packet.encode();
        
        buffer.push(&encoded);
        let decoded = buffer.try_pop().unwrap().unwrap();
        
        assert_eq!(packet.header.message_id, decoded.header.message_id);
        assert_eq!(packet.payload, decoded.payload);
    }
}