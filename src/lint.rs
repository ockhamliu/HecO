use crate::command::CommandRunner;
use crate::config::Config;
use crate::project::{find_project_root, load_project};
use anyhow::{Context, Result};
use clap::Parser;
use owo_colors::OwoColorize;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "lint")]
pub struct LintArgs {
    #[arg(long, help = "自动修复可以修复的问题")]
    pub fix: bool,
    #[arg(
        long,
        value_delimiter = ',',
        help = "指定一个或多个 product 名称，使用逗号分隔"
    )]
    pub products: Option<Vec<String>>,
    #[arg(short, long, help = "安静模式，减少输出信息")]
    pub quiet: bool,
}

fn run_codelinter(
    project_root: &std::path::Path,
    config: &Config,
    check_path: &str,
    fix: bool,
    product: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let node_path = config
        .node_path()
        .context("找不到 Node 运行时。请检查 DevEco Studio 安装路径配置。")?;

    let codelinter_path = config
        .codelinter_path()
        .context("找不到 codelinter 工具。请检查 DevEco Studio 安装路径配置。")?;

    if !codelinter_path.exists() {
        return Err(anyhow::anyhow!(
            "找不到 codelinter 工具，路径: {}。请检查 DevEco Studio 安装路径配置。",
            codelinter_path.display()
        ));
    }

    let codelinter_str = codelinter_path.to_string_lossy().to_string();

    let mut cmd_args: Vec<String> = vec![codelinter_str];

    if fix {
        cmd_args.push("--fix".to_string());
    }

    if let Some(product_name) = product {
        cmd_args.push("-p".to_string());
        cmd_args.push(product_name.to_string());
    }

    if !check_path.is_empty() && check_path != "." {
        cmd_args.push(check_path.to_string());
    }

    let cmd_args_ref: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    let node_path_str = node_path.to_str().unwrap_or("node");
    let runner = CommandRunner::new(project_root.to_path_buf());
    
    if quiet {
        let output = runner.run_captured_merged(node_path_str, &cmd_args_ref)?;
        if !output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            anyhow::bail!("error: lint failed");
        }
        Ok(())
    } else {
        runner.run(node_path_str, &cmd_args_ref)
    }
}

pub fn handle_lint(args: LintArgs) -> Result<()> {
    let project_root = find_project_root()
        .context("未找到 HarmonyOS 项目根目录（缺少 build-profile.json5 或 oh-package.json5）")?;

    let project = load_project().context("无法加载项目信息")?;

    let config = Config::load(Some(&project_root))?;

    let current_dir = std::env::current_dir()?;

    let relative_path = current_dir
        .strip_prefix(&project_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let check_path = if relative_path.is_empty() || relative_path == "." {
        ".".to_string()
    } else {
        relative_path
    };

    let start = Instant::now();

    if let Some(ref products) = args.products {
        if !args.quiet {
            println!(
                "{} {} ({})",
                "Linting".green(),
                products.join(", "),
                project_root.display()
            );
        }

        for product in products {
            project.validate_product(product)?;
            run_codelinter(&project_root, &config, &check_path, args.fix, Some(product), args.quiet)?;
        }

        if !args.quiet {
            println!("\n{} in {:.2?}", "Finished".green(), start.elapsed());
        }
    } else {
        if !args.quiet {
            println!("{} ({})", "Linting".green(), project_root.display());
        }

        run_codelinter(&project_root, &config, &check_path, args.fix, None, args.quiet)?;

        if !args.quiet {
            println!("\n{} in {:.2?}", "Finished".green(), start.elapsed());
        }
    }

    Ok(())
}
