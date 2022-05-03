pub use super::{
  Tag,
  fmt::FMT_RTP_PAYLOAD_DYNAMIC,
};

pub trait MediaAttributes {

  fn media_attributes(&self) -> Vec<Tag>;

}

pub enum CodecInfo {
  H264 {
    sps: Vec<u8>,
    pps: Vec<Vec<u8>>,
  },
}

impl MediaAttributes for CodecInfo {

  fn media_attributes(&self) -> Vec<Tag> {
    match self {
      CodecInfo::H264 {
        sps,
        pps,
      } => vec![
        Tag::Value("rtpmap".to_string(), format!("{} H264/90000", FMT_RTP_PAYLOAD_DYNAMIC)),
        Tag::Value("fmtp".to_string(), format!("{} packetization-mode=1, sprop-parameter-sets={}", FMT_RTP_PAYLOAD_DYNAMIC, ...)),
      ],
    }
  }

}