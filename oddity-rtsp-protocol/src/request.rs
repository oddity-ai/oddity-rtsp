use std::fmt;

use super::{
  message::{
    Message,
    Headers,
    Bytes,
    Version,
    Method,
    Uri,
  },
};

#[derive(Clone, Debug)]
pub struct Request {
  pub method: Method,
  pub uri: Uri,
  pub version: Version,
  pub headers: Headers,
  pub body: Option<Bytes>,
}

impl Message for Request {
  type Metadata = RequestMetadata;

  fn new(
    metadata: RequestMetadata,
    headers: Headers,
    body: Option<Bytes>,
  ) -> Self {
    Self {
      method: metadata.method,
      uri: metadata.uri,
      version: metadata.version,
      headers,
      body,
    }
  }

}

impl fmt::Display for Request {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "Version: {}, Method: {}, Uri: {}",
      self.version,
      self.method,
      self.uri)?;

    writeln!(f, "Headers:")?;
    for (var, val) in &self.headers {
      writeln!(f, " - {}: {}", &var, &val)?;
    }

    if let Some(body) = &self.body {
      writeln!(f, "[{} bytes]", body.len())?;
    }

    Ok(())
  }

}

#[derive(Clone, Debug)]
pub struct RequestMetadata {
  method: Method,
  uri: Uri,
  version: Version,
}

impl RequestMetadata {

  pub(super) fn new(
    method: Method,
    uri: Uri,
    version: Version
  ) -> Self {
    Self {
      method,
      uri,
      version,
    }
  }

}

// TODO(gerwin) Builder for [`Request`].