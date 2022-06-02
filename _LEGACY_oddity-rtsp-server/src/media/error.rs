use std::error;
use std::fmt;

pub use oddity_video::Error as VideoError;

#[derive(Debug, Clone)]
pub enum Error {
  CodecNotSupported,
  TransportNotSupported,
  DestinationInvalid,
  Media(VideoError),
}

impl fmt::Display for Error {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::CodecNotSupported
        => write!(f, "codec not supported"),
      Error::TransportNotSupported
        => write!(f, "transport not mapsupported"),
      Error::DestinationInvalid
        => write!(f, "failed to extract destination from transport information"),
      Error::Media(err)
        => write!(f, "{}", err),
    }
  }

}

impl error::Error for Error {}