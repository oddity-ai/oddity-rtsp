use std::collections::HashMap;

pub trait Message {
  type Metadata;

  fn new(metadata: Self::Metadata, headers: Headers, body: Bytes) -> Self;
}

pub struct Request {
  metadata: RequestMetadata,
  headers: Headers,
  body: Bytes,
}

impl Message for Request {
  type Metadata = RequestMetadata;

  fn new(
    metadata: RequestMetadata,
    headers: Headers,
    body: Bytes,
  ) -> Self {
    Self {
      metadata,
      headers,
      body,
    }
  }

}

pub struct Response {
  metadata: ResponseMetadata,
  headers: Headers,
  body: Bytes,
}

impl Message for Response {
  type Metadata = ResponseMetadata;

  fn new(
    metadata: ResponseMetadata,
    headers: Headers,
    body: Bytes,
  ) -> Self {
    Self {
      metadata,
      headers,
      body,
    }
  }

}

#[derive(Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub enum Version {
  V1,
  V2,
  Unknown,
}

pub struct RequestMetadata {
  method: Method,
  uri: String, // TODO Uri type?
  version: Version,
}

impl RequestMetadata {

  pub(super) fn new(
    method: Method,
    uri: String,
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
pub type Headers = HashMap<String, String>;

pub type Bytes = Vec<u8>;