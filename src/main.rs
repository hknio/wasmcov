mod cli;
mod common;
mod near;

use cli::{Environment, NearCommand};

fn main() {
    let cli = cli::build_cli();

    match &cli.environment {
        Environment::Init { home } => {
            common::set_wasmcov_dir(home);
        }
        Environment::Radix => {}
        Environment::Near(command) => match command {
            NearCommand::Init { home } => {
                common::set_wasmcov_dir(home);
                near::init();
            }
            NearCommand::Run {} => {}
            NearCommand::Show {} => {}
        },
    }
}
