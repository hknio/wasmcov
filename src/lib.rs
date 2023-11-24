use anyhow::anyhow;
use anyhow::Result;
use std::process::Command;

pub mod dir;
pub mod llvm;
pub mod report;

fn run_command(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command).args(args).output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Command {} failed with status code {}: {}",
            command,
            output.status.code().unwrap_or(-1),
            String::from_utf8(output.stderr)?
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| anyhow!("Failed to read command output"))
}

// Blockchain-specific modules.
#[cfg(feature = "near")]
pub mod near;
