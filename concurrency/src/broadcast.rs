use std::sync::{Arc, Mutex};

use crossbeam_channel::{
  bounded,
  Sender as UnderlyingSender,
  Receiver,
  TrySendError,
};

#[derive(Clone)]
pub struct Broadcaster<T: Clone> {
  cap: usize,
  txs: Arc<Mutex<Vec<UnderlyingSender<T>>>>,
}

impl<T: Clone> Broadcaster<T> {

  pub fn new(cap: usize) -> Self {
    Self {
      cap,
      txs: Arc::new(
        Mutex::new(
          Vec::new()
        )
      ),
    }
  }

  pub fn subscribe(
    &mut self,
  ) -> Receiver<T> {
    let (tx, rx) = bounded(self.cap);
    self.txs.lock().unwrap().push(tx);
    rx
  }

  pub fn broadcast(
    &mut self,
    item: T,
  ) {
    {
      let mut txs = self.txs.lock().unwrap();
      *txs = txs
        .drain(..)
        .filter(|tx| {
          match tx.try_send(item.clone()) {
            Ok(()) => true,
            Err(TrySendError::Disconnected(_)) => {
              tracing::trace!("source cleaning up disconnected tx");
              false
            },
            Err(TrySendError::Full(_)) => {
              tracing::trace!(
                "source subscriber is not keeping up and being \
                forcefully disconnected",
              );
              false
            },
          }
        })
        .collect();
    }
  }

  pub fn num(
    &self,
  ) -> usize {
    self.txs.lock().unwrap().len()
  }

}
