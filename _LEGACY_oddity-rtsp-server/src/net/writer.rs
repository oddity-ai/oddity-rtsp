use std::net::TcpStream;

use oddity_rtsp_protocol::RtspResponseWriter;

pub fn run_loop(
  mut writer: RtspResponseWriter<TcpStream>,
  writer_rx: WriterRx,
  stop_rx: StopRx,
) {
  let stop_rx = stop_rx.into_rx();
  loop {
    channel::select! {
      recv(writer_rx) -> response => {
        if let Ok(response) = response {
          if let Err(err) = writer.write(response) {
            tracing::error!(%err, "write failed");
            break;
          }
        } else {
          tracing::error!("writer channel failed unexpectedly");
          break;
        }
      },
      recv(stop_rx) -> _ => {
        tracing::trace!("connection writer stopping");
        break;
      },
    };
  }
}