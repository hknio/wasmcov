use crate::dir::{get_output_dir, get_profraw_dir, get_wasmcov_dir};
use crate::run_command;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};

use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn merge_profraw_to_profdata(llvm_major_version: &str) -> Result<()> {
    let profraw_dir = get_profraw_dir()?;

    let command = format!(
        "llvm-profdata-{} merge -sparse {}/*.profraw -o {}/coverage.profdata",
        llvm_major_version,
        profraw_dir.to_str().unwrap(),
        profraw_dir.to_str().unwrap()
    );

    // TODO: Improve this ugly temp fix - otheriwe it throws can't find files error
    let output = run_command("sh", &["-c", command.as_str()])?;

    print!("{:?}", output);

    Ok(())
}

pub(crate) fn modify_ll_files() -> Result<()> {
    let output_dir = get_output_dir()?;

    // Modify all .ll files in the output directory, use glob
    for entry in glob::glob(output_dir.join("*.ll").to_str().unwrap()).unwrap() {
        match entry {
            Ok(path) => {
                let mut ll_contents = String::new();

                File::open(&path)
                    .expect(&format!("Failed to open LL file {:?}.", path))
                    .read_to_string(&mut ll_contents)?;

                let modified_ll_contents = Regex::new(r"(?ms)^(define[^\n]*\n).*?^}\s*$")
                    .unwrap()
                    .replace_all(&ll_contents, "${1}start:\n  unreachable\n}\n")
                    .to_string();

                File::create(&path)
                    .expect(&format!("Failed to open LL file {:?}", path))
                    .write_all(modified_ll_contents.as_bytes())?;
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}

pub(crate) fn generate_object_file(llvm_major_version: &str) -> Result<(), anyhow::Error> {
    let output_dir = get_output_dir()?;

    // Run on every .ll file in the output directory, use glob
    for entry in glob::glob(output_dir.join("*.ll").to_str().unwrap()).unwrap() {
        match entry {
            Ok(path) => {
                let name = path.file_stem().unwrap().to_str().unwrap();

                let output = run_command(
                    &format!("clang-{}", llvm_major_version),
                    &[
                        // name.ll is the input file
                        path.to_str().unwrap(),
                        "-Wno-override-module",
                        "-c",
                        "-o",
                        output_dir.join(format!("{}.ll.o", name)).to_str().unwrap(),
                    ],
                )?;
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}

pub(crate) fn generate_coverage_report(
    object_file: &Path,
    llvm_major_version: &str,
) -> Result<(), anyhow::Error> {
    let profdata_path = get_profraw_dir()?.join("coverage.profdata");
    let coverage_report_path = get_wasmcov_dir()?.join("coverage-report");
    let object_file_path = object_file;

    let output = run_command(
        &format!("llvm-cov-{}", llvm_major_version),
        &[
            "show",
            "--instr-profile",
            profdata_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid path"))?,
            object_file_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid path"))?,
            "--show-instantiations=false",
            "--format=html",
            "--output-dir",
            coverage_report_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid path"))?,
        ],
    )?;

    println!(
        "Coverage report was successfully generated, it is available in {:?} directory.",
        coverage_report_path
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::dir::get_output_dir;

    use super::*;

    #[test]
    fn test_merge_profraw_to_profdata() {
        std::env::set_var(
            "WASMCOV_DIR",
            std::env::current_dir().unwrap().join("tests"),
        );

        merge_profraw_to_profdata("16").unwrap();

        // Compare coverage.profdata and expected (bytes, not text)
        let mut profdata_contents = Vec::new();
        let mut profdata_expected_contents = Vec::new();
        File::open(get_profraw_dir().unwrap().join("coverage.profdata"))
            .expect("Failed to open coverage.profdata")
            .read_to_end(&mut profdata_contents)
            .unwrap();
        File::open(
            get_profraw_dir()
                .unwrap()
                .join("coverage-expected.profdata"),
        )
        .expect("Failed to open coverage-expected.profdata")
        .read_to_end(&mut profdata_expected_contents)
        .unwrap();

        assert_eq!(profdata_contents, profdata_expected_contents);

        // Clean up
        std::fs::remove_file(get_profraw_dir().unwrap().join("coverage.profdata")).unwrap();
    }

    #[test]
    fn test_modify_ll_file() {
        std::env::set_var(
            "WASMCOV_DIR",
            std::env::current_dir().unwrap().join("tests"),
        );

        let ll_path = Path::new("tests/output").join("fibonacci.ll");
        let ll_modified_path = Path::new("tests/output").join("fibonacci-modified.ll");
        let ll_expected_path = Path::new("tests/output").join("fibonacci-modified.ll");
        std::fs::copy(&ll_path, &ll_modified_path).unwrap();

        modify_ll_files().unwrap();

        // Compare fibonacci-modified.ll and expected
        let mut ll_modified_contents = String::new();
        let mut ll_expected_contents = String::new();
        File::open(&ll_modified_path)
            .expect(&format!("Failed to open LL file {:?}", ll_modified_path))
            .read_to_string(&mut ll_modified_contents)
            .unwrap();
        File::open(&ll_expected_path)
            .expect(&format!("Failed to open LL file {:?}", ll_expected_path))
            .read_to_string(&mut ll_expected_contents)
            .unwrap();
        assert_eq!(ll_modified_contents, ll_expected_contents);

        // Clean up
        std::fs::remove_file(&ll_modified_path).unwrap();
    }

    #[test]
    fn test_generate_object_file() {
        std::env::set_var(
            "WASMCOV_DIR",
            std::env::current_dir().unwrap().join("tests"),
        );

        generate_object_file("16").unwrap();

        // Compare fibonacci.ll.o and expected (bytes, not text)
        let mut object_file_contents = Vec::new();
        let mut object_file_expected_contents = Vec::new();
        File::open(get_output_dir().unwrap().join("fibonacci.ll.o"))
            .expect("Failed to open fibonacci.ll.o")
            .read_to_end(&mut object_file_contents)
            .unwrap();
        File::open(get_output_dir().unwrap().join("fibonacci-expected.ll.o"))
            .expect("Failed to open fibonacci-expected.ll.o")
            .read_to_end(&mut object_file_expected_contents)
            .unwrap();

        assert_eq!(object_file_contents, object_file_expected_contents);

        // Clean up
        std::fs::remove_file(get_output_dir().unwrap().join("fibonacci.ll.o")).unwrap();
    }

    #[test]
    fn generate_coverage_report() {}
}
