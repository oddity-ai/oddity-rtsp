use std::collections::HashMap;

pub use http::uri::Uri;

pub trait Message {
  type Metadata;

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

}

pub type Headers = HashMap<String, String>;

pub type Bytes = Vec<u8>;