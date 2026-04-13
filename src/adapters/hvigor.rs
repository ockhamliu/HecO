use crate::build::BuildArgs;
use crate::clean::CleanArgs;
use crate::command::CommandRunner;
use crate::config::Config;
use crate::project::{load_project, ModuleType};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};

impl BuildArgs {
    pub fn to_command_args(&self, project_root: &PathBuf) -> anyhow::Result<Vec<String>> {
        let mut args = Vec::new();

        let (module_name, target_name) = self.parse_module().unwrap_or((String::new(), None));

        args.push(resolve_task(&module_name, &target_name, project_root)?);

        if let Some(module) = &self.module {
            args.push("-p".to_string());
            args.push(format!("module={}", module));
        }

        let mode = if self.release { "release" } else { "debug" };
        args.push("-p".to_string());
        args.push(format!("buildMode={}", mode));

        // 默认禁用 daemon 以避免 uv_cwd 报错问题
        args.push("--no-daemon".to_string());

        Ok(args)
    }

    pub fn to_app_command_args(&self, product: &str) -> anyhow::Result<Vec<String>> {
        let mut args = Vec::new();

        args.push("assembleApp".to_string());

        let mode = if self.release { "release" } else { "debug" };
        args.push("-p".to_string());
        args.push(format!("buildMode={}", mode));
        args.push("-p".to_string());
        args.push(format!("product={}", product));

        // 默认禁用 daemon 以避免 uv_cwd 报错问题
        args.push("--no-daemon".to_string());

        Ok(args)
    }
}

pub fn build(args: &BuildArgs, project_root: &PathBuf, config: &Config) -> anyhow::Result<()> {
    let node_path = config
        .node_path()
        .ok_or_else(|| anyhow::anyhow!("未找到 Node 路径"))?;

    let hvigorw_js_path = config
        .hvigorw_js_path()
        .ok_or_else(|| anyhow::anyhow!("未找到 hvigorw.js 路径"))?;

    let sdk_path = config
        .sdk_path()
        .ok_or_else(|| anyhow::anyhow!("未找到 SDK 路径"))?;

    let java_path = config.java_path().ok_or_else(|| {
        anyhow::anyhow!("未找到 Java 路径，请确保 JAVA_HOME 环境变量已设置或 Java 在 PATH 中")
    })?;

    let java_home = java_path.parent().unwrap().parent().unwrap();
    let java_bin = java_home.join("bin");

    let current_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", java_bin.to_str().unwrap_or(""), current_path);

    let command_args = args.to_command_args(project_root)?;
    let runner = CommandRunner::new(project_root.clone())
        .env("DEVECO_SDK_HOME", sdk_path.to_str().unwrap_or(""))
        .env("JAVA_HOME", java_home.to_str().unwrap_or(""))
        .env("PATH", &new_path);

    let program_args: Vec<&str> = std::iter::once(hvigorw_js_path.to_str().unwrap())
        .chain(command_args.iter().map(|s| s.as_str()))
        .collect();

    let node_path_str = node_path.to_str().unwrap_or("node");

    if args.quiet {
        let output = runner.run_captured_merged(node_path_str, &program_args)?;
        if !output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            anyhow::bail!("{}", "error: build failed".red());
        }
        Ok(())
    } else {
        runner.run(node_path_str, &program_args)
    }
}

pub fn build_for_app(
    args: &BuildArgs,
    project_root: &Path,
    config: &Config,
    product: &str,
) -> anyhow::Result<()> {
    let node_path = config
        .node_path()
        .ok_or_else(|| anyhow::anyhow!("未找到 Node 路径"))?;

    let hvigorw_js_path = config
        .hvigorw_js_path()
        .ok_or_else(|| anyhow::anyhow!("未找到 hvigorw.js 路径"))?;

    let sdk_path = config
        .sdk_path()
        .ok_or_else(|| anyhow::anyhow!("未找到 SDK 路径"))?;

    let java_path = config.java_path().ok_or_else(|| {
        anyhow::anyhow!("未找到 Java 路径，请确保 JAVA_HOME 环境变量已设置或 Java 在 PATH 中")
    })?;

    let java_home = java_path.parent().unwrap().parent().unwrap();
    let java_bin = java_home.join("bin");

    let current_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", java_bin.to_str().unwrap_or(""), current_path);

    let command_args = args.to_app_command_args(product)?;
    let runner = CommandRunner::new(project_root.to_path_buf())
        .env("DEVECO_SDK_HOME", sdk_path.to_str().unwrap_or(""))
        .env("JAVA_HOME", java_home.to_str().unwrap_or(""))
        .env("PATH", &new_path);

    let program_args: Vec<&str> = std::iter::once(hvigorw_js_path.to_str().unwrap())
        .chain(command_args.iter().map(|s| s.as_str()))
        .collect();

    let node_path_str = node_path.to_str().unwrap_or("node");

    if args.quiet {
        let output = runner.run_captured_merged(node_path_str, &program_args)?;
        if !output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            anyhow::bail!("{}", "error: build failed".red());
        }
        Ok(())
    } else {
        runner.run(node_path_str, &program_args)
    }
}

fn resolve_task(
    module_name: &str,
    target_name: &Option<String>,
    project_root: &PathBuf,
) -> anyhow::Result<String> {
    if !module_name.is_empty() {
        if module_name == "app" {
            // 构建 assembleApp 包
            if let Some(product) = target_name {
                if let Ok(project) = load_project() {
                    project.validate_product(product)?;
                }
            }
            return Ok("assembleApp".to_string());
        }

        if let Ok(project) = load_project() {
            if project.root == *project_root {
                if let Some(m) = project.find_module(module_name) {
                    if let Some(target) = target_name {
                        project.validate_target(module_name, target)?;
                    }
                    return Ok(match m.module_type {
                        ModuleType::Har => "assembleHar".to_string(),
                        ModuleType::Shared => "assembleHsp".to_string(),
                        _ => "assembleHap".to_string(),
                    });
                } else {
                    let available: Vec<&str> =
                        project.modules.iter().map(|m| m.name.as_str()).collect();
                    let msg = format!(
                        "error: module '{}' not found in project\n\nAvailable modules:\n  {}",
                        module_name.red(),
                        available.join("\n  ")
                    );
                    anyhow::bail!("{}", msg);
                }
            }
        }
    }
    Ok("assembleHap".to_string())
}

pub fn clean(args: &CleanArgs, project_root: &Path, config: &Config) -> anyhow::Result<()> {
    let node_path = config
        .node_path()
        .ok_or_else(|| anyhow::anyhow!("未找到 Node 路径"))?;

    let hvigorw_js_path = config
        .hvigorw_js_path()
        .ok_or_else(|| anyhow::anyhow!("未找到 hvigorw.js 路径"))?;

    let sdk_path = config
        .sdk_path()
        .ok_or_else(|| anyhow::anyhow!("未找到 SDK 路径"))?;

    let command_args = ["clean", "--no-daemon"];
    let runner = CommandRunner::new(project_root.to_path_buf())
        .env("DEVECO_SDK_HOME", sdk_path.to_str().unwrap_or(""));

    let program_args: Vec<&str> = std::iter::once(hvigorw_js_path.to_str().unwrap())
        .chain(command_args.iter().copied())
        .collect();

    let node_path_str = node_path.to_str().unwrap_or("node");

    if args.quiet {
        let output = runner.run_captured_merged(node_path_str, &program_args)?;
        if !output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            anyhow::bail!("{}", "error: clean failed".red());
        }
        Ok(())
    } else {
        runner.run(node_path_str, &program_args)
    }
}
