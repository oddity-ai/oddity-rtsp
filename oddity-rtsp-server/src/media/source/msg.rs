use std::fmt;

use oddity_video::{
  StreamInfo,
  Packet,
};

/// Message sent between producer service and subscribers.
#[derive(Clone)]
pub enum Msg {
  /// Subscriber should reinitialize stream with the given properties.
  Init(StreamInfo),
  /// Subscriber should handle media packet.
  Packet(Packet),
}

impl fmt::Display for Msg {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Msg::Init(_) => write!(f, "init"),
      Msg::Packet(_) => write!(f, "packet"),
    }
  }

}