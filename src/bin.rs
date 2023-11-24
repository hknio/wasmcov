use wasmcov::llvm::verify_tooling;

fn main() {
    let cmd = clap::Command::new("cargo")
        .bin_name("cargo")
        .subcommand_required(true)
        .subcommand(
            clap::command!("wasmcov")
                .about("Generate coverage reports for WASM targets")
                .version("0.1.0"),
        );

    let matches = cmd.get_matches();
    let matches = match matches.subcommand() {
        Some(("wasmcov", _)) => verify_tooling(),
        _ => unreachable!("clap should ensure we don't get here"),
    };
}
