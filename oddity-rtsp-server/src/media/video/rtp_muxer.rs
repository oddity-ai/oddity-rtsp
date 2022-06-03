//! Async wrapper functions for [`oddity_video::RtpMuxer`].

use tokio::task;

use oddity_video::{self as video, RtpMuxer};

type Result<T> = std::result::Result<T, video::Error>;

pub async fn make_rtp_muxer() -> Result<RtpMuxer> {
  task::spawn_blocking(move || {
      RtpMuxer::new()
    })
    .await
    .unwrap()
}

pub async fn mux(
  rtp_muxer: &'static mut RtpMuxer,
  packet: video::Packet,
) -> Result<video::RtpBuf> {
  task::spawn_blocking(move || {
      rtp_muxer.mux(packet)
    })
    .await
    .unwrap()
}

pub async fn finish(
  rtp_muxer: &'static mut RtpMuxer,
) -> Result<video::RtpBuf> {
  task::spawn_blocking(move || {
      rtp_muxer.finish()
    })
    .await
    .unwrap()
}