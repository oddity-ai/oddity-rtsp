use std::net::{TcpStream, Shutdown};
use std::io::{BufReader, BufWriter, Error};

pub fn split(
  stream: TcpStream,
) -> (
  BufReader<TcpStream>,
  BufWriter<TcpStream>,
  ShutdownHandle,
) {
  (
    BufReader::new(stream.try_clone().unwrap()),
    BufWriter::new(stream.try_clone().unwrap()),
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