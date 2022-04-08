use bytes::{
  BytesMut,
  BufMut,
};

use super::{
  message::{
    Request,
    RequestMetadata,
    Response,
    ResponseMetadata,
    Version,
    Method,
    StatusCode,
    Uri,
  },
  error::{
    Error,
    Result,
  },
};

pub trait Serialize {

  fn serialize(self, dst: &mut BytesMut) -> Result<()>;

}

impl Serialize for Request {

  fn serialize(self, dst: &mut BytesMut) -> Result<()> {
    self.metadata.serialize(dst)?;

    for (var, val) in self.headers.into_iter() {
      dst.put(format!("{}: {}\r\n", var, val).as_bytes());
    }

    dst.put(b"\r\n".as_slice());

    if let Some(body) = self.body {
      dst.put(body);
    }

    Ok(())
  }

}

impl Serialize for RequestMetadata {

  fn serialize(self, dst: &mut BytesMut) -> Result<()> {
    self.method.serialize(dst)?;
    dst.put_u8(b' ');
    self.uri.serialize(dst)?;
    dst.put_u8(b' ');
    self.version.serialize(dst)?;
    dst.put_u8(b'\r');
    dst.put_u8(b'\n');

    Ok(())
  }

}

impl Serialize for Response {

  fn serialize(self, dst: &mut BytesMut) -> Result<()> {
    self.metadata.serialize(dst)?;

    for (var, val) in self.headers.into_iter() {
      dst.put(format!("{}: {}\r\n", var, val).as_bytes());
    }

    dst.put(b"\r\n".as_slice());

    if let Some(body) = self.body {
      dst.put(body);
    }

    Ok(())
  }

}

impl Serialize for ResponseMetadata {

  fn serialize(self, dst: &mut BytesMut) -> Result<()> {
    self.version.serialize(dst)?;
    dst.put_u8(b' ');
    self.status.serialize(dst)?;
    dst.put_u8(b' ');
    dst.put(self.reason.as_bytes());
    dst.put_u8(b'\r');
    dst.put_u8(b'\n');

    Ok(())
  }

}

impl Serialize for Version {

  fn serialize(self, dst: &mut BytesMut) -> Result<()> {
    let version = match self {
      Version::V1 => b"RTSP/1.0".as_slice(),
      Version::V2 => b"RTSP/2.0".as_slice(),
      Version::Unknown => return Err(Error::VersionUnknown),
    };

    dst.put(version);
    Ok(())
  }

}

impl Serialize for Method {

  fn serialize(self, dst: &mut BytesMut) -> Result<()> {
    let method = match self {
      Method::Describe     => b"DESCRIBE".as_slice(),
      Method::Announce     => b"ANNOUNCE".as_slice(),
      Method::Setup        => b"SETUP".as_slice(),
      Method::Play         => b"PLAY".as_slice(),
      Method::Pause        => b"PAUSE".as_slice(),
      Method::Record       => b"RECORD".as_slice(),
      Method::Options      => b"OPTIONS".as_slice(),
      Method::Redirect     => b"REDIRECT".as_slice(),
      Method::Teardown     => b"TEARDOWN".as_slice(),
      Method::GetParameter => b"GET_PARAMETER".as_slice(),
      Method::SetParameter => b"SET_PARAMETER".as_slice(),
    };

    dst.put(method);
    Ok(())
  }

}

impl Serialize for Uri {


  fn serialize(self, dst: &mut BytesMut) -> Result<()> {
    dst.put(self.to_string().as_bytes());
    Ok(())
  }

}

impl Serialize for StatusCode {


  fn serialize(self, dst: &mut BytesMut) -> Result<()> {
    dst.put(self.to_string().as_bytes());
    Ok(())
  }

}