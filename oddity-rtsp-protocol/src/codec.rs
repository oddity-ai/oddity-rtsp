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
  message::Message,
  request::Request,
  response::Response,
  error::Error,
};

pub trait Target {
  type Send: Message + Serialize;
  type Receive: Message;
}

pub struct AsClient;

impl Target for AsClient {
  type Send = Request;
  type Receive = Response;
}

pub struct AsServer;

impl Target for AsServer {
  type Send = Response;
  type Receive = Request;
}

pub struct Codec<T: Target> {
  parser: Parser<T::Receive>,
}

impl<T: Target> Codec<T> {

  pub fn new() -> Self {
    Self {
      parser: Parser::new(),
    }
  }

}

impl<T: Target> Decoder for Codec<T> {
  type Item = T::Receive;
  type Error = Error;

  fn decode(
    &mut self,
    src: &mut BytesMut,
  ) -> Result<Option<Self::Item>, Self::Error> {
    Ok(match self.parser.parse(src)? {
      Status::Done => {
        // Extract parser and replace with all new one since this one
        // is now consumed and we don't need it anymore
        let parser = std::mem::replace(&mut self.parser, Parser::<T::Receive>::new());
        Some(parser.into_message()?)
      },
      Status::Hungry => None,
    })
  }

}

impl<T: Target> Encoder<T::Send> for Codec<T> {
  type Error = Error;

  fn encode(
    &mut self,
    item: T::Send,
    dst: &mut BytesMut,
  ) -> Result<(), Self::Error> {
    item.serialize(dst)
  }

}