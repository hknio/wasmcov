use anyhow::{anyhow, Result};
use regex::Regex;
use std::{env, process::Command};

use crate::{dir::get_output_dir, run_command};

fn check_rustc_version() -> Result<(bool, String)> {
    let output_str = run_command("rustc", &["--version", "--verbose"])?;
    let is_nightly = output_str.contains("nightly");
    let llvm_major_version = Regex::new(r"LLVM version: (\d+)")
        .unwrap()
        .captures(&output_str)
        .and_then(|cap| cap.get(1).map(|m| m.as_str()))
        .map(String::from)
        .ok_or(anyhow!("Failed to parse rustc output: {}", output_str))?;
    Ok((is_nightly, llvm_major_version))
}

fn check_wasm_target(nightly: bool) -> Result<()> {
    let output_str = run_command("rustup", &["target", "list", "--installed"])?;
    let is_wasm_target_installed = output_str.contains("wasm32-unknown-unknown");

    if !is_wasm_target_installed {
        let nightly_str = if nightly { "nightly " } else { "" };
        let toolchain_str = if nightly { " --toolchain=nightly" } else { "" };
        Err(anyhow!(
            "The {}wasm32-unknown-unknown target is not installed.\nYou can install it by using the following command:\nrustup target add wasm32-unknown-unknown{}",
            nightly_str,
            toolchain_str
        ))
    } else {
        Ok(())
    }
}

fn check_command_availability(command: String) -> Result<()> {
    if Command::new(&command).arg("--version").output().is_err() {
        let llvm_version = command.split('-').last().unwrap_or("Unknown");
        Err(anyhow!("Missing command: {}. Please install LLVM version matching rustc LLVM version, which is {}.\nFor more information, check https://apt.llvm.org/", 
              command, llvm_version))
    } else {
        Ok(())
    }
}

fn set_target_dir() {
    let target_dir = get_output_dir().unwrap();
    env::set_var("CARGO_TARGET_DIR", target_dir);
}

pub struct VerifyToolingResult {
    pub is_nightly: bool,
    pub llvm_major_version: String,
    pub should_cleanup: bool,
}

pub fn verify_tooling() -> Result<VerifyToolingResult> {
    check_wasm_target(false)?;

    let (mut is_nightly, mut llvm_major_version) = check_rustc_version()?;

    let mut should_cleanup = false;

    if !is_nightly {
        env::set_var("RUSTUP_TOOLCHAIN", "nightly");

        if let Ok((nightly_toolchain, toolchain_version)) = check_rustc_version() {
            is_nightly = nightly_toolchain;
            llvm_major_version = toolchain_version;

            if !is_nightly {
                return Err(anyhow!("Coverage tool requires nightly version of the rust toolchain.\nYou can install it by using the following commands:\nrustup target add wasm32-unknown-unknown --toolchain=nightly"));
            }

            check_wasm_target(is_nightly)?;
            should_cleanup = true;
        } else {
            return Err(anyhow!("Failed to set nightly toolchain."));
        }
    }

    check_command_availability(format!("clang-{}", &llvm_major_version))?;
    check_command_availability(format!("llvm-cov-{}", &llvm_major_version))?;
    check_command_availability(format!("llvm-profdata-{}", &llvm_major_version))?;

    set_target_dir();

    // return object with is_ngihtly, llvm_major_version and should_cleanup
    Ok(VerifyToolingResult {
        is_nightly,
        llvm_major_version,
        should_cleanup,
    })
}

pub fn unset_rustup_toolchain(should_cleanup: bool) {
    if should_cleanup {
        env::remove_var("RUSTUP_TOOLCHAIN");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    // Paste your rustc --version --verbose output here
    static RUSTC_OUTPUT: &str = "rustc 1.72.0-nightly (5ea666864 2023-06-27)\nbinary: rustc\ncommit-hash: 5ea66686467d3ec5f8c81570e7f0f16ad8dd8cc3\ncommit-date: 2023-06-27\nhost: x86_64-unknown-linux-gnu\nrelease: 1.72.0-nightly\nLLVM version: 16.0.5\n";

    #[test]
    fn test_run_command() -> Result<()> {
        let command = "rustc";
        let args = &["--version", "--verbose"];

        let result = run_command(command, args)?;

        assert_eq!(result, RUSTC_OUTPUT.to_string());
        Ok(())
    }

    #[test]
    fn test_check_rustc_version() -> Result<()> {
        let result = check_rustc_version()?;

        let is_nightly = RUSTC_OUTPUT.contains("nightly");
        let llvm_major_version = Regex::new(r"LLVM version: (\d+)")
            .unwrap()
            .captures(RUSTC_OUTPUT)
            .and_then(|cap| cap.get(1).map(|m| m.as_str()))
            .map(String::from)
            .ok_or(anyhow!("Failed to parse rustc output: {}", RUSTC_OUTPUT))?;

        assert_eq!(result, (is_nightly, llvm_major_version));
        Ok(())
    }

    #[test]
    fn test_check_wasm_target() -> Result<()> {
        let is_nightly = RUSTC_OUTPUT.contains("nightly");

        check_wasm_target(is_nightly)?;

        Ok(())
    }

    #[test]
    fn test_check_command_availability() {
        let command = "rustc";

        let result = check_command_availability(command.to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_tooling_and_cleanup() {
        let result = verify_tooling();

        assert!(result.is_ok());

        let verify_tooling_result = result.unwrap();

        unset_rustup_toolchain(verify_tooling_result.should_cleanup);
    }
}
