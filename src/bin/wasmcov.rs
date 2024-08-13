use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::{env, fs, path::PathBuf, process::Command};
use wasmcov::{build, dir, llvm, near_sandbox, report, utils};

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Wasmcov(WasmcovArgs),
}

#[derive(Parser)]
struct WasmcovArgs {
    #[command(subcommand)]
    command: WasmcovCommands,

    /// Sets the wasmcov directory
    #[arg(long, global = true, help = "Sets the wasmcov directory (can be also set by WASMCOV_DIR env var)")]
    wasmcov_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum WasmcovCommands {
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
    /// Merge profraw into profdata by running llvm-profdata merge
    Merge {
        //. Additional llvm-profdata arguments
        #[arg(last = true)]
        llvm_profdata_args: Vec<String>,
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
        #[arg(
            long,
            help = "Removes entire wasmcov directory content when true",
            default_value = "false"
        )]
        all: bool,
    },
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let cli = if args.len() > 1 && args[1] == "wasmcov" {
        Cli::parse()
    } else {
        Cli {
            command: Commands::Wasmcov(WasmcovArgs::parse()),
        }
    };

    match cli.command {
        Commands::Wasmcov(args) => handle_wasmcov(args),
    }
}

fn handle_wasmcov(args: WasmcovArgs) -> Result<()> {
    let wasmcov_dir = env::var("WASMCOV_DIR").unwrap_or_default();
    if args.wasmcov_dir.is_some() || wasmcov_dir.is_empty() {
        dir::set_wasmcov_dir(args.wasmcov_dir.as_ref());
    }

    match args.command {
        WasmcovCommands::Build { cargo_args } => build_command(cargo_args),
        WasmcovCommands::Run { near, cargo_args } => run_or_test_command("run", near, cargo_args),
        WasmcovCommands::Test { near, cargo_args } => run_or_test_command("test", near, cargo_args),
        WasmcovCommands::Merge { llvm_profdata_args } => merge_command(llvm_profdata_args),
        WasmcovCommands::Report { llvm_cov_args } => report_command(llvm_cov_args),
        WasmcovCommands::Clean { all } => clean_command(all),
    }
}

fn build_command(cargo_args: Vec<String>) -> Result<()> {
    let target_dir = prepare_target_directory()?;
    set_env_vars();
    execute_command("cargo", "build", &cargo_args);
    process_wasm_files(&target_dir)
}

fn run_or_test_command(command: &str, near: Option<String>, cargo_args: Vec<String>) -> Result<()> {
    setup_near_sandbox_if_needed(&near)?;
    let (cargo_args, binary_args) = split_args(cargo_args);
    let executables = if command == "test" {
        build_test_binaries(cargo_args)?
    } else {
        build_binaries(cargo_args)?
    };

    let target_dir = prepare_target_directory()?;
    set_env_vars();
    for binary in executables {
        println!("Running binary: {}", binary);
        execute_command(&binary, "", &binary_args);
    }
    process_wasm_files(&target_dir)
}

fn merge_command(llvm_profdata_args: Vec<String>) -> Result<()> {
    let profdata_dir = dir::get_profdata_dir();
    for entry in fs::read_dir(dir::get_profraw_dir())? {
        let dir_path = entry?.path();
        if !dir_path.is_dir() || fs::read_dir(&dir_path)?.next().is_none() {
            continue;
        }

        let dir_name = dir_path.file_name().unwrap().to_str().unwrap();
        println!("Merging profraw files for {}", dir_name);

        let profdata_path = profdata_dir.join(format!("{}.profdata", dir_name));
        report::merge_profraw_to_profdata(&dir_path, &profdata_path, llvm_profdata_args.clone())?;

        println!("Profdata file has been saved to {:?}", profdata_path);
    }
    Ok(())
}

fn report_command(llvm_cov_args: Vec<String>) -> Result<()> {
    let target_dir = dir::get_target_dir();

    merge_command(Vec::new())?;

    for entry in fs::read_dir(dir::get_profdata_dir())? {
        let file_path = entry?.path();
        if file_path.is_dir() {
            continue;
        }

        let file_name = file_path.with_extension("");
        let file_name = file_name.file_name().unwrap().to_str().unwrap();
        println!("Generating coverage report for {}", file_name);

        let object_file = match utils::find_file(
            &target_dir,
            &[
                &format!("{}.o", file_name.replace("-", "_")),
                &format!("{}.o", file_name),
            ],
        ) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("Warning: object file not found for {:?}", file_name);
                eprintln!("Object files should be placed in the wasmcov target directory");
                continue;
            }
        };

        let report_dir = dir::get_report_dir().join(file_name);
        report::generate_report(&file_path, &object_file, &report_dir, &llvm_cov_args)?;
        println!("Coverage report has been saved to {:?}", report_dir);
    }
    Ok(())
}

