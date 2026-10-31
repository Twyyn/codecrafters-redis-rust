use bytes::BytesMut;

use super::{SpannedArg, SpannedFrame};

const MAX_ARGS: usize = 1024;
const MAX_BUFFERED: usize = 1024 * 1024;
const MAX_BULK_BYTES: usize = 512 * 1024;
const MAX_HEADER_LENGTH: usize = 20;

#[derive(Debug)]
enum Phase {
    ArrayHeader,
    BulkHeader {
        remaining_len: usize,
    },
    BulkBody {
        remaining_len: usize,
        end: usize,
        after_crlf: usize,
    },
}

#[derive(Debug)]
pub struct Parser {
    buf: BytesMut,
    phase: Phase,
    cursor: usize,
    spannedv: Vec<SpannedArg>,
}

impl Parser {
    pub fn new(buf: BytesMut) -> Self {
        Self {
            buf,
            phase: Phase::ArrayHeader,
            cursor: 0,
            spannedv: Vec::new(),
        }
    }

    pub fn buf_mut(&mut self) -> &mut BytesMut {
        &mut self.buf
    }

    pub fn buf(&self) -> &BytesMut {
        &self.buf
    }

    pub fn remaining_capacity(&self) -> usize {
        MAX_BUFFERED.saturating_sub(self.buf.len())
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn parse(&mut self) -> Result<Option<SpannedFrame>, ParseError> {
        loop {
            match self.phase {
                Phase::ArrayHeader => {
                    let Some(b'*') = self.buf.get(self.cursor) else {
                        return if self.cursor < self.buf.len() {
                            Err(ParseError::InvalidType)
                        } else {
                            Ok(None)
                        };
                    };

                    let Some((len, offset)) = self.parse_length(self.cursor)? else {
                        return Ok(None);
                    };

                    self.phase = Phase::BulkHeader { remaining_len: len };
                    self.cursor = offset;
                    self.spannedv = Vec::with_capacity(len);
                }

                Phase::BulkHeader { remaining_len } => {
                    let Some(b'$') = self.buf.get(self.cursor) else {
                        return if self.cursor < self.buf.len() {
                            Err(ParseError::InvalidType)
                        } else {
                            Ok(None)
                        };
                    };

                    if remaining_len >= MAX_BULK_BYTES {
                        return Err(ParseError::InvalidLength);
                    }

                    let Some((length, offset)) = self.parse_length(self.cursor)? else {
                        return Ok(None);
                    };

                    let end = offset
                        .checked_add(length + 1)
                        .ok_or(ParseError::LengthOverflow)?;

                    let after_crlf = end.checked_add(2).ok_or(ParseError::LengthOverflow)?;

                    if after_crlf > MAX_BUFFERED {
                        return Err(ParseError::BufferLimitExceeded);
                    }

                    self.cursor = offset;

                    self.phase = Phase::BulkBody {
                        remaining_len,
                        end,
                        after_crlf,
                    };
                }

                Phase::BulkBody {
                    remaining_len,
                    end,
                    after_crlf,
                } => {
                    if after_crlf > self.buf.len() {
                        return Ok(None);
                    }

                    if &self.buf[end..after_crlf] != b"\r\n" {
                        return Err(ParseError::MissingCrlf);
                    }

                    self.spannedv
                        .push(SpannedArg::new(self.cursor, end - self.cursor));

                    self.cursor = after_crlf;

                    if remaining_len > 1 {
                        self.phase = Phase::BulkHeader { remaining_len };
                        continue;
                    }

                    let source = self.buf.split_to(self.cursor).freeze();
                    let spannedv = std::mem::take(&mut self.spannedv);

                    self.cursor = 0;
                    self.phase = Phase::ArrayHeader;

                    return Ok(Some(SpannedFrame::new(source, spannedv)));
                }
            }
        }
    }

    pub fn feed(&mut self, received: &[u8]) -> Result<(), ParseError> {
        if received.len() > self.remaining_capacity() {
            return Err(ParseError::BufferLimitExceeded);
        }

        self.buf.extend_from_slice(received);

        Ok(())
    }

    fn parse_length(&self, offset: usize) -> Result<Option<(usize, usize)>, ParseError> {
        if offset >= self.buf.len() {
            return Ok(None);
        }

        let start = offset + 1;
        let remaining = &self.buf[start..];
        let scan_len = remaining.len().min(MAX_HEADER_LENGTH + 1);

        let Some(len) = memchr::memchr(b'\r', &remaining[..scan_len]) else {
            if remaining.len() > MAX_HEADER_LENGTH {
                return Err(ParseError::InvalidLength);
            }

            return Ok(None);
        };

        if len == 0 {
            return Err(ParseError::InvalidLength);
        }

        let Some(b'\n') = remaining.get(len + 1) else {
            return if len + 1 < remaining.len() {
                Err(ParseError::MissingCrlf)
            } else {
                Ok(None)
            };
        };

        let mut result = 0usize;

        for &digit in &remaining[..len] {
            if !digit.is_ascii_digit() {
                return Err(ParseError::InvalidLength);
            }

            result = result
                .checked_mul(10)
                .and_then(|n| n.checked_add((digit - b'0') as usize))
                .ok_or(ParseError::LengthOverflow)?;
        }

        Ok(Some((result, start + len + 2)))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("incomplete frame")]
    Incomplete,

    #[error("unexpected type prefix")]
    InvalidType,

    #[error("invalid length")]
    InvalidLength,

    #[error("length overflow")]
    LengthOverflow,

    #[error("missing CRLF terminator")]
    MissingCrlf,

    #[error("invalid command argument count")]
    InvalidArgCount,

    #[error("input buffer exceeds the size limit")]
    BufferLimitExceeded,
}
