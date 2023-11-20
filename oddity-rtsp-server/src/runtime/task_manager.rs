use std::future::Future;

use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::task;

pub struct TaskManager {
    hold_tx: Mutex<Option<mpsc::Sender<()>>>,
    hold_rx: Mutex<mpsc::Receiver<()>>,
    stop_tx: broadcast::Sender<()>,
}

impl TaskManager {
    pub fn new() -> Self {
        let (hold_tx, hold_rx) = mpsc::channel(1);
        let (stop_tx, _) = broadcast::channel(1);
        Self {
            // Must protect by mutex since another task might
            // invalidate the `hold_tx` once shutdown begins.
            hold_tx: Mutex::new(Some(hold_tx)),
            // Must protect `hold_rx` by mutex to allow for
            // internal mutability.
            hold_rx: Mutex::new(hold_rx),
            stop_tx,
        }
    }

    #[allow(clippy::let_underscore_future)]
    pub async fn spawn<F, T>(&self, f: F) -> Task
    where
        F: FnOnce(TaskContext) -> T + Send + 'static,
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        // Unlock the `hold_tx` mutex and grab a copy to pass
        // on to the task context. `hold_tx` could be empty if
        // another task already asked the manager to stop, in
        // which case we ignore the request and don't start a
        // task at all.
        if let Some(hold_all_tx) = self.hold_tx.lock().await.as_ref().cloned() {
            let (hold_tx, hold_rx) = oneshot::channel();
            let (stop_tx, stop_rx) = mpsc::channel(1);
            let stop_all_rx = self.stop_tx.subscribe();
            let _ = task::spawn(async move {
                // Instantiate task context here. After the fut-
                // ure genrated by `f` has finished, it will be
                // dropped automatically, which will cause the
                // hold to be released as well.
                let task_context = TaskContext {
                    _token: hold_tx,
                    _token_all: hold_all_tx,
                    stop: stop_rx,
                    stop_all: stop_all_rx,
                };

                f(task_context).await;
            });
            Task::new(hold_rx, stop_tx)
        } else {
            Task::none()
        }
    }

    pub async fn stop(&self) {
        // If we don't drop the apex hold_tx here then the call
        // to recv() below will block forever since there would
        // be one remaing hold.
        drop(self.hold_tx.lock().await.take());

        // Send stop signal to all tasks using the stop signal
        // broadcast channel; tasks must respond! Note: It is
        // very important that this happens after the remaining
        // `hold_tx` is dropped, since dropping `hold_tx` also
        // causes futher invocations of `spawn` to be ignored.
        // If they were not ignored, then a new task could pot-
        // entially be spawned that never received the stop re-
        // quest which would cause a deadlock.
        let _ = self.stop_tx.send(());

        // Wait for the channel to break after all `hold_tx` are
        // dropped, which means all tasks have finished.
        let _ = self.hold_rx.lock().await.recv().await;
    }
}

pub struct Task {
    hold: Option<oneshot::Receiver<()>>,
    stop: Option<mpsc::Sender<()>>,
}

impl Task {
    pub fn new(hold_rx: oneshot::Receiver<()>, stop_tx: mpsc::Sender<()>) -> Task {
        Self {
            hold: Some(hold_rx),
            stop: Some(stop_tx),
        }
    }

    pub fn none() -> Task {
        Task {
            hold: None,
            stop: None,
        }
    }

    pub async fn stop(&mut self) {
        if let Some(stop) = self.stop.as_ref() {
            let _ = stop.send(()).await;
        }
        if let Some(hold) = self.hold.take() {
            let _ = hold.await;
        }
    }
}

pub struct TaskContext {
    stop: mpsc::Receiver<()>,
    stop_all: broadcast::Receiver<()>,
    _token: oneshot::Sender<()>,
    _token_all: mpsc::Sender<()>,
}

impl TaskContext {
    pub async fn wait_for_stop(&mut self) {
        select! {
          // CANCEL SAFETY: `mpsc::Receiver::recv` is cancel safe.
          _ = self.stop.recv() => {},
          // CANCEL SAFETY: `broadcast::Receiver::recv` is cancel safe.
          _ = self.stop_all.recv() => {},
        };
    }
}
