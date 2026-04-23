use std::sync::atomic::{AtomicBool, Ordering};

/// Atomic bool used to determine if logging is enabled
static LOG_TO_STDOUT: AtomicBool = AtomicBool::new(true);

/// Update the state of the global logging flag
pub fn set_enabled(enabled: bool) {
    LOG_TO_STDOUT.store(enabled, Ordering::Relaxed);
}

/// Getter for the global logging flag
#[inline]
pub fn enabled() -> bool {
    LOG_TO_STDOUT.load(Ordering::Relaxed)
}

/// Print a string to stdout, but only if the global logging flag is set to true
pub fn log(msg: &str) {
    if enabled() {
        println!("{msg}");
    }
}
