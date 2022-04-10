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

pub type Headers = BTreeMap<String, String>;

// TODO(gerwin) Builder APIs that do some stuff automatically such as setting
//  Content-Length header and handling encoding etc. Also need pre-set values
//  for error responses.

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