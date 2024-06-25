use anyhow::Ok;
use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;

// Sets the WASMCOV_DIR environment variable to the wasmcov directory.
pub fn set_wasmcov_dir(wasmcov_dir: Option<&PathBuf>) -> PathBuf {
    let default_directory = &env::current_dir().unwrap().join("wasmcov");
    let coverage_directory = wasmcov_dir.unwrap_or(default_directory);

    env::set_var("WASMCOV_DIR", coverage_directory);
    if !coverage_directory.exists() {
        fs::create_dir_all(coverage_directory).expect("Failed to create coverage directory");
    }

    coverage_directory.to_owned()
}

// Get the coverage directory from the WASMCOV_DIR environment variable.
pub fn get_wasmcov_dir() -> PathBuf {
    let coverage_directory = env::var("WASMCOV_DIR")
        .map(PathBuf::from)
        .expect("WASMCOV_DIR is not set.");

    if !coverage_directory.exists() {
        panic!("WASMCOV_DIR {:?} does not exist.", coverage_directory);
    }

    coverage_directory
}

// Directory with the profraw files.
pub fn get_profraw_dir() -> PathBuf {
    let profraw_dir = get_wasmcov_dir().join("profraw");

    if !profraw_dir.exists() {
        fs::create_dir_all(&profraw_dir).expect("Failed to create profraw directory");
    }

    profraw_dir
}

// Directory with the profdata files.
pub fn get_profdata_dir() -> PathBuf {
    let profdata_dir = get_wasmcov_dir().join("profdata");
    if !profdata_dir.exists() {
        fs::create_dir_all(&profdata_dir).expect("Failed to create profdata directory");
    }
    profdata_dir
}

// Directory with the output files.
pub fn get_target_dir() -> PathBuf {
    let target_dir = get_wasmcov_dir().join("target");
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).expect("Failed to create target directory");
    }
    target_dir
}

// Directory with the report files.
pub fn get_report_dir() -> PathBuf {
    let report_dir = get_wasmcov_dir().join("report");
    if !report_dir.exists() {
        fs::create_dir_all(&report_dir).expect("Failed to create report directory");
    }
    report_dir
}

// Remove "profdata", "profraw", "report" directories from coverage directory
// or clear all files and directories from coverage directory when full is true.
pub fn clean_wasmcov_directory(full: bool) -> Result<()> {
    let coverage_dir = get_wasmcov_dir();

    if full {
        // Remove all files and directories from coverage_directory
        fs::remove_dir_all(&coverage_dir)?;
        fs::create_dir_all(&coverage_dir)?;
    } else {
        // Remove only profdata, report, and profraw
        let dirs_to_clear = vec![get_profdata_dir(), get_report_dir(), get_profraw_dir()];

        for dir in dirs_to_clear {
            if dir.exists() {
                fs::remove_dir_all(&dir)?;
                fs::create_dir_all(&dir)?;
            }
        }
    }

    Ok(())
}
