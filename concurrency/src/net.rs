use std::net::{TcpStream, Shutdown};
use std::io::Error;

pub fn split<R, W>(
  stream: TcpStream,
) -> (R, W, ShutdownHandle)
where
  R: From<TcpStream>,
  W: From<TcpStream>,
{
  (
    stream.try_clone().unwrap().into(),
    stream.try_clone().unwrap().into(),
    ShutdownHandle(stream),
  )
}

pub struct ShutdownHandle(TcpStream);

impl ShutdownHandle {

  pub fn shutdown(
    &mut self,
    how: Shutdown,
  ) -> Result<(), Error> {
    self.0.shutdown(how)
  }

}