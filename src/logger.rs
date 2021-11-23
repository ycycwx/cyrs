extern crate clap;
extern crate env_logger;
extern crate log;

use std::io::Write;

use log::Level;

pub fn init_logger_env() {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            let mut style = buf.style();
            style
                .set_color(match record.level() {
                    Level::Error => env_logger::fmt::Color::Red,
                    Level::Warn => env_logger::fmt::Color::Yellow,
                    Level::Info => env_logger::fmt::Color::Green,
                    _ => env_logger::fmt::Color::White,
                })
                .set_bold(false);

            writeln!(buf, "[cy] [{}]: {}", style.value(record.level()), style.value(record.args()))
        })
        .init();
}
