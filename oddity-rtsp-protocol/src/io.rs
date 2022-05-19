use std::marker::PhantomData;
use std::io::{Read, Write};

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
  interleaved::ResponseMaybeInterleaved,
  error::Error,
};

pub trait Target {
  type Inbound: Message;
  type Outbound: Serialize;
}

pub struct AsClient;

impl Target for AsClient {
  type Inbound = Response;
  type Outbound = Request;
}

pub struct AsServer;

impl Target for AsServer {
  type Inbound = Request;
  type Outbound = ResponseMaybeInterleaved;
}

pub struct RtspWriter<W: Write, T: Target> {
  inner: W,
  _marker: PhantomData<T>,
}

pub type RtspRequestWriter<W> = RtspWriter<W, AsClient>;
pub type RtspResponseReader<R> = RtspReader<R, AsClient>;

pub type RtspResponseWriter<W> = RtspWriter<W, AsServer>;
pub type RtspRequestReader<R> = RtspReader<R, AsServer>;

impl<W: Write, T: Target> RtspWriter<W, T> {

  pub fn new(
    inner: W,
  ) -> Self {
    Self {
      inner,
      _marker: Default::default(),
    }
  }

  pub fn write(
    &mut self,
    item: T::Outbound,
  ) -> Result<(), Error> {
    let mut bytes = BytesMut::new();
    item.serialize(&mut bytes)?;
    self
      .inner
      .write_all(&bytes)
      .map_err(Error::Io)
  }

}

impl<W: Write, T: Target> From<W> for RtspWriter<W, T> {

  fn from(inner: W) -> Self {
    Self::new(inner)
  }

}

pub struct RtspReader<R: Read, T: Target> {
  inner: R,
  parser: Parser<T::Inbound>,
  buffer_codec: BytesMut,
  buffer_read: [u8; 1024],
}

impl<R: Read, T: Target> RtspReader<R, T> {

  pub fn new(
    inner: R,
  ) -> Self {
    Self {
      inner,
      parser: Parser::new(),
      buffer_codec: BytesMut::new(),
      buffer_read: [0_u8; 1024],
    }
  }

  pub fn read(
    &mut self,
  ) -> Result<T::Inbound, Error> {
    loop {
      let num_bytes = self
        .inner
        .read(&mut self.buffer_read)
        .map_err(Error::Io)?;

      if num_bytes == 0 {
        return Err(Error::Shutdown);
      }

      self
        .buffer_codec
        .extend_from_slice(&self.buffer_read[..num_bytes]);

      if let Status::Done = self
          .parser
          .parse(&mut self.buffer_codec)? {
        // Replace the parser embedded in the writer with a new
        // one so we can consume this one and convert it into an
        // actual read item.
        let parser = std::mem::replace(
          &mut self.parser,
          Parser::<T::Inbound>::new()
        );

        return parser.into_message();
      }
    }
  }

}

impl<R: Read, T: Target> From<R> for RtspReader<R, T> {

  fn from(inner: R) -> Self {
    Self::new(inner)
  }

}