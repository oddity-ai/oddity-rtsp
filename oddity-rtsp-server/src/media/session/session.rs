use std::net::UdpSocket;

use concurrency::{
  Service,
  StopRx,
};

use oddity_rtsp_protocol::ResponseMaybeInterleaved;

use oddity_video::{
  RtpMuxer,
  RtpBuf,
};

use crate::media::source::{
  Source,
  Rx as SourceRx,
  Msg as SourceMsg,
};

use super::context::{
  Context,
  Destination,
  UdpDestination,
  TcpInterleavedDestination,
};

pub struct Session {
  service: Option<Service>,
  source_rx: SourceRx,
}

impl Session {

  pub fn new(
    source: &mut Source,
    context: Context,
  ) -> Self {
    let service = Service::spawn({
      let source_rx = source.subscribe();
      move |stop| {
        match context.dest {
          Destination::Udp(dest) => {
            Self::run_udp(
              source_rx,
              context.muxer,
              dest,
              stop,
            )
          },
          Destination::TcpInterleaved(dest) => {
            Self::run_tcp_interleaved(
              source_rx,
              context.muxer,
              dest,
              stop,
            )
          }
        }
      }
    });

    Self {
      service: Some(service),
      source_rx: source.subscribe(),
    }
  }

  pub fn play() {
    // TODO
  }

  // TODO drop() = teardown (?)
  pub fn teardown(self) {
    
  }

  // TODO refactor
  fn run_udp(
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
      let msg = source_rx.recv();
      if let Ok(msg) = msg {
        match msg {
          SourceMsg::Init(stream_info) => {
            // TODO
          },
          SourceMsg::Packet(packet) => {
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
            }
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
    source_rx: SourceRx,
    mut muxer: RtpMuxer,
    dest: TcpInterleavedDestination,
    stop: StopRx,
  ) {
    // TODO setup muxer with stream info, but how to get it once the source
    //  already started much earlier?

    loop {
      let msg = source_rx.recv();
      if let Ok(msg) = msg {
        match msg {
          SourceMsg::Init(stream_info) => {
            // TODO
          },
          SourceMsg::Packet(packet) => {
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