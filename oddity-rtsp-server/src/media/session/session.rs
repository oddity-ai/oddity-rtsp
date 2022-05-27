use std::net::UdpSocket;

use concurrency::{
  Service,
  StopRx,
};

use oddity_rtsp_protocol::ResponseMaybeInterleaved;

use oddity_video::{
  RtpMuxer,
  RtpBuf,
  StreamInfo,
};

use crate::media::{
  source::{
    Source,
    Rx as SourceRx,
  },
  Error,
};

use super::context::{
  Context,
  Destination,
  UdpDestination,
  TcpInterleavedDestination,
};

pub struct Session {
  service: Option<Service>,
}

impl Session {

  pub fn new(
    source: &mut Source,
    context: Context,
  ) -> Result<Self, Error> {
    let service = Service::spawn({
      let (source_rx, source_stream_info) = source.subscribe()?;
      move |stop| {
        match context.dest {
          Destination::Udp(dest) => {
            Self::run_udp(
              source_stream_info,
              source_rx,
              context.muxer,
              dest,
              stop,
            )
          },
          Destination::TcpInterleaved(dest) => {
            Self::run_tcp_interleaved(
              source_stream_info,
              source_rx,
              context.muxer,
              dest,
              stop,
            )
          }
        }
      }
    });

    Ok(
      Self {
        service: Some(service),
      }
    )
  }

  pub fn play() {
    // TODO
  }

  // TODO drop() = teardown (?)
  pub fn teardown(self) {
    
  }

  // TODO refactor
  fn run_udp(
    stream_info: StreamInfo,
    source_rx: SourceRx,
    mut muxer: RtpMuxer,
    dest: UdpDestination,
    stop: StopRx,
  ) {
    let socket_rtp = match UdpSocket::bind("0.0.0.0:0") {
      Ok(socket) => socket,
      Err(err) => {
        // TODO error
        return;
      },
    };

    let socket_rtcp = match UdpSocket::bind("0.0.0.0:0") {
      Ok(socket) => socket,
      Err(err) => {
        // TODO error
        return;
      },
    };

    // TODO setup muxer with stream info, but how to get it once the source
    //  already started much earlier?

    loop {
      let packet = source_rx.recv();
      if let Ok(packet) = packet {
        match muxer.mux(packet) {
          Ok(output) => {
            match output {
              RtpBuf::Rtp(buf) => {
                socket_rtp.send_to(&buf, dest.rtp_remote).unwrap(); // TODO
              },
              RtpBuf::Rtcp(buf) => {
                socket_rtp.send_to(&buf, dest.rtcp_remote).unwrap(); // TODO
              }
            }
          },
          Err(err) => {
            // TODO
          },
        };
      } else {
        // TODO
      }
      /*
      channel::select! {
        recv(source_rx) -> msg => {
        },
        recv(stop.into_rx()) -> _ => {
          // TODO
          break;
        },
      };
      */
    }
  }

  // TODO refactor
  fn run_tcp_interleaved(
    stream_info: StreamInfo,
    source_rx: SourceRx,
    muxer: RtpMuxer,
    dest: TcpInterleavedDestination,
    stop: StopRx,
  ) {
    // TODO setup muxer with stream info, but how to get it once the source
    //  already started much earlier?
    let mut muxer =
      match muxer.with_stream(stream_info) {
        Ok(muxer) => muxer,
        Err(err) => {
          // TODO
          return;
        },
      };

    loop {
      let packet = source_rx.recv();
      if let Ok(packet) = packet {
        match muxer.mux(packet) {
          Ok(output) => {
            let response_interleaved_message = match output {
              RtpBuf::Rtp(buf) => {
                ResponseMaybeInterleaved::Interleaved {
                  channel: dest.rtp_channel,
                  payload: buf.into(),
                }
              },
              RtpBuf::Rtcp(buf) => {
                ResponseMaybeInterleaved::Interleaved {
                  channel: dest.rtcp_channel,
                  payload: buf.into(),
                }
              },
            };
            dest.tx.send(response_interleaved_message).unwrap(); // TODO error handling
          },
          Err(err) => {
            // TODO
          },
        };
      } else {
        // TODO
      }
      /*
      TODO
      channel::select! {
        recv(source_rx) -> msg => {
        },
        recv(stop.into_rx()) -> _ => {
          // TODO
          break;
        },
      };
      */
    }

  }

}