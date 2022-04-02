mod buffer;

use std::collections::HashMap;
use std::io::{Cursor, BufRead};

type Result<T> = std::result::Result<T, Error>;

pub enum State {
  Head(Head),
  Body(Body),
  Failed(Error),
}

pub enum Head {
  FirstLine,
  Header,
  Done,
}

pub enum Body {
  Incomplete(usize),
  Complete,
}

pub enum Status {
  Hungry,
  HungryFor(usize),
  Done,
}

pub enum Line {
  Complete(String),
  Incomplete(String),
}

pub enum Version {
  V1,
  V2,
  Unknown,
}

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

pub struct Request {
  metadata: Metadata,
  headers: HashMap<String, String>,
  body: Vec<u8>,
}

impl Request {

  fn new(
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

pub struct Metadata {
  method: Method,
  uri: String, // TODO Uri type?
  version: Version,
}

impl Metadata {

  pub fn new(method: Method, uri: String, version: Version) -> Self {
    Self {
      method,
      uri,
      version,
    }
  }

}

pub struct Parser {
  state: State,
  metadata: Option<Metadata>,
  headers: HashMap<String, String>,
  /// This variable is used to hold buffered bytes during parsing
  /// of head and body of the request. After parsing is done, this
  /// buffer will hold the body data.
  buf: Vec<u8>,
}

impl Parser {

  pub fn new() -> Self {
    Self {
      state: State::Head(Head::FirstLine),
      metadata: None,
      headers: HashMap::new(),
      buf: Vec::new(),
    }
  }

  pub fn parse(&mut self, buffer: &[u8]) -> Result<Status> {
    self.state = self.parse_inner(buffer);

    match self.state {
      State::Body(Body::Complete) =>
        Ok(Status::Done),
      State::Body(Body::Incomplete(need)) =>
        Ok(Status::HungryFor(need)),
      State::Head(_) =>
        Ok(Status::Hungry),
      State::Failed(err) =>
        Err(err),
    }
  }

  pub fn into_request(self) -> Result<Request> {
    match self.state {
      State::Body(Body::Complete) =>
        Ok(Request::new(
          self.metadata
            .ok_or(Error::MetadataNotParsed)?,
          self.headers,
          self.buf,
        )),
      State::Failed(err) =>
        Err(err),
      _ =>
        Err(Error::NotDone)
    }
  }

  fn parse_inner(&mut self, buffer: &[u8]) -> State {
    match self.state {
      State::Head(head) => {
        let buffer = if self.buf.len() > 0 {
          self.buf.extend_from_slice(buffer);
          &self.buf
        } else {
          buffer
        };

        let (read_bytes, next_head) =
          match self.parse_inner_head(buffer, head) {
            Ok(parse_inner_head_result) => {
              parse_inner_head_result
            },
            Err(err) => {
              return State::Failed(err);
            }
          };

        // TODO(gerwin) Is this correct?
        if read_bytes != 0 {
          self.buf.clear();
        }

        if read_bytes < buffer.len() {
          self.buf.extend_from_slice(&buffer[read_bytes..]);
        }

        match next_head {
          Head::Done => {
            match self.find_content_length() {
              Ok(content_length) => {
                let content_length_remaining = content_length - self.buf.len();
                State::Body(Body::Incomplete(content_length_remaining))
              },
              Err(err) =>
                State::Failed(err)
            }
          },
          _ =>
            State::Head(next_head),
        }
      },
      State::Body(Body::Incomplete(need)) => {
        let got = buffer.len();
        let body_bytes = &buffer[..need.min(got)];
        self.buf.extend_from_slice(body_bytes);
        if got == need {
          State::Body(Body::Complete)
        } else if got < need {
          State::Body(Body::Incomplete(need - got))
        } else {
          State::Failed(Error::BodyOverflow { need, got })
        }
      },
      State::Body(Body::Complete) => {
        State::Failed(Error::BodyAlreadyDone)
      },
      State::Failed(_) => {
        State::Failed(Error::AlreadyError)
      },
    }
  }

  fn parse_inner_head(
    &mut self,
    buffer: &[u8],
    mut head: Head,
  ) -> Result<(usize, Head)> {
    let mut cursor = Cursor::new(buffer);
    let mut line = String::new();
    let mut total_read_bytes = 0;
    loop {
      let mut read_bytes = cursor.read_line(&mut line)
        .map_err(|_| Error::Encoding)?;
      if read_bytes == 0 {
        // If `read_line` returns `0`, then it means that it could
        // not read a full line. We break out of this loop, signal
        // to the caller that we have only read part of the buffer
        // by returning `total_read_bytes`.
        break;
      }

      total_read_bytes += read_bytes;
      head = self.parse_inner_head_item(line, head)?;
    }

    Ok((total_read_bytes, head))
  }

  fn parse_inner_head_item(
    &mut self,
    line: String,
    head: Head,
  ) -> Result<Head> {
    let line = line.trim();
    match head {
      Head::FirstLine => {
        self.metadata = Some(parse_metadata(&line)?);
        Ok(Head::Header)
      },
      Head::Header => {
        Ok(if !line.is_empty() {
          let (var, val) = parse_header(&line)?;
          self.headers.insert(var, val);
          Head::Header
        } else {
          // The line is empty, so we got CRLF, which signals end of
          // headers for this request.
          Head::Done
        })
      },
      Head::Done =>
        Err(Error::HeadAlreadyDone),
    }
  }

  fn find_content_length(&self) -> Result<usize> {
    let content_length = self
      .headers
      .get("Content-Length")
      .ok_or_else(|| Error::ContentLengthMissing)?;

    content_length
      .parse::<usize>()
      .map_err(|_| Error::ContentLengthNotInteger)
  }

}

fn parse_metadata(line: &str) -> Result<Metadata> {
  let parts = line.split_whitespace();

  let method = parts
    .next()
    .ok_or_else(|| Error::FirstLineMalformed {
      line: line.to_string(),
    })?;

  let method = match method {
    "DESCRIBE"      => Method::Describe,
    "ANNOUNCE"      => Method::Announce,
    "SETUP"         => Method::Setup,
    "PLAY"          => Method::Play,
    "PAUSE"         => Method::Pause,
    "RECORD"        => Method::Record,
    "OPTIONS"       => Method::Options,
    "REDIRECT"      => Method::Redirect,
    "TEARDOWN"      => Method::Teardown,
    "GET_PARAMETER" => Method::GetParameter,
    "SET_PARAMETER" => Method::SetParameter,
    _ => {
      return Err(Error::MethodUnknown {
        line: line.to_string(),
        method: method.to_string(),
      });
    },
  };

  let uri = parts
    .next()
    .ok_or_else(|| Error::UriMissing {
      line: line.to_string(),
    })?
    .to_string();

  let version = parts
    .next()
    .ok_or_else(|| Error::VersionMissing {
      line: line.to_string(),
    })?;

  let version = if version.starts_with("RTSP/") {
    match &version[5..] {
      "1.0" => Version::V1,
      "2.0" => Version::V2,
      _     => Version::Unknown,
    }
  } else {
    return Err(Error::VersionMalformed {
      line: line.to_string(),
      version: version.to_string(),
    });
  };

  Ok(Metadata::new(method, uri, version))
}

fn parse_header(line: &str) -> Result<(String, String)> {
  let parts = line
    .split(":")
    .map(|part| part.trim());
    
  let var = parts
    .next()
    .ok_or_else(|| Error::HeaderVariableMissing {
      line: line.to_string(),
    })?
    .to_string();

  let val = parts
    .next()
    .ok_or_else(|| Error::HeaderValueMissing {
      line: line.to_string(),
      var,
    })?
    .to_string();

  Ok((var, val))
}

pub enum Error {
  /// An error occurred decoding the header due to incorrect usage
  /// of text encoding by the sender.
  Encoding,
  /// The first line of the head part is malformed.
  FirstLineMalformed {
    line: String
  },
  /// The specified method is not a valid method.
  MethodUnknown {
    line: String,
    method: String
  },
  /// The header first line does have a method, but it does not have
  /// a target URI, which is the required second part of the first
  /// line of the head.
  UriMissing {
    line: String
  },
  /// The header first line does have a method and target URI, but
  /// it does not have a version, which is the required third part
  /// of the first line of the head.
  VersionMissing {
    line: String
  },
  /// The version specifier is incorrect. It should start with "RTSP/"
  /// followed by a digit, "." and another digit.
  VersionMalformed {
    line: String,
    version: String
  },
  /// Header line is missing the header variable.
  HeaderVariableMissing {
    line: String,
  },
  /// Header does not have value.
  HeaderValueMissing {
    line: String,
    var: String,
  },
  /// The Content-Length header is missing, but it is required.
  ContentLengthMissing,
  /// The Content-Length header is not an integer value, or cannot be
  /// converted to an unsigned integer.
  ContentLengthNotInteger,
  /// This occurs when the caller invokes the state machine with a
  /// state that signals that parsing the head part of the request
  /// was already done before.
  HeadAlreadyDone,
  /// This occurs when the caller invokes the state machine with a
  /// state that signals that parsing the body part of the request
  /// was already done before.
  BodyAlreadyDone,
  /// This occurs when the client provided more bytes than expected,
  /// and appending any more bytes to the body would cause it to
  /// become larger than the provided Content-Length.
  BodyOverflow {
    need: usize,
    got: usize,
  },
  /// This occurs when the caller tries to feed a state machine that
  /// is already in an error state.
  AlreadyError,
  /// Metadata was not parsed for some reason.
  MetadataNotParsed,
  /// This occurs when the caller tries to turn the parser into an
  /// actual request, but the parser was not ready yet.
  NotDone,
}