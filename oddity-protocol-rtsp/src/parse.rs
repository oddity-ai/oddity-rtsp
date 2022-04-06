use std::collections::HashMap;

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
    Uri,
    Headers,
    Bytes,
  },
  buffer::Buffer,
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
  body: Option<Bytes>,
}

impl<M: Message> Parser<M>
  where M::Metadata: Parse
{

  pub fn new() -> Self {
    Self {
      state: State::Head(Head::FirstLine),
      metadata: None,
      headers: HashMap::new(),
      body: None,
    }
  }

  pub fn parse(&mut self, buffer: &mut Buffer) -> Result<Status> {
    self.parse_loop(buffer)?;

    match &self.state {
      State::Body(Body::Complete) =>
        Ok(Status::Done),
      State::Body(Body::Incomplete) =>
        Ok(Status::Hungry),
      State::Head(_) =>
        Ok(Status::Hungry),
    }
  }

  fn parse_loop(&mut self, buffer: &mut Buffer) -> Result<()> {
    let mut again = true;
    while again {
      (self.state, again) = self.parse_inner(buffer)?;
    }

    Ok(())
  }

  fn parse_inner(&mut self, buffer: &mut Buffer) -> Result<(State, Again)> {
    match self.state {
      State::Head(head) => {
        let next_head = self.parse_inner_head(buffer, head)?;
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
        let got = buffer.remaining();

        if got == need {
          self.body = Some(buffer.read(need));
          Ok((State::Body(Body::Complete), false))
        } else if got > need {
          self.body = Some(buffer.read(need));
          Err(Error::BodyOverflow {
            need: need,
            got
          })
        } else {
          Ok((State::Body(Body::Incomplete), false))
        }
      },
      State::Body(Body::Complete) => {
        Err(Error::BodyAlreadyDone)
      },
    }
  }

  fn parse_inner_head(
    &mut self,
    buffer: &mut Buffer,
    mut head: Head,
  ) -> Result<Head> {
    while head != Head::Done {
      let line = match buffer.read_line() {
        Some(line) => line.map_err(|_| Error::Encoding)?,
        None => break,
      };

      head = Self::parse_inner_head_line(
        &mut self.metadata,
        &mut self.headers,
        line,
        head)?;
    }

    Ok(head)
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

  fn parse_bytes_and_into(mut self, bytes: Bytes) -> Result<M> {
    let mut buffer = Buffer::from(bytes);
    self.parse(&mut buffer)?;
    self.into()
  }

  fn into(self) -> Result<M> {
    match self.state {
      State::Body(Body::Complete) =>
        Ok(M::new(
          self.metadata
            .ok_or(Error::MetadataNotParsed)?,
          self.headers,
          self.body,
        )),
      _ =>
        Err(Error::NotDone)
    }
  }

}

impl Parser<Request> {

  pub fn parse_bytes_and_into_request(self, bytes: Bytes) -> Result<Request> {
    self.parse_bytes_and_into(bytes)
  }

  pub fn into_request(self) -> Result<Request> {
    self.into()
  }

}

impl Parser<Response> {

  pub fn parse_bytes_and_into_response(self, bytes: Bytes) -> Result<Response> {
    self.parse_bytes_and_into(bytes)
  }

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

    let uri = uri.parse::<Uri>()
      .map_err(|_| Error::UriMalformed {
        line: line.to_string(),
        uri: uri.to_string(),
      })?;

    let uri = if uri.authority().is_some() || uri.path() == "*" {
      Ok(uri)
    } else {
      // Relative URI's are not allowed in RTSP.
      Err(Error::UriNotAbsolute { uri, })
    }?;

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
    Bytes,
    Buffer,
  };

  #[test]
  fn parse_options_request() {
    let request = br###"OPTIONS rtsp://example.com/media.mp4 RTSP/1.0
CSeq: 1
Require: implicit-play
Proxy-Require: gzipped-messages

"###.to_vec();

    let request = RequestParser::new().parse_bytes_and_into_request(request).unwrap();
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

"###.to_vec();

    let request = RequestParser::new().parse_bytes_and_into_request(request).unwrap();
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

"###.to_vec();

    let response = ResponseParser::new().parse_bytes_and_into_response(response).unwrap();
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

"###.to_vec();

    let response = ResponseParser::new().parse_bytes_and_into_response(response).unwrap();
    assert_eq!(response.metadata.version, Version::V1);
    assert_eq!(response.metadata.status, 404);
    assert_eq!(response.metadata.reason, "Stream Not Found");
    assert_eq!(response.headers.get("CSeq"), Some(&"1".to_string()));
  }

  #[test]
  fn parse_describe_request() {
    let request = br###"DESCRIBE rtsp://example.com/media.mp4 RTSP/1.0
CSeq: 2

"###.to_vec();

    let request = RequestParser::new().parse_bytes_and_into_request(request).unwrap();
    assert_eq!(request.metadata.method, Method::Describe);
    assert_eq!(request.metadata.uri, "rtsp://example.com/media.mp4");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"2".to_string()));
  }

  #[test]
  fn parse_describe_request_v2() {
    let request = br###"DESCRIBE rtsp://example.com/media.mp4 RTSP/2.0
CSeq: 2

"###.to_vec();

    let request = RequestParser::new().parse_bytes_and_into_request(request).unwrap();
    assert_eq!(request.metadata.method, Method::Describe);
    assert_eq!(request.metadata.uri, "rtsp://example.com/media.mp4");
    assert_eq!(request.metadata.version, Version::V2);
    assert_eq!(request.headers.get("CSeq"), Some(&"2".to_string()));
  }

  #[test]
  fn parse_describe_request_v3() {
    let request = br###"DESCRIBE rtsp://example.com/media.mp4 RTSP/3.0
CSeq: 2

"###.to_vec();

    let request = RequestParser::new().parse_bytes_and_into_request(request).unwrap();
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
a=StreamName:string;"hinted audio track""###.to_vec();

    let response = ResponseParser::new().parse_bytes_and_into_response(response).unwrap();
    assert_eq!(response.metadata.version, Version::V1);
    assert_eq!(response.metadata.status, 200);
    assert_eq!(response.metadata.reason, "OK");
    assert_eq!(response.headers.get("CSeq"), Some(&"2".to_string()));
    assert_eq!(response.headers.get("Content-Base"), Some(&"rtsp://example.com/media.mp4".to_string()));
    assert_eq!(response.headers.get("Content-Type"), Some(&"application/sdp".to_string()));
    assert_eq!(response.headers.get("Content-Length"), Some(&"443".to_string()));
  }
  
  // TODO TEST PIPELINING

  const EXAMPLE_REQUEST_PLAY_CRLN: &[u8] = b"PLAY rtsp://example.com/stream/0 RTSP/1.0\x0d\x0a\
CSeq: 1\x0d\x0a\
Session: 1234abcd\x0d\x0a\
Content-Length: 16\x0d\x0a\
\x0d\x0a\
0123456789abcdef";

  #[test]
  fn parse_play_request() {
    let request = RequestParser::new()
      .parse_bytes_and_into_request(EXAMPLE_REQUEST_PLAY_CRLN.to_vec())
      .unwrap();
    assert_eq!(request.metadata.method, Method::Play);
    assert_eq!(request.metadata.uri, "rtsp://example.com/stream/0");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"1".to_string()));
    assert_eq!(request.headers.get("Session"), Some(&"1234abcd".to_string()));
    assert_eq!(request.headers.get("Content-Length"), Some(&"16".to_string()));
    assert_eq!(request.body, Some(b"0123456789abcdef".to_vec()));
  }

  #[test]
  fn parse_play_request_partial_piece1_ln() {
    parse_play_request_partial_piece1(&request_play_ln());
  }

  #[test]
  fn parse_play_request_partial_piece2_ln() {
    parse_play_request_partial_piece(&request_play_ln(), 2);
  }

  #[test]
  fn parse_play_request_partial_piece3_ln() {
    parse_play_request_partial_piece(&request_play_ln(), 3);
  }

  #[test]
  fn parse_play_request_partial_piece_varying_ln() {
    parse_play_request_partial_piece_varying(&request_play_ln());
  }

  #[test]
  fn parse_play_request_partial_piece1_cr() {
    parse_play_request_partial_piece1(&request_play_cr());
  }

  #[test]
  fn parse_play_request_partial_piece2_cr() {
    parse_play_request_partial_piece(&request_play_cr(), 2);
  }

  #[test]
  fn parse_play_request_partial_piece3_cr() {
    parse_play_request_partial_piece(&request_play_cr(), 3);
  }

  #[test]
  fn parse_play_request_partial_piece_varying_cr() {
    parse_play_request_partial_piece_varying(&request_play_cr());
  }

  #[test]
  fn parse_play_request_partial_piece1_crln() {
    parse_play_request_partial_piece1(&request_play_crln());
  }

  #[test]
  fn parse_play_request_partial_piece2_crln() {
    parse_play_request_partial_piece(&request_play_crln(), 2);
  }

  #[test]
  fn parse_play_request_partial_piece3_crln() {
    parse_play_request_partial_piece(&request_play_crln(), 3);
  }

  #[test]
  fn parse_play_request_partial_piece_varying_crln() {
    parse_play_request_partial_piece_varying(&request_play_crln());
  }

  fn request_play_ln() -> Bytes {
    EXAMPLE_REQUEST_PLAY_CRLN
      .to_vec()
      .into_iter()
      .filter(|b| *b != b'\x0d')
      .collect::<Bytes>()
  }

  fn request_play_cr() -> Bytes {
    EXAMPLE_REQUEST_PLAY_CRLN
      .to_vec()
      .into_iter()
      .filter(|b| *b != b'\x0a')
      .collect::<Bytes>()
  }

  fn request_play_crln() -> Bytes {
    EXAMPLE_REQUEST_PLAY_CRLN.to_vec()
  }

  fn parse_play_request_partial_piece1(request_bytes: &[u8]) {
    let mut buffer = Buffer::new();
    let mut parser = RequestParser::new();

    let upto_last = request_bytes.len() - 1;
    for i in 0..upto_last {
      let i_range = i..i + 1;
      buffer.extend_from_slice(&request_bytes[i_range]);
      assert_eq!(parser.parse(&mut buffer).unwrap(), Status::Hungry);
    }

    let last_range = request_bytes.len() - 1..;
      buffer.extend_from_slice(&request_bytes[last_range]);
    assert_eq!(parser.parse(&mut buffer).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Play);
    assert_eq!(request.metadata.uri, "rtsp://example.com/stream/0");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"1".to_string()));
    assert_eq!(request.headers.get("Session"), Some(&"1234abcd".to_string()));
    assert_eq!(request.headers.get("Content-Length"), Some(&"16".to_string()));
    assert_eq!(request.body, Some(b"0123456789abcdef".to_vec()));
  }
  
  fn parse_play_request_partial_piece(request_bytes: &[u8], piece_size: usize) {
    let mut buffer = Buffer::new();
    let mut parser = RequestParser::new();

    let pieces_upto_last = (request_bytes.len() / piece_size) - 1;
    for i in 0..pieces_upto_last {
      let piece_range = (i * piece_size)..(i * piece_size) + piece_size;
      buffer.extend_from_slice(&request_bytes[piece_range]);
      assert_eq!(parser.parse(&mut buffer).unwrap(), Status::Hungry);
    }

    let last_piece = pieces_upto_last;
    let leftover_piece_range = last_piece * piece_size..;
    buffer.extend_from_slice(&request_bytes[leftover_piece_range]);
    assert_eq!(parser.parse(&mut buffer).unwrap(), Status::Done);

    let request = parser.into_request().unwrap();
    assert_eq!(request.metadata.method, Method::Play);
    assert_eq!(request.metadata.uri, "rtsp://example.com/stream/0");
    assert_eq!(request.metadata.version, Version::V1);
    assert_eq!(request.headers.get("CSeq"), Some(&"1".to_string()));
    assert_eq!(request.headers.get("Session"), Some(&"1234abcd".to_string()));
    assert_eq!(request.headers.get("Content-Length"), Some(&"16".to_string()));
    assert_eq!(request.body, Some(b"0123456789abcdef".to_vec()));
  }

  fn parse_play_request_partial_piece_varying(request_bytes: &[u8]) {
    let mut buffer = Buffer::new();
    let mut parser = RequestParser::new();

    let mut start = 0;
    let mut size = 1;
    loop {
      let piece_range = start..(start + size).min(request_bytes.len());
      buffer.extend_from_slice(&request_bytes[piece_range]);
      if let Status::Done = parser.parse(&mut buffer).unwrap() {
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
    assert_eq!(request.body, Some(b"0123456789abcdef".to_vec()));
  }

}