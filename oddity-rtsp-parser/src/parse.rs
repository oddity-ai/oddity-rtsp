use std::collections::HashMap;
use std::io::{Cursor, BufRead};

use super::{
  request::{
    Request,
    Metadata,
    Method,
    Version,
  },
  error::{
    Result,
    Error,
  },
};

#[derive(Clone, Debug)]
pub enum Status {
  Hungry,
  HungryFor(usize),
  Done,
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
    self.state = self.parse_inner(buffer)?;

    match &self.state {
      State::Body(Body::Complete) =>
        Ok(Status::Done),
      State::Body(Body::Incomplete(need)) =>
        Ok(Status::HungryFor(*need)),
      State::Head(_) =>
        Ok(Status::Hungry),
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
      _ =>
        Err(Error::NotDone)
    }
  }

  fn parse_inner(&mut self, buffer: &[u8]) -> Result<State> {
    match self.state {
      State::Head(head) => {
        let (read_bytes, next_head) =
          self.parse_inner_head(buffer, head)?;

        // TODO(gerwin) Is this correct?
        if read_bytes != 0 {
          self.buf.clear();
        }

        if read_bytes < buffer.len() {
          self.buf.extend_from_slice(&buffer[read_bytes..]);
        }

        match next_head {
          Head::Done => {
            self
              .find_content_length()
              .map(|content_length| {
                let content_length_remaining = content_length - self.buf.len();
                State::Body(Body::Incomplete(content_length_remaining))
              })
          },
          _ =>
            Ok(State::Head(next_head)),
        }
      },
      State::Body(Body::Incomplete(need)) => {
        let got = buffer.len();
        let body_bytes = &buffer[..need.min(got)];
        self.buf.extend_from_slice(body_bytes);
        if got == need {
          Ok(State::Body(Body::Complete))
        } else if got < need {
          Ok(State::Body(Body::Incomplete(need - got)))
        } else {
          Err(Error::BodyOverflow {
            need: need,
            got
          })
        }
      },
      State::Body(Body::Complete) => {
        Err(Error::BodyAlreadyDone)
      },
    }
  }

  fn parse_inner_head(
    &mut self,
    buffer: &[u8],
    mut head: Head,
  ) -> Result<(usize, Head)> {
    let buffer = if self.buf.len() > 0 {
      self.buf.extend_from_slice(buffer);
      &self.buf
    } else {
      buffer
    };

    let mut cursor = Cursor::new(buffer);
    let mut total_read_bytes = 0;
    loop {
      let mut line = String::new();
      let read_bytes = cursor.read_line(&mut line)
        .map_err(|_| Error::Encoding)?;
      if read_bytes == 0 {
        // If `read_line` returns `0`, then it means that it could
        // not read a full line. We break out of this loop, signal
        // to the caller that we have only read part of the buffer
        // by returning `total_read_bytes`.
        break;
      }

      total_read_bytes += read_bytes;
      head = Self::parse_inner_head_item(
        &mut self.metadata,
        &mut self.headers,
        line,
        head)?;
    }

    Ok((total_read_bytes, head))
  }

  fn parse_inner_head_item(
    metadata: &mut Option<Metadata>,
    headers: &mut HashMap<String, String>,
    line: String,
    head: Head,
  ) -> Result<Head> {
    let line = line.trim();
    match head {
      Head::FirstLine => {
        *metadata = Some(parse_metadata(&line)?);
        Ok(Head::Header)
      },
      Head::Header => {
        Ok(if !line.is_empty() {
          let (var, val) = parse_header(&line)?;
          headers.insert(var, val);
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
      .map_err(|_| Error::ContentLengthNotInteger {
        value: content_length.clone(),
      })
  }

}

#[derive(Clone, Copy)]
enum State {
  Head(Head),
  Body(Body),
}

#[derive(Clone, Copy)]
enum Head {
  FirstLine,
  Header,
  Done,
}

#[derive(Clone, Copy)]
enum Body {
  Incomplete(usize),
  Complete,
}

fn parse_metadata(line: &str) -> Result<Metadata> {
  let mut parts = line.split_whitespace();

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
  let mut parts = line
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
      var: var.clone(),
    })?
    .to_string();

  Ok((var, val))
}
