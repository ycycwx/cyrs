pub mod chalk;
pub mod db;

extern crate clap;
extern crate env_logger;
extern crate log;

use std::error::Error;

use clap::{App, Arg, ArgMatches};

use crate::db::{DataBase, DataBaseHandler};

fn unwrap_args(arg_matches: &ArgMatches, id: &str) -> Vec<String> {
    match arg_matches.values_of(id) {
        Some(matches) => matches.map(|s| s.to_string()).collect::<Vec<_>>(),
        None => vec![],
    }
}

fn init_logger_env() {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);
}

fn main() -> Result<(), Box<dyn Error>> {
    init_logger_env();

    let matches = App::new("cyrs")
        .about("A simple C-c C-v tool in command line.")
        .license("MIT")
        .version("0.1")
        .author("ycycwx <yytcjcy@gmail.com>")
        .subcommand(
            App::new("add").visible_alias("a").about("add files").arg(
                Arg::new("file")
                    .about("Mark files to `COPY/MOVE` into temporary zone.")
                    .required(true)
                    .min_values(1),
            ),
        )
        .subcommand(App::new("copy").visible_aliases(&["c", "cp"]).about("copy files").arg(
            Arg::new("file").about("Copy files into target folder").required(true).max_values(1),
        ))
        .subcommand(App::new("move").visible_aliases(&["m", "mv"]).about("move files").arg(
            Arg::new("file").about("Move files into target folder.").required(true).max_values(1),
        ))
        .subcommand(App::new("list").visible_aliases(&["l", "ls", "show"]).about("list files"))
        .subcommand(App::new("reset").visible_alias("clear").about("reset temporary zone"))
        .get_matches();

    let mut database = DataBase::new()?;

    match matches.subcommand() {
        Some(("add", matches)) => database.add(&unwrap_args(matches, "file"))?,
        Some(("copy", matches)) => database.cp(&unwrap_args(matches, "file"))?,
        Some(("move", matches)) => database.mv(&unwrap_args(matches, "file"))?,
        Some(("list", _)) => database.list(),
        Some(("reset", _)) => database.reset()?,
        None => chalk::error("No subcommand was used"),
        _ => unreachable!(),
    }

    Ok(())
}
