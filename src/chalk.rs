use log::error;
use log::info;
use log::warn;

pub fn error<S: AsRef<str>>(str: S) {
    error!("\x1b[0;31m{}\x1b[0m", str.as_ref());
}

pub fn info<S: AsRef<str>>(str: S) {
    info!("\x1b[0;32m{}\x1b[0m", str.as_ref());
}

pub fn warn<S: AsRef<str>>(str: S) {
    warn!("\x1b[0;33m{}\x1b[0m", str.as_ref());
}
