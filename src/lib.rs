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

pub fn setup() {
    // Verify tooling is installed.
    let llvm::VerifyToolingResult {
        is_nightly,
        llvm_major_version,
    } = llvm::verify_tooling().expect("Failed to verify tooling");

    // If we are not on nightly, we need to set the RUSTC_BOOTSTRAP environment variable.
    if !is_nightly {
        println!("Setting RUSTC_BOOTSTRAP=1");
        std::env::set_var("RUSTC_BOOTSTRAP", "1");
    }

    // Set the RUSTFLAGS environment variable.
    // export RUSTFLAGS="-Cinstrument-coverage -Zno-profiler-runtime -Zlocation-detail=none --emit=llvm-ll"
    // This is used to pass arguments to rustc.
    let mut rustflags = String::from(
        "-Cinstrument-coverage -Zno-profiler-runtime -Zlocation-detail=none --emit=llvm-ll",
    );
    if llvm_major_version >= String::from("12") {
        rustflags.push_str(" -Zinstrument-coverage-note");
    }
    println!("Setting RUSTFLAGS={}", rustflags);
    std::env::set_var("RUSTFLAGS", rustflags);

    // Set wasmcov directory.
    dir::set_wasmcov_dir(None);
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
    if object_files.len() == 0 {
        panic!("No object file found in the output directory.");
    }
    let object_file: &std::path::PathBuf = object_files[0].as_ref().unwrap();
    report::generate_coverage_report(&object_file, &llvm_major_version)
        .expect("Failed to generate report");
}

// Blockchain-specific modules.
#[cfg(feature = "near")]
pub mod near;
