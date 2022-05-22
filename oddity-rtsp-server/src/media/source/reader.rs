use concurrency::{
  Broadcaster,
  StopRx,
};

use oddity_video::{
  Reader,
  StreamInfo,
};

use crate::media::{
  Descriptor,
  VideoError,
};

use super::msg::Msg;

/// Receiver channel type for source-produced messages.
pub type Rx = concurrency::channel::Receiver<Msg>;

/// Internal service function that performs the actual reading process.
pub fn run(
  descriptor: Descriptor,
  mut tx: Broadcaster<Msg>,
  mut stop: StopRx,
) {
  fn retry_timeout() {
    // TODO
  }

  'outer:
  while !stop.should() {
    let (mut reader, stream_id) = match Reader::new(&descriptor.clone().into()) {
      Ok(reader) => {
        match fetch_stream_info(&reader) {
          Ok((stream_id, stream_info)) => {
            tx.broadcast(Msg::Init(stream_info));
            (reader, stream_id)
          },
          Err(err) => {
            tracing::error!(
              %descriptor, %err,
              "failed to fetch stream information"
            );
            retry_timeout();
            continue 'outer;
          },
        }
      },
      Err(err) => {
        tracing::error!(
          %descriptor, %err,
          "failed to open media"
        );
        retry_timeout();
        continue 'outer;
      },
    };

    while !stop.should() {
      match reader.read(stream_id) {
        Ok(packet) => {
          tx.broadcast(Msg::Packet(packet));
        },
        Err(err) => {
          tracing::error!(
            %descriptor, %err,
            "reading from video stream failed",
          );
          retry_timeout();
          continue 'outer;
        },
      };
    }
  }
}

/// Helper function for acquiring stream information.
fn fetch_stream_info(
  reader: &Reader,
) -> Result<(usize, StreamInfo), VideoError> {
  let stream_index = reader.best_video_stream_index()?;

  Ok((
    stream_index,
    reader.stream_info(stream_index)?,
  ))
}