fn clean_command(all: bool) -> Result<()> {
    dir::clean_wasmcov_directory(all)
}

fn set_env_vars() {
    env::set_var(
        "CARGO_ENCODED_RUSTFLAGS",
        build::get_build_flags().join("\x1f"),
    );
    let (is_nightly, _version) = llvm::check_rustc_version().expect("Failed to run rustc command");
    if !is_nightly {
        println!("Setting RUSTUP_TOOLCHAIN to nightly");
        env::set_var("RUSTUP_TOOLCHAIN", "nightly");
    }
}

fn execute_command(command: &str, subcommand: &str, args: &[String]) -> String {
    let output = Command::new(command)
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

fn setup_near_sandbox_if_needed(near: &Option<String>) -> Result<()> {
    if let Some(version) = near {
        let near_sandbox_dir = dir::get_wasmcov_dir().join("near_sandbox");
        let neard_path = near_sandbox::setup_near_sandbox(near_sandbox_dir, version.clone())?;
        env::set_var("NEAR_SANDBOX_BIN_PATH", neard_path);
    }
    Ok(())
}

fn build_binaries(cargo_args: Vec<String>) -> Result<Vec<String>> {
    execute_command("cargo", "build", &cargo_args);
    parse_cargo_output("build", &cargo_args)
}

fn build_test_binaries(mut cargo_args: Vec<String>) -> Result<Vec<String>> {
    cargo_args.push("--no-run".to_string());
    execute_command("cargo", "test", &cargo_args);
    parse_cargo_output("test", &cargo_args)
}

fn parse_cargo_output(command: &str, extra_args: &[String]) -> Result<Vec<String>> {
    let mut args = vec![command, "--message-format=json"];
    args.extend(extra_args.iter().map(String::as_str));
    let output = Command::new("cargo").args(&args).output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to build {} binaries", command));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|json| {
            if json["reason"] == "compiler-artifact"
                && json["profile"]["test"] == (command == "test")
            {
                json["executable"].as_str().map(String::from)
            } else {
                None
            }
        })
        .collect())
}

fn prepare_target_directory() -> Result<PathBuf> {
    let target_dir = dir::get_target_dir();
    for wasm_file in build::find_wasm_files_with_coverage(&target_dir)? {
        fs::remove_file(&wasm_file)?;
    }
    env::set_var("CARGO_TARGET_DIR", &target_dir);
    Ok(target_dir)
}

fn process_wasm_files(target_dir: &PathBuf) -> Result<()> {
    for wasm_file in build::find_wasm_files_with_coverage(target_dir)? {
        println!("Processing WASM file: {:?}", wasm_file);
        match build::find_ll_file(&wasm_file) {
            Ok(ll_file) => {
                let new_ll_file = target_dir.join(ll_file.file_name().unwrap());
                build::correct_ll_file(&ll_file, &new_ll_file)?;

                let obj_file = target_dir.join(format!(
                    "{}.o",
                    new_ll_file.file_stem().unwrap().to_str().unwrap()
                ));
                build::compile_ll_file(&new_ll_file, &obj_file)?;

                let wasm_file_target = target_dir.join(wasm_file.file_name().unwrap());
                fs::copy(&wasm_file, &wasm_file_target)?;
            }
            Err(_) => {
                eprintln!("Warning: LL file not found for {:?}", wasm_file);
            }
        }
    }
    println!("Processed files have been saved to {:?}", target_dir);
    Ok(())
}

fn split_args(args: Vec<String>) -> (Vec<String>, Vec<String>) {
    match args.iter().position(|arg| arg == "--") {
        Some(index) => {
            let (cargo_args, binary_args) = args.split_at(index);
            (cargo_args.to_vec(), binary_args[1..].to_vec())
        }
        None => (args, Vec::new()),
    }
}
