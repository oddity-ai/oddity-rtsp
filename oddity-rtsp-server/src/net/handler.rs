//! Ideally, `Handler` were a trait that looked something like this:
//!
//! ```
//! use oddity_rtsp_protocol::{Request, Response};
//!
//! pub trait Handler {
//!   async fn handle(request: &Request) -> Response;
//! }
//! ```
//!
//! Then, we could have the `AppHandler` implement it and generalize
//! over `H: Handler` in `Connection::spawn` and `ConnectionManager`.
//! This requires `async_trait` though, and would incur a heap alloc-
//! action on the hot path. It probably does not matter but why intr-
//! oduce a heap allocation for something that can more easily be ac-
//! hieved by a type alias.

/// Alias for `Handler` used to handle messages received from incomi-
/// ng connections.
pub type Handler = crate::app::handler::AppHandler;
