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
      Status::Done => {
        // Extract parser and replace with all new one since this one
        // is now consumed and we don't need it anymore
        let parser = std::mem::replace(&mut self.parser, Parser::<M>::new());
        Some(parser.into()?)
      },
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