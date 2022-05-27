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
  descriptor: Descriptor,
  mut reader: Reader,
  mut tx: Broadcaster<Packet>,
  mut stop: StopRx,
) {
  fn retry_timeout() {
    // TODO
  }

  let stream_info = match fetch_stream_info(&reader) {
    Ok(stream_info) => {
      stream_info
    },
    Err(err) => {
      tracing::error!(
        %descriptor, %err,
        "failed to fetch stream information"
      );
      return;
    },
  };

  while !stop.should() {
    match reader.read(stream_info.index) {
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

/// Helper function to initialize a reader and produce stream information.
pub fn initialize(
  descriptor: &Descriptor,
) -> Result<(Reader, StreamInfo), VideoError> {
  match Reader::new(&descriptor.clone().into()) {
    Ok(reader) => {
      match fetch_stream_info(&reader) {
        Ok(stream_info) => {
          Ok((reader, stream_info))
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
) -> Result<StreamInfo, VideoError> {
  let stream_index = reader.best_video_stream_index()?;
  reader.stream_info(stream_index)
}