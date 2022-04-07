// TODO Not sure if this is the right API...

use super::{
  message::{
    Message,
    Request,
    Response,
  },
  error::Result,
};

use bytes::Bytes;

pub struct Writer<M: Message> {
  _phantom: std::marker::PhantomData<M>,
}

pub trait Serialize {

  fn write(&mut self, bytes: &mut Bytes) -> Result<()>;

}

impl Serialize for Writer<Request> {

  fn write(&mut self, bytes: &mut Bytes) -> Result<()> {
    unimplemented!()
  }

}

impl Serialize for Writer<Response> {

  fn write(&mut self, bytes: &mut Bytes) -> Result<()> {
    unimplemented!()
  }

}