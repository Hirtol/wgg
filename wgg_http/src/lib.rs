use std::sync::Arc;
use tokio::sync::Notify;

pub mod api;
pub mod config;
mod cross_system;
mod db;
pub mod setup;
pub mod telemetry;
mod utils;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A notifier to be able to shut down all systems appropriately, and in time.
pub fn get_quit_notifier() -> Arc<Notify> {
    Arc::new(Notify::new())
}
