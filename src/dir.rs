use anyhow::Result;
use glob::glob;
use std::env;
use std::fs;
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
    env::set_var("CARGO_TARGET_DIR", &coverage_directory.join("target"));

    // Create the coverage directory if it does not exist.
    if !Path::new(&coverage_directory).exists() {
        fs::create_dir_all(&coverage_directory).unwrap();
    }
}

// Get the coverage directory from the WASMCOV_DIR environment variable.
// If that variable is not set, use the current directory.
pub fn get_wasmcov_dir() -> Result<PathBuf> {
    let default_directory = env::current_dir().unwrap().join("wasmcov");
    let coverage_directory = env::var("WASMCOV_DIR")
        .map(PathBuf::from)
        .unwrap_or(default_directory);

    if !Path::new(&coverage_directory).exists() {
        // Create it if it doesn't exist
        fs::create_dir_all(&coverage_directory).unwrap();
    }

    Ok(coverage_directory)
}

pub fn get_profraw_dir() -> Result<PathBuf> {
    let wasmcov_dir = get_wasmcov_dir().unwrap();
    let profraw_dir = wasmcov_dir.join("profraw");

    if !Path::new(&profraw_dir).exists() {
        // Create it if it doesn't exist
        fs::create_dir_all(&profraw_dir).unwrap();
    }

    Ok(profraw_dir)
}

pub fn get_output_dir() -> Result<PathBuf> {
    let wasmcov_dir = get_wasmcov_dir().unwrap();
    let output_dir = wasmcov_dir.join("output");

    if !Path::new(&output_dir).exists() {
        // Create it if it doesn't exist
        fs::create_dir_all(&output_dir).unwrap();
    }

    Ok(output_dir)
}

pub fn get_target_dir() -> Result<PathBuf> {
    let wasmcov_dir = get_wasmcov_dir().unwrap();
    let output_dir = wasmcov_dir.join("target");

    if !Path::new(&output_dir).exists() {
        // Create it if it doesn't exist
        fs::create_dir_all(&output_dir).unwrap();
    }

    Ok(output_dir)
}

// This code writes a profile to disk in the profraw format. The profile is
// written to the profraw directory under the wasmcov directory. The file name
// is a UUID. The data is passed as a byte vector.
pub fn write_profraw(data: Vec<u8>) {
    let id = Uuid::new_v4();

    let profraw_dir = get_profraw_dir().unwrap();
    if !Path::new(&profraw_dir).exists() {
        fs::create_dir_all(&profraw_dir).unwrap();
    }

    let profraw_path = profraw_dir.join(format!("{}.profraw", id));
    fs::write(profraw_path, data).unwrap();
}

// get all .wasm files in the target directory, copy them and .ll files of the same name to the output directory
// we are interested only in wasm files that are in the deps folder of the target directory
// but the deps folder is not a direct child of the target directory - it is likely to be in targetdirectory/wasm32-unknown-unknown/profilename/deps
// or in targetdirectory/profilename/deps so we need to
pub fn extract_compiled_artefacts() -> Result<()> {
    let target_dir = get_target_dir()?;
    let output_dir = get_output_dir()?;

    // get all .wasm files in a deps directory within the target directory
    let wasm_files = glob(&format!("{}/**/deps/*.wasm", target_dir.to_str().unwrap()))?;
    // for each wasm_files, copy the wasm file and the .ll file of the same name to the output directory, handle no ll file as error
    for wasm_file in wasm_files {
        let wasm_file = wasm_file?;
        let wasm_file_name = wasm_file.file_name().unwrap().to_str().unwrap();
        let wasm_file_stem = wasm_file.file_stem().unwrap().to_str().unwrap();
        let wasm_file_path = wasm_file.parent().unwrap();
        let ll_file_path = wasm_file_path.join(format!("{}.ll", wasm_file_stem));
        if !ll_file_path.exists() {
            return Err(anyhow::anyhow!(
                "No .ll file found for wasm file {}",
                wasm_file_name
            ));
        }
        let wasm_file_output_path = output_dir.join(wasm_file_name);
        let ll_file_output_path = output_dir.join(format!("{}.ll", wasm_file_stem));
        fs::copy(wasm_file, wasm_file_output_path)?;
        fs::copy(ll_file_path, ll_file_output_path)?;
    }
    Ok(())
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
    fn test_get_profraw_dir() {
        // Set the WASMCOV_DIR environment variable to a temporary directory.
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        set_wasmcov_dir(Some(&temp_dir_path));

        // Check that the directory exists.
        let profraw_dir = get_profraw_dir().unwrap();
        assert_eq!(profraw_dir, temp_dir_path.join("profraw"));

        // Clean up.
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_get_output_dir() {
        // Set the WASMCOV_DIR environment variable to a temporary directory.
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = &temp_dir.path().to_path_buf();
        set_wasmcov_dir(Some(&temp_dir_path));

        // Check that the directory exists.
        let output_dir = get_output_dir().unwrap();
        assert_eq!(output_dir, temp_dir_path.join("output"));
        assert!(Path::new(&output_dir).exists());

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
}
