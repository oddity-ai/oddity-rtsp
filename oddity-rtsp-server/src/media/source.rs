mod msg;

use std::sync::Arc;

use oddity_video::{
  Reader,
  StreamInfo,
};

use super::{
  Descriptor,
  sdp::create as create_sdp,
  super::{
    worker::{
      Worker,
      Stopper,
    },
  },
  Error,
  VideoError,
};

use comm::Communication;

pub use comm::Rx;
pub use msg::Message;

/// Media source that produces media packets when active. The source
/// can produce packets and send them to one or more subscribers. This
/// way, there is not need to instantiate multiple readers for the same
/// media resource.
/// 
/// # Example
/// 
/// ```
/// // TODO
/// ```
pub struct Source {
  /// Describes the underlying media item.
  descriptor: Descriptor,
  /// Contains a handle to the worker thread and a origin subscriber
  /// from which new subscribers can be created. If the worker is not
  /// active, it is `None`.
  worker: Option<Worker>,
  /// Contains underlying communications handling for worker and the
  /// subscribers to this source.
  comms: Communication,
}

impl Source {

  /// Create a new source.
  /// 
  /// # Arguments
  /// 
  /// * `descriptor` - Path or URI to underlying media source.
  pub fn new(descriptor: &Descriptor) -> Self {
    Self {
      descriptor: descriptor.clone(),
      worker: None,
      comms: Communication::new(),
    }
  }

  /// Fetch media description in the Session Description Protocol
  /// format, as a string.
  /// 
  /// # Example
  /// 
  /// ```
  /// // TODO
  /// ```
  pub fn describe(&self) -> Result<String, Error> {
    let sdp = create_sdp(
      "No Name".to_string(), // TODO
      &self.descriptor,
    )?;

    Ok(format!("{}", sdp))
  }

  /// Retrieve a `Rx` through which the receiver can fetch media control
  /// packets such as re-initialization or media objects.
  pub fn subscribe(&mut self) -> Rx {
    let receiver = match self.worker.as_ref() {
      // If the worker is already active for this source and producing
      // packets just return another receiver end for the source producer.
      Some(_) => {
        self.comms.subscribe()
      },
      // If the worker is inactive (because there are no subscribers until
      // now), start the work internally and acquire a subscriber to it.
      None => {
        let rx = self.comms.subscribe();
        let worker = Worker::new({
          let descriptor = self.descriptor.clone();
          let comms = self.comms.clone();
          move |stop| {
            Self::run(
              descriptor,
              comms,
              stop,
            )
          }
        });

        self.worker = Some(worker);
        rx
      }
    };

    receiver
  }

  /// Unsubscribe from the source.
  pub fn unsubscribe(&mut self, rx: Rx) {
    self.comms.unsubscribe(rx);

    // If there's no receivers left, then we can stop the worker
    // thread since it is not necessary anymore. It will be restarted
    // the next time there's a subscription.
    if self.comms.num_subscribers() <= 0 {
      if let Some(worker) = self.worker.take() {
        worker.stop(false);
      }
    }
  }

  /// Internal worker function that performs the actual reading process.
  fn run(
    descriptor: Descriptor,
    mut comms: Communication,
    mut stop: Stopper,
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
              comms.broadcast(Message::Init(stream_info));
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
            comms.broadcast(Message::Packet(packet));
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