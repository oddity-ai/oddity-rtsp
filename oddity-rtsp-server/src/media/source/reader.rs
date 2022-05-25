use concurrency::{
  Broadcaster,
  BroadcastError,
  StopRx,
};

use oddity_video::{
  Reader,
  StreamInfo,
  Packet,
};

use crate::media::{
  Descriptor,
  VideoError,
};

/// Receiver channel type for source-produced messages.
pub type Rx = concurrency::channel::Receiver<Packet>;

/// Internal service function that performs the actual reading process.
pub fn run(
  reader: Reader,
  mut tx: Broadcaster<Packet>,
  mut stop: StopRx,
) {
  fn retry_timeout() {
    // TODO
  }

  while !stop.should() {
    match reader.read(stream_id) {
      Ok(packet) => {
        // If there's no receivers left, then we can stop the loop
        // since it is not necessary anymore. It will be restarted
        // the next time there's a subscription.
        if let Err(BroadcastError::NoSubscribers) =
            tx.broadcast(packet) {
          break;
        }
      },
      Err(err) => {
        tracing::error!(
          %descriptor, %err,
          "reading from video stream failed",
        );
        retry_timeout();
        continue;
      },
    };

    // TODO handle reset of input stream!
  }
}

pub fn initialize(
  descriptor: &Descriptor,
) -> Result<(Reader, StreamInfo)> {
  match Reader::new(&descriptor.clone().into()) {
    Ok(reader) => {
      match fetch_stream_info(&reader) {
        // TODO
        Ok((stream_id, stream_info)) => {
          Ok((reader, stream_id))
        },
        Err(err) => {
          tracing::error!(
            %descriptor, %err,
            "failed to fetch stream information"
          );
          Err(err)
        },
      }
    },
    Err(err) => {
      tracing::error!(
        %descriptor, %err,
        "failed to open media"
      );
      Err(err)
    },
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
