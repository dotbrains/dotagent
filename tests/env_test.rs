use dotagent::env::load_env_from_current_working_directory;
use serial_test::serial;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const ENV_KEY: &str = "DOTAGENT_ENV_TEST_VALUE";

struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    fn set(path: &Path) -> Self {
        let original = env::current_dir().expect("failed to read current directory");
        env::set_current_dir(path).expect("failed to change current directory");
        Self { original }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

#[test]
#[serial]
fn loads_env_file_from_current_working_directory() {
    env::remove_var(ENV_KEY);
    let temp = tempdir().expect("failed to create temp dir");
    let _guard = CwdGuard::set(temp.path());

    fs::write(".env", format!("{ENV_KEY}=from-file\n")).expect("failed to write .env");

    let env_file_path = load_env_from_current_working_directory()
        .expect("failed to load .env")
        .expect("expected .env path");

    let expected_env_path = env::current_dir()
        .expect("failed to read current directory")
        .join(".env");

    assert_eq!(env_file_path, expected_env_path);
    assert_eq!(env::var(ENV_KEY).expect("missing env key"), "from-file");

    env::remove_var(ENV_KEY);
}

#[test]
#[serial]
fn does_nothing_when_env_file_is_missing() {
    env::remove_var(ENV_KEY);
    let temp = tempdir().expect("failed to create temp dir");
    let _guard = CwdGuard::set(temp.path());

    let env_file_path = load_env_from_current_working_directory().expect("unexpected io error");

    assert!(env_file_path.is_none());
    assert!(env::var(ENV_KEY).is_err());
}

#[test]
#[serial]
fn does_not_override_existing_shell_variable() {
    env::set_var(ENV_KEY, "from-shell");
    let temp = tempdir().expect("failed to create temp dir");
    let _guard = CwdGuard::set(temp.path());

    fs::write(".env", format!("{ENV_KEY}=from-file\n")).expect("failed to write .env");

    let _ = load_env_from_current_working_directory().expect("unexpected io error");

    assert_eq!(env::var(ENV_KEY).expect("missing env key"), "from-shell");

    env::remove_var(ENV_KEY);
}
