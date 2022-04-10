use std::collections::BTreeMap;
use std::fmt;

use super::{
  parse::Parse,
  serialize::Serialize,
};

pub use http::uri::Uri;
pub use bytes::Bytes;

pub trait Message: Serialize {
  type Metadata: Parse;

  fn new(
    metadata: Self::Metadata,
    headers: Headers,
    body: Option<Bytes>
  ) -> Self;

}

// TODO(gerwin) Builder APIs that do some stuff automatically such as setting
//  Content-Length header and handling encoding etc. Also need pre-set values
//  for error responses.

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
pub struct Response {
  pub version: Version,
  pub status: StatusCode,
  pub reason: String,
  pub headers: Headers,
  pub body: Option<Bytes>,
}

impl Message for Response {
  type Metadata = ResponseMetadata;

  fn new(
    metadata: ResponseMetadata,
    headers: Headers,
    body: Option<Bytes>,
  ) -> Self {
    Self {
      version: metadata.version,
      status: metadata.status,
      reason: metadata.reason,
      headers,
      body,
    }
  }

}

impl Response {

  pub fn error(
    status: StatusCode,
    reason: &str,
  ) -> Response {
    Response {
      version: Default::default(),
      status,
      reason: reason.to_string(),
      headers: Default::default(),
      body: Default::default(),
    }
  }

  pub fn status(&self) -> StatusCategory {
    match self.status {
      s if s >= 600 => StatusCategory::Unknown,
      s if s >= 500 => StatusCategory::ServerError,
      s if s >= 400 => StatusCategory::ClientError,
      s if s >= 300 => StatusCategory::Redirection,
      s if s >= 200 => StatusCategory::Success,
      s if s >= 100 => StatusCategory::Informational,
      _             => StatusCategory::Unknown,
    }
  }

}

impl fmt::Display for Response {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "Version: {}, Status Code: {}, Reason Phrase: {}",
      self.version,
      self.status,
      &self.reason)?;

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

#[derive(Clone, PartialEq, Debug)]
pub enum Method {
  Describe,
  Announce,
  Setup,
  Play,
  Pause,
  Record,
  Options,
  Redirect,
  Teardown,
  GetParameter,
  SetParameter,
}

impl fmt::Display for Method {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Method::Describe     => write!(f, "DESCRIBE"),
      Method::Announce     => write!(f, "ANNOUNCE"),
      Method::Setup        => write!(f, "SETUP"),
      Method::Play         => write!(f, "PLAY"),
      Method::Pause        => write!(f, "PAUSE"),
      Method::Record       => write!(f, "RECORD"),
      Method::Options      => write!(f, "OPTIONS"),
      Method::Redirect     => write!(f, "REDIRECT"),
      Method::Teardown     => write!(f, "TEARDOWN"),
      Method::GetParameter => write!(f, "GET_PARAMETER"),
      Method::SetParameter => write!(f, "SET_PARAMETER"),
    }
  }

}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Version {
  V1,
  V2,
  Unknown,
}

impl Default for Version {

  #[inline]
  fn default() -> Version {
    Version::V1
  }

}

impl fmt::Display for Version {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Version::V1      => write!(f, "1.0"),
      Version::V2      => write!(f, "2.0"),
      Version::Unknown => write!(f, "?"),
    }
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

pub type StatusCode = usize;

#[derive(Clone, PartialEq, Debug)]
pub enum StatusCategory {
  Informational,
  Success,
  Redirection,
  ClientError,
  ServerError,
  Unknown,
}

#[derive(Clone, Debug)]
pub struct ResponseMetadata {
  version: Version,
  status: StatusCode,
  reason: String,
}

impl ResponseMetadata {

  pub(super) fn new(
    version: Version,
    status: StatusCode,
    reason: String
  ) -> Self {
    Self {
      version,
      status,
      reason,
    }
  }

}

pub type Headers = BTreeMap<String, String>;