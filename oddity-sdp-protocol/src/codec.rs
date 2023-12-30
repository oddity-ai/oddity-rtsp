use std::fmt::Write;

pub use super::{fmt::FMT_RTP_PAYLOAD_DYNAMIC, Tag};

pub trait MediaAttributes {
    fn media_attributes(&self) -> Vec<Tag>;
}

pub enum CodecInfo<'params> {
    H264(H264CodecParameters<'params>),
}

impl<'params> CodecInfo<'params> {
    #[allow(clippy::similar_names)]
    #[must_use]
    pub const fn h264(
        sps: &'params [u8],
        pps: &'params [&'params [u8]],
        packetization_mode: usize,
    ) -> Self {
        Self::H264(H264CodecParameters {
            sps,
            pps,
            packetization_mode,
        })
    }
}

pub struct H264CodecParameters<'params> {
    sps: &'params [u8],
    pps: &'params [&'params [u8]],
    packetization_mode: usize,
}

impl MediaAttributes for CodecInfo<'_> {
    fn media_attributes(&self) -> Vec<Tag> {
        match self {
            CodecInfo::H264(params) => vec![
                h264_rtpmap(),
                h264_fmtp(params.packetization_mode, params.sps, params.pps),
            ],
        }
    }
}

fn h264_rtpmap() -> Tag {
    Tag::Value(
        "rtpmap".to_string(),
        format!("{FMT_RTP_PAYLOAD_DYNAMIC} H264/90000"),
    )
}

#[allow(clippy::similar_names)]
fn h264_fmtp(packetization_mode: usize, sps: &[u8], pps: &[&[u8]]) -> Tag {
    let profile_level_id_bytes = &sps[1..4];
    let profile_level_id = profile_level_id_bytes
        .iter()
        .fold(String::new(), |mut output, b| {
            let _ = write!(output, "{b:02X}");
            output
        });

    let mut parameter_sets = Vec::with_capacity(1 + pps.len());
    parameter_sets.push(base64::encode(sps));
    parameter_sets.extend(pps.iter().map(base64::encode));
    let sprop_parameter_sets = parameter_sets.join(",");

    Tag::Value(
        "fmtp".to_string(),
        format!(
            "{FMT_RTP_PAYLOAD_DYNAMIC} packetization-mode={packetization_mode}; profile-level-id={profile_level_id}; sprop-parameter-sets={sprop_parameter_sets}",
        ),
    )
}
