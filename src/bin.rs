use clap::{arg, Arg};
use wasmcov::{finalize, post_build, setup};

fn main() {
    let wasmcov = clap::command!("wasmcov")
        .subcommand_required(true)
        .subcommand(
            clap::command!("setup")
            .about("Setup wasmcov, including a check for the required version of LLVM, environment variables, wasmcov directories etc.")
            // TODO: add in future version
            // .arg(
            //     arg!("--wasmcov-dir -d")
            //         .help("Specify the version of LLVM to use")
            //         .default_value("12.0.0")
            // )
        )
        .subcommand(
            clap::command!("post-build")
            .about("Used after the build step to provide a path for the compiled WASM binary with coverage instrumentation.")
        )
        .subcommand(
            clap::command!("finalize")
            .about("Used after the build step to finalize the coverage data. Merges the coverage data, modifies all needed compiled artefacts. ")
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
        ("setup", _) => {
            setup();
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
