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
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
    let config_dir = PathBuf::from(&home).join(".cache");
    create_dir_all(config_dir.join("cyrs"))?;
    Ok(config_dir.join("cyrs/cy.json"))
}

fn read_config<P: AsRef<Path>>(config_path: P) -> Result<Vec<String>> {
    let path = config_path.as_ref();
    let json_str = if path.exists() {
        read_to_string(path)?
    } else {
        File::create(path)?.write_all(b"[]")?;
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
    fn reset(&mut self) -> Result<()>;
}

impl DataBase {
    fn save(&self) -> Result<()> {
        let mut config = File::create(&self.cache_path)?;
        let json = serde_json::to_string_pretty(&self.db)?;
        write!(config, "{}", json)?;
        Ok(())
    }
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

        self.save()
    }

    fn create<S: AsRef<str>>(&mut self, files: &[S]) -> Result<()> {
        let mut db = vec![];
        for file in files {
            match canonicalize(file.as_ref()) {
                Ok(full_path) => {
                    let real_path = full_path.as_path().display().to_string();
                    db.push(real_path);
                }
                Err(err) => {
                    error!("Fail to add \"{}\". Reason: {}", file.as_ref(), err);
                }
            }
        }
        if !files.is_empty() && db.is_empty() {
            anyhow::bail!("No valid files to put into clipboard.");
        }
        self.db = db;
        self.save()
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

        let target_dir = canonicalize(files[0].as_ref())?.as_path().display().to_string();
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
        let target_dir = canonicalize(files[0].as_ref())?.as_path().display().to_string();
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
            info!("{}. {}", index + 1, item);
        })
    }

    fn reset(&mut self) -> Result<()> {
        File::create(&self.cache_path)?.write_all(b"[]")?;
        self.db.clear();
        info!("Reset clipboard successfully.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{read_config, DataBase, DataBaseHandler};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_DIRECTORY_ID: AtomicUsize = AtomicUsize::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let id = NEXT_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir()
                .join(format!("cyrs-db-test-{}-{timestamp}-{id}", std::process::id()));
            fs::create_dir(&directory).unwrap();
            Self(directory)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn reads_the_supplied_config_path() {
        let directory = TestDirectory::new();
        let config_path = directory.0.join("clipboard.json");
        fs::write(&config_path, r#"["provided-config"]"#).unwrap();

        assert_eq!(read_config(config_path).unwrap(), vec!["provided-config"]);
    }

    #[test]
    fn create_and_add_write_to_the_database_cache_path() {
        let directory = TestDirectory::new();
        let cache_path = directory.0.join("clipboard.json");
        fs::write(&cache_path, b"[]").unwrap();
        let first = directory.0.join("first.txt");
        let second = directory.0.join("second.txt");
        fs::write(&first, "first").unwrap();
        fs::write(&second, "second").unwrap();
        let mut database = DataBase { cache_path: cache_path.clone(), db: vec![] };

        database.create(&[first.to_str().unwrap()]).unwrap();

        let saved: Vec<String> =
            serde_json::from_str(&fs::read_to_string(&cache_path).unwrap()).unwrap();
        let first_path = fs::canonicalize(&first).unwrap().to_str().unwrap().to_string();
        assert_eq!(saved, vec![first_path.clone()]);

        database.add(&[second.to_str().unwrap()]).unwrap();

        let saved: Vec<String> =
            serde_json::from_str(&fs::read_to_string(&cache_path).unwrap()).unwrap();
        let second_path = fs::canonicalize(&second).unwrap().to_str().unwrap().to_string();
        assert_eq!(saved, vec![first_path, second_path]);
    }

    #[test]
    fn reset_propagates_save_errors_and_preserves_memory() {
        let directory = TestDirectory::new();
        let mut database =
            DataBase { cache_path: directory.0.clone(), db: vec!["original.txt".to_string()] };

        let result = database.reset();

        assert!(result.is_err(), "reset must report a failed clipboard write");
        assert_eq!(database.db, vec!["original.txt"]);
    }

    #[test]
    fn reset_clears_persisted_and_in_memory_clipboard() {
        let directory = TestDirectory::new();
        let cache_path = directory.0.join("clipboard.json");
        fs::write(&cache_path, r#"["original.txt"]"#).unwrap();
        let mut database =
            DataBase { cache_path: cache_path.clone(), db: vec!["original.txt".to_string()] };

        database.reset().unwrap();

        let saved: Vec<String> =
            serde_json::from_str(&fs::read_to_string(cache_path).unwrap()).unwrap();
        assert!(saved.is_empty());
        assert!(database.db.is_empty());
    }

    #[test]
    fn create_with_only_invalid_inputs_preserves_clipboard() {
        let directory = TestDirectory::new();
        let cache_path = directory.0.join("clipboard.json");
        let original = r#"["original.txt"]"#;
        fs::write(&cache_path, original).unwrap();
        let mut database =
            DataBase { cache_path: cache_path.clone(), db: vec!["original.txt".to_string()] };
        let first = directory.0.join("missing-first.txt");
        let second = directory.0.join("missing-second.txt");

        let result = database.create(&[first.to_str().unwrap(), second.to_str().unwrap()]);

        assert!(result.is_err(), "create must report when every input is invalid");
        assert_eq!(database.db, vec!["original.txt"]);
        assert_eq!(fs::read_to_string(cache_path).unwrap(), original);
    }
}
