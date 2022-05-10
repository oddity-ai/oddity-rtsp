use oddity_video::{
  Reader,
  RtpMuxer,
};

use oddity_sdp_protocol::{
  Sdp,
  TimeRange,
  Kind,
  Protocol,
  CodecInfo,
};

use super::{
  Descriptor,
  Error,
};

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
pub fn create(
  name: String,
  descriptor: &Descriptor,
) -> Result<Sdp, Error> {
  const ORIGIN_DUMMY_HOST: [u8; 4] = [0, 0, 0, 0];
  const TARGET_DUMMY_HOST: [u8; 4] = [0, 0, 0, 0];
  const TARGET_DUMMY_URL: &str = "rtp://0.0.0.0";
  const TARGET_DUMMY_PORT: u16 = 0;

  let reader = Reader::new(&descriptor.clone().into())
    .map_err(Error::Media)?;
  let best_video_stream = reader.best_video_stream_index()
    .map_err(Error::Media)?;

  let time_range = match descriptor {
    Descriptor::File(_)
      => unimplemented!() /* TODO */,
    Descriptor::Stream(_)
      => TimeRange::Live,
  };

  let muxer = RtpMuxer::new(TARGET_DUMMY_URL.parse().unwrap())
    .and_then(|muxer|
      muxer.with_stream(reader.stream_info(best_video_stream)?))
    .map_err(Error::Media)?;

  let (sps, pps) = muxer
    .parameter_sets_h264()
    .into_iter()
    // The `parameter_sets` function will return an error if the
    // underlying stream codec is not supported, we filter out
    // the stream in that case, and return `CodecNotSupported`.
    .filter_map(Result::ok)
    .next()
    .ok_or_else(|| Error::CodecNotSupported)?;

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