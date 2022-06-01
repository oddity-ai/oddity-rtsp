mod app;
mod net;
mod media;
mod source;
mod session;
mod runtime;

use app::App;

use tokio::signal::ctrl_c;

#[tokio::main]
async fn main() {
  let mut app = App::new();
  app.start().await;

  ctrl_c().await.expect("failed to listen for signal");
  app.stop().await;
}