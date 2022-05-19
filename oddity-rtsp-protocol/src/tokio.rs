use tokio_util::codec::{
  Decoder,
  Encoder
};

use bytes::BytesMut;

use super::{
  parse::{
    Parser,
    Status,
  },
  serialize::Serialize,
  error::Error,
  io::Target,
};

pub struct Codec<T: Target> {
  parser: Parser<T::Inbound>,
}

impl<T: Target> Codec<T> {

  pub fn new() -> Self {
    Self {
      parser: Parser::new(),
    }
  }

}

impl<T: Target> Decoder for Codec<T> {
  type Item = T::Inbound;
  type Error = Error;

  fn decode(
    &mut self,
    src: &mut BytesMut,
  ) -> Result<Option<Self::Item>, Self::Error> {
    Ok(match self.parser.parse(src)? {
      Status::Done => {
        // Extract parser and replace with all new one since this one
        // is now consumed and we don't need it anymore
        let parser = std::mem::replace(&mut self.parser, Parser::<T::Inbound>::new());
        Some(parser.into_message()?)
      },
      Status::Hungry => None,
    })
  }

}

impl<T: Target> Encoder<T::Outbound> for Codec<T> {
  type Error = Error;

  fn encode(
    &mut self,
    item: T::Outbound,
    dst: &mut BytesMut,
  ) -> Result<(), Self::Error> {
    item.serialize(dst)
  }

}