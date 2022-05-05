use tokio::sync::watch::{
  channel,
  Receiver,
  Sender,
};

use oddity_video::{
  Reader,
  RtpMuxer,
  Locator,
};
use oddity_sdp_protocol::{
  Sdp,
  Kind,
  Protocol,
  TimeRange,
  CodecInfo,
};

use crate::worker::{Worker, Stopper}; // TODO own crate

use super::{
  Descriptor,
};

pub type Producer = Sender<Packet>;
pub type Subscriber = Receiver<Packet>;

pub struct Source {
  descriptor: Descriptor,
  worker: Option<(Worker, Subscriber)>,
  subscriber_count: usize,
}

impl Source {

  pub fn new(descriptor: &Descriptor) -> Self {
    Self {
      descriptor: descriptor.clone(),
      worker: None,
      subscriber_count: 0,
    }
  }

  // TODO improve interface
  pub fn describe(&self) -> String {
    // TODO query sdp
    let reader = Reader::new(&self.descriptor.clone().into()).unwrap(); // TODO unwrap
    let rtp_muxer = RtpMuxer::new("rtp://0.0.0.0".parse().unwrap()).unwrap()
      .with_stream(&reader, reader.best_video_stream_index().unwrap()).unwrap();
    //let writer = Writer::new_with_format("rtp://0.0.0.0", "rtp").unwrap();

    println!("libavcodec: {}", rtp_muxer.sdp().unwrap());

    let packetization_mode = rtp_muxer.packetization_mode();
    let parameter_sets = rtp_muxer.parameter_sets();

    let (sps, pps) = parameter_sets[0].as_ref().unwrap();

    let sdp = Sdp::new([0, 0, 0, 0].into(), "-".to_string(), [0, 0, 0, 0].into(), TimeRange::Live)
      .with_media(
        Kind::Video,
        1234,
        Protocol::RtpAvp,
        CodecInfo::h264(
          sps,
          pps.as_slice(),
          packetization_mode,
        ));

    println!("ours: {}", sdp); // TODO TEST
    
    "".to_string()
  }

  pub fn subscribe(&mut self) -> Receiver<Packet> {
    let receiver = match self.worker.as_ref() {
      Some((_, subscriber)) => {
        subscriber.clone()
      },
      None => {
        let (producer, subscriber) = channel(Default::default());
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

pub type Packet = Vec<u8>; // TODO