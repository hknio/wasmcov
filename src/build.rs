use crate::llvm;
use crate::utils::run_command;
use anyhow::anyhow;
use anyhow::Result;
use glob::glob;
use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn get_build_flags() -> Vec<&'static str> {
    vec![
        "--emit=llvm-ir",
        "-Cinstrument-coverage",
        "-Clto=off",
        "-Zlocation-detail=none",
        "-Zno-profiler-runtime",
    ]
}

pub fn find_wasm_files_with_coverage(dir: &PathBuf) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut matching_files = Vec::new();
    let pattern = format!("{}/**/deps/*.wasm", dir.to_str().unwrap());
    let search_pattern = b"__llvm_profile_init";

    for entry in glob(&pattern)? {
        let entry = entry?;
        let mut file = File::open(&entry)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if buffer
            .windows(search_pattern.len())
            .any(|window| window == search_pattern)
        {
            matching_files.push(entry);
        }
    }

    Ok(matching_files)
}

pub fn find_ll_file(wasm_file: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    // check if wasm_file_dir/deps/wasm_file_name.ll exists
    let wasm_file_dir = wasm_file.parent().unwrap();
    let wasm_file_name = wasm_file.file_stem().unwrap().to_str().unwrap();
    let ll_file = wasm_file_dir.join(format!("{}.ll", wasm_file_name));

    if !ll_file.exists() {
        return Err(anyhow!("LL file {:?} does not exist.", ll_file));
    }

    Ok(ll_file)
}

pub fn correct_ll_file(ll_file: &PathBuf, new_ll_file: &PathBuf) -> Result<(), anyhow::Error> {
    let mut ll_contents = String::new();

    File::open(&ll_file)
        .unwrap_or_else(|_| panic!("Failed to open LL file {:?}.", ll_file))
        .read_to_string(&mut ll_contents)?;

    let modified_ll_contents = Regex::new(r"(?ms)^(define[^\n]*\n).*?^}\s*$")
        .unwrap()
        .replace_all(&ll_contents, "${1}start:\n  unreachable\n}\n")
        .to_string();

    File::create(&new_ll_file)
        .unwrap_or_else(|_| panic!("Failed to open LL file {:?}", new_ll_file))
        .write_all(modified_ll_contents.as_bytes())?;

    Ok(())
}

pub fn compile_ll_file(ll_file: &PathBuf, obj_file: &PathBuf) -> Result<()> {
    run_command(
        &llvm::get_tooling()?.clang,
        &[
            "--target=x86_64-unknown-linux-gnu",
            "-Wno-override-module",
            "-c",
            "-o",
            &obj_file.to_str().unwrap(),
            &ll_file.to_str().unwrap(),
        ],
        None,
    )?;
    Ok(())
}
