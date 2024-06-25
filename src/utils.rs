use anyhow::{anyhow, Result};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub fn run_command(command: &str, args: &[&str], dir: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new(command);
    if let Some(dir) = dir {
        cmd.current_dir(dir);
    }
    cmd.args(args);
    let output = cmd
        .output()
        .map_err(|_| anyhow!("Failed to execute command \"{command} {args:?}\""))?;
    if !output.status.success() {
        return Err(anyhow!(
            "Command \"{command} {args:?}\" failed with status code {}: {}",
            output.status.code().unwrap_or(-1),
            String::from_utf8(output.stderr)?
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|_| anyhow!("Failed to read command \"{command} {args:?}\" output"))
}

pub fn find_file(dir: &Path, alternatives: &[&str]) -> Result<PathBuf> {
    alternatives
        .iter()
        .map(|path| dir.join(path))
        .find(|path| path.exists())
        .ok_or_else(|| {
            anyhow!(
                "Could not find any of the alternative paths: {:?}",
                alternatives
            )
        })
}

pub enum FileOperation {
    ReplaceText {
        pattern: String,
        replacement: String,
    },
    AddBefore {
        pattern: String,
        new_line: String,
    },
    AddAfter {
        pattern: String,
        new_line: String,
    },
}

pub fn modify_file(file_path: PathBuf, operation: FileOperation) -> std::io::Result<()> {
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    let mut modified_content = Vec::new();
    let mut modified = false;

    for line in reader.lines() {
        let line = line?;
        match &operation {
            FileOperation::ReplaceText {
                pattern,
                replacement,
            } => {
                if line.contains(pattern) {
                    modified_content.push(line.replace(pattern, replacement));
                    modified = true;
                } else {
                    modified_content.push(line);
                }
            }
            FileOperation::AddBefore { pattern, new_line } => {
                if line.contains(pattern) {
                    modified_content.push(new_line.clone());
                    modified = true;
                }
                modified_content.push(line);
            }
            FileOperation::AddAfter { pattern, new_line } => {
                modified_content.push(line.clone());
                if line.contains(pattern) {
                    modified_content.push(new_line.clone());
                    modified = true;
                }
            }
        }
    }

    if modified {
        let mut output_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file_path)?;
        for line in modified_content {
            writeln!(output_file, "{}", line)?;
        }
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("No modifications were made in {file_path:?}"),
        ))
    }
}
