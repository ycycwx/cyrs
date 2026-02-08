extern crate clap;
extern crate env_logger;
extern crate log;

use std::io::Write;

use log::Level;

pub fn init_logger_env() {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            let style = buf.default_level_style(record.level());
            let style_start = style.render();
            let style_end = style.render_reset();

            if record.level() == Level::Info {
                writeln!(
                    buf,
                    "[cy]: {}{}{}",
                    style_start,
                    record.args(),
                    style_end
                )
            } else {
                writeln!(
                    buf,
                    "[cy] [{}{}{}]: {}{}{}",
                    style_start,
                    record.level(),
                    style_end,
                    style_start,
                    record.args(),
                    style_end
                )
            }
        })
        .init();
}
