use std::env;
use std::fs;
use std::io;
use std::io::Result;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

pub fn set_wasmcov_dir(wasmcov_dir: Option<&PathBuf>) {
    // Set the directory used to store coverage data.
    // If n --o directory is specified, use the default directory.
    let default_directory = &env::current_dir().unwrap().join("wasmcov");
    let coverage_directory = wasmcov_dir.unwrap_or(default_directory);

    // Set the directory that wasm-cov will store coverage data in.
    env::set_var("WASMCOV_DIR", &coverage_directory);

    // Create the coverage directory if it does not exist.
    if !Path::new(&coverage_directory).exists() {
        fs::create_dir_all(&coverage_directory).unwrap();
    }

    // If bin directory does not exist, create it
    if !Path::new(&coverage_directory.join("bin")).exists() {
        fs::create_dir_all(&coverage_directory.join("bin")).unwrap();
    }

    // If profraw directory does not exist, create it
    if !Path::new(&coverage_directory.join("profraw")).exists() {
        fs::create_dir_all(&coverage_directory.join("profraw")).unwrap();
    }

    // Removed due to crate size limits.
    // Include neard binar from bin/neard and write it to WASMCOV_DIR/bin/neard
    // let neard_bin = include_bytes!("../bin/neard");
    // let neard_bin_path = coverage_directory.join("bin").join("neard");
    // fs::write(neard_bin_path, neard_bin).expect("Failed to create neard binary");
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

// This function is called on NEAR call or view logs, which can be fetched using the logs()
// function on either an ExecutionResult or similar objects produced by near-workspaces/src/result.rs
// This call needs to be added to every function call definitiion.
#[cfg(feature = "near")]
pub fn near_coverage(&logs: Vec<&str>) {
    let coverage: Vec<u8> = near_sdk::base64::decode(&logs.last().unwrap()).unwrap();
    write_profraw(coverage);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_set_wasmcov_dir() {
        // Set the WASMCOV_DIR environment variable to a temporary directory.
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        set_wasmcov_dir(Some(&temp_dir_path));

        // Check that the directory exists.
        let wasmcov_dir = get_wasmcov_dir().unwrap();
        assert_eq!(&wasmcov_dir, &temp_dir_path);

        // Check that the bin and profraw subdirectories exist.
        let bin_dir = wasmcov_dir.join("bin");
        let profraw_dir: PathBuf = wasmcov_dir.join("profraw");
        assert!(Path::new(&bin_dir).exists());
        assert!(Path::new(&profraw_dir).exists());

        // Check that the neard binary exists.
        let neard_bin = wasmcov_dir.join("bin").join("neard");
        assert!(Path::new(&neard_bin).exists());

        // Clean up.
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_get_wasmcov_dir() {
        // Set the WASMCOV_DIR environment variable to a temporary directory.
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        set_wasmcov_dir(Some(&temp_dir_path));

        // Check that the directory exists.
        let wasmcov_dir = get_wasmcov_dir().unwrap();
        assert_eq!(wasmcov_dir, temp_dir_path);

        // Clean up.
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_write_profraw() {
        // Set the WASMCOV_DIR environment variable to a temporary directory.
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        set_wasmcov_dir(Some(&temp_dir_path));

        // Write a profile to disk.
        let data = vec![1, 2, 3];
        write_profraw(data);

        // Check that the profile exists.
        let profraw_dir = temp_dir_path.join("profraw");
        let profraw_files = fs::read_dir(profraw_dir).unwrap();
        let profraw_file = profraw_files.into_iter().next().unwrap().unwrap();
        let profraw_path = profraw_file.path();
        assert!(Path::new(&profraw_path).exists());
    }

    #[test]
    fn test_near_coverage() {
        // Set the WASMCOV_DIR environment variable to a temporary directory.
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        set_wasmcov_dir(Some(&temp_dir_path));

        // Write a profile to disk.
        let data = vec![1, 2, 3];
        write_profraw(data);

        // Check that the profile exists.
        let profraw_dir = temp_dir_path.join("profraw");
        let profraw_files = fs::read_dir(profraw_dir).unwrap();
        let profraw_file = profraw_files.into_iter().next().unwrap().unwrap();
        let profraw_path = profraw_file.path();
        assert!(Path::new(&profraw_path).exists());
    }
}
