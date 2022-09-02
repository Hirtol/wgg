use std::sync::Arc;
use tokio::sync::Notify;

mod api;
pub mod config;
mod db;
pub mod setup;
mod utils;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A notifier to be able to shut down all systems appropriately, and in time.
pub fn get_quit_notifier() -> Arc<Notify> {
    Arc::new(Notify::new())
}
