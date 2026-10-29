use super::CRLF;

use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    Simple(Bytes),
    Error(Bytes),
    Integer(i64),
    Bulk(Bytes),
    Array(Vec<Self>),
    Nil,
}

impl Frame {
    pub fn serialize(&self, buffer: &mut BytesMut) -> Bytes {
        let mut number = itoa::Buffer::new();

        match self {
            Self::Simple(message) => {
                buffer.put_u8(b'+');
                buffer.extend_from_slice(message);
                buffer.extend_from_slice(CRLF);
            }

            Self::Error(message) => {
                buffer.put_u8(b'-');
                buffer.extend_from_slice(message);
                buffer.extend_from_slice(CRLF);
            }

            Self::Integer(value) => {
                buffer.put_u8(b':');
                buffer.extend_from_slice(value.to_string().as_bytes());
                buffer.extend_from_slice(CRLF);
            }

            Self::Bulk(bytes) => {
                buffer.put_u8(b'$');
                buffer.extend_from_slice(bytes.len().to_string().as_bytes());
                buffer.extend_from_slice(CRLF);
                buffer.extend_from_slice(bytes);
                buffer.extend_from_slice(CRLF);
            }

            Self::Array(frames) => {
                buffer.put_u8(b'*');
                buffer.extend_from_slice(frames.len().to_string().as_bytes());
                buffer.extend_from_slice(CRLF);
                for _ in frames {
                    self.serialize();
                }
            }

            Self::Nil => {
                buffer.extend_from_slice(b"$-1\r\n");
            }
        }

        buffer.freeze()
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}
