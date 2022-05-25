use concurrency::{
  Service,
  Broadcaster,
};
use oddity_video::StreamInfo;

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
  /// Stream information. If the reader is currently inactive, then
  /// there is no stream, and it is set to `None`.
  stream_info: Option<StreamInfo>,
  /// Contains a handle to the reader service. If the worker loop is
  /// not active, it is `None`.
  service: Option<Service>,
  /// Contains underlying broadcaster handling packets produced by the
  /// reader and broadcasting them to subscribers.
  tx: Broadcaster<Packet>,
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
      stream_info: None,
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
  pub fn subscribe(
    &mut self
  ) -> Result<(StreamInfo, Rx), Error> {
    Ok(
      match self.service.as_ref() {
        // If the service is already active for this source and producing
        // packets just return another receiver end for the source producer.
        Some(service) if service.is_running() => {
          (
            // TODO store streaminfo with service
            self.stream_info.unwrap(),
            self.tx.subscribe(),
          )
        },
        // If the service is inactive (because there are no subscribers until
        // now), start the work internally and acquire a subscriber to it.
        _ => {
          let (reader, stream_info) = reader::initialize(descriptor)?;
          let rx = self.tx.subscribe();
          let service = Service::spawn({
            let descriptor = self.descriptor.clone();
            let tx = self.tx.clone();
            move |stop| {
              reader::run(
                reader,
                tx,
                stop,
              )
            }
          });

          self.stream = Some(stream_info);
          self.service = Some(service);
          
          (stream_info, rx)
        }
      }
    )
  }

}