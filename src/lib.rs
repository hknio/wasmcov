use std::env;
use std::fs;
use std::io;
use std::io::Result;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

pub fn set_wasmcov_dir(wasmcov_dir: Option<PathBuf>) {
    // Set the directory used to store coverage data.
    // If no directory is specified, use the default directory.
    let default_directory = env::current_dir().unwrap().join("wasmcov");
    let coverage_directory = wasmcov_dir.unwrap_or(default_directory);

    // Set the directory that wasm-cov will store coverage data in.
    env::set_var("WASMCOV_DIR", &coverage_directory);

    // Create the coverage directory if it does not exist.
    // Also create bin and profraw subdirectories.
    if !Path::new(&coverage_directory).exists() {
        fs::create_dir_all(&coverage_directory).unwrap();
        fs::create_dir_all(&coverage_directory.join("bin")).unwrap();
        fs::create_dir_all(&coverage_directory.join("profraw")).unwrap();
    }

    // Include neard binar from bin/neard and write it to WASMCOV_DIR/bin/neard
    let neard_bin = include_bytes!("../bin/neard");
    let neard_bin_path = coverage_directory.join("bin").join("neard");
    fs::write(neard_bin_path, neard_bin).unwrap();
}

// Get the coverage directory from the WASMCOV_DIR environment variable.
// If that variable is not set, use the current directory.
pub fn get_wasmcov_dir() -> Result<PathBuf> {
    let default_directory = env::current_dir().unwrap().join("wasmcov");
    let coverage_directory = env::var("WASMCOV_DIR")
        .map(PathBuf::from)
        .unwrap_or(default_directory);

    if !Path::new(&coverage_directory).exists() {
        // Throw an error if the directory doesn't exist
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Coverage directory not found at {}",
                coverage_directory.display()
            ),
        ));
    }

    Ok(coverage_directory)
}

// This code writes a profile to disk in the profraw format. The profile is
// written to the profraw directory under the wasmcov directory. The file name
// is a UUID. The data is passed as a byte vector.
pub fn write_profraw(data: Vec<u8>) {
    let id = Uuid::new_v4();

    let wasmcov_dir = get_wasmcov_dir().unwrap();
    let profraw_dir = wasmcov_dir.join("profraw");
    if !Path::new(&profraw_dir).exists() {
        fs::create_dir_all(&profraw_dir).unwrap();
    }

    let profraw_path = profraw_dir.join(format!("{}.profraw", id));
    fs::write(profraw_path, data).unwrap();
}
