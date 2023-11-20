use oddity_rtsp_protocol as rtsp;

pub fn resolve_transport(rtsp_transport: &rtsp::Transport) -> rtsp::Transport {
    if rtsp_transport.interleaved_channel().is_some() {
        rtsp_transport.clone()
    } else {
        // Use default channels 0 and 1 if client did not specify preferred
        // interleaved channels.
        rtsp_transport
            .clone()
            .with_parameter(rtsp::Parameter::Interleaved(rtsp::Channel::Range(0, 1)))
    }
}

pub fn is_supported(transport: &rtsp::Transport) -> bool {
    return transport
        .lower_protocol()
        .map(is_lower_protocol_supported)
        .unwrap_or(true)
        && transport.parameters_iter().all(is_parameter_supported);
}

fn is_lower_protocol_supported(lower: &rtsp::Lower) -> bool {
    match lower {
        rtsp::Lower::Udp => false,
        rtsp::Lower::Tcp => true,
    }
}

fn is_parameter_supported(parameter: &rtsp::Parameter) -> bool {
    /*
      Supported parameters are:
      - `unicast`
      - `interleaved`
      - `mode` (if value is "PLAY")
    */
    match parameter {
        rtsp::Parameter::Unicast => true,
        rtsp::Parameter::Multicast => false, // Multicast not supported
        rtsp::Parameter::Destination(_) => false, // UDP not supported
        rtsp::Parameter::Interleaved(_) => true,
        rtsp::Parameter::Append => false,    // RECORD not supported
        rtsp::Parameter::Ttl(_) => false,    // Multicast not supported
        rtsp::Parameter::Layers(_) => false, // Multicast not supported
        rtsp::Parameter::Port(_) => false,   // Multicast not supported
        rtsp::Parameter::ClientPort(_) => false, // UDP not supported
        rtsp::Parameter::ServerPort(_) => false, // Client cannot choose server ports
        rtsp::Parameter::Ssrc(_) => false,   // Client cannot choose ssrc
        rtsp::Parameter::Mode(rtsp::Method::Play) => true,
        rtsp::Parameter::Mode(_) => false, // Only PLAY is supported for session.
    }
}
