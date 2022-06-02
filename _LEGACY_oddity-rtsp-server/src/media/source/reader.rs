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
// pub type Rx = concurrency::channel::Receiver<Packet>;

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
