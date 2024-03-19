//! Async wrapper functions for [`video_rs::RtpMuxer`].

use tokio::task;

use video_rs as video;
use video_rs::rtp::{RtpMuxer, RtpMuxerBuilder};

type Result<T> = std::result::Result<T, video::Error>;

pub async fn make_rtp_muxer_builder() -> Result<RtpMuxerBuilder> {
    task::spawn_blocking(RtpMuxerBuilder::new).await.unwrap()
}

pub async fn muxed(
    mut rtp_muxer: RtpMuxer,
    packet: video::Packet,
) -> (RtpMuxer, Result<Vec<video::rtp::RtpBuf>>) {
    task::spawn_blocking(move || {
        let out = rtp_muxer.mux(packet);
        (rtp_muxer, out)
    })
    .await
    .unwrap()
}

pub async fn finish(mut rtp_muxer: RtpMuxer) -> Result<Option<Vec<video::rtp::RtpBuf>>> {
    task::spawn_blocking(move || rtp_muxer.finish())
        .await
        .unwrap()
}
