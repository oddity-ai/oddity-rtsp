use oddity_rtsp_protocol::{
  Transport,
  Parameter,
  Lower,
};

pub struct Connect {
  socket: Socket,
  transport: Transport,
}

pub fn determine_transport(
  constraints: impl IntoIterator<Item=Transport>,
) -> Option<Transport> {
  constraints
    .into_iter()
    .filter_map(|constraint| {
      if is_supported(&constraint) {
        Some(
          
        )
      } else {
        None
      }
    })
    .next()
}

fn resolve() -> Transport {
  
}

fn is_supported(
  transport: &Transport,
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
  lower: &Lower,
) -> bool {
  match lower {
    Lower::Udp => true,
    Lower::Tcp => true,
  }
}

fn is_parameter_supported(
  parameter: &Parameter,
) -> bool {
  match parameter {
    Parameter::Unicast        => true,
    Parameter::Multicast      => false,
    Parameter::Destination(_) => true,
    Parameter::Interleaved(_) => false,
    Parameter::Append         => false,
    Parameter::Ttl(_)         => true,
    Parameter::Layers(_)      => false,
    Parameter::Port(_)        => true,
    Parameter::ClientPort(_)  => true,
    Parameter::ServerPort(_)  => false,
    Parameter::Ssrc(_)        => false,
    Parameter::Mode(_)        => false,
  }
}