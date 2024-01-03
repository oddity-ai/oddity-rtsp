use std::fmt::Write;

use base64::engine::Engine;

pub use super::{fmt::FMT_RTP_PAYLOAD_DYNAMIC, Tag};

pub trait MediaAttributes {
    fn media_attributes(&self) -> Vec<Tag>;
}

pub enum CodecInfo<'params> {
    H264(H264CodecParameters<'params>),
}

impl<'params> CodecInfo<'params> {
    pub fn h264(
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
        format!("{} H264/90000", FMT_RTP_PAYLOAD_DYNAMIC),
    )
}

fn h264_fmtp(packetization_mode: usize, sps: &[u8], pps: &[&[u8]]) -> Tag {
    let profile_level_id_bytes = &sps[1..4];
    let profile_level_id = profile_level_id_bytes
        .iter()
        .fold(String::new(), |mut output, b| {
            let _ = write!(output, "{b:02X}");
            output
        });

    let mut parameter_sets = Vec::with_capacity(1 + pps.len());
    parameter_sets.push(base64::engine::general_purpose::STANDARD.encode(sps));
    parameter_sets.extend(
        pps.iter()
            .map(|pps| base64::engine::general_purpose::STANDARD.encode(pps)),
    );
    let sprop_parameter_sets = parameter_sets.join(",");

    Tag::Value(
        "fmtp".to_string(),
        format!(
            "{} packetization-mode={}; profile-level-id={}; sprop-parameter-sets={}",
            FMT_RTP_PAYLOAD_DYNAMIC, packetization_mode, profile_level_id, sprop_parameter_sets,
        ),
    )
}
