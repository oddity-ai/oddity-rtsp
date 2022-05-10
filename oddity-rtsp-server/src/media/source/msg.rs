use std::fmt;

use oddity_video::{
  StreamInfo,
  Packet,
};

/// Message sent between producer worker and subscribers.
#[derive(Clone)]
pub enum Message {
  /// Subscriber should reinitialize stream with the given properties.
  Init(StreamInfo),
  /// Subscriber should handle media packet.
  Packet(Packet),
}

impl fmt::Display for Message {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Message::Init(_) => write!(f, "init"),
      Message::Packet(_) => write!(f, "packet"),
    }
  }

}
