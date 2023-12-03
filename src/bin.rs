use std::path::PathBuf;

use clap::{arg, command};
use wasmcov::{finalize, post_build, setup};

fn main() {
    let wasmcov = clap::command!("wasmcov")
        .subcommand_required(true)
        .subcommand(
            command!("setup")
            .about("Setup wasmcov, including a check for the required version of LLVM, environment variables, wasmcov directories etc.\nNeeds to be ran inside an eval block eval $(cargo wasmcov setup)")
            // .arg(
            //     arg!("--wasmcov-dir -d")
            //         .help("Specify the version of LLVM to use")
            //         .default_value(None)
            // )
      
        )
        .subcommand(
            command!("post-build")
            .about("Used after the build step to provide a path for the compiled WASM binary with coverage instrumentation.")
        )
        .subcommand(
            command!("finalize")
            .about("Finalizes the creation of coverage data after tests. Merges the coverage data, modifies all needed compiled artefacts. ")
            .arg(
                arg!("--wasmcov-dir -d")
                    .help("Specify the version of LLVM to use")
                    .default_value("12.0.0")
            )
        );

    let cmd = clap::Command::new("cargo")
        .bin_name("cargo")
        .subcommand_required(true)
        .subcommand(wasmcov);

    let matches = cmd.get_matches();

    let matches = matches.subcommand_matches("wasmcov").unwrap();

    // Should print help if no subcommand is provided to wasmcov

    match matches.subcommand().unwrap() {
        // Takes wasmcov_dir argument
        ("setup", args) => {
            let result = setup(args.get_one::<PathBuf>("wasmcov-dir"));
            // If show-env is used, print the environment variables to be used in eval block
            println!("{}", result.unwrap());
        }
        ("finalize", _) => {
            finalize();
        }
        ("post-build", _) => post_build(),
        _ => unreachable!("clap should ensure we don't get here"),
    }
}

#[cfg(test)]
mod tests {}
