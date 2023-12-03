use anyhow::anyhow;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

pub mod dir;
pub mod llvm;
pub mod report;

pub use minicov::capture_coverage;

pub fn run_command(command: &str, args: &[&str]) -> Result<String> {
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

pub fn setup(wasmcov_dir: Option<&PathBuf>) -> Result<String> {
    // Verify tooling is installed.
    let llvm::VerifyToolingResult {
        is_nightly,
        llvm_major_version: _,
    } = llvm::verify_tooling().expect("Failed to verify tooling");

    let mut env_string = dir::set_wasmcov_dir(wasmcov_dir)?;

    // If we are not on nightly, we need to set the RUSTC_BOOTSTRAP environment variable.
    if !is_nightly {
        // Add to env string
        env_string.push_str("; export RUSTC_BOOTSTRAP=1");
        std::env::set_var("RUSTC_BOOTSTRAP", "1");
    }

    // Set the RUSTFLAGS environment variable.
    // export RUSTFLAGS="-Cinstrument-coverage -Zno-profiler-runtime -Zlocation-detail=none --emit=llvm-ir"
    let mut rustflags = String::from(
        "-Cinstrument-coverage -Zno-profiler-runtime -Zlocation-detail=none --emit=llvm-ir",
    );

    // Add "-C lto=no" to disable LTO.
    rustflags.push_str(" -Clto=no");

    std::env::set_var("RUSTFLAGS", rustflags);

    // Combine the environment string with the RUSTFLAGS environment variable.
    env_string.push_str(&format!(
        "; export RUSTFLAGS=\"{}\"",
        std::env::var("RUSTFLAGS").unwrap()
    ));

    Ok(env_string)
}

pub fn finalize() {
    // Process all the build artefacts extract_compiled_artefacts
    dir::extract_compiled_artefacts().expect("Failed to extract compiled artefacts");

    let (_, llvm_major_version) =
        llvm::check_rustc_version().expect("Failed to check rustc version");

    // Modify ll files and generate object file
    report::modify_ll_files().expect("Failed to modify LL files");
    report::generate_object_file(&llvm_major_version).expect("Failed to generate object file");

    // Merge profraw files to profdata.
    report::merge_profraw_to_profdata(&llvm_major_version)
        .expect("Failed to merge profraw to profdata");

    // Generate report. If there is more than one .o file, throw an error, because we don't know which one to use.
    let output_dir = dir::get_output_dir().expect("Failed to get output directory");
    let object_files = glob::glob(output_dir.join("*.o").to_str().unwrap())
        .expect("Failed to get object files")
        .collect::<Vec<_>>();
    if object_files.len() > 1 {
        panic!("More than one object file found in the output directory. We don't know which one to use.");
    }
    if object_files.is_empty() {
        panic!("No object file found in the output directory.");
    }
    let object_file: &std::path::PathBuf = object_files[0].as_ref().unwrap();
    report::generate_coverage_report(object_file, &llvm_major_version)
        .expect("Failed to generate report");
}

// Find the path to the compiled WASM binary with coverage instrumentation.
pub fn post_build() -> Vec<PathBuf> {
    let target_dir = dir::get_target_dir().expect("Failed to get target directory");
    let wasm_files = glob::glob(target_dir.join("**/deps/*.wasm").to_str().unwrap())
        .expect("Failed to get wasm files")
        .collect::<Vec<_>>();

    println!("Found {} wasm compiles", wasm_files.len());

    // Unwrap all wasm_files from Result<PathBuf> to PathBuf
    let wasm_files = wasm_files
        .into_iter()
        .map(|x| x.unwrap())
        .collect::<Vec<PathBuf>>();

    // Print all wasm_file paths
    for wasm_file in &wasm_files {
        println!("{}", wasm_file.display());
    }

    // Return wasm_files
    wasm_files
}

// Blockchain-specific modules.
#[cfg(feature = "near")]
pub mod near;
