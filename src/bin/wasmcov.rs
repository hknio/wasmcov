use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use wasmcov::build;
use wasmcov::dir;
use wasmcov::llvm;
use wasmcov::near_sandbox;
use wasmcov::report;
use wasmcov::utils;
use anyhow::Result;

/// A cargo subcommand for WASM coverage
#[derive(Parser)]
#[command(name = "cargo-wasmcov")]
#[command(about = "A cargo subcommand for WASM coverage")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Sets the wasmcov directory
    #[arg(long, global = true, help = "Sets the wasmcov directory")]
    wasmcov_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the project with WASM coverage instrumentation
    Build {
        /// Additional cargo arguments
        #[arg(last = true)]
        cargo_args: Vec<String>,
    },
    /// Run the project with WASM coverage
    Run {
        /// Specify NEAR sandbox version (e.g. 1.35.0)
        #[arg(long, value_name = "VERSION")]
        near: Option<String>,
        /// Additional cargo arguments
        #[arg(last = true)]
        cargo_args: Vec<String>,
    },
    /// Run tests with WASM coverage
    Test {
        /// Specify NEAR sandbox version (e.g. 1.35.0)
        #[arg(long, value_name = "VERSION")]
        near: Option<String>,
        /// Additional cargo arguments
        #[arg(last = true)]
        cargo_args: Vec<String>,
    },
    /// Generate coverage report
    Report {
        /// Additional llvm-cov arguments
        #[arg(last = true)]
        llvm_cov_args: Vec<String>,
    },
    /// Clean coverage data
    Clean {
        /// Removes entire wasmcov directory content when true
        #[arg(long, help = "Removes entire wasmcov directory content when true", default_value = "false")]
        all: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    llvm::get_tooling()?;
    dir::set_wasmcov_dir(cli.wasmcov_dir.as_ref());

    match cli.command {
        Commands::Build { cargo_args } => {
            let target_dir = prepare_target_directory()?;
            env::set_var(
                "CARGO_ENCODED_RUSTFLAGS",
                build::get_build_flags().join("\x1f"),
            );
            env::set_var(
                "RUSTUP_TOOLCHAIN",
                "nightly",
            );
            execute_command("cargo", "build", &cargo_args);
            process_wasm_files(&target_dir)?;
        }
        Commands::Run { near, cargo_args } => {
            setup_near_sandbox_if_needed(&near)?;
            let executables = build_binaries()?;
            let target_dir = prepare_target_directory()?;
            for binary in executables {
                println!("Running binary: {}", binary);
                env::set_var(
                    "CARGO_ENCODED_RUSTFLAGS",
                    build::get_build_flags().join("\x1f"),
                );
                env::set_var(
                    "RUSTUP_TOOLCHAIN",
                    "nightly",
                );
                execute_command(&binary, "", &vec![]);
            }
            process_wasm_files(&target_dir)?;            
        }
        Commands::Test { near, cargo_args } => {
            println!("{:?}", near);
            setup_near_sandbox_if_needed(&near)?;
            let tests = build_test_binaries()?;
            let target_dir = prepare_target_directory()?;
            for binary in tests {
                println!("Running binary: {}", binary);
                env::set_var(
                    "CARGO_ENCODED_RUSTFLAGS",
                    build::get_build_flags().join("\x1f"),
                );
                env::set_var(
                    "RUSTUP_TOOLCHAIN",
                    "nightly",
                );
                execute_command(&binary, "", &vec![]);
            }
            process_wasm_files(&target_dir)?;
        }
        Commands::Report { llvm_cov_args } => {
            let profdata_dir = dir::get_profdata_dir();
            let target_dir = dir::get_target_dir();
            for entry in fs::read_dir(dir::get_profraw_dir())? {
                let dir_path = entry?.path();
                if !dir_path.is_dir() {
                    continue;
                }

                let dir_name = dir_path.file_name().unwrap().to_str().unwrap();
                if fs::read_dir(&dir_path)?.next().is_none() {
                    continue; // Skip empty directories
                }

                println!("Generating coverage report for {}", dir_name);
                let object_file = utils::find_file(
                    &target_dir,
                    &[
                        &format!("{}.o", dir_name.replace("-", "_")),
                        &format!("{}.o", dir_name),
                    ],
                );

                if object_file.is_err() {
                    println!("Warning: Object file not found for {:?}", dir_name);
                    continue;
                }
                let object_file = object_file.unwrap();
                let profdata_path = profdata_dir.join(format!("{}.profdata", dir_name));
                report::merge_profraw_to_profdata(&dir_path, &profdata_path).unwrap();

                let report_dir = dir::get_report_dir().join(dir_name);
                report::generate_report(&profdata_path, &object_file, &report_dir, &llvm_cov_args)
                    .unwrap();
                println!("Coverage report has been saved to {:?}", report_dir);
            }
        }
        Commands::Clean { all } => {
            dir::clean_wasmcov_directory(all)?;
        }
    }

    Ok(())
}

