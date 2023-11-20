pub mod task_manager;

use task_manager::TaskManager;

pub struct Runtime {
    task_manager: TaskManager,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            task_manager: TaskManager::new(),
        }
    }

    pub fn task(&self) -> &TaskManager {
        &self.task_manager
    }

    pub async fn stop(&self) {
        self.task_manager.stop().await
    }
}
