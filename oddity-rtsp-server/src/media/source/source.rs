use oddity_video::{
  StreamInfo,
  Packet,
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
};

pub struct Source {
  descriptor: Descriptor,
  reader: Option<(Service, StreamInfo)>,
  tx: Broadcaster<Packet>,
}

impl Source {
  const TX_CAP: usize = 1024;

  pub fn new(descriptor: &Descriptor) -> Self {
    Self {
      descriptor: descriptor.clone(),
      reader: None,
      tx: Broadcaster::new(Self::TX_CAP),
    }
  }

  pub fn describe(&self) -> Result<String, Error> {
    let sdp = create_sdp(
      "No Name".to_string(), // TODO
      &self.descriptor,
    )?;

    Ok(format!("{}", sdp))
  }

  pub fn subscribe(
    &mut self
  ) -> Result<(Rx, StreamInfo), Error> {
    Ok(
      match self.reader.as_ref() {
        // If the service is already active for this source and producing
        // packets just return another receiver end for the source producer.
        Some((service, stream_info)) if service.is_running() => {
          (self.tx.subscribe(), stream_info.clone())
        },
        // If the service is inactive (because there are no subscribers until
        // now), start the work internally and acquire a subscriber to it.
        _ => {
          let (reader, stream_info) =
            reader::initialize(&self.descriptor)
              .map_err(Error::Media)?;

          let rx = self.tx.subscribe();
          let service = Service::spawn({
            let descriptor = self.descriptor.clone();
            let tx = self.tx.clone();
            move |stop| {
              reader::run(
                descriptor,
                reader,
                tx,
                stop,
              )
            }
          });

          self.reader = Some((service, stream_info.clone()));
          (rx, stream_info)
        }
      }
    )
  }

}