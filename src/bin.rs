use wasmcov::llvm::{unset_rustup_toolchain, verify_tooling, VerifyToolingResult};

fn main() {
    let wasmcov = clap::command!("wasmcov")
        .subcommand_required(true)
        .subcommand(clap::command!("verify-tooling").about("Verify tooling is installed"));

    let cmd = clap::Command::new("cargo")
        .bin_name("cargo")
        .subcommand_required(true)
        .subcommand(wasmcov);

    let matches = cmd.get_matches();

    let matches = matches.subcommand_matches("wasmcov").unwrap();

    // Should print help if no subcommand is provided to wasmcov

    match matches.subcommand().unwrap() {
        ("verify-tooling", _) => {
            println!("Checking if tooling is installed...");
            let result = verify_tooling();
            match result {
                Ok(VerifyToolingResult {
                    is_nightly,
                    llvm_major_version,
                    should_cleanup,
                }) => {
                    println!("Tooling is installed!");
                    println!("is_nightly: {}", is_nightly);
                    println!("llvm_major_version: {}", llvm_major_version);
                    if should_cleanup {
                        unset_rustup_toolchain(should_cleanup);
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        _ => unreachable!("clap should ensure we don't get here"),
    }
}

#[cfg(test)]
mod tests {}
