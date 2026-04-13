use clap::Parser;

mod adapters;
mod build;
mod clean;
mod command;
mod config;
mod emulator;
mod lint;
mod project;
mod setup;

use crate::build::{handle_build, BuildArgs};
use clean::{handle_clean, CleanArgs};
use emulator::{handle_emulator, EmulatorArgs};
use lint::{handle_lint, LintArgs};
use setup::{handle_setup, SetupArgs};

#[derive(Parser, Debug)]
#[command(name = "heco")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    Build(BuildArgs),
    Clean(CleanArgs),
    Setup(SetupArgs),
    Lint(LintArgs),
    Emulator(EmulatorArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => {
            handle_build(args);
        }
        Commands::Clean(args) => {
            handle_clean(args);
        }
        Commands::Setup(args) => {
            handle_setup(args);
        }
        Commands::Lint(args) => {
            handle_lint(args)?;
        }
        Commands::Emulator(args) => {
            handle_emulator(args)?;
        }
    }
    Ok(())
}
