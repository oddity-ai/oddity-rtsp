use std::error;
use std::fmt;

use oddity_video::Error as MediaError;

#[derive(Debug, Clone)]
pub enum Error {
  CodecNotSupported,
  Media(MediaError),
}

impl fmt::Display for Error {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::CodecNotSupported => write!(f, "codec not supported"),
      Error::Media(err) => write!(f, "{}", err),
    }
  }

}

impl error::Error for Error {}