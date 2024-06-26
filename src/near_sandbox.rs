use crate::utils::{find_file, modify_file, run_command, FileOperation};
use anyhow::anyhow;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn modify_cargo_toml(file_path: PathBuf) -> std::io::Result<()> {
    let operation = FileOperation::ReplaceText {
        pattern: String::from("profile.quick-release"),
        replacement: String::from("profile.dev-release"),
    };
    modify_file(file_path, operation)
}

pub fn modify_near_vm_runner(file_path: PathBuf) -> std::io::Result<()> {
    let operation =
        FileOperation::AddBefore {
            pattern: String::from("if let Err(trap) = res {"),
            new_line: String::from(
                "if std::env::var(\"WASMCOV_DIR\").is_ok() {
                if let Some(near_vm_types::ExportIndex::Function(index)) = artifact.export_field(\"capture_coverage\") {
    if let Some(function) = instance.function_by_index(index) {
        instance.invoke_function(
            function.vmctx,
            function.call_trampoline.unwrap(),
            function.address,
            [].as_mut_ptr() as *mut _,
        ).expect(\"capture_coverage function should not fail\");
    }
}
}"),
        };
    modify_file(file_path, operation)
}

pub fn modify_wasmtime_runner(file_path: PathBuf) -> std::io::Result<()> {
    let operation =
        FileOperation::ReplaceText {
            pattern: String::from("Ok(run) => match run.call(&mut store, ()) {"),
            replacement: String::from(
                "Ok(run) => match (|| {
                    let result = run.call(&mut store, ());
                    if std::env::var(\"WASMCOV_DIR\").is_ok() {
                        if let Some(func) = instance.get_func(&mut store, \"capture_coverage\") {
                            if let Some(run) = func.typed::<(), ()>(&mut store).ok() {
                                run.call(&mut store, ()).expect(\"capture_coverage function should not fail\");
                            }
                        }
                    }
                    result
                })() {"),
        };
    modify_file(file_path, operation)
}

pub fn modify_imports(file_path: PathBuf) -> std::io::Result<()> {
    let operation = FileOperation::AddAfter {
            pattern: String::from("sandbox_debug_log<[len: u64, ptr: u64]"),
            new_line: String::from("##[\"sandbox\"] sandbox_capture_coverage<[file_len: u64, file_ptr: u64, coverage_len: u64, coverage_ptr: u64] -> []>,"),
        };
    modify_file(file_path, operation)
}

pub fn modify_logic(file_path: PathBuf) -> std::io::Result<()> {
    let operation =
        FileOperation::AddBefore {
            pattern: String::from("pub fn panic(&mut self)"),
            new_line: String::from("
            #[cfg(feature = \"sandbox\")]
            pub fn sandbox_capture_coverage(&mut self, file_len: u64, file_ptr: u64, coverage_len: u64, coverage_ptr: u64) -> Result<()> {
                use near_primitives_core::hash::CryptoHash;
                use std::path::PathBuf;
            
                let file = self.memory.view_for_free(MemSlice { ptr: file_ptr, len: file_len })?.into_owned();
                let file = String::from_utf8(file).expect(\"Invalid coverage file name\");
                let coverage = self.memory.view_for_free(MemSlice { ptr: coverage_ptr, len: coverage_len })?.into_owned();
        
                let wasmcov_dir = std::env::var(\"WASMCOV_DIR\").map(|s| PathBuf::from(s)).expect(\"WASMCOV_DIR is not set.\");
                if !wasmcov_dir.exists() {
                    panic!(\"WASMCOV_DIR {wasmcov_dir:?} does not exist.\");
                }
                let profraw_directory = wasmcov_dir.join(\"profraw\").join(file);
                let _ = std::fs::create_dir_all(&profraw_directory); // it may fail if multiple threads are trying to create the same directory
                
                const HEX_DIGITS: &[u8; 16] = b\"0123456789abcdef\";
                let mut coverage_hash = String::with_capacity(64);
                for &byte in CryptoHash::hash_bytes(&coverage).as_bytes() {
                    coverage_hash.push(HEX_DIGITS[(byte >> 4) as usize] as char);
                    coverage_hash.push(HEX_DIGITS[(byte & 0x0F) as usize] as char);
                }
            
                let file_path = profraw_directory.join(format!(\"{coverage_hash}.profraw\"));
                std::fs::write(&file_path, coverage).expect(\"Failed to write coverage file\");
                Ok(())
            }"),
        };
    modify_file(file_path, operation)
}

pub fn add_wasmcov_to_nearcore(nearcore_dir: &PathBuf) -> Result<()> {
    let near_vm_runner_path = find_file(
        &nearcore_dir,
        &[
            "runtime/near-vm-runner/src/near_vm_runner.rs",
            "runtime/near-vm-runner/src/near_vm_runner/runner.rs",
        ],
    )
    .map_err(|_| anyhow!("Could not find near_vm_runner.rs"))?;

    let imports_path = find_file(&nearcore_dir, &["runtime/near-vm-runner/src/imports.rs"])
        .map_err(|_| anyhow!("Could not find imports.rs"))?;
    let wasmtime_runner_path = find_file(
        &nearcore_dir,
        &["runtime/near-vm-runner/src/wasmtime_runner.rs"],
    )
    .map_err(|_| anyhow!("Could not find wasmtime_runner.rs"))?;

    let logic_path = find_file(
        &nearcore_dir,
        &[
            "runtime/near-vm-runner/src/logic/logic.rs",
            "runtime/near-vm-logic/src/logic.rs",
        ],
    )
    .map_err(|_| anyhow!("Could not find logic.rs"))?;

    let _ = modify_cargo_toml(nearcore_dir.join("Cargo.toml")); // it can fail
    modify_near_vm_runner(near_vm_runner_path)
        .map_err(|_| anyhow!("Failed to modify near_vm_runner.rs"))?;
    modify_imports(imports_path).map_err(|_| anyhow!("Failed to modify imports.rs"))?;
    modify_logic(logic_path).map_err(|_| anyhow!("Failed to modify logic.rs"))?;
    modify_wasmtime_runner(wasmtime_runner_path)
        .map_err(|_| anyhow!("Failed to modify wasmtime_runner.rs"))?;

    Ok(())
}

pub fn setup_near_sandbox(dir: PathBuf, version: String) -> Result<PathBuf> {
    let version = if version.is_empty() {
        "1.35.0".to_string()
    } else {
        version
    };

    let mut version_split = version.split(".");
    let version_major = version_split
        .next()
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or_default();
    let version_minor = version_split
        .next()
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or_default();
    if version_major != 1 || version_minor < 34 {
        eprintln!(
            "Version {} is not supported. Please use version 1.34.0 or higher.",
            version
        );
        std::process::exit(1);
    }

    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create location directory");
    }

    let neard_path = dir.join(format!("neard-{}", version));

    // If neard-version already exists, return its path
    if neard_path.exists() {
        return Ok(neard_path);
    }

    // Clone the repo
    let near_repository_dir = dir.join(&version);
    if !near_repository_dir.exists() {
        println!("Cloning nearcore version {}", version);
        run_command(
            "git",
            &[
                "clone",
                "--depth=1",
                "--branch",
                &version,
                "https://github.com/near/nearcore",
                near_repository_dir.to_str().unwrap(),
            ],
            None,
        )?;
        add_wasmcov_to_nearcore(&near_repository_dir)?;
    }

    // librocksdb-sys requires update because there are issues with the version in the lock file in older versions
    run_command(
        "cargo",
        &["update", "-p", "librocksdb-sys"],
        Some(&near_repository_dir),
    )?;
    println!("Building neard, it may take a while");
    run_command(
        "cargo",
        &[
            "build",
            "-p",
            "neard",
            "--locked",
            "--features",
            "sandbox",
            "--profile",
            "dev-release",
            "--target-dir",
            near_repository_dir.join("target").to_str().unwrap(),
        ],
        Some(&near_repository_dir),
    )?;

    // Copy the built neard to neard-version
    let source_path = near_repository_dir
        .join("target")
        .join("dev-release")
        .join("neard");
    fs::copy(&source_path, &neard_path).expect("Failed to copy neard binary");

    // make sure neard works
    run_command(neard_path.to_str().unwrap(), &["--version"], None)
        .expect(format!("Failed to run {}", neard_path.to_str().unwrap()).as_str());

    // remove the repo
    fs::remove_dir_all(&near_repository_dir).expect("Failed to remove near repository dir");

    Ok(neard_path)
}
