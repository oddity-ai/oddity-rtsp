use concurrency::{
  Service,
  Broadcaster,
};

use crate::media::{
  sdp::create as create_sdp,
  Descriptor,
  Error,
};

use super::{
  reader::{
    self,
    Rx,
  },
  msg::Msg,
};

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
  /// Contains a handle to the worker service. If the worker loop is
  /// not active, it is `None`.
  service: Option<Service>,
  /// Contains underlying broadcaster handling message produced by the
  /// service and broadcasting them to subscribers.
  tx: Broadcaster<Msg>,
}

impl Source {
  /// Subscriber max backlog size before it fails.
  const TX_CAP: usize = 1024;

  /// Create a new source.
  /// 
  /// # Arguments
  /// 
  /// * `descriptor` - Path or URI to underlying media source.
  pub fn new(descriptor: &Descriptor) -> Self {
    Self {
      descriptor: descriptor.clone(),
      service: None,
      tx: Broadcaster::new(Self::TX_CAP),
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
    let receiver = match self.service.as_ref() {
      // If the service is already active for this source and producing
      // packets just return another receiver end for the source producer.
      Some(_) => {
        self.tx.subscribe()
      },
      // If the service is inactive (because there are no subscribers until
      // now), start the work internally and acquire a subscriber to it.
      None => {
        let rx = self.tx.subscribe();
        let service = Service::spawn({
          let descriptor = self.descriptor.clone();
          let tx = self.tx.clone();
          move |stop| {
            reader::run(
              descriptor,
              tx,
              stop,
            )
          }
        });

        self.service = Some(service);
        rx
      }
    };

    receiver
  }

  /// Unsubscribe from the source.
  pub fn unsubscribe(&mut self, rx: Rx) {
    // Dropping the rx will cause it to become invalid.
    drop(rx);

    // If there's no receivers left, then we can stop the service
    // thread since it is not necessary anymore. It will be restarted
    // the next time there's a subscription.
    if self.tx.num() <= 0 {
      if let Some(service) = self.service.take() {
        // Dropping the service will cause it to stop.
        drop(service);
      }
    }
  }

}