use std::fmt;
use std::error;
use std::io;
use std::convert;

use super::message::Uri;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  /// An error occurred decoding the header due to incorrect usage
  /// of text encoding by the sender.
  Encoding,
  /// The request line of the head part is malformed.
  RequestLineMalformed {
    line: String
  },
  /// The header first line does have a method and target URI, but
  /// it does not have a version, which is the required third part
  /// of the first line of the head.
  VersionMissing {
    line: String
  },
  /// The response status line does have a version, but does not have
  /// a status code which is required.
  StatusCodeMissing {
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
  /// The header first line has a Request-URI, but it could not be
  /// parsed correctly.
  UriMalformed {
    line: String,
    uri: String,
  },
  /// The Request-URI is correct, but represents a relative path,
  /// which is not allowed in RTSP.
  UriNotAbsolute {
    uri: Uri,
  },
  /// The response status line has a version and status code, but is
  /// missing a reason phrase which is required.
  ReasonPhraseMissing {
    line: String,
  },
  /// The version specifier is incorrect. It should start with "RTSP/"
  /// followed by a digit, "." and another digit.
  VersionMalformed {
    line: String,
    version: String
  },
  /// The provided status code is not an unsigned integer or cannot be
  /// converted to one. It must be a 3-digit non-negative number.
  StatusCodeNotInteger {
    line: String,
    status_code: String
  },
  /// Header line is malformed.
  HeaderMalformed {
    line: String,
  },
  /// The Content-Length header is missing, but it is required.
  ContentLengthMissing,
  /// The Content-Length header is not an integer value, or cannot be
  /// converted to an unsigned integer.
  ContentLengthNotInteger {
    value: String,
  },
  /// This occurs when the caller invokes the state machine with a
  /// state that signals that parsing the head part of the request
  /// was already done before.
  HeadAlreadyDone,
  /// This occurs when the caller invokes the state machine with a
  /// state that signals that parsing the body part of the request
  /// was already done before.
  BodyAlreadyDone,
  /// Metadata was not parsed for some reason.
  MetadataNotParsed,
  /// This occurs when the caller tries to turn the parser into an
  /// actual request, but the parser was not ready yet.
  NotDone,
  /// This occurs when trying to serialize a request that does not
  /// have a known version.
  VersionUnknown,
  /// I/O error occurred.
  Io(io::Error),
}

impl fmt::Display for Error {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::Encoding =>
        write!(f, "encoding incorrect"),
      Error::RequestLineMalformed { line, } =>
        write!(f, "request line malformed: {}", &line),
      Error::VersionMissing { line, } =>
        write!(f, "version missing in request line: {}", &line),
      Error::StatusCodeMissing { line, } =>
        write!(f, "status code missing in response line: {}", &line),
      Error::MethodUnknown { line, method, } =>
        write!(f, "method unknown: {} (in request line: {})", &method, &line),
      Error::UriMissing { line, } =>
        write!(f, "uri missing in request line: {}", &line),
      Error::UriMalformed { line, uri, } =>
        write!(f, "uri malformed: {} (in line: {})", &uri, &line),
      Error::UriNotAbsolute { uri, } =>
        write!(f, "uri must be absolute, but it is relative: {}", &uri),
      Error::ReasonPhraseMissing { line, } =>
        write!(f, "reason phrase missing in response line: {}", &line),
      Error::VersionMalformed { line, version, } =>
        write!(f, "version malformed: {} (in line: {})", &version, &line),
      Error::StatusCodeNotInteger { line, status_code } =>
        write!(f, "response has invalid status code: {} (in response line: {})", &status_code, &line),
      Error::HeaderMalformed { line, } =>
        write!(f, "header line malformed: {}", &line),
      Error::ContentLengthMissing =>
        write!(f, "request does not have Content-Length header"),
      Error::ContentLengthNotInteger { value, } =>
        write!(f, "request has invalid value for Content-Length: {}", &value),
      Error::HeadAlreadyDone =>
        write!(f, "head already done (cycle in state machine)"),
      Error::BodyAlreadyDone =>
        write!(f, "body already done (cycle in state machine)"),
      Error::MetadataNotParsed =>
        write!(f, "metadata not parsed"),
      Error::NotDone =>
        write!(f, "parser not done yet"),
      Error::VersionUnknown =>
        write!(f, "response has unknown version"),
      Error::Io(err) =>
        write!(f, "{}", err),
    }
  }

}

impl convert::From<io::Error> for Error {

  fn from(error: io::Error) -> Self {
    Error::Io(error)
  }

}

impl error::Error for Error {}