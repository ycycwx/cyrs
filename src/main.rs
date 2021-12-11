mod db;
mod logger;

extern crate clap;

use std::error::Error;

use clap::{App, Arg, ArgMatches};
use log::error;

use crate::db::{DataBase, DataBaseHandler};
use crate::logger::init_logger_env;

fn unwrap_args(arg_matches: &ArgMatches, id: &str) -> Vec<String> {
    match arg_matches.values_of(id) {
        Some(matches) => matches.map(|s| s.to_string()).collect::<Vec<_>>(),
        None => vec![],
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    init_logger_env();

    let matches = App::new("cyrs")
        .about("A simple C-c C-v tool in command line.")
        .version("0.0.6")
        .author("ycycwx <yytcjcy@gmail.com>")
        .subcommand(
            App::new("add").visible_alias("a").about("Mark files into clipboard").arg(
                Arg::new("file")
                    .help("Mark <file>s into clipboard for `COPY/MOVE`")
                    .required(true)
                    .min_values(1),
            ),
        )
        .subcommand(
            App::new("copy")
                .visible_aliases(&["c", "cp"])
                .about("Copy all files in clipboard to target dir")
                .arg(
                    Arg::new("dir")
                        .help("Copy files into target <dir>")
                        .required(true)
                        .max_values(1),
                ),
        )
        .subcommand(
            App::new("move")
                .visible_aliases(&["m", "mv"])
                .about("Move all files in clipboard to target dir")
                .arg(
                    Arg::new("dir")
                        .help("Move files into target <dir>")
                        .required(true)
                        .max_values(1),
                ),
        )
        .subcommand(
            App::new("list")
                .visible_aliases(&["l", "ls", "show"])
                .about("List all files in clipboard"),
        )
        .subcommand(App::new("reset").visible_alias("clear").about("Reset clipboard"))
        .get_matches();

    let mut database = DataBase::new()?;

    match matches.subcommand() {
        Some(("add", matches)) => database.add(&unwrap_args(matches, "file"))?,
        Some(("copy", matches)) => database.cp(&unwrap_args(matches, "dir"))?,
        Some(("move", matches)) => database.mv(&unwrap_args(matches, "dir"))?,
        Some(("list", _)) => database.list(),
        Some(("reset", _)) => database.reset()?,
        None => error!("No subcommand was used"),
        _ => unreachable!(),
    }

    Ok(())
}
