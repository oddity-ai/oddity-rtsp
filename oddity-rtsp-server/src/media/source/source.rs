use oddity_video::{
  Reader,
  StreamInfo,
  Packet,
};

use crate::link::Receiver;

use crate::media::{
  sdp::create as create_sdp,
  Descriptor,
  Error,
  VideoError,
};

use super::{
  reader::{
    self,
    Rx,
  },
};

pub enum Msg {
  StreamInfo,
}

pub enum Reply {
  StreamInfo(StreamInfo),
}

pub async fn run(
  descriptor: Descriptor,
  link: Receiver<(), StreamInfo>,
  stop: Receiver<(), ()>,
  tx: Broadcast,
) {
  // TODO
  let (reader, _) = reader::initialize(&descriptor).unwrap();

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

  loop {
    tokio::select! {
      packet = tokio::task::spawn_blocking(move || reader.read(stream_info.index)) => {

      },
      _ = link.recv() => {
        link.reply(stream_info); // TODO error handling
      },
      _ = stop.recv() => {
        stop.reply(());
        break;
      },
    }
  }

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

/// Helper function for acquiring stream information.
fn fetch_stream_info(
  reader: &Reader,
) -> Result<StreamInfo, VideoError> {
  let stream_index = reader.best_video_stream_index()?;
  reader.stream_info(stream_index)
}

pub struct Source {
  descriptor: Descriptor,
  //reader: Option<(Service, StreamInfo)>,
  reader: tokio::task::JoinHandle<()>,
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