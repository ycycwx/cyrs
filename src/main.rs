mod db;
mod logger;

extern crate clap;

use anyhow::Result;

use clap::{App, AppSettings, Arg, ArgMatches};

use crate::db::{DataBase, DataBaseHandler};
use crate::logger::init_logger_env;

fn unwrap_args(arg_matches: &ArgMatches, id: &str) -> Vec<String> {
    match arg_matches.values_of(id) {
        Some(matches) => matches.map(|s| s.to_string()).collect::<Vec<_>>(),
        None => vec![],
    }
}

fn main() -> Result<()> {
    init_logger_env();

    let matches = App::new("cyrs")
        .about("A simple C-c C-v tool in command line.")
        .version("0.1.1")
        .author("ycycwx <yytcjcy@gmail.com>")
        .setting(AppSettings::ArgsNegateSubcommands)
        .arg(
            Arg::new("INPUT")
                .help("Mark files into clipboard")
                .multiple_values(true)
                .takes_value(true),
        )
        .subcommand(
            App::new("add").visible_alias("a").about("Add files into clipboard").arg(
                Arg::new("file")
                    .help("Add <file>s into clipboard for `COPY/MOVE`")
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

    if let Some(inputs) = matches.values_of("INPUT") {
        database.create(&inputs.collect::<Vec<_>>())?;
    } else {
        match matches.subcommand() {
            Some(("add", matches)) => database.add(&unwrap_args(matches, "file"))?,
            Some(("copy", matches)) => database.cp(&unwrap_args(matches, "dir"))?,
            Some(("move", matches)) => database.mv(&unwrap_args(matches, "dir"))?,
            Some(("list", _)) => database.list(),
            Some(("reset", _)) => database.reset()?,
            _ => unreachable!(),
        }
    }

    Ok(())
}
