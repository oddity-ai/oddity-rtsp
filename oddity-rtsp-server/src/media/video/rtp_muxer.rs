//! Async wrapper functions for [`oddity_video::RtpMuxer`].

use std::net;

use tokio::task;

use oddity_video::{self as video, RtpMuxer};

type Result<T> = std::result::Result<T, video::Error>;

pub async fn make_rtp_muxer(
  dest: net::Ip
) -> Result<RtpMuxer> {
  // TODO transport client_addr and port
  task::spawn_blocking(move || {
      RtpMuxer::new(

      )
    })
    .await
    .unwrap()
}

pub async fn muxed(
  mut rtp_muxer: RtpMuxer,
  packet: video::Packet,
) -> (RtpMuxer, Result<video::RtpBuf>) {
  task::spawn_blocking(move || {
      let out = rtp_muxer.mux(packet);
      (rtp_muxer, out)
    })
    .await
    .unwrap()
}

pub async fn finish(
  mut rtp_muxer: RtpMuxer,
) -> Result<video::RtpBuf> {
  task::spawn_blocking(move || {
      rtp_muxer.finish()
    })
    .await
    .unwrap()
}