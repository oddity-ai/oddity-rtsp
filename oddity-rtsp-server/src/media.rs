mod source;
mod session;
mod controller;
mod descriptor;
mod sdp;
mod error;

pub use controller::{
  Controller as MediaController,
  RegisterSessionError,
};
pub use source::{
  Source,
  Rx as SourceRx,
  Message as SourceMsg,
};
pub use session::{
  SessionId,
  Session,
};
pub use descriptor::Descriptor;
pub use error::{
  Error,
  VideoError,
};

pub type SharedMediaController =
  std::sync::Arc<std::sync::Mutex<MediaController>>;