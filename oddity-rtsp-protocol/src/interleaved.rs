use super::{
  response::Response,
  serialize::Serialize,
  error::{
    Error,
    Result,
  },
};

use bytes::{
  Bytes,
  BytesMut,
  BufMut,
};

pub enum ResponseMaybeInterleaved {
  Message(Response),
  Interleaved {
    channel: u8,
    payload: Bytes,
  }
}

impl Serialize for ResponseMaybeInterleaved {

  fn serialize(self, dst: &mut BytesMut) -> Result<()> {
    match self {
      ResponseMaybeInterleaved::Message(response) => {
        response.serialize(dst)
      },
      ResponseMaybeInterleaved::Interleaved {
        channel,
        payload,
      } => {
        dst.put_u8(0x24); // $
        dst.put_u8(channel);
        dst.put_u16(payload
          .len()
          .try_into()
          .map_err(|_| Error::InterleavedPayloadTooLarge)?
        );
        dst.put(payload);

        Ok(())
      },
    }
  }

}