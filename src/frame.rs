use bytes::{Buf, BufMut, Bytes, BytesMut};
use memchr::memmem;

const CRLF: &[u8] = b"\r\n";

#[derive(Debug, Clone)]
pub enum Frame {
    String(String),
    Error(String),
    Integer(i64),
    Bulk(Bytes),
    Array(Vec<Self>),
    Null,
}

impl Frame {
    pub fn serialize(&self, buffer: &mut BytesMut) {
        buffer.clear();
        match self {
            Self::String(text) => {
                buffer.put_u8(b'+');
                buffer.extend_from_slice(text.as_bytes());
                buffer.extend_from_slice(CRLF);
            }

            Self::Error(message) => {
                buffer.put_u8(b'-');
                buffer.extend_from_slice(message.as_bytes());
                buffer.extend_from_slice(CRLF);
            }

            Self::Integer(value) => {
                buffer.put_u8(b':');
                buffer.extend_from_slice(format!("{value}").as_bytes());
                buffer.extend_from_slice(CRLF);
            }

            Self::Bulk(data) => {
                buffer.put_u8(b'$');
                buffer.extend_from_slice(format!("{}", data.len()).as_bytes());
                buffer.extend_from_slice(CRLF);
                buffer.extend_from_slice(data);
                buffer.extend_from_slice(CRLF);
            }

            Self::Array(frames) => {
                buffer.put_u8(b'*');
                buffer.extend_from_slice(format!("{}", frames.len()).as_bytes());
                buffer.extend_from_slice(CRLF);

                for frame in frames {
                    frame.serialize(buffer);
                }
            }

            Self::Null => {
                buffer.extend_from_slice(b"*-1\r\n");
            }
        }
    }

    pub fn parse(buffer: &mut BytesMut) -> Result<Self, FrameError> {
        let mut parser = Parser::new(buffer);

        let frame = parser.parse_frame()?;

        let consumed = parser.consumed();
        buffer.advance(consumed);

        Ok(frame)
    }
}

struct Parser<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    fn parse_frame(&mut self) -> Result<Frame, FrameError> {
        match self.read()? {
            b'+' => {
                let line = self.read_line()?;

                let text = String::from_utf8(line.to_vec())
                    .map_err(|_| FrameError::Invalid("invalid string"))?;

                Ok(Frame::String(text))
            }

            b'-' => {
                let line = self.read_line()?;

                let message = String::from_utf8(line.to_vec())
                    .map_err(|_| FrameError::Invalid("invalid error string"))?;

                Ok(Frame::Error(message))
            }

            b':' => {
                let value = self.read_decimal()?;
                Ok(Frame::Integer(value))
            }

            b'$' => {
                if self.peek()? == b'-' {
                    let line = self.read_line()?;

                    if line != b"-1" {
                        return Err(FrameError::Invalid("invalid null frame"));
                    }

                    return Ok(Frame::Null);
                }

                let length = usize::try_from(self.read_decimal()?)
                    .map_err(|_| FrameError::Invalid("invalid bulk string length"))?;

                let total = length
                    .checked_add(CRLF.len())
                    .ok_or(FrameError::Invalid("bulk string length overflow"))?;

                if self.remaining().len() < total {
                    return Err(FrameError::Incomplete);
                }

                let data = &self.remaining()[..length];
                let end_slice = &self.remaining()[length..total];

                if end_slice != CRLF {
                    return Err(FrameError::Invalid("invalid bulk string terminator"));
                }

                let data = Bytes::copy_from_slice(data);

                self.advance(total)?;

                Ok(Frame::Bulk(data))
            }

            b'*' => {
                let length = usize::try_from(self.read_decimal()?)
                    .map_err(|_| FrameError::Invalid("invalid array length"))?;

                let mut frames = Vec::with_capacity(length);
                for _ in 0..length {
                    let frame = self.parse_frame()?;
                    frames.push(frame);
                }

                Ok(Frame::Array(frames))
            }

            _ => Err(FrameError::Invalid("unknown frame type")),
        }
    }

    fn consumed(&self) -> usize {
        self.position
    }

    fn remaining(&self) -> &'a [u8] {
        &self.buffer[self.position..]
    }

    fn advance(&mut self, nth: usize) -> Result<(), FrameError> {
        if self.remaining().len() < nth {
            return Err(FrameError::Incomplete);
        }

        self.position += nth;
        Ok(())
    }

    fn peek(&self) -> Result<u8, FrameError> {
        self.buffer.first().copied().ok_or(FrameError::Incomplete)
    }

    fn read(&mut self) -> Result<u8, FrameError> {
        let byte = self.peek()?;
        self.buffer.advance(1);
        Ok(byte)
    }

    fn read_line(&mut self) -> Result<&'a [u8], FrameError> {
        let remaining = self.buffer;

        let Some(crlf_index) = memmem::find(remaining, CRLF) else {
            return Err(FrameError::Incomplete);
        };

        let line = &remaining[..crlf_index];

        self.position += crlf_index + CRLF.len();

        Ok(line)
    }

    fn read_decimal(&mut self) -> Result<i64, FrameError> {
        let line = self.read_line()?;

        if line.is_empty() {
            return Err(FrameError::Invalid("empty integer"));
        }

        let text = std::str::from_utf8(line).map_err(|_| FrameError::Invalid("invalid integer"))?;

        text.parse::<i64>()
            .map_err(|_| FrameError::Invalid("invalid integer"))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("incomplete frame")]
    Incomplete,

    #[error("invalid frame: {0}")]
    Invalid(&'static str),
}
