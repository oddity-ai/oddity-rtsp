use std::fmt;
use std::net::{SocketAddr, UdpSocket};
use rand::Rng;

use concurrency::{
  Service,
  StopRx,
  channel,
};

use oddity_rtsp_protocol::{
  RtspResponseWriter,
  Serialize,
};

use oddity_video::Packet;

use crate::conn::WriterTx;

use super::{
  Source,
  SourceRx,
  SourceMsg,
};

pub enum Destination {
  Udp(UdpTarget),
  Interleaved(WriterTx),
}

// TODO maybe this should just be connection
pub struct UdpTarget {
  pub muxer: RtpMuxer, // TODO need to have it here i think
  pub rtp_remote: SocketAddr,
  pub rtcp_remote: SocketAddr,
}

pub struct Session {
  service: Option<Service>,
  source_rx: SourceRx,
}

impl Session {

  pub fn new(
    source: &mut Source,
    destination: Destination,
  ) -> Self {
    let service = Service::spawn({
      let source_rx = source.subscribe();
      move |stop| {
        match destination {
          Destination::Udp(socket_addr) => {
            Self::run_udp(
              source_rx,
              socket_addr,
              stop,
            )
          },
          Destination::Interleaved(writer_tx) => {
            Self::run_interleaved(
              source_rx,
              writer_tx,
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

  fn run_udp(
    source_rx: SourceRx,
    dest: UdpTarget,
    stop: StopRx,
  ) {
    // TODO
    // !!! How to handle `server_port` pair, do we need to receive data there and if so, what
    //  do we do with it ??!?!?!
    // okay i think we can get 'm like this: https://www.ffmpeg.org/doxygen/3.4/rtpproto_8h.html

    let socket_rtp = match UdpSocket::bind(dest.rtp_local) {
      Ok(socket) => socket,
      Err(err) => {
        // TODO error
        return;
      },
    };

    let socket_rtcp = match UdpSocket::bind(dest.rtcp_local) {
      Ok(socket) => socket,
      Err(err) => {
        // TODO error
        return;
      },
    };

    loop {
      let msg = source_rx.recv();
      if let Ok(msg) = msg {
        match msg {
          SourceMsg::Packet(packet) => {
            packet.
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

  fn run_interleaved(
    source_rx: SourceRx,
    writer_tx: WriterTx,
    stop: StopRx,
  ) {

  }

}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
  const SESSION_ID_LEN: usize = 16;

  pub fn generate() -> SessionId {
    SessionId(
      rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(Self::SESSION_ID_LEN)
        .map(char::from)
        .collect())
  }

}

impl fmt::Display for SessionId {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.0.fmt(f)
  }

}