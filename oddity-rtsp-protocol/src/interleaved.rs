use std::fmt;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{
    error::{Error, Result},
    message::Message,
    request::Request,
    response::Response,
    serialize::Serialize,
};

pub const MAGIC: u8 = 0x24; // $

pub type ChannelId = u8;

pub type RequestMaybeInterleaved = MaybeInterleaved<Request>;
pub type ResponseMaybeInterleaved = MaybeInterleaved<Response>;

pub enum MaybeInterleaved<M: Message> {
    Message(M),
    Interleaved { channel: ChannelId, payload: Bytes },
}

impl<M: Message> fmt::Display for MaybeInterleaved<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Message(message) => write!(f, "{}", message),
            Self::Interleaved { channel, payload } => write!(
                f,
                "interleaved payload over channel: {}, size: {}",
                channel,
                payload.len()
            ),
        }
    }
}

impl<M: Message> Serialize for MaybeInterleaved<M> {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        match self {
            Self::Message(response) => response.serialize(dst),
            Self::Interleaved { channel, payload } => {
                dst.put_u8(MAGIC); // $
                dst.put_u8(channel);
                dst.put_u16(
                    payload
                        .len()
                        .try_into()
                        .map_err(|_| Error::InterleavedPayloadTooLarge)?,
                );
                dst.put(payload);

                Ok(())
            }
        }
    }
}

pub struct InterleavedParser {
    channel_and_size: Option<(u8, u16)>,
}

impl InterleavedParser {
    pub fn new() -> Self {
        Self {
            channel_and_size: None,
        }
    }

    pub fn parse(&mut self, buffer: &mut impl Buf) -> Option<Result<(ChannelId, Bytes)>> {
        if let Some((channel, size)) = self.channel_and_size {
            if buffer.remaining() >= size.into() {
                let payload = buffer.copy_to_bytes(size.into());
                Some(Ok((channel, payload)))
            } else {
                None
            }
        } else if buffer.remaining() >= 4 {
            let header = &buffer.chunk()[..4];
            if header[0] != MAGIC {
                return Some(Err(Error::InterleavedInvalid));
            }

            let channel = header[1];
            let size = u16::from_be_bytes([header[2], header[3]]);

            self.channel_and_size = Some((channel, size));

            buffer.advance(4);

            self.parse(buffer)
        } else {
            None
        }
    }
}
