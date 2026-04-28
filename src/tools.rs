use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

pub fn built_in_tool_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "read_file",
            description: "Read the contents of a given relative file path. Use this when you want to see what's inside a file. Do not use this with directory names.",
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The relative path of a file in the working directory."
                    }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolSpec {
            name: "list_files",
            description: "List files and directories at a given path. If no path is provided, lists files in the current directory.",
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": ["string", "null"],
                        "description": "Optional relative path to list files from. Defaults to the current directory if not provided."
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolSpec {
            name: "edit_file",
            description: "Make edits to a text file. Replaces 'old_str' with 'new_str' in the given file. 'old_str' and 'new_str' MUST be different from each other. If the file specified with path doesn't exist and old_str is empty, it will be created with new_str as its contents.",
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file"
                    },
                    "old_str": {
                        "type": "string",
                        "description": "Text to search for — must match exactly and must only have one match. Pass an empty string to create a new file with new_str as its contents."
                    },
                    "new_str": {
                        "type": "string",
                        "description": "Text to replace old_str with"
                    }
                },
                "required": ["path", "old_str", "new_str"],
                "additionalProperties": false
            }),
        },
    ]
}

pub fn to_openai_tools(tools: &[ToolSpec]) -> Value {
    Value::Array(
        tools
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.parameters
                    }
                })
            })
            .collect(),
    )
}

pub fn execute_tool_call(name: &str, args_json: &str) -> Result<String> {
    match name {
        "read_file" => {
            let args: ReadFileArgs =
                serde_json::from_str(args_json).context("invalid arguments for read_file")?;
            read_file(&args.path)
        }
        "list_files" => {
            let args: ListFilesArgs =
                serde_json::from_str(args_json).context("invalid arguments for list_files")?;
            list_files(args.path)
        }
        "edit_file" => {
            let args: EditFileArgs =
                serde_json::from_str(args_json).context("invalid arguments for edit_file")?;
            edit_file(&args.path, &args.old_str, &args.new_str)
        }
        _ => bail!("Unknown tool: {name}"),
    }
}

#[derive(Debug, Deserialize)]
struct ReadFileArgs {
    path: String,
}

fn read_file(path: &str) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read file: {path}"))
}

#[derive(Debug, Deserialize)]
struct ListFilesArgs {
    #[serde(default)]
    path: Option<String>,
}

fn list_files(path: Option<String>) -> Result<String> {
    let root = path
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| ".".to_string());
    let root_path = Path::new(&root);

    if !root_path.exists() {
        bail!("path not found: {root}");
    }
    if !root_path.is_dir() {
        bail!("path is not a directory: {root}");
    }

    let mut out = Vec::new();

    for entry in WalkDir::new(root_path).min_depth(1) {
        let entry =
            entry.with_context(|| format!("failed to read directory entry under {root}"))?;
        let rel = entry
            .path()
            .strip_prefix(root_path)
            .map_err(|_| anyhow!("failed to create relative path under {root}"))?;
        let mut rel_str = rel.to_string_lossy().replace('\\', "/");
        if entry.file_type().is_dir() {
            rel_str.push('/');
        }
        out.push(rel_str);
    }

    out.sort();
    serde_json::to_string(&out).context("failed to serialize list_files output")
}

#[derive(Debug, Deserialize)]
struct EditFileArgs {
    path: String,
    old_str: String,
    new_str: String,
}

fn edit_file(path: &str, old_str: &str, new_str: &str) -> Result<String> {
    if old_str == new_str {
        bail!("old_str and new_str must be different");
    }

    let file_path = PathBuf::from(path);

    match fs::read_to_string(&file_path) {
        Ok(content) => {
            if old_str.is_empty() {
                bail!("file {path} already exists; pass a non-empty old_str to edit it");
            }

            let occurrences = content.match_indices(old_str).count();
            if occurrences == 0 {
                bail!("old_str not found in {path}");
            }
            if occurrences > 1 {
                bail!("old_str matched {occurrences} times in {path}; must be unique");
            }

            let replaced = content.replacen(old_str, new_str, 1);
            fs::write(&file_path, replaced)
                .with_context(|| format!("failed to write file: {path}"))?;
            Ok("OK".to_string())
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            if old_str.is_empty() {
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("failed to create parent directory for {path}"))?;
                }
                fs::write(&file_path, new_str)
                    .with_context(|| format!("failed to write file: {path}"))?;
                Ok("OK".to_string())
            } else {
                bail!("failed to read file: {path}: {err}");
            }
        }
        Err(err) => bail!("failed to read file: {path}: {err}"),
    }
}
