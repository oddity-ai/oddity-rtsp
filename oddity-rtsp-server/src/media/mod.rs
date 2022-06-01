use oddity_video::StreamInfo;

pub use oddity_video::Packet;

#[derive(Clone)]
pub struct MediaInfo {
  streams: Vec<StreamInfo>,
}