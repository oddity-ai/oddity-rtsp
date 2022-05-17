use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::{
  self,
  spawn,
  JoinHandle,
  ThreadId,
};

use super::{
  broadcast::Broadcaster,
  stop::StopRx,
};

type ServiceHashMap = HashMap<ThreadId, JoinHandle<()>>;

pub struct ServicePool {
  stop_tx: Broadcaster<()>,
  handles: Arc<Mutex<ServiceHashMap>>,
}

impl ServicePool {

  pub fn new() -> Self {
    Self {
      stop_tx: Broadcaster::new(1),
      handles: Arc::new(
        Mutex::new(
          ServiceHashMap::new(),
        ),
      ),
    }
  }

  pub fn spawn<F>(
    &mut self,
    f: F,
  )
  where
    F: FnOnce(StopRx) -> (),
    F: Send + 'static, 
  {
    let stop_rx = self.stop_tx.subscribe();
    let join_handle = 
      spawn({
        let handles = self.handles.clone();
        move || {
          f(stop_rx.into());
          handles
            .lock()
            .unwrap()
            .remove(&thread::current().id());
        }
      });

    self
      .handles
      .lock()
      .unwrap()
      .insert(
        join_handle.thread().id(),
        join_handle,
      );
  }

}

impl Drop for ServicePool {

  fn drop(&mut self) {
    self.stop_tx.broadcast(());
    self
      .handles
      .lock()
      .unwrap()
      .drain()
      .for_each(|(_, handle)| {
        let _ = handle.join();
      });
  }

}