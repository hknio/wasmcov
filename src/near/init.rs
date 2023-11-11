use std::env;
use std::fs;
use std::io::Result;
use std::path::Path;
use std::process::Command;

use crate::common::get_wasmcov_dir;

const NEARD_BINARY: &'static [u8] = include_bytes!("../../bin/neard");

fn init_neard() -> Result<()> {
    let neard_path = get_wasmcov_dir()
        .expect("Failed to get wasmcov directory")
        .join("bin/neard");

    if !Path::new(&neard_path).exists() {
        fs::create_dir_all(neard_path.parent().unwrap()).unwrap();
        fs::write(&neard_path, NEARD_BINARY).expect("Failed to write neard binary");
    }

    let near_path = get_wasmcov_dir()
        .expect("Failed to get wasmcov directory")
        .join(".near");

    let status: std::process::ExitStatus = Command::new(neard_path)
        .arg("--home")
        .arg(near_path)
        .arg("init")
        .status()
        .expect("Failed to run neard init");

    if !status.success() {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to run neard init",
        ))?;
    }

    Ok(())
}

fn init_config() {
    let config_path = env::current_dir()
        .expect("Failed to get current directory")
        .join(".cargo/config");

    // Read the current contents of the .cargo/config file
    let config_content = fs::read_to_string(&config_path).unwrap();

    // Append or update the [profile.coverage] section
    let new_config_content = format!(
        "{}\n[profile.coverage]\ninherits = \"release\"\nstrip = true\ncodegen-units = 1\nopt-level = \"z\"\ndebug = false\npanic = \"abort\"\noverflow-checks = true\n",
        config_content
    );

    // Write the modified content back to the .cargo/config file
    fs::write(&config_path, new_config_content).expect("Failed to write .cargo/config");
}

pub fn init() {
    init_neard();
    init_config();
}
