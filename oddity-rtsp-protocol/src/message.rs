use std::collections::BTreeMap;

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
//   Content-Length header and handling encoding etc.

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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Version {
  V1,
  V2,
  Unknown,
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