use crate::command::CommandRunner;
use crate::config::Config;
use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

#[derive(Args, Debug)]
pub struct EmulatorArgs {
    #[command(subcommand)]
    pub command: EmulatorCommands,
}

#[derive(Subcommand, Debug)]
pub enum EmulatorCommands {
    /// Start an emulator
    Start(StartArgs),
    /// Stop a running emulator
    Stop(StopArgs),
    /// List all emulators
    List(ListArgs),
}

#[derive(Args, Debug)]
pub struct StartArgs {
    /// Emulator name
    pub name: String,
}

#[derive(Args, Debug)]
pub struct StopArgs {
    /// Emulator name
    pub name: String,
    /// Force stop (kill process)
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct ListArgs {}

pub fn handle_emulator(args: EmulatorArgs) -> Result<()> {
    match args.command {
        EmulatorCommands::Start(start_args) => handle_start(start_args),
        EmulatorCommands::Stop(stop_args) => handle_stop(stop_args),
        EmulatorCommands::List(list_args) => handle_list(list_args),
    }
}

fn handle_start(args: StartArgs) -> Result<()> {
    println!("Starting emulator '{}'...", args.name);

    let emulator_cmd = find_emulator_binary()?;
    let config = Config::load(None)?;

    // 构建命令参数
    let mut cmd_args: Vec<String> = vec!["-hvd".to_string(), args.name.clone()];

    // 添加模拟器实例路径
    if let Some(instance_path) = config.get_emulator_instance_path() {
        cmd_args.extend_from_slice(&[
            "-path".to_string(),
            instance_path.to_str().unwrap().to_string(),
        ]);
        println!("  Instance path: {}", instance_path.display());
    } else {
        bail!("Could not find emulator instance path. Please configure it via 'heco setup'");
    }

    // 添加模拟器镜像路径
    if let Some(image_root) = config.get_emulator_image_root() {
        cmd_args.extend_from_slice(&[
            "-imageRoot".to_string(),
            image_root.to_str().unwrap().to_string(),
        ]);
        println!("  Image root: {}", image_root.display());
    } else {
        bail!("Could not find emulator image root. Please configure it via 'heco setup'");
    }

    // 创建 CommandRunner
    let runner = CommandRunner::new(std::env::current_dir()?);

    // 执行命令并设置超时
    let timeout = Duration::from_secs(2);

    // 启动命令执行
    let cmd_args_slice: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    let output = runner.run_captured_merged_with_timeout(
        emulator_cmd.to_str().unwrap(),
        &cmd_args_slice,
        Some(timeout),
    )?;

    // 处理执行结果
    let output_str = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        if output_str.contains("already exist") || output_str.contains("already running") {
            println!("⚠️  Emulator '{}' is already running", args.name);
        } else {
            println!("✓ Emulator '{}' started successfully", args.name);
        }
    } else {
        if output_str.contains("already exist") || output_str.contains("already running") {
            println!("⚠️  Emulator '{}' is already running", args.name);
        } else {
            bail!("Failed to start emulator: {}", output_str.trim());
        }
    }

    Ok(())
}

fn handle_stop(args: StopArgs) -> Result<()> {
    println!("Stopping emulator '{}'...", args.name);

    let emulator_cmd = find_emulator_binary()?;

    let mut cmd = Command::new(&emulator_cmd);
    cmd.arg("-stop").arg(&args.name);

    if args.force {
        println!("  Force stopping...");
    }

    let output = cmd.output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("{}", stdout);
        }
        println!("✓ Emulator '{}' stopped successfully", args.name);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // 如果模拟器已经停止，不算错误
        if stderr.contains("not running") || stderr.contains("stopped") {
            println!("✓ Emulator '{}' is already stopped", args.name);
        } else {
            bail!("Failed to stop emulator: {}", stderr);
        }
    }

    Ok(())
}

fn handle_list(_args: ListArgs) -> Result<()> {
    println!("Emulators:");

    let emulator_cmd = find_emulator_binary()?;

    let mut cmd = Command::new(&emulator_cmd);
    cmd.arg("-list");

    let output = cmd.output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.is_empty() || stdout.trim().is_empty() {
            println!("  No emulators found.");
            return Ok(());
        }

        let mut found_any = false;

        // 解析输出并显示
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 原生命令输出格式就是模拟器名称，如 "Mate 80"
            let name = line;
            found_any = true;
            println!("  {}", name);
        }
        if !found_any {
            println!("  No emulators found.");
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to list emulators: {}", stderr);
    }

    Ok(())
}

/// 查找 Emulator 可执行文件路径
fn find_emulator_binary() -> Result<PathBuf> {
    // 从 heco 配置中读取
    let config = Config::load(None)?;

    if let Some(emulator_path) = config.emulator_path() {
        return Ok(emulator_path);
    }

    bail!("Could not find Emulator binary. Please ensure DevEco Studio is installed and configured in heco setup, or set DEVECO_STUDIO_ROOT environment variable.")
}

