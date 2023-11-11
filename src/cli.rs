use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Environment {
    #[clap(subcommand)]
    Near(NearCommand),
    Radix,
    Init {
        #[arg(value_name = "WASMCOV_DIR")]
        home: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum NearCommand {
    Init {
        #[arg(value_name = "NEAR_HOME")]
        home: Option<PathBuf>,
    },
    Run {},
    Show {},
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// The first argument
    #[clap(subcommand)]
    pub environment: Environment,
}

pub fn build_cli() -> Cli {
    let cli: Cli = Cli::parse();
    cli
}
