use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct SpannedArg {
    pub pos: usize,
    pub len: usize,
}

impl SpannedArg {
    pub fn new(pos: usize, len: usize) -> Self {
        Self { pos, len }
    }
}

#[derive(Debug)]
pub struct SpannedFrame {
    pub buf: Bytes,
    pub spanv: Vec<SpannedArg>,
}

impl SpannedFrame {
    pub fn new(buf: Bytes, spanv: Vec<SpannedArg>) -> Self {
        Self { buf, spanv }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    Simple(Bytes),
    Error(Bytes),
    Integer(i64),
    Bulk(Bytes),
    Array(Vec<Self>),
    Null,
}

impl Frame {
    pub fn encode(&self, buf: &mut BytesMut) {
        self.encode_inner(buf);
        buf.clear();
    }

    pub fn encode_inner(&self, buf: &mut BytesMut) {
        let mut digits = itoa::Buffer::new();

        match self {
            Self::Simple(message) => {
                buf.put_u8(b'+');
                buf.extend_from_slice(message);
                buf.extend_from_slice(b"\r\n");
            }
            Self::Error(message) => {
                buf.put_u8(b'-');
                buf.extend_from_slice(message);
                buf.extend_from_slice(b"\r\n");
            }

            Self::Integer(value) => {
                buf.put_u8(b':');
                buf.extend_from_slice(digits.format(*value).as_bytes());
                buf.extend_from_slice(b"\r\n");
            }

            Self::Bulk(bytes) => {
                buf.put_u8(b'$');
                buf.extend_from_slice(digits.format(bytes.len()).as_bytes());
                buf.extend_from_slice(b"\r\n");
                buf.extend_from_slice(bytes);
                buf.extend_from_slice(b"\r\n");
            }

            Self::Array(frames) => {
                buf.put_u8(b'*');
                buf.extend_from_slice(digits.format(frames.len()).as_bytes());
                buf.extend_from_slice(b"\r\n");

                for frame in frames {
                    frame.encode_inner(buf);
                }
            }

            Self::Null => {
                buf.extend_from_slice(b"$-1\r\n");
            }
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

impl From<SpannedFrame> for Frame {
    fn from(spanned: SpannedFrame) -> Self {
        let args = spanned
            .spanv
            .into_iter()
            .map(|span| {
                let start = span.pos;
                let end = start + span.len;

                Self::Bulk(spanned.buf.slice(start..end))
            })
            .collect();

        Self::Array(args)
    }
}
