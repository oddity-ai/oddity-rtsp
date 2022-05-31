mod app;
mod net;
mod runtime;

use app::App;

#[tokio::main]
async fn main() {
  App::new().run().await
}