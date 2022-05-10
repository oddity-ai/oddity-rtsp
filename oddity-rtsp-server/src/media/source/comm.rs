// TODO(gerwin) Refactor into `oddity-threading` crate.
// TODO(gerwin) For correct refactoring this requires splitting the API
// into Rx and Tx similar to channel.

use std::sync::{
  Arc,
  Mutex,
};

use crossbeam_channel::{
  bounded,
  Receiver,
  Sender,
  TrySendError,
};

use super::msg::Message;

/// The maximum number of messages in the underlying channel that
/// communicates contorl messages between pubs and subs.
const CHANNEL_CAP: usize = 128;

/// Handles channel communication between source producer of media
/// events and subscribers.
#[derive(Clone)]
pub(super) struct Communication {
  inner: Arc<Mutex<Inner>>,
}

/// Inner data holds members protected by `Arc` and `Mutex`.
struct Inner {
  txs: Vec<Sender<Message>>,
  count: usize,
}

impl Communication {

  /// Create a new communication object.
  pub(super) fn new() -> Self {
    Self {
      inner: Arc::new(
        Mutex::new(
          Inner {
            txs: Vec::new(),
            count: 0,
          }
        )
      )
    }
  }

  /// Subscribe and acquire `Rx` to receive items.
  pub(super) fn subscribe(
    &mut self,
  ) -> Rx {
    let (tx, rx) = bounded(CHANNEL_CAP);
    {
      let mut inner = self.inner.lock().unwrap();
      inner.txs.push(tx);
      inner.count += 1;
    }
    rx
  }

  /// Unsubscribe `Rx`.
  pub(super) fn unsubscribe(
    &mut self,
    rx: Rx,
  ) {
    {
      let mut inner = self.inner.lock().unwrap();
      inner.count -= 1;
    }
    drop(rx);
  }

  /// Broadcast a message to all subscribers.
  /// 
  /// Note: To be called by producer.
  pub(super) fn broadcast(
    &mut self,
    msg: Message,
  ) {
    {
      let mut inner = self.inner.lock().unwrap();
      inner.txs = inner.txs
        .drain(..)
        .filter(|tx| {
          match tx.try_send(msg.clone()) {
            Ok(()) => true,
            Err(TrySendError::Disconnected(_)) => {
              tracing::trace!("source cleaning up disconnected tx");
              false
            },
            Err(TrySendError::Full(msg)) => {
              tracing::trace!(
                %msg,
                "source subscriber is not keeping up and being \
                forcefully disconnected",
              );
              false
            },
          }
        })
        .collect()
    }
  }

  /// Number of subscribers registered.
  pub(super) fn num_subscribers(
    &self,
  ) -> usize {
    self.inner.lock().unwrap().count
  }

}

/// Receiver end of channel between media producer and consumer.
pub type Rx = Receiver<Message>;