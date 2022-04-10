use std::fmt;

use super::{
  message::{
    Message,
    Headers,
    Bytes,
    Version,
    StatusCode,
    StatusCategory,
  },
  request::Request,
};

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

  pub fn to(
    request: &Request,
    mut headers: Headers,
  ) -> Response {
    if let Some(val) = request.headers.get("CSeq") {
      headers.insert("CSeq".to_string(), val.clone());
    }
    
    Response {
      version: Default::default(),
      status: 200,
      reason: "OK".to_string(),
      headers,
      body: Default::default(),
    }
  }

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