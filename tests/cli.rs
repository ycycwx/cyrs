use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_HOME_ID: AtomicUsize = AtomicUsize::new(0);

struct TestHome(PathBuf);

impl TestHome {
    fn new() -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let id = NEXT_HOME_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("cyrs-test-{}-{timestamp}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cy")).args(args).env("HOME", &self.0).output().unwrap()
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn no_arguments_shows_help_before_initializing_clipboard() {
    let home = TestHome::new();
    let output = home.run(&[]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("Usage: cy"), "{stderr}");
    assert!(stderr.contains("Commands:"), "{stderr}");
    assert!(!home.0.join(".cache/cyrs").exists());
}

#[test]
fn explicit_help_exits_successfully() {
    let home = TestHome::new();
    let output = home.run(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "{output:?}");
    assert!(stdout.contains("Usage: cy"), "{stdout}");
    assert!(!home.0.join(".cache/cyrs").exists());
}

#[test]
fn positional_file_can_still_initialize_clipboard_without_a_subcommand() {
    let home = TestHome::new();
    let file = home.0.join("input.txt");
    fs::write(&file, "clipboard input").unwrap();

    let output = home.run(&[file.to_str().unwrap()]);

    assert!(output.status.success(), "{output:?}");
    let clipboard = fs::read_to_string(home.0.join(".cache/cyrs/cy.json")).unwrap();
    let files: Vec<String> = serde_json::from_str(&clipboard).unwrap();
    assert_eq!(files, vec![fs::canonicalize(file).unwrap().to_str().unwrap()]);
}

#[test]
fn userprofile_is_used_when_home_is_unset() {
    let home = TestHome::new();
    let output = Command::new(env!("CARGO_BIN_EXE_cy"))
        .arg("list")
        .env_remove("HOME")
        .env("USERPROFILE", &home.0)
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let clipboard = fs::read_to_string(home.0.join(".cache/cyrs/cy.json")).unwrap();
    let files: Vec<String> = serde_json::from_str(&clipboard).unwrap();
    assert!(files.is_empty());
}

#[test]
fn all_invalid_positional_files_preserve_existing_clipboard() {
    let home = TestHome::new();
    let cache_path = home.0.join(".cache/cyrs/cy.json");
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    let original = "[\n  \"original.txt\"\n]\n";
    fs::write(&cache_path, original).unwrap();
    let first = home.0.join("missing-first.txt");
    let second = home.0.join("missing-second.txt");

    let output = home.run(&[first.to_str().unwrap(), second.to_str().unwrap()]);

    assert!(!output.status.success(), "{output:?}");
    assert_eq!(fs::read_to_string(cache_path).unwrap(), original);
}

#[test]
fn mixed_positional_files_replace_clipboard_with_valid_files() {
    let home = TestHome::new();
    let cache_path = home.0.join(".cache/cyrs/cy.json");
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    fs::write(&cache_path, r#"["original.txt"]"#).unwrap();
    let missing = home.0.join("missing.txt");
    let valid = home.0.join("valid.txt");
    fs::write(&valid, "valid clipboard input").unwrap();

    let output = home.run(&[missing.to_str().unwrap(), valid.to_str().unwrap()]);

    assert!(output.status.success(), "{output:?}");
    let files: Vec<String> =
        serde_json::from_str(&fs::read_to_string(cache_path).unwrap()).unwrap();
    assert_eq!(files, vec![fs::canonicalize(valid).unwrap().to_str().unwrap()]);
}

#[test]
fn all_invalid_add_files_fail_and_preserve_existing_clipboard() {
    let home = TestHome::new();
    let cache_path = home.0.join(".cache/cyrs/cy.json");
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    let original = "[\n  \"original.txt\"\n]\n";
    fs::write(&cache_path, original).unwrap();
    let first = home.0.join("missing-first.txt");
    let second = home.0.join("missing-second.txt");

    let output = home.run(&["add", first.to_str().unwrap(), second.to_str().unwrap()]);

    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No valid files to add to clipboard."), "{stderr}");
    assert_eq!(fs::read_to_string(cache_path).unwrap(), original);
}

#[test]
fn mixed_add_files_append_valid_files_to_existing_clipboard() {
    let home = TestHome::new();
    let cache_path = home.0.join(".cache/cyrs/cy.json");
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    let original = home.0.join("original.txt");
    fs::write(&original, "original clipboard input").unwrap();
    let original_path = fs::canonicalize(original).unwrap().to_str().unwrap().to_string();
    fs::write(&cache_path, serde_json::to_string(&vec![&original_path]).unwrap()).unwrap();
    let missing = home.0.join("missing.txt");
    let valid = home.0.join("valid.txt");
    fs::write(&valid, "valid clipboard input").unwrap();

    let output = home.run(&["add", missing.to_str().unwrap(), valid.to_str().unwrap()]);

    assert!(output.status.success(), "{output:?}");
    let files: Vec<String> =
        serde_json::from_str(&fs::read_to_string(cache_path).unwrap()).unwrap();
    let valid_path = fs::canonicalize(valid).unwrap().to_str().unwrap().to_string();
    assert_eq!(files, vec![original_path, valid_path]);
}

#[test]
fn duplicate_add_files_count_as_valid_inputs_without_adding_duplicates() {
    let home = TestHome::new();
    let cache_path = home.0.join(".cache/cyrs/cy.json");
    fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
    let valid = home.0.join("valid.txt");
    fs::write(&valid, "valid clipboard input").unwrap();
    let valid_path = fs::canonicalize(&valid).unwrap().to_str().unwrap().to_string();
    fs::write(&cache_path, serde_json::to_string(&vec![&valid_path]).unwrap()).unwrap();
    let missing = home.0.join("missing.txt");

    let output = home.run(&[
        "add",
        missing.to_str().unwrap(),
        valid.to_str().unwrap(),
        valid.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "{output:?}");
    let files: Vec<String> =
        serde_json::from_str(&fs::read_to_string(cache_path).unwrap()).unwrap();
    assert_eq!(files, vec![valid_path]);
}
