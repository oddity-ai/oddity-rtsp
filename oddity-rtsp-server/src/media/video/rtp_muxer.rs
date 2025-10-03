//! Async wrapper functions for [`video_rs::RtpMuxer`].

use tokio::task;

use video_rs as video;
use video_rs::rtp::RtpMuxer as BlockingRtpMuxer;
pub use video_rs::rtp::RtpMuxerBuilder;

type Result<T> = std::result::Result<T, video::Error>;

pub async fn make_rtp_muxer_builder() -> Result<RtpMuxerBuilder> {
    task::spawn_blocking(RtpMuxerBuilder::new).await.unwrap()
}

pub struct RtpMuxer {
    inner: BlockingRtpMuxer,
    sps_packet_annex_b: Option<Vec<u8>>,
    pps_packet_annex_b: Option<Vec<u8>>,
    first_packet_sent: bool,
}

impl RtpMuxer {
    pub fn from_builder(rtp_muxer_builder: RtpMuxerBuilder) -> Result<RtpMuxer> {
        let blocking_muxer = rtp_muxer_builder.build();

        let mut parameter_sets = blocking_muxer.parameter_sets_h264();
        let (sps_packet, pps_packet) = if !parameter_sets.is_empty() {
            let (sps, ppss) = parameter_sets.remove(0)?;

            let mut sps_packet = vec![0, 0, 0, 1]; // annex b start code
            sps_packet.extend_from_slice(sps);

            let mut pps_packet = Vec::new();
            for pps in ppss {
                pps_packet.extend_from_slice(&[0, 0, 0, 1]);
                pps_packet.extend_from_slice(pps);
            }

            (Some(sps_packet), Some(pps_packet))
        } else {
            (None, None)
        };

        Ok(RtpMuxer {
            inner: blocking_muxer,
            sps_packet_annex_b: sps_packet,
            pps_packet_annex_b: pps_packet,
            first_packet_sent: false,
        })
    }

    #[inline]
    pub fn seq_and_timestamp(&self) -> (u16, u32) {
        self.inner.seq_and_timestamp()
    }

    pub async fn muxed(
        mut self,
        packet: video::Packet,
    ) -> (RtpMuxer, Result<Vec<video::rtp::RtpBuf>>) {
        task::spawn_blocking(move || {
            let packet = if (packet.is_key() || !self.first_packet_sent)
                && (self.sps_packet_annex_b.is_some() || self.pps_packet_annex_b.is_some())
            {
                let old_dts = packet.dts();
                let old_pts = packet.pts();
                let (old_packet, time_base) = packet.into_inner_parts();
                let mut new_packet_data = Vec::new();
                if let Some(sps_packet_data) = &self.sps_packet_annex_b {
                    new_packet_data.extend_from_slice(sps_packet_data);
                }
                if let Some(pps_packet_data) = &self.pps_packet_annex_b {
                    new_packet_data.extend_from_slice(pps_packet_data);
                }
                new_packet_data.extend_from_slice(&[0, 0, 0, 1]);
                new_packet_data.extend_from_slice(&old_packet.data().unwrap()[4..]);
                let mut new_packet =
                    video::Packet::new(video::ffmpeg::Packet::copy(&new_packet_data), time_base);
                new_packet.set_dts(old_dts);
                new_packet.set_pts(old_pts);
                new_packet
            } else {
                packet
            };
            let out = self.inner.mux(packet);
            self.first_packet_sent = true;
            (self, out)
        })
        .await
        .unwrap()
    }

    pub async fn finish(mut self) -> Result<Option<Vec<video::rtp::RtpBuf>>> {
        task::spawn_blocking(move || self.inner.finish())
            .await
            .unwrap()
    }
}
