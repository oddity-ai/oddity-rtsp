use std::collections::HashMap;

pub struct Request {
  metadata: Metadata,
  headers: HashMap<String, String>,
  body: Vec<u8>,
}

impl Request {

  pub(super) fn new(
    metadata: Metadata,
    headers: HashMap<String, String>,
    body: Vec<u8>
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

#[derive(Clone, Debug)]
pub enum Version {
  V1,
  V2,
  Unknown,
}

pub(super) struct Metadata {
  method: Method,
  uri: String, // TODO Uri type?
  version: Version,
}

impl Metadata {

  pub(super) fn new(method: Method, uri: String, version: Version) -> Self {
    Self {
      method,
      uri,
      version,
    }
  }

}