fn execute_command(command: &str, subcommand: &str, args: &Vec<String>) -> String {
    let mut cmd = Command::new(command);
    let output = cmd
        .arg(subcommand)
        .args(args)
        .spawn()
        .expect("Failed to execute command")
        .wait_with_output()
        .expect("Failed to wait on child");

    if !output.status.success() {
        eprintln!(
            "Command `{}` failed with status: {}",
            subcommand, output.status
        );
        std::process::exit(output.status.code().unwrap_or(1));
    }
    String::from_utf8(output.stdout).expect("Failed to read command output")
}

fn setup_near_sandbox_if_needed(near: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(version) = near {
        let near_sandbox_dir = dir::get_wasmcov_dir().join("near_sandbox");
        let neard_path = near_sandbox::setup_near_sandbox(near_sandbox_dir, version.clone())?;
        env::set_var("NEAR_SANDBOX_BIN_PATH", neard_path);
    }
    Ok(())
}

fn build_binaries() -> Result<Vec<String>> {
    execute_command("cargo", "build", &vec![]);
    let output = Command::new("cargo")
        .args(["build", "--message-format=json"])
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to build test binaries"));
    }
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|json| {
            if json["reason"] == "compiler-artifact" && json["profile"]["test"] == false {
                json["executable"].as_str().map(String::from)
            } else {
                None
            }
        })
        .collect())
}

fn build_test_binaries() -> Result<Vec<String>> {
    execute_command("cargo", "test", &vec!["--no-run".to_string()]);
    let output = Command::new("cargo")
        .args(["test", "--no-run", "--message-format=json"])
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to build test binaries"));
    }
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|json| {
            if json["reason"] == "compiler-artifact" && json["profile"]["test"] == true {
                json["executable"].as_str().map(String::from)
            } else {
                None
            }
        })
        .collect())
}

fn prepare_target_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let target_dir = dir::get_target_dir();
    let wasm_files = build::find_wasm_files_with_coverage(&target_dir)?;
    for wasm_file in wasm_files {
        fs::remove_file(&wasm_file)?;
    }
    env::set_var("CARGO_TARGET_DIR", &target_dir);
    Ok(target_dir)
}

fn process_wasm_files(target_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let wasm_files = build::find_wasm_files_with_coverage(target_dir)?;
    for wasm_file in wasm_files {
        println!("Processing WASM file: {:?}", wasm_file);
        let ll_file = build::find_ll_file(&wasm_file);
        if ll_file.is_err() {
            eprintln!("Warning: LL file not found for {:?}", wasm_file);
            continue;
        }
        let ll_file = ll_file.unwrap();
        let new_ll_file = target_dir.join(ll_file.file_name().unwrap());
        build::correct_ll_file(&ll_file, &new_ll_file)?;
        let obj_file = target_dir.join(format!(
            "{}.o",
            new_ll_file.file_stem().unwrap().to_str().unwrap()
        ));
        build::compile_ll_file(&new_ll_file, &obj_file)?;
        let wasm_file_name = wasm_file.file_name().unwrap();
        let wasm_file_target = target_dir.join(wasm_file_name);
        fs::copy(&wasm_file, &wasm_file_target)?;
    }
    println!("Processed files has been saved to {:?}", target_dir);
    Ok(())
}
