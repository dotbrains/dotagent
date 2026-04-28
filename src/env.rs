use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

const ENV_FILE_NAME: &str = ".env";

pub fn load_env_from_current_working_directory() -> io::Result<Option<PathBuf>> {
    let env_file_path = env::current_dir()?.join(ENV_FILE_NAME);

    if !env_file_path.exists() {
        return Ok(None);
    }

    let raw_contents = fs::read_to_string(&env_file_path)?;

    for line in raw_contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((raw_key, raw_value)) = trimmed.split_once('=') else {
            continue;
        };

        let key = raw_key.trim();
        if key.is_empty() || env::var_os(key).is_some() {
            continue;
        }

        env::set_var(key, normalize_env_value(raw_value.trim()));
    }

    Ok(Some(env_file_path))
}

fn normalize_env_value(raw_value: &str) -> String {
    if raw_value.len() >= 2 {
        let first = raw_value.as_bytes()[0];
        let last = raw_value.as_bytes()[raw_value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return raw_value[1..raw_value.len() - 1].to_string();
        }
    }

    raw_value.to_string()
}
