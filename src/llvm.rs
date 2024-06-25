use crate::utils::run_command;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::sync::Once;

static LLVM_TOOLING: Once = Once::new();
static mut LLVM_TOOLING_RESULT: Option<LlvmToolingResult> = None;
pub fn get_tooling() -> Result<&'static LlvmToolingResult> {
    LLVM_TOOLING.call_once(|| unsafe {
        LLVM_TOOLING_RESULT = Some(find_tooling().expect("Failed to initialize LLVM tooling"));
    });
    unsafe { Ok(LLVM_TOOLING_RESULT.as_ref().unwrap()) }
}

pub fn check_rustc_version() -> Result<(bool, String)> {
    let output_str = run_command("rustc", &["--version", "--verbose"], None)?;
    let is_nightly = output_str.contains("nightly");
    let llvm_major_version = Regex::new(r"LLVM version: (\d+)")
        .unwrap()
        .captures(&output_str)
        .and_then(|cap| cap.get(1).map(|m| m.as_str()))
        .map(String::from)
        .ok_or(anyhow!("Failed to parse rustc output: {}", output_str))?;
    Ok((is_nightly, llvm_major_version))
}

pub fn check_llvm_tool_version(command: &str) -> Result<String> {
    let output = run_command(&command, &["--version"], None)?;
    let llvm_major_version = Regex::new(r"version (\d+)")
        .unwrap()
        .captures(&output)
        .and_then(|cap| cap.get(1).map(|m| m.as_str()))
        .map(String::from)
        .ok_or(anyhow!("Failed to parse {command} output:\n{output}"))?;
    Ok(llvm_major_version)
}

pub fn find_llvm_tool(tool: &str, major_version: &str) -> Result<String> {
    let version = check_llvm_tool_version(&format!("{tool}-{major_version}"));
    if version.is_err() {
        let version = check_llvm_tool_version(&tool);
        if version.is_err() {
            return Err(anyhow!("Failed to find {tool}-{major_version}"));
        }
        let version = version.unwrap();
        if version != major_version {
            Err(anyhow!(
                "Found {tool} version {version}, but expected {major_version}"
            ))
        } else {
            Ok(tool.to_string())
        }
    } else {
        Ok(format!("{tool}-{major_version}"))
    }
}

pub struct LlvmToolingResult {
    pub rustc_is_nightly: bool,
    pub llvm_major_version: String,
    pub clang: String,
    pub llvm_cov: String,
    pub llvm_profdata: String,
}

pub fn find_tooling() -> Result<LlvmToolingResult> {
    let (rustc_is_nightly, llvm_major_version) = check_rustc_version()?;
    let clang = find_llvm_tool("clang", &llvm_major_version)?;
    let llvm_cov = find_llvm_tool("llvm-cov", &llvm_major_version)?;
    let llvm_profdata = find_llvm_tool("llvm-profdata", &llvm_major_version)?;

    Ok(LlvmToolingResult {
        rustc_is_nightly,
        llvm_major_version,
        clang,
        llvm_cov,
        llvm_profdata,
    })
}

/*
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
    }
}
 */
