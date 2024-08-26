use std::fs::OpenOptions;

use slog::Drain;
use slog_scope::GlobalLoggerGuard;


const SLOG_FILE_NAME: &str = "slog.log";

pub fn init_slog() -> GlobalLoggerGuard {
    let work_dir = std::env::current_dir().unwrap();
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(work_dir.join(SLOG_FILE_NAME)).unwrap();

    let decorator = slog_term::PlainSyncDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    
    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let guard = slog_scope::set_global_logger(logger);

    // register slog_stdlog as the log handler with the log crate
    slog_stdlog::init().unwrap();

    guard
}