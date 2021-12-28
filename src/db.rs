extern crate fs_extra;
extern crate log;
extern crate serde;
extern crate serde_json;

use std::env;
use std::fs::canonicalize;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use fs_extra::move_items;
use serde::{Deserialize, Serialize};

use log::error;
use log::info;
use log::warn;

#[derive(Serialize, Deserialize, Debug)]
pub struct DataBase {
    cache_path: PathBuf,
    db: Vec<String>,
}

fn initialize_cache_path() -> Result<PathBuf> {
    let home = env::var("HOME")?;
    let config_dir = PathBuf::from(&home).join(".cache");
    create_dir_all(&config_dir.join("cyrs"))?;
    Ok(config_dir.join("cyrs/cy.json"))
}

fn read_config<P: AsRef<Path>>(config_path: P) -> Result<Vec<String>> {
    let json_str = if config_path.as_ref().exists() {
        read_to_string(initialize_cache_path()?)?
    } else {
        File::create(config_path)?.write_all("[]".as_bytes())?;
        "[]".to_string()
    };
    let value: Vec<String> = serde_json::from_str(&json_str)?;
    Ok(value)
}

pub trait DataBaseHandler: Sized {
    fn new() -> Result<Self>;
    fn add<S: AsRef<str>>(&mut self, files: &[S]) -> Result<()>;
    fn create<S: AsRef<str>>(&mut self, files: &[S]) -> Result<()>;
    fn cp<S: AsRef<str>>(&self, files: &[S]) -> Result<()>;
    fn mv<S: AsRef<str>>(&self, files: &[S]) -> Result<()>;
    fn list(&self);
    fn reset(&self) -> Result<()>;
}

impl DataBaseHandler for DataBase {
    fn new() -> Result<Self> {
        let config_path = initialize_cache_path()?;
        let db = read_config(&config_path)?;
        Ok(DataBase { cache_path: config_path, db })
    }

    fn add<S: AsRef<str>>(&mut self, files: &[S]) -> Result<()> {
        for file in files {
            let full_path = canonicalize(file.as_ref())?;
            let real_path = full_path.as_path().display().to_string();
            if self.db.contains(&real_path) {
                warn!("\"{}\" is duplicated in clipboard.", &real_path);
            } else {
                self.db.push(full_path.as_path().display().to_string());
            }
        }

        let mut config = File::create(initialize_cache_path()?)?;
        let json = serde_json::to_string_pretty(&self.db)?;
        write!(config, "{}", json)?;
        Ok(())
    }

    fn create<S: AsRef<str>>(&mut self, files: &[S]) -> Result<()> {
        let mut db = vec![];
        for file in files {
            let full_path = canonicalize(file.as_ref())?;
            let real_path = full_path.as_path().display().to_string();
            db.push(real_path);
        }
        self.db = db;

        let mut config = File::create(initialize_cache_path()?)?;
        let json = serde_json::to_string_pretty(&self.db)?;
        write!(config, "{}", json)?;

        Ok(())
    }

    fn cp<S: AsRef<str>>(&self, files: &[S]) -> Result<()> {
        if self.db.is_empty() {
            warn!("You must exec `cy add <file>...` first.");
            return Ok(());
        }

        if files.is_empty() {
            warn!("You must choose an existing target <dir>.");
            return Ok(());
        }

        let target_dir = canonicalize(&files[0].as_ref())?.as_path().display().to_string();
        for file in &self.db {
            let full_path = canonicalize(file)?;
            let options = CopyOptions::new();
            match copy_items(&[full_path], &target_dir, &options) {
                Ok(_) => {
                    info!("Success copy file \"{}\" to \"{}\".", file, target_dir);
                }
                Err(err) => {
                    error!(
                        "Fail to copy file \"{}\" to \"{}\". Reason: {}.",
                        file, target_dir, err
                    );
                }
            };
        }
        Ok(())
    }

    fn mv<S: AsRef<str>>(&self, files: &[S]) -> Result<()> {
        if self.db.is_empty() {
            warn!("You must exec `cy add <file>...` first.");
            return Ok(());
        }

        if files.is_empty() {
            warn!("You must choose an existing target <dir>.");
            return Ok(());
        }

        let mut failed_items: Vec<String> = vec![];
        let target_dir = canonicalize(&files[0].as_ref())?.as_path().display().to_string();
        for file in &self.db {
            let full_path = canonicalize(file)?;
            let options = CopyOptions::new();
            match move_items(&[full_path], &target_dir, &options) {
                Ok(_) => info!("Success move file \"{}\" to \"{}\"", file, target_dir),
                Err(err) => {
                    error!("Fail to move file \"{}\" to \"{}\". Reason: {}", file, target_dir, err);

                    failed_items.push(file.to_string());
                }
            };
        }

        // write config with failed files after move
        let mut config = File::create(&self.cache_path)?;
        write!(config, "{}", serde_json::to_string_pretty(&failed_items)?)?;

        Ok(())
    }

    fn list(&self) {
        if self.db.is_empty() {
            warn!("Clipboard is empty. You can exec `cy add <file>...` to add files.");
            return;
        }

        self.db.iter().enumerate().for_each(|(index, item)| {
            info!("{}", (index + 1).to_string() + ". " + item);
        })
    }

    fn reset(&self) -> Result<()> {
        let mut config = File::create(&self.cache_path)?;
        match write!(config, "[]") {
            Ok(()) => info!("Reset clipboard successfully."),
            Err(err) => error!("{}", err.to_string()),
        };
        Ok(())
    }
}
