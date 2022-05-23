use oddity_rtsp_protocol::{
  Transport,
  Parameter,
  Port,
  Channel,
  Lower,
  Method,
};

use oddity_video::RtpMuxer;

use crate::{
  net::WriterTx,
  media::Error,
};

use super::context::{
  Context,
  Destination,
  UdpDestination,
  TcpInterleavedDestination,
};

pub fn make_context_from_transport(
  candidate_transports: impl IntoIterator<Item=Transport>,
  writer_tx: WriterTx,
) -> Result<Context, Error> {
  let transport = candidate_transports
    .into_iter()
    .filter(|transport| is_supported(&transport))
    .next()
    .ok_or_else(|| Error::TransportNotSupported)?;
  
  RtpMuxer::new()
    .map_err(Error::Media)
    .and_then(|muxer| {
      let transport = resolve_transport(&transport, &muxer);
      let dest = resolve_destination(&transport, writer_tx)
        .ok_or_else(|| Error::DestinationInvalid)?;

      Ok(Context {
        muxer,
        transport,
        dest,
      })
    })
}

fn resolve_transport(
  transport: &Transport,
  rtp_muxer: &RtpMuxer,
) -> Transport {
  let (rtp_port, rtcp_port) = rtp_muxer.local_ports();

  transport
    .clone()
    .with_parameter(
      Parameter::ServerPort(
        Port::Range(
          rtp_port,
          rtcp_port,
        )
      )
    )
}

fn resolve_destination(
  transport: &Transport,
  writer_tx: WriterTx,
) -> Option<Destination> {
  Some(
    match transport.lower_protocol()? {
      Lower::Udp => {
        let client_ip_addr = transport.destination()?;
        let (client_rtp_port, client_rtcp_port) =
          match transport.client_port()? {
            Port::Single(rtp_port)
              => (*rtp_port, rtp_port + 1),
            Port::Range(rtp_port, rtcp_port)
              => (*rtp_port, *rtcp_port),
          };

        Destination::Udp(
          UdpDestination {
            rtp_remote: (*client_ip_addr, client_rtp_port).into(),
            rtcp_remote: (*client_ip_addr, client_rtcp_port).into(),
          }
        )
      },
      Lower::Tcp => {
        let (rtp_channel, rtcp_channel) =
          match transport.interleaved_channel()? {
            Channel::Single(rtp_channel)
              => (*rtp_channel, rtp_channel + 1),
            Channel::Range(rtp_channel, rtcp_channel)
              => (*rtp_channel, *rtcp_channel),
          };

        Destination::TcpInterleaved(
          TcpInterleavedDestination {
            rtp_channel,
            rtcp_channel,
            tx: writer_tx,
          }
        )
      },
    }
  )
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
    Parameter::Unicast            => true,
    Parameter::Multicast          => false, // Multicast not supported
    Parameter::Destination(_)     => true,
    Parameter::Interleaved(_)     => true,
    Parameter::Append             => false, // RECORD not supported
    Parameter::Ttl(_)             => true,
    Parameter::Layers(_)          => false, // Multicast not supported
    Parameter::Port(_)            => false, // Multicast not supported
    Parameter::ClientPort(_)      => true,
    Parameter::ServerPort(_)      => false, // Client cannot choose server ports
    Parameter::Ssrc(_)            => false, // Client cannot choose ssrc
    Parameter::Mode(Method::Play) => true,
    Parameter::Mode(_)            => false, // Only PLAY is supported for session.
  }
}