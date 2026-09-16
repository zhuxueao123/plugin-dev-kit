use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::Serialize;

static OUTPUT_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

#[derive(Serialize)]
pub struct CommandOutput<T>
where
    T: Serialize,
{
    pub success: bool,
    pub command: String,
    pub data: T,
}

#[derive(Serialize)]
pub struct ErrorOutput {
    pub success: bool,
    pub command: String,
    pub error: String,
}

pub fn configure_output(path: Option<PathBuf>) {
    let _ = OUTPUT_PATH.set(path);
}

pub fn print_success<T>(command: &str, data: T) -> Result<()>
where
    T: Serialize,
{
    let payload = CommandOutput {
        success: true,
        command: command.to_string(),
        data,
    };
    emit(&payload)
}

pub fn print_error(command: &str, error: &str) -> Result<()> {
    let payload = ErrorOutput {
        success: false,
        command: command.to_string(),
        error: error.to_string(),
    };
    emit(&payload)
}

fn emit<T>(payload: &T) -> Result<()>
where
    T: Serialize,
{
    let raw = serde_json::to_string_pretty(payload)?;
    if let Some(path) = configured_output_path() {
        write_output_file(path, &raw)?;
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        if let Err(error) = writeln!(handle, "{raw}") {
            if error.kind() != io::ErrorKind::BrokenPipe {
                return Err(error).context("Failed to write CLI output.");
            }
        }
    }

    Ok(())
}

fn configured_output_path() -> Option<&'static PathBuf> {
    OUTPUT_PATH.get().and_then(|value| value.as_ref())
}

fn write_output_file(path: &Path, raw: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }
    }

    fs::write(path, raw)
        .with_context(|| format!("Failed to write output file: {}", path.display()))?;

    Ok(())
}
