use std::sync::atomic::{AtomicBool, Ordering};

static LOG_TO_STDOUT: AtomicBool = AtomicBool::new(true);

pub fn set_enabled(enabled: bool) {
    LOG_TO_STDOUT.store(enabled, Ordering::Relaxed);
}

#[inline]
pub fn enabled() -> bool {
    LOG_TO_STDOUT.load(Ordering::Relaxed)
}

pub fn log(msg: &str) {
    if enabled() {
        println!("{msg}");
    }
}
