use crate::llvm;
use crate::utils::run_command;

use anyhow::Result;

use glob::glob;
use std::path::PathBuf;

pub fn merge_profraw_to_profdata(profraw_dir: &PathBuf, profdata_path: &PathBuf) -> Result<()> {
    // find all .profraw files in the profraw directory
    let profraw_files: Vec<String> = glob(profraw_dir.join("*.profraw").to_str().unwrap())?
        .filter_map(|entry| entry.ok())
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    // Prepare the command arguments
    let mut args = vec![
        "merge".to_string(),
        "-sparse".to_string(),
        "-o".to_string(),
        profdata_path.to_str().unwrap().to_string(),
    ];
    args.extend(profraw_files);

    // Run the command
    run_command(
        &llvm::get_tooling()?.llvm_profdata,
        args.as_slice()
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>()
            .as_slice(),
        None,
    )?;

    Ok(())
}

pub fn generate_report(
    profdata_path: &PathBuf,
    object_file: &PathBuf,
    report_dir: &PathBuf,
    llvm_cov_args: &Vec<String>,
) -> Result<()> {
    let mut cov_args = vec![
        "show",
        "--instr-profile",
        profdata_path.to_str().unwrap(),
        object_file.to_str().unwrap(),
        "--output-dir",
        report_dir.to_str().unwrap(),
        "--show-instantiations=false",
        "--format=html",
        "-show-directory-coverage",
    ];
    cov_args.extend(
        llvm_cov_args
            .as_slice()
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>()
            .as_slice(),
    );
    run_command(&llvm::get_tooling()?.llvm_cov, cov_args.as_slice(), None)?;
    Ok(())
}
