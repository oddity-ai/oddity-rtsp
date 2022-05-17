use std::sync::{
  Arc,
  atomic::{
    AtomicBool,
    Ordering,
  },
};
use std::thread::{
  spawn,
  JoinHandle,
};

use crossbeam_channel::{
  bounded,
  Sender,
};

use super::stop::StopRx;

pub struct Service {
  handle: Option<JoinHandle<()>>,
  stop_tx: Sender<()>,
  is_running: Arc<AtomicBool>,
}

impl Service {

  pub fn spawn<F>(f: F) -> Self
  where
    F: FnOnce(StopRx) -> (),
    F: Send + 'static,
  {
    let is_running = Arc::new(AtomicBool::new(false));
    let (stop_tx, stop_rx) = bounded(1);
    let handle = spawn({
      let is_running = is_running.clone();
      move || {
        is_running.store(true, Ordering::Relaxed);
        f(stop_rx.into());
        is_running.store(false, Ordering::Relaxed);
      }
    });

    Self {
      handle: Some(handle),
      stop_tx,
      is_running,
    }
  }

  pub fn is_running(&self) -> bool {
    self.is_running.load(Ordering::Relaxed)
  }

}

impl Drop for Service {

  fn drop(&mut self) {
    if let Some(handle) = self.handle.take() {
      if let Ok(()) = self.stop_tx.send(()) {
        let _ = handle.join();
      }
    }
  }

}