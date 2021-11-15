pub fn red(str: &str) -> String {
    format!("\x1b[0;31m[cy] {}\x1b[0m", str)
}

pub fn green(str: &str) -> String {
    format!("\x1b[0;32m[cy] {}\x1b[0m", str)
}

pub fn yellow(str: &str) -> String {
    format!("\x1b[0;33m[cy] {}\x1b[0m", str)
}
