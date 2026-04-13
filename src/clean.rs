use crate::adapters::hvigor;
use crate::config::Config;
use crate::project::find_project_root;
use clap::Parser;
use owo_colors::OwoColorize;
use std::time::Instant;

#[derive(Parser, Debug)]
pub struct CleanArgs {
    #[arg(long, short)]
    pub quiet: bool,
}

pub(crate) fn handle_clean(args: CleanArgs) {
    let project_root = match find_project_root() {
        Some(path) => path,
        None => {
            eprintln!(
                "{}",
                "error: no project root found (build-profile.json5)".red()
            );
            std::process::exit(1);
        }
    };

    let config = match Config::load(Some(&project_root)) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}", format!("error: failed to load config: {}", e).red());
            std::process::exit(1);
        }
    };

    let start = Instant::now();

    if !args.quiet {
        println!("{} ({})", "Cleaning".green(), project_root.display());
    }

    match hvigor::clean(&args, &project_root, &config) {
        Ok(_) => {
            if !args.quiet {
                println!("\n{} in {:.2?}", "Finished".green(), start.elapsed());
            }
        }
        Err(e) => {
            eprintln!("{}", format!("error: clean failed: {}", e).red());
            std::process::exit(1);
        }
    }
}
