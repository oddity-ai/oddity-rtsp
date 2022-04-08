use std::collections::HashMap;

use super::{
  parse::Parse,
  serialize::Serialize,
};

pub use http::uri::Uri;
pub use bytes::Bytes;

pub trait Message {
  type Metadata: Parse + Serialize;

  fn new(
    metadata: Self::Metadata,
    headers: Headers,
    body: Option<Bytes>
  ) -> Self;

}

#[derive(Clone, Debug)]
pub struct Request {
  pub metadata: RequestMetadata,
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
      metadata,
      headers,
      body,
    }
  }

}

#[derive(Clone, Debug)]
pub struct Response {
  pub metadata: ResponseMetadata,
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
      metadata,
      headers,
      body,
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
  pub method: Method,
  pub uri: Uri,
  pub version: Version,
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
  pub version: Version,
  pub status: StatusCode,
  pub reason: String,
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

pub type Headers = HashMap<String, String>;