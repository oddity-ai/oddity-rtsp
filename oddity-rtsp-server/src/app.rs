use crate::runtime::Runtime;

pub struct App {
  runtime: Runtime,
}

impl App {

  pub fn new() -> Self {
    Self {
      runtime: Runtime::new(),
    }
  }

  pub async fn run(mut self) {
    
  }

}
