use std::fmt;

use oddity_rtsp_protocol::Uri;

pub struct Multiplexer {
  /// Source URI from which to read and multiplex stream.
  uri: Uri,
}

impl Multiplexer {

  pub fn new(uri: Uri) -> Multiplexer {
    Self {
      uri,
    }
  }

  // TODO Some thread management. The internal thread actually
  //   drives the media object. Communication for play, pause etc. over
  //   a channel.

  // TODO It would be great if we could make the internal media
  //   pull stop automatically if there are no sinks.

  // TODO Implement
  // pub fn add_sink();

  // TODO Implement
  // pub fn remove_sink();

  // TODO Implement
  // fn worker();

}

impl fmt::Display for Multiplexer {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "multiplex from source: {}", self.uri)
  }

}