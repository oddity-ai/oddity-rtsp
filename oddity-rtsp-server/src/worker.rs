use std::thread::{
  spawn,
  JoinHandle,
};

use tokio::sync::oneshot::{
  channel,
  Sender,
  Receiver,
};

// TODO refactor into crate

pub struct Worker {
  handle: Option<JoinHandle<()>>,
  stop_tx: Sender<()>,
}

impl Worker {

  pub fn new<F>(f: F) -> Self
  where
    F: FnOnce(Stopper) -> (),
    F: Send + 'static,
  {
    let (stop_tx, stop_rx) = channel();
    let handle = spawn(move || {
      f(stop_rx)
    });

    Self {
      handle: Some(handle),
      stop_tx,
    }
  }

  pub fn stop(self, wait: bool) {
    if let Ok(()) = self.stop_tx.send(()) {
      // We take the handle here to make sure the destructor isn't
      // going to be waiting as well. This also allows destructing
      // the worker without waiting at all by calling `stop(false)`
      // before the worker is dropped.
      let handle_taken = self.handle.take();
      if wait {
        if let Some(handle) = handle_taken {
          let _ = handle.join();
        }
      }
    }
  }

}

impl Drop for Worker {

  fn drop(&mut self) {
    if let Some(handle) = self.handle.take() {
      if let Ok(()) = self.stop_tx.send(()) {
        let _ = handle.join();
      }
    }
  }

}

pub type Stopper = Receiver<()>;