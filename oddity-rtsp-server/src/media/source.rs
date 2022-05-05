use tokio::sync::watch::{
  channel,
  Receiver,
  Sender,
};

use super::{
  Descriptor,
  Error,
  sdp::create as create_sdp,
  super::{
    worker::{
      Worker,
      Stopper,
    },
  },
};

/// Represents a media packet or no packet if none is available.
pub type Packet = Option<oddity_video::Packet>;

/// Sender end of a channel that produces media packets.
pub type Producer = Sender<Packet>;

/// Receiver end of a channel that produces media packets.
pub type Subscriber = Receiver<Packet>;

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
  worker: Option<(Worker, Subscriber)>,
  /// Number of subscribers. When this becomes zero, we can stop the
  /// worker to not waste any resources.
  subscriber_count: usize,
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
      subscriber_count: 0,
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

  /// Retrieve a `Subscriber` through which the receiver can fetch media
  /// packets (produced by the worker).
  pub fn subscribe(&mut self) -> Receiver<Packet> {
    let receiver = match self.worker.as_ref() {
      // If the worker is already active for this source and producing
      // packets just return another receiver end for the source producer.
      Some((_, subscriber)) => {
        subscriber.clone()
      },
      // If the worker is inactive (because there are no subscribers until
      // now), start the work internally and acquire a subscriber to it.
      None => {
        let (producer, subscriber) = channel(None);
        let worker = Worker::new({
          let descriptor = self.descriptor.clone();
          move |stop| {
            Self::run(
              descriptor,
              producer,
              stop)
          }});

        self.worker = Some((worker, subscriber.clone()));
        subscriber
      }
    };

    self.subscriber_count += 1;

    receiver
  }

  /// Unsubscribe from the source.
  pub fn unsubscribe(&mut self, _receiver: Receiver<Packet>) {
    self.subscriber_count -= 1;

    // If there's no subscribers left, then we can stop the worker
    // thread since it is not necessary anymore. It will be restarted
    // the next time there's a subscription.
    if self.subscriber_count <= 0 {
      if let Some((worker, _)) = self.worker.take() {
        worker.stop(false);
      }
    }
  }

  /// Internal worker function that performs the actual reading process.
  fn run(
    descriptor: Descriptor,
    producer: Producer,
    stop: Stopper,
  ) {
    // TODO
    // - muxer = Muxer::new(descriptor)
    // - while NOT stop_rx
    //   - packet = muxer.mux()
    //   - producer.send(packet)
    // and some error handling here and there
  }

}