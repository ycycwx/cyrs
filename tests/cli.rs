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
