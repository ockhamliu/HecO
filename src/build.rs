use crate::adapters::hvigor;
use crate::config::Config;
use crate::project::find_project_root;
use clap::Parser;
use owo_colors::OwoColorize;
use std::time::Instant;

#[derive(Parser, Debug)]
pub struct BuildArgs {
    /// 模块名称 (格式: module 或 module@target)
    #[arg(short, long)]
    pub module: Option<String>,
    /// Debug 构建模式
    #[arg(long, conflicts_with = "release")]
    pub debug: bool,
    /// Release 构建模式
    #[arg(long, conflicts_with = "debug")]
    pub release: bool,
    #[arg(long, short)]
    pub quiet: bool,
}

impl BuildArgs {
    pub fn parse_module(&self) -> Option<(String, Option<String>)> {
        self.module.as_ref().map(|m| {
            if let Some(idx) = m.find('@') {
                let module_name = m[..idx].to_string();
                let target_name = m[idx + 1..].to_string();
                (module_name, Some(target_name))
            } else {
                (m.clone(), None)
            }
        })
    }
}

pub(crate) fn handle_build(args: BuildArgs) {
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
    let build_type = if args.release { "release" } else { "debug" };

    let args = if args.module.is_none() {
        if let Ok(project) = crate::project::load_project() {
            if let Ok(current_dir) = std::env::current_dir() {
                if let Some(module) = project.find_module_by_path(&current_dir) {
                    BuildArgs {
                        module: Some(module.name.clone()),
                        debug: args.debug,
                        release: args.release,
                        quiet: args.quiet,
                    }
                } else {
                    args
                }
            } else {
                args
            }
        } else {
            args
        }
    } else {
        args
    };

    let (module_name, target_name) = args.parse_module().unwrap_or((String::new(), None));

    if module_name.is_empty() {
        // 不在模块目录下，构建 assembleApp 包
        if let Ok(project) = crate::project::load_project() {
            if project.products.is_empty() {
                eprintln!("{}", "error: no products found in project".red());
                std::process::exit(1);
            }

            for product in &project.products {
                if !args.quiet {
                    println!(
                        "{} product ({}) ({})",
                        "Compiling".green(),
                        product,
                        project_root.display()
                    );
                }

                // 直接构建 assembleApp 包，不指定 module 参数
                let app_args = BuildArgs {
                    module: None,
                    debug: args.debug,
                    release: args.release,
                    quiet: args.quiet,
                };

                match hvigor::build_for_app(&app_args, &project_root, &config, product) {
                    Ok(_) => {
                        if !args.quiet {
                            println!(
                                "{} {} product ({}) in {:.2?}",
                                "Finished".green(),
                                build_type,
                                product,
                                start.elapsed()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "{}",
                            format!("error: build failed for product {}: {}", product, e).red()
                        );
                        std::process::exit(1);
                    }
                }
            }

            if !args.quiet {
                println!(
                    "\n{} {} product(s) in {:.2?}",
                    "Finished".green(),
                    build_type,
                    start.elapsed()
                );
            }
            return;
        }
    }

    let display_name = if let Some(target) = target_name {
        format!("{}@{}", module_name, target)
    } else {
        module_name
    };

    if !args.quiet {
        println!(
            "{} {} ({})",
            "Compiling".green(),
            display_name,
            project_root.display()
        );
    }

    match hvigor::build(&args, &project_root, &config) {
        Ok(_) => {
            if !args.quiet {
                println!(
                    "\n{} {} module(s) in {:.2?}",
                    "Finished".green(),
                    build_type,
                    start.elapsed()
                );
            }
        }
        Err(e) => {
            eprintln!("{}", format!("error: build failed: {}", e).red());
            std::process::exit(1);
        }
    }
}
