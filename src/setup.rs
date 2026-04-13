use clap::Args;
use dialoguer::{Input, Select};



fn save_config(
    deveco_path: Option<&str>,
    java_path: Option<&str>,
    emulator_instance_path: Option<&str>,
    emulator_image_root: Option<&str>,
    is_global: bool,
) {
    let config_file = if is_global {
        dirs::home_dir()
            .expect("无法获取 HOME 目录")
            .join(".config")
            .join("heco")
            .join("config.toml")
    } else {
        std::env::current_dir()
            .expect("无法获取当前目录")
            .join(".heco")
            .join("config.toml")
    };

    if let Some(parent) = config_file.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            println!("创建配置目录失败：{}", e);
            return;
        }
    }

    let mut content = String::from("[setup]\n");

    if let Some(path) = deveco_path {
        content = content + &format!("deveco_studio_root = \"{}\"\n", path);
    }

    if let Some(path) = java_path {
        content = content + &format!("java_home = \"{}\"\n", path);
    }

    if let Some(path) = emulator_instance_path {
        content = content + &format!("emulator_instance_path = \"{}\"\n", path);
    }

    if let Some(path) = emulator_image_root {
        content = content + &format!("emulator_image_root = \"{}\"\n", path);
    }

    if let Err(e) = std::fs::write(&config_file, content) {
        println!("写入配置文件失败：{}", e);
        return;
    }

    println!("配置文件已保存：{:?}", config_file);
}



fn handle_deveco_studio_setup(is_global: bool) {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = vec![
        "/Applications/DevEco-Studio.app".to_string(),
        format!("{}/Applications/DevEco-Studio.app", home),
    ];

    let default_paths: Vec<String> = candidates
        .into_iter()
        .filter(|p| std::path::Path::new(p).exists())
        .collect();

    if default_paths.is_empty() {
        println!("错误: 未找到 DevEco Studio，请手动输入路径");
        return;
    }

    let welcome = "已识别的可能的 DevEco Studio 路径：".to_string() + &default_paths.join("\n");
    let input = Input::new()
        .with_prompt(format!("{} 输入目录 (回车使用第一个)", welcome))
        .default(default_paths[0].clone())
        .interact()
        .unwrap_or_else(|_| default_paths[0].clone());

    save_config(Some(&input), None, None, None, is_global);
    println!("成功配置 DevEco Studio: {}", input);
}



fn handle_emulator_setup(is_global: bool) {
    let home = std::env::var("HOME").unwrap_or_default();

    // 默认模拟器实例路径
    let default_instance_paths = [
        format!("{}/Library/Huawei/Emulator", home),
        format!("{}/.huawei/emulator", home),
    ];

    let instance_welcome = "已识别的可能的模拟器实例路径：".to_string()
        + &default_instance_paths
            .iter()
            .filter(|p| std::path::Path::new(p).exists())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

    let instance_path = Input::new()
        .with_prompt(format!(
            "{} 输入模拟器实例路径 (回车使用第一个或跳过)",
            instance_welcome
        ))
        .default(
            default_instance_paths
                .iter()
                .find(|p| std::path::Path::new(p).exists())
                .cloned()
                .unwrap_or(String::new()),
        )
        .interact()
        .unwrap_or_default();

    // 默认模拟器镜像路径
    let default_image_paths = [
        format!("{}/Library/Huawei/Sdk/system-images", home),
        format!("{}/.huawei/sdk/system-images", home),
    ];

    let image_welcome = "已识别的可能的模拟器镜像路径：".to_string()
        + &default_image_paths
            .iter()
            .filter(|p| std::path::Path::new(p).exists())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

    let image_path = Input::new()
        .with_prompt(format!(
            "{} 输入模拟器镜像路径 (回车使用第一个或跳过)",
            image_welcome
        ))
        .default(
            default_image_paths
                .iter()
                .find(|p| std::path::Path::new(p).exists())
                .cloned()
                .unwrap_or(String::new()),
        )
        .interact()
        .unwrap_or_default();

    let instance_path_opt = if instance_path.is_empty() {
        None
    } else {
        Some(instance_path.as_str())
    };
    let image_path_opt = if image_path.is_empty() {
        None
    } else {
        Some(image_path.as_str())
    };

    save_config(
        None,
        None,
        instance_path_opt,
        image_path_opt,
        is_global,
    );

    println!("成功配置模拟器路径");
}

#[derive(Args, Debug)]
pub struct SetupArgs {
    #[arg(long, short)]
    pub scope: Option<String>,
}

pub fn handle_setup(args: SetupArgs) {
    let is_global = match args.scope.as_deref() {
        Some("global") => true,
        Some("project") => false,
        _ => {
            let scope_select = Select::new()
                .with_prompt("请选择配置范围：")
                .items(&["全局", "项目"])
                .default(0)
                .interact()
                .unwrap_or(0);
            scope_select == 0
        }
    };

    if is_global {
        println!("设置 Heco 配置");
        println!("范围：全局");
    } else {
        println!("设置 Heco 配置");
        println!("范围：项目");
    }

    let install_type = Select::new()
        .with_prompt("请选择配置项：")
        .items(&[
            "DevEco Studio (DevEco Studio 安装目录)",
            "模拟器路径 (模拟器实例和镜像路径)",
            "跳过配置",
        ])
        .default(0)
        .interact()
        .unwrap_or(2);

    match install_type {
        0 => handle_deveco_studio_setup(is_global),
        1 => handle_emulator_setup(is_global),
        _ => println!("已跳过配置"),
    }
}
