use tokio::sync::watch::{
  channel,
  Receiver,
  Sender,
};

use super::{
  Descriptor,
};

use crate::worker::{Worker, Stopper}; // TODO own crate

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

  pub fn subscribe(&self) -> Receiver<Packet> {
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

  pub fn unsubscribe(&self, _receiver: Receiver<Packet>) {
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