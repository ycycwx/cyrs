extern crate dirs;
extern crate fs_extra;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use std::fs::canonicalize;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use dirs::home_dir;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use fs_extra::move_items;
use serde::{Deserialize, Serialize};

use crate::chalk;

#[derive(Serialize, Deserialize, Debug)]
pub struct DataBase {
    config_path: PathBuf,
    db: Vec<String>,
}

fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    Ok(match home_dir() {
        Some(dir) => dir.join(".cy"),
        None => panic!("{}", chalk::red("Cannot find $HOME path")),
    })
}

fn read_config(config_path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let json_str = if config_path.exists() {
        read_to_string(get_config_path()?)?
    } else {
        File::create(config_path)?.write_all("[]".as_bytes())?;
        "[]".to_string()
    };
    let value: Vec<String> = serde_json::from_str(&json_str)?;
    Ok(value)
}

pub trait DataBaseHandler: Sized {
    fn new() -> Result<Self, Box<dyn Error>>;
    fn add(&mut self, files: &[String]) -> Result<(), Box<dyn Error>>;
    fn reset(&self) -> Result<(), Box<dyn Error>>;
    fn cp(&self, files: &[String]) -> Result<(), Box<dyn Error>>;
    fn mv(&self, files: &[String]) -> Result<(), Box<dyn Error>>;
    fn list(&self);
}

impl DataBaseHandler for DataBase {
    fn new() -> Result<Self, Box<dyn Error>> {
        let config_path = get_config_path()?;
        let db = read_config(&config_path)?;
        Ok(DataBase { config_path, db })
    }

    fn add(&mut self, files: &[String]) -> Result<(), Box<dyn Error>> {
        for file in files {
            let full_path = canonicalize(file)?;
            if !self.db.contains(&full_path.as_path().display().to_string()) {
                let full_path = canonicalize(file)?;
                self.db.push(full_path.as_path().display().to_string());
            }
        }

        let mut config = File::create(get_config_path()?)?;
        let json = serde_json::to_string_pretty(&self.db)?;
        write!(config, "{}", json)?;
        Ok(())
    }

    fn reset(&self) -> Result<(), Box<dyn Error>> {
        let mut config = File::create(&self.config_path)?;
        write!(config, "[]")?;
        Ok(())
    }

    fn cp(&self, files: &[String]) -> Result<(), Box<dyn Error>> {
        if files.len() != 1 {
            panic!("{}", chalk::red("You must choose one target dir"));
        }

        let target_dir = canonicalize(&files[0])?.as_path().display().to_string();
        for file in &self.db {
            let full_path = canonicalize(file)?;
            let options = CopyOptions::new();
            match copy_items(&[full_path], &target_dir, &options) {
                Ok(_) => {
                    println!(
                        "{}",
                        chalk::green(&format!("Success copy file {} to {}", file, target_dir))
                    );
                }
                Err(err) => {
                    eprintln!(
                        "{}",
                        chalk::red(&format!(
                            "Fail to copy file {} to {}. Reason: {}",
                            file, target_dir, err
                        ))
                    );
                }
            };
        }
        Ok(())
    }

    fn mv(&self, files: &[String]) -> Result<(), Box<dyn Error>> {
        if files.len() != 1 {
            panic!("{}", chalk::red("You must choose one target dir"));
        }

        let mut failed_items: Vec<String> = vec![];
        let target_dir = canonicalize(&files[0])?.as_path().display().to_string();
        for file in &self.db {
            let full_path = canonicalize(file)?;
            let options = CopyOptions::new();
            match move_items(&[full_path], &target_dir, &options) {
                Ok(_) => {
                    println!(
                        "{}",
                        chalk::green(&format!("Success move file {} to {}", file, target_dir))
                    );
                }
                Err(err) => {
                    eprintln!(
                        "{}",
                        chalk::red(&format!(
                            "Fail to move file {} to {}. Reason: {}",
                            file, target_dir, err
                        ))
                    );

                    failed_items.push(file.to_string());
                }
            };
        }

        // write config with failed files after move
        let mut config = File::create(&self.config_path)?;
        println!("{}", serde_json::to_string_pretty(&failed_items)?);
        write!(config, "{}", serde_json::to_string_pretty(&failed_items)?)?;

        Ok(())
    }

    fn list(&self) {
        self.db.iter().enumerate().for_each(|(index, item)| {
            println!("{}", chalk::green(&((index + 1).to_string() + ". " + item)))
        })
    }
}
