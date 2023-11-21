use crate::dir::write_profraw;

// This function is called on NEAR call or view logs, which can be fetched using the logs()
// function on either an ExecutionResult or similar objects produced by near-workspaces/src/result.rs
// This call needs to be added to every function call definitiion.
pub fn near_coverage(logs: &Vec<&str>) {
    let coverage: Vec<u8> = near_sdk::base64::decode(&logs.last().unwrap()).unwrap();
    write_profraw(coverage);
}

#[cfg(test)]
mod tests {
    use crate::dir::set_wasmcov_dir;

    use super::*;
    use std::{fs, path::Path};
    use tempfile::tempdir;

    #[test]
    fn test_near_coverage() {
        // Set the WASMCOV_DIR environment variable to a temporary directory.
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        set_wasmcov_dir(Some(&temp_dir_path));

        // Encode some dummy data using near_sdk::base64::encode
        let dummy_data = "dummy data";
        let encoded_data = near_sdk::base64::encode(dummy_data.as_bytes());

        // Call near_coverage with the encoded data
        near_coverage(&vec![encoded_data.as_str()]);

        // The file name is a UUID, so we need to get the file name first.
        let profraw_dir = temp_dir_path.join("profraw");
        let file_name = fs::read_dir(&profraw_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .file_name()
            .into_string()
            .unwrap();
        let profraw_path = profraw_dir.join(file_name);

        // Assert that the profraw file exists and the data is correct
        assert!(Path::new(&profraw_path).exists());
        assert_eq!(fs::read_to_string(profraw_path).unwrap(), dummy_data);
    }
}
