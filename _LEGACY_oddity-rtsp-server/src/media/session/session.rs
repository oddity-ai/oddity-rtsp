use tokio::spawn;
use tokio::net::UdpSocket;

use oddity_rtsp_protocol::ResponseMaybeInterleaved;

use oddity_video::{
  RtpMuxer,
  RtpBuf,
  StreamInfo,
};

use crate::media::{
  source::{
    Source,
    //Rx as SourceRx,
  },
  Error,
};

type SourceRx = tokio::sync::mpsc::UnboundedReceiver<oddity_video::Packet>;

use super::context::{
  Context,
  Destination,
  UdpDestination,
  TcpInterleavedDestination,
};


pub async fn start(
  source: &mut Source,
  context: Context,
) -> Result<(), Error> {
  let (rx, stream_info) = source.subscribe().await?;
  match context.dest {
    Destination::Udp(dest) => {
      spawn(
        run_udp(
          stream_info,
          rx,
          context.muxer,
          dest,
        )
      )
    },
    Destination::TcpInterleaved(dest) => {
      spawn(
        run_tcp_interleaved(
          stream_info,
          rx,
          context.muxer,
          dest,
        )
      )
    }
  };

  Ok(())
}

/*
pub fn play() {
  // TODO
}

// TODO drop() = teardown (?)
pub fn teardown(self) {
  
}
*/

// TODO refactor
async fn run_udp(
  stream_info: StreamInfo,
  source_rx: SourceRx,
  mut muxer: RtpMuxer,
  dest: UdpDestination,
) {
  let socket_rtp = match UdpSocket::bind("0.0.0.0:0").await {
    Ok(socket) => socket,
    Err(err) => {
      // TODO error
      return;
    },
  };

  let socket_rtcp = match UdpSocket::bind("0.0.0.0:0").await {
    Ok(socket) => socket,
    Err(err) => {
      // TODO error
      return;
    },
  };

  // TODO setup muxer with stream info, but how to get it once the source
  //  already started much earlier?

  loop {
    tokio::select! {
      packet = source_rx.recv() => {
        if let Some(packet) = packet {

        } else {
          // TODO
        }
      },
      _ = stop_rx.recv() => {
        // TODO
      },
    }
  }

  loop {
    let packet = source_rx.recv();
    if let Ok(packet) = packet {
      match muxer.mux(packet) {
        Ok(output) => {
          match output {
            RtpBuf::Rtp(buf) => {
              socket_rtp.send_to(&buf, dest.rtp_remote).await.unwrap(); // TODO
            },
            RtpBuf::Rtcp(buf) => {
              socket_rtp.send_to(&buf, dest.rtcp_remote).await.unwrap(); // TODO
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
async fn run_tcp_interleaved(
  stream_info: StreamInfo,
  source_rx: SourceRx,
  muxer: RtpMuxer,
  dest: TcpInterleavedDestination,
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
