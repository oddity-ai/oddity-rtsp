use oddity_video::{Reader, RtpMuxer};
use oddity_sdp_protocol::{
  TimeRange,
  Kind,
  Protocol,
  CodecInfo,
};

use crate::media::MediaDescriptor;

pub use oddity_sdp_protocol::Sdp;

/// Create a new SDP description for the given media descriptor. The
/// SDP contents can be used over RTSP when the client requested a
/// stream description.
/// 
/// Note: This function only handles the most appropriate video stream
/// and tosses any audio or other streams.
/// 
/// # Arguments
/// 
/// * `name` - Name of stream.
/// * `descriptor` - Media stream descriptor.
pub async fn create(
  name: String,
  descriptor: &MediaDescriptor,
) -> Result<Sdp, SdpError> {
  // TODO SPAWN_BLOCKING for BLOCKING PARTS !!!!!!!!!!!!
  const ORIGIN_DUMMY_HOST: [u8; 4] = [0, 0, 0, 0];
  const TARGET_DUMMY_HOST: [u8; 4] = [0, 0, 0, 0];
  const TARGET_DUMMY_PORT: u16 = 0;

  let reader = Reader::new(&descriptor.clone().into())
    .map_err(SdpError::Media)?;
  let best_video_stream = reader.best_video_stream_index()
    .map_err(SdpError::Media)?;

  let time_range = match descriptor {
    MediaDescriptor::File(_)
      => unimplemented!() /* TODO */,
    MediaDescriptor::Stream(_)
      => TimeRange::Live,
  };

  let muxer = RtpMuxer::new()
    .and_then(|muxer|
      muxer.with_stream(reader.stream_info(best_video_stream)?))
    .map_err(SdpError::Media)?;

  let (sps, pps) = muxer
    .parameter_sets_h264()
    .into_iter()
    // The `parameter_sets` function will return an error if the
    // underlying stream codec is not supported, we filter out
    // the stream in that case, and return `CodecNotSupported`.
    .filter_map(Result::ok)
    .next()
    .ok_or_else(|| SdpError::CodecNotSupported)?;

  // Since the previous call to `parameter_sets_h264` can only
  // return a result if the underlying stream is H.264, we can
  // assume H.264 from this point onwards.
  let codec_info = CodecInfo::h264(
    sps,
    pps.as_slice(),
    muxer.packetization_mode(),
  );

  let sdp = Sdp::new(
    ORIGIN_DUMMY_HOST.into(),
    name,
    TARGET_DUMMY_HOST.into(),
    time_range
  );

  let sdp = sdp
    .with_media(
      Kind::Video,
      TARGET_DUMMY_PORT,
      Protocol::RtpAvp,
      codec_info,
    );

  Ok(sdp)
}

pub enum SdpError {
  CodecNotSupported,
  Media(oddity_video::Error),
}