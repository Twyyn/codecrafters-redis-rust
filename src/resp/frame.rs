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
    pub fn serialize(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        self.serialize_into(&mut buffer);
        buffer.freeze()
    }

    pub fn serialize_into(&self, buffer: &mut BytesMut) {
        let mut digits = itoa::Buffer::new();

        match self {
            Self::Simple(message) => {
                buffer.put_u8(if self.is_error() { b'-' } else { b'+' });
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
                buffer.extend_from_slice(digits.format(*value).as_bytes());
                buffer.extend_from_slice(CRLF);
            }

            Self::Bulk(bytes) => {
                buffer.put_u8(b'$');
                buffer.extend_from_slice(digits.format(bytes.len()).as_bytes());
                buffer.extend_from_slice(CRLF);
                buffer.extend_from_slice(bytes);
                buffer.extend_from_slice(CRLF);
            }

            Self::Array(frames) => {
                buffer.put_u8(b'*');
                buffer.extend_from_slice(digits.format(frames.len()).as_bytes());
                buffer.extend_from_slice(CRLF);

                for frame in frames {
                    frame.serialize_into(buffer);
                }
            }

            Self::Nil => {
                buffer.extend_from_slice(b"$-1\r\n");
            }
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}
