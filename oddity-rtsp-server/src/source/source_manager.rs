use std::sync::Arc;
use std::collections::HashMap;

use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use crate::runtime::Runtime;
use crate::runtime::task_manager::TaskContext;
use crate::source::source::{
  Source,
  SourcePath,
  SourceState,
  SourceStateTx,
  SourceStateRx,
};

type SourceMap = Arc<Mutex<HashMap<SourcePath, Source>>>;

pub struct SourceManager {
  sources: SourceMap,
  source_state_tx: SourceStateTx,
  runtime: Arc<Runtime>,
}

impl SourceManager {

  pub async fn new(
    runtime: Arc<Runtime>,
  ) -> Self {
    let sources = Arc::new(Mutex::new(HashMap::new()));
    let (source_state_tx, source_state_rx) =
      mpsc::unbounded_channel();

    runtime
      .task()
      .spawn({
        let sources = sources.clone();
        move |task_context| {
          Self::run(
            sources.clone(),
            source_state_rx,
            task_context,
          )
        }
      })
      .await;

    Self {
      sources,
      source_state_tx,
      runtime,
    }
  }

  pub async fn register_and_start(
    &mut self,
    path: SourcePath,
  ) {
    let source = Source::start(
        path,
        self.source_state_tx.clone(),
        self.runtime.as_ref(),
      )
      .await;
  }

  // TODO stop all

  async fn run(
    sources: SourceMap,
    mut source_state_rx: SourceStateRx,
    mut task_context: TaskContext,
  ) {
    loop {
      select! {
        state = source_state_rx.recv() => {
          match state {
            Some(SourceState::Stopped(source_id)) => {
              let _ = sources.lock().await.remove(&source_id);
            },
            None => {
              // TODO
              break;
            },
          }
        },
        _ = task_context.wait_for_stop() => {
          break;
        },
      }
    }
  }
  
}