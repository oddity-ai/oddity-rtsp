use oddity_rtsp_protocol as rtsp;
use oddity_video as video;

pub fn resolve_transport(
  rtsp_transport: &rtsp::Transport,
  rtp_muxer: &video::RtpMuxer,
) -> rtsp::Transport {
  let (rtp_port, rtcp_port) = rtp_muxer.local_ports(); // TODO! segfault

  rtsp_transport
    .clone()
    .with_parameter(
      rtsp::Parameter::ServerPort(
        rtsp::Port::Range(
          rtp_port,
          rtcp_port,
        )
      )
    )
}

pub fn is_supported(
  transport: &rtsp::Transport,
) -> bool {
  return
    transport
      .lower_protocol()
      .map(|lower| is_lower_protocol_supported(lower))
      .unwrap_or(true) &&
    transport
      .parameters_iter()
      .all(|parameter| is_parameter_supported(parameter))
}

fn is_lower_protocol_supported(
  lower: &rtsp::Lower,
) -> bool {
  match lower {
    rtsp::Lower::Udp => true,
    rtsp::Lower::Tcp => true,
  }
}

fn is_parameter_supported(
  parameter: &rtsp::Parameter,
) -> bool {
  /*
    Supported parameters are:
    - `unicast`
    - `destination`
    - `interleaved`
    - `ttl`
    - `client_port`
    - `mode` (if value is "PLAY")
  */
  match parameter {
    rtsp::Parameter::Unicast                  => true,
    rtsp::Parameter::Multicast                => false, // Multicast not supported
    rtsp::Parameter::Destination(_)           => true,
    rtsp::Parameter::Interleaved(_)           => true,
    rtsp::Parameter::Append                   => false, // RECORD not supported
    rtsp::Parameter::Ttl(_)                   => true,
    rtsp::Parameter::Layers(_)                => false, // Multicast not supported
    rtsp::Parameter::Port(_)                  => false, // Multicast not supported
    rtsp::Parameter::ClientPort(_)            => true,
    rtsp::Parameter::ServerPort(_)            => false, // Client cannot choose server ports
    rtsp::Parameter::Ssrc(_)                  => false, // Client cannot choose ssrc
    rtsp::Parameter::Mode(rtsp::Method::Play) => true,
    rtsp::Parameter::Mode(_)                  => false, // Only PLAY is supported for session.
  }
}