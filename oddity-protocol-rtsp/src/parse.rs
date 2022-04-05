use std::collections::HashMap;
use std::io::{Cursor, BufRead};

use super::{
  message::{
    Message,
    Request,
    RequestMetadata,
    Response,
    ResponseMetadata,
    Version,
    StatusCode,
    Method,
    Headers,
    Bytes,
  },
  error::{
    Result,
    Error,
  },
};

pub type RequestParser = Parser<Request>;
pub type ResponseParser = Parser<Response>;

#[derive(Clone, PartialEq, Debug)]
pub enum Status {
  Hungry,
  Done,
}

pub struct Parser<M: Message>
  where M::Metadata: Parse
{
  state: State,
  metadata: Option<M::Metadata>,
  headers: Headers,
  /// This variable is used to hold buffered bytes during parsing
  /// of head and body of the request. After parsing is done, this
  /// buffer will hold the body data.
  buf: Bytes,
}

impl<M: Message> Parser<M>
  where M::Metadata: Parse
{

  pub fn new() -> Self {
    Self {
      state: State::Head(Head::FirstLine),
      metadata: None,
      headers: HashMap::new(),
      buf: Vec::new(),
    }
  }

  pub fn parse(&mut self, buf: &[u8]) -> Result<Status> {
    self.buf.extend_from_slice(buf);
    self.parse_loop()?;

    match &self.state {
      State::Body(Body::Complete) =>
        Ok(Status::Done),
      State::Body(Body::Incomplete) =>
        Ok(Status::Hungry),
      State::Head(_) =>
        Ok(Status::Hungry),
    }
  }

  fn parse_loop(&mut self) -> Result<()> {
    let mut again = true;
    while again {
      (self.state, again) = self.parse_inner()?;
    }

    Ok(())
  }

  fn parse_inner(&mut self) -> Result<(State, Again)> {
    match self.state {
      State::Head(head) => {
        let (read_bytes, next_head) =
          self.parse_inner_head(head)?;

        self.buf = self.buf[read_bytes..].to_vec(); // TODO(gerwin) Good style?

        match next_head {
          Head::Done => {
            if self.have_content_length() {
              Ok((State::Body(Body::Incomplete), true))
            } else {
              Ok((State::Body(Body::Complete), false))
            }
          },
          _ => {
            Ok((State::Head(next_head), false))
          }
        }
      },
      State::Body(Body::Incomplete) => {
        let need = self.find_content_length()?
          .ok_or_else(|| Error::ContentLengthMissing)?;
        let got = self.buf.len();
        if got == need {
          Ok((State::Body(Body::Complete), false))
        } else if got < need {
          Ok((State::Body(Body::Incomplete), false))
        } else {
          self.buf = self.buf[..need].to_vec(); // TODO
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
    mut head: Head,
  ) -> Result<(usize, Head)> {
    let mut cursor = Cursor::new(&self.buf); // TODO this might be a bit innefficient doing it every time
    let mut total_read_bytes = 0;
    while head != Head::Done {
      let mut line = String::new();

      let read_bytes = cursor.read_line(&mut line)
        .map_err(|_| Error::Encoding)?;

      if read_bytes == 0 {
        // Reached EOF.
        break;
      }

      if !line.ends_with('\n') {
        // No full line available: Break out of this loop, signal
        // to the caller that we have only read part of the buffer
        // by returning `total_read_bytes`.
        break;
      }

      total_read_bytes += read_bytes;
      head = Self::parse_inner_head_line(
        &mut self.metadata,
        &mut self.headers,
        line,
        head)?;
    }

    Ok((total_read_bytes, head))
  }

  fn parse_inner_head_line(
    metadata: &mut Option<M::Metadata>,
    headers: &mut HashMap<String, String>,
    line: String,
    head: Head,
  ) -> Result<Head> {
    let line = line.trim();
    match head {
      Head::FirstLine => {
        *metadata = Some(Self::parse_metadata(&line)?);
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

  fn parse_metadata(line: &str) -> Result<M::Metadata> {
    M::Metadata::parse(line)
  }

  fn have_content_length(&self) -> bool {
    self
      .headers
      .contains_key("Content-Length")
  }

  fn find_content_length(&self) -> Result<Option<usize>> {
    if let Some(content_length) = self
        .headers
        .get("Content-Length") {

      Ok(Some(content_length
        .parse::<usize>()
        .map_err(|_| Error::ContentLengthNotInteger {
          value: content_length.clone(),
        })?))
    } else {
      Ok(None)
    }
  }

  fn into(self) -> Result<M> {
    match self.state {
      State::Body(Body::Complete) =>
        Ok(M::new(
          self.metadata
            .ok_or(Error::MetadataNotParsed)?,
          self.headers,
          self.buf,
        )),
      _ =>
        Err(Error::NotDone)
    }
  }

}

impl Parser<Request> {

  pub fn into_request(self) -> Result<Request> {
    self.into()
  }

}

impl Parser<Response> {

  pub fn into_response(self) -> Result<Response> {
    self.into()
  }

}

type Again = bool;

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
  Head(Head),
  Body(Body),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Head {
  FirstLine,
  Header,
  Done,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Body {
  Incomplete,
  Complete,
}

pub trait Parse: Sized {
  fn parse(line: &str) -> Result<Self>;
}

impl Parse for RequestMetadata {

  fn parse(line: &str) -> Result<RequestMetadata> {
    let mut parts = line.split(' ');

    let method = parts
      .next()
      .ok_or_else(|| Error::RequestLineMalformed {
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

    let version = parse_version(version, line)?;

    Ok(RequestMetadata::new(method, uri, version))
  }

}

impl Parse for ResponseMetadata {

  fn parse(line: &str) -> Result<ResponseMetadata> {
    let (version, rest) = line
      .split_once(' ')
      .ok_or_else(|| Error::StatusCodeMissing {
        line: line.to_string()
      })?;

    let version = parse_version(version.trim(), line)?;

    let (status_code, rest) = rest
      .split_once(' ')
      .ok_or_else(|| Error::ReasonPhraseMissing {
        line: line.to_string()
      })?;

    let status_code = status_code
      .trim()
      .parse::<StatusCode>()
      .map_err(|_| Error::StatusCodeNotInteger {
        line: line.to_string(),
        status_code: status_code.to_string(),
      })?;

    let reason = rest
      .trim()
      .to_string();

    Ok(ResponseMetadata::new(version, status_code, reason))
  }

}

fn parse_version(part: &str, line: &str) -> Result<Version> {
  if part.starts_with("RTSP/") {
    Ok(match &part[5..] {
      "1.0" => Version::V1,
      "2.0" => Version::V2,
      _     => Version::Unknown,
    })
  } else {
    return Err(Error::VersionMalformed {
      line: line.to_string(),
      version: part.to_string(),
    });
  }
}

fn parse_header(line: &str) -> Result<(String, String)> {
  let (var, val) = line
    .split_once(":")
    .ok_or_else(|| Error::HeaderMalformed {
      line: line.to_string()
    })?;
    
  Ok((var.trim().to_string(), val.trim().to_string()))
}

#[cfg(test)]
mod tests {

  use super::{
    RequestParser,
    ResponseParser,
    Status,
    Version,
    Method,
  };

  #[test]
  fn parse_options_request() {
    let request = br###"OPTIONS rtsp://example.com/media.mp4 RTSP/1.0
CSeq: 1
Require: implicit-play
Proxy-Require: gzipped-messages

"###;

    let mut parser = RequestParser::new();
    assert_eq!(parser.parse(request).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Options);
    assert_eq!(request.metadata.uri, "rtsp://example.com/media.mp4");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"1".to_string()));
    assert_eq!(request.headers.get("Require"), Some(&"implicit-play".to_string()));
    assert_eq!(request.headers.get("Proxy-Require"), Some(&"gzipped-messages".to_string()));
  }

  #[test]
  fn parse_options_request_any() {
    let request = br###"OPTIONS * RTSP/1.0
CSeq: 1

"###;

    let mut parser = RequestParser::new();
    assert_eq!(parser.parse(request).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Options);
    assert_eq!(request.metadata.uri, "*");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"1".to_string()));
  }

  #[test]
  fn parse_options_response() {
    let response = br###"RTSP/1.0 200 OK
CSeq: 1
Public: DESCRIBE, SETUP, TEARDOWN, PLAY, PAUSE

"###;

    let mut parser = ResponseParser::new();
    assert_eq!(parser.parse(response).unwrap(), Status::Done);

    let response = parser.into_response().unwrap();
    assert_eq!(response.metadata.version, Version::V1);
    assert_eq!(response.metadata.status, 200);
    assert_eq!(response.metadata.reason, "OK");
    assert_eq!(response.headers.get("CSeq"), Some(&"1".to_string()));
    assert_eq!(response.headers.get("Public"), Some(&"DESCRIBE, SETUP, TEARDOWN, PLAY, PAUSE".to_string()));
  }

  #[test]
  fn parse_options_response_error() {
    let response = br###"RTSP/1.0 404 Stream Not Found
CSeq: 1

"###;

    let mut parser = ResponseParser::new();
    assert_eq!(parser.parse(response).unwrap(), Status::Done);

    let response = parser.into_response().unwrap();
    assert_eq!(response.metadata.version, Version::V1);
    assert_eq!(response.metadata.status, 404);
    assert_eq!(response.metadata.reason, "Stream Not Found");
    assert_eq!(response.headers.get("CSeq"), Some(&"1".to_string()));
  }

  #[test]
  fn parse_describe_request() {
    let request = br###"DESCRIBE rtsp://example.com/media.mp4 RTSP/1.0
CSeq: 2

"###;

    let mut parser = RequestParser::new();
    assert_eq!(parser.parse(request).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Describe);
    assert_eq!(request.metadata.uri, "rtsp://example.com/media.mp4");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"2".to_string()));
  }

  #[test]
  fn parse_describe_request_v2() {
    let request = br###"DESCRIBE rtsp://example.com/media.mp4 RTSP/2.0
CSeq: 2

"###;

    let mut parser = RequestParser::new();
    assert_eq!(parser.parse(request).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Describe);
    assert_eq!(request.metadata.uri, "rtsp://example.com/media.mp4");
    assert_eq!(request.metadata.version, Version::V2);
    assert_eq!(request.headers.get("CSeq"), Some(&"2".to_string()));
  }

  #[test]
  fn parse_describe_request_v3() {
    let request = br###"DESCRIBE rtsp://example.com/media.mp4 RTSP/3.0
CSeq: 2

"###;

    let mut parser = RequestParser::new();
    assert_eq!(parser.parse(request).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Describe);
    assert_eq!(request.metadata.uri, "rtsp://example.com/media.mp4");
    assert_eq!(request.metadata.version, Version::Unknown);
    assert_eq!(request.headers.get("CSeq"), Some(&"2".to_string()));
  }

  #[test]
  fn parse_describe_response() {
    let response = br###"RTSP/1.0 200 OK
CSeq: 2
Content-Base: rtsp://example.com/media.mp4
Content-Type: application/sdp
Content-Length: 443

m=video 0 RTP/AVP 96
a=control:streamid=0
a=range:npt=0-7.741000
a=length:npt=7.741000
a=rtpmap:96 MP4V-ES/5544
a=mimetype:string;"video/MP4V-ES"
a=AvgBitRate:integer;304018
a=StreamName:string;"hinted video track"
m=audio 0 RTP/AVP 97
a=control:streamid=1
a=range:npt=0-7.712000
a=length:npt=7.712000
a=rtpmap:97 mpeg4-generic/32000/2
a=mimetype:string;"audio/mpeg4-generic"
a=AvgBitRate:integer;65790
a=StreamName:string;"hinted audio track""###;

    let mut parser = ResponseParser::new();
    assert_eq!(parser.parse(response).unwrap(), Status::Done);

    let response = parser.into_response().unwrap();
    assert_eq!(response.metadata.version, Version::V1);
    assert_eq!(response.metadata.status, 200);
    assert_eq!(response.metadata.reason, "OK");
    assert_eq!(response.headers.get("CSeq"), Some(&"2".to_string()));
    assert_eq!(response.headers.get("Content-Base"), Some(&"rtsp://example.com/media.mp4".to_string()));
    assert_eq!(response.headers.get("Content-Type"), Some(&"application/sdp".to_string()));
    assert_eq!(response.headers.get("Content-Length"), Some(&"443".to_string()));
  }

  const EXAMPLE_REQUEST_PLAY: &[u8] = br###"PLAY rtsp://example.com/stream/0 RTSP/1.0
CSeq: 1
Session: 1234abcd
Content-Length: 16

0123456789abcdef"###;

  #[test]
  fn parse_play_request() {
    let mut parser = RequestParser::new();
    assert_eq!(parser.parse(EXAMPLE_REQUEST_PLAY).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Play);
    assert_eq!(request.metadata.uri, "rtsp://example.com/stream/0");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"1".to_string()));
    assert_eq!(request.headers.get("Session"), Some(&"1234abcd".to_string()));
    assert_eq!(request.headers.get("Content-Length"), Some(&"16".to_string()));
    assert_eq!(request.body, b"0123456789abcdef");
  }

  #[test]
  fn parse_play_request_partial_piece1() {
    let mut parser = RequestParser::new();

    let upto_last = EXAMPLE_REQUEST_PLAY.len() - 1;
    for i in 0..upto_last {
      let i_range = i..i + 1;
      assert_eq!(parser.parse(&EXAMPLE_REQUEST_PLAY[i_range]).unwrap(), Status::Hungry);
    }

    let last_range = EXAMPLE_REQUEST_PLAY.len() - 1..;
    assert_eq!(parser.parse(&EXAMPLE_REQUEST_PLAY[last_range]).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Play);
    assert_eq!(request.metadata.uri, "rtsp://example.com/stream/0");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"1".to_string()));
    assert_eq!(request.headers.get("Session"), Some(&"1234abcd".to_string()));
    assert_eq!(request.headers.get("Content-Length"), Some(&"16".to_string()));
    assert_eq!(request.body, b"0123456789abcdef");
  }
  
  #[test]
  fn parse_play_request_partial_piece2() {
    parse_play_request_partial_piece(2);
  }

  #[test]
  fn parse_play_request_partial_piece3() {
    parse_play_request_partial_piece(3);
  }

  fn parse_play_request_partial_piece(piece_size: usize) {
    let mut parser = RequestParser::new();

    let pieces_upto_last = (EXAMPLE_REQUEST_PLAY.len() / piece_size) - 1;
    for i in 0..pieces_upto_last {
      let piece_range = (i * piece_size)..(i * piece_size) + piece_size;
      assert_eq!(parser.parse(&EXAMPLE_REQUEST_PLAY[piece_range]).unwrap(), Status::Hungry);
    }

    let last_piece = pieces_upto_last;
    let leftover_piece_range = last_piece * piece_size..;
    assert_eq!(parser.parse(&EXAMPLE_REQUEST_PLAY[leftover_piece_range]).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Play);
    assert_eq!(request.metadata.uri, "rtsp://example.com/stream/0");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"1".to_string()));
    assert_eq!(request.headers.get("Session"), Some(&"1234abcd".to_string()));
    assert_eq!(request.headers.get("Content-Length"), Some(&"16".to_string()));
    assert_eq!(request.body, b"0123456789abcdef");
  }

  #[test]
  fn parse_play_request_partial_piece_varying() {
    let mut parser = RequestParser::new();

    let mut start = 0;
    let mut size = 1;
    loop {
      let piece_range = start..(start + size).min(EXAMPLE_REQUEST_PLAY.len());
      if let Status::Done = parser.parse(&EXAMPLE_REQUEST_PLAY[piece_range]).unwrap() {
        break;
      }
      start += size;
      size = (size * 2) % 9;
    }

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Play);
    assert_eq!(request.metadata.uri, "rtsp://example.com/stream/0");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"1".to_string()));
    assert_eq!(request.headers.get("Session"), Some(&"1234abcd".to_string()));
    assert_eq!(request.headers.get("Content-Length"), Some(&"16".to_string()));
    assert_eq!(request.body, b"0123456789abcdef");
  }

}