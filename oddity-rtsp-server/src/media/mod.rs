mod controller;
mod descriptor;
mod sdp;
mod error;

pub mod source;
pub mod session;

pub use controller::{
  Controller as MediaController,
  RegisterSessionError,
};
pub use error::{
  Error,
  VideoError,
};
pub use descriptor::Descriptor;

pub type SharedMediaController =
  std::sync::Arc<std::sync::Mutex<MediaController>>;