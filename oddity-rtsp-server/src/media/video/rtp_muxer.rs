//! Async wrapper functions for [`video_rs::RtpMuxer`].

use tokio::task;

use video_rs::{self as video, RtpMuxer};

type Result<T> = std::result::Result<T, video::Error>;

pub async fn make_rtp_muxer() -> Result<RtpMuxer> {
    task::spawn_blocking(RtpMuxer::new).await.unwrap()
}

pub async fn muxed(
    mut rtp_muxer: RtpMuxer,
    packet: video::Packet,
) -> (RtpMuxer, Result<Vec<video::RtpBuf>>) {
    task::spawn_blocking(move || {
        let out = rtp_muxer.mux(packet);
        (rtp_muxer, out)
    })
    .await
    .unwrap()
}

pub async fn finish(mut rtp_muxer: RtpMuxer) -> Result<Option<Vec<video::RtpBuf>>> {
    task::spawn_blocking(move || rtp_muxer.finish())
        .await
        .unwrap()
}
