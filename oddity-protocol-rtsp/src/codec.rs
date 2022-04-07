use tokio_util::codec::{Decoder, Encoder};

use bytes::BytesMut;

use super::{
  parse::{
    Parser,
    Status,
  },
  message::Message,
  error::Error,
};

pub struct Codec<M: Message> {
  parser: Parser<M>,
}

impl<M: Message> Decoder for Codec<M> {
  type Item = M;
  type Error = Error;

  fn decode(
    &mut self,
    src: &mut BytesMut,
  ) -> Result<Option<Self::Item>, Self::Error> {
    Ok(match self.parser.parse(src)? {
      Status::Done => Some(self.parser.into()?),
      Status::Hungry => None,
    })
  }

}

impl<M: Message> Encoder<M> for Codec<M> {
  type Error = Error;

  fn encode(
    &mut self,
    item: M,
    dst: &mut BytesMut,
  ) -> Result<(), Self::Error> {
    unimplemented!() // TODO
  }

}