#![feature(error_generic_member_access)]

use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;

pub mod storage;
pub(crate) mod app_details;
pub mod vault_handler;
pub mod daemon;

pub fn log_init(level: Level) -> WorkerGuard {
    let (writer, guard) = tracing_appender::non_blocking(std::io::stdout());
    let builder = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(level);
    if is_debug() {
        builder
            .pretty()
            .init()
    } else {
        builder.init();
    }

    guard
}

#[allow(unreachable_code)]
pub fn is_debug() -> bool {
    #[cfg(debug_assertions)] {
        return true;
    }
    return false;
}