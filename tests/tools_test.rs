use dotagent::tools::{built_in_tool_specs, execute_tool_call, to_openai_tools};
use serial_test::serial;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

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
fn creates_and_updates_file_through_edit_file() {
    let temp = tempdir().expect("failed to create temp dir");
    let _guard = CwdGuard::set(temp.path());

    let create_result = execute_tool_call(
        "edit_file",
        r#"{"path":"note.txt","old_str":"","new_str":"hello"}"#,
    )
    .expect("create should succeed");
    assert_eq!(create_result, "OK");

    let created = fs::read_to_string("note.txt").expect("failed to read created file");
    assert_eq!(created, "hello");

    let update_result = execute_tool_call(
        "edit_file",
        r#"{"path":"note.txt","old_str":"hello","new_str":"hello world"}"#,
    )
    .expect("update should succeed");
    assert_eq!(update_result, "OK");

    let updated = fs::read_to_string("note.txt").expect("failed to read updated file");
    assert_eq!(updated, "hello world");
}

#[test]
#[serial]
fn lists_nested_files_relative_to_path() {
    let temp = tempdir().expect("failed to create temp dir");
    let _guard = CwdGuard::set(temp.path());

    fs::create_dir_all("nested").expect("failed to create nested dir");
    fs::write("nested/example.txt", "content").expect("failed to write nested file");

    let result = execute_tool_call("list_files", r#"{"path":"."}"#).expect("list_files failed");
    let listed: Vec<String> = serde_json::from_str(&result).expect("invalid list_files JSON");

    assert!(listed.contains(&"nested/".to_string()));
    assert!(listed.contains(&"nested/example.txt".to_string()));
}

#[test]
#[serial]
fn rejects_non_unique_replacements() {
    let temp = tempdir().expect("failed to create temp dir");
    let _guard = CwdGuard::set(temp.path());

    fs::write("duplicate.txt", "same\nsame\n").expect("failed to write duplicate file");

    let error = execute_tool_call(
        "edit_file",
        r#"{"path":"duplicate.txt","old_str":"same","new_str":"different"}"#,
    )
    .expect_err("expected edit_file to reject non-unique replacement");

    assert!(error.to_string().contains("must be unique"));
}

#[test]
fn exposes_openai_compatible_function_tools() {
    let tools = built_in_tool_specs();
    let openai_tools = to_openai_tools(&tools);

    let tool_array = openai_tools
        .as_array()
        .expect("openai_tools should be an array");
    assert_eq!(tool_array.len(), 3);

    let names: Vec<&str> = tool_array
        .iter()
        .map(|tool| {
            tool.get("function")
                .and_then(|function| function.get("name"))
                .and_then(|name| name.as_str())
                .expect("missing function name")
        })
        .collect();

    assert_eq!(names, vec!["read_file", "list_files", "edit_file"]);
}
