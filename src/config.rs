use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "deveco_studio_root", default)]
    pub deveco_studio_root: Option<PathBuf>,
    #[serde(rename = "java_home", default)]
    pub java_home: Option<PathBuf>,
    #[serde(rename = "emulator_instance_path", default)]
    pub emulator_instance_path: Option<PathBuf>,
    #[serde(rename = "emulator_image_root", default)]
    pub emulator_image_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ConfigFile {
    #[serde(rename = "setup", default)]
    pub setup: Option<Config>,
}

impl Config {
    pub fn global_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".config").join("heco").join("config.toml"))
    }

    pub fn project_path() -> Option<PathBuf> {
        std::env::current_dir()
            .ok()
            .map(|p| p.join(".heco").join("config.toml"))
    }

    pub fn load(project_root: Option<&PathBuf>) -> anyhow::Result<Self> {
        let global_config = Self::load_from_file(Self::global_path());
        let project_config = match project_root {
            Some(root) => Self::load_from_file(Some(root.join(".heco").join("config.toml"))),
            None => Self::load_from_file(Self::project_path()),
        };

        let mut merged = Config::default();

        let highest = |a: Option<PathBuf>, b: Option<PathBuf>| -> Option<PathBuf> {
            if a.is_some() {
                a
            } else {
                b
            }
        };

        merged.deveco_studio_root = highest(
            project_config
                .as_ref()
                .and_then(|c| c.deveco_studio_root.clone()),
            global_config
                .as_ref()
                .and_then(|c| c.deveco_studio_root.clone()),
        );

        merged.java_home = highest(
            project_config.as_ref().and_then(|c| c.java_home.clone()),
            global_config.as_ref().and_then(|c| c.java_home.clone()),
        );

        merged.emulator_instance_path = highest(
            project_config
                .as_ref()
                .and_then(|c| c.emulator_instance_path.clone()),
            global_config
                .as_ref()
                .and_then(|c| c.emulator_instance_path.clone()),
        );

        merged.emulator_image_root = highest(
            project_config
                .as_ref()
                .and_then(|c| c.emulator_image_root.clone()),
            global_config
                .as_ref()
                .and_then(|c| c.emulator_image_root.clone()),
        );

        // 检查默认DevEco Studio路径（仅在macOS上）
        #[cfg(target_os = "macos")]
        {
            if merged.deveco_studio_root.is_none() {
                // 检查全局Applications目录
                let global_path = PathBuf::from("/Applications/DevEco-Studio.app");
                if global_path.exists() {
                    merged.deveco_studio_root = Some(global_path);
                } else {
                    // 检查用户目录下的Applications目录
                    if let Some(home) = dirs::home_dir() {
                        let user_path = home.join("Applications").join("DevEco-Studio.app");
                        if user_path.exists() {
                            merged.deveco_studio_root = Some(user_path);
                        }
                    }
                }
            }
        }

        Ok(merged)
    }

    fn load_from_file(path: Option<PathBuf>) -> Option<Config> {
        let path = path?;
        if !path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&path).ok()?;
        let config_file: ConfigFile = toml::from_str(&content).ok()?;
        config_file.setup
    }

    pub fn node_path(&self) -> Option<PathBuf> {
        if let Some(ref root) = self.deveco_studio_root {
            let node = root
                .join("Contents")
                .join("tools")
                .join("node")
                .join("bin")
                .join("node");
            if node.exists() {
                return Some(node);
            }
        }

        None
    }

    pub fn hvigorw_js_path(&self) -> Option<PathBuf> {
        if let Some(ref root) = self.deveco_studio_root {
            let hvigorw_js = root
                .join("Contents")
                .join("tools")
                .join("hvigor")
                .join("bin")
                .join("hvigorw.js");
            if hvigorw_js.exists() {
                return Some(hvigorw_js);
            }
        }

        None
    }

    pub fn sdk_path(&self) -> Option<PathBuf> {
        if let Some(ref root) = self.deveco_studio_root {
            let sdk = root.join("Contents").join("sdk");
            if sdk.exists() {
                return Some(sdk);
            }
        }

        // 然后尝试用户提供的默认 SDK 路径
        let home = std::env::var("HOME").unwrap_or_default();
        let default_sdk = PathBuf::from(&home)
            .join("Library")
            .join("Huawei")
            .join("Sdk");
        if default_sdk.exists() {
            return Some(default_sdk);
        }

        None
    }

    pub fn java_path(&self) -> Option<PathBuf> {
        if let Some(ref java_home) = self.java_home {
            let java = java_home.join("bin").join("java");
            if java.exists() {
                return Some(java);
            }
        }

        if let Ok(java_path) = std::env::var("JAVA_HOME") {
            let java = PathBuf::from(java_path).join("bin").join("java");
            if java.exists() {
                return Some(java);
            }
        }

        if let Ok(output) = std::process::Command::new("which").arg("java").output() {
            if output.status.success() {
                let java_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !java_path.is_empty() {
                    return Some(PathBuf::from(java_path));
                }
            }
        }

        None
    }

    pub fn emulator_path(&self) -> Option<PathBuf> {
        // 从 deveco_studio_root 构建 emulator 路径
        if let Some(ref root) = self.deveco_studio_root {
            // macOS 路径
            #[cfg(target_os = "macos")]
            let emulator_path = root
                .join("Contents")
                .join("tools")
                .join("emulator")
                .join("Emulator");

            // Windows 路径
            #[cfg(target_os = "windows")]
            let emulator_path = root.join("tools").join("emulator").join("Emulator.exe");

            // Linux 路径
            #[cfg(target_os = "linux")]
            let emulator_path = root.join("tools").join("emulator").join("Emulator");

            if emulator_path.exists() {
                return Some(emulator_path);
            }
        }

        // 尝试从环境变量 DEVECO_STUDIO_ROOT 查找
        if let Ok(deveco_root) = std::env::var("DEVECO_STUDIO_ROOT") {
            let root = PathBuf::from(&deveco_root);

            #[cfg(target_os = "macos")]
            let emulator_path = root
                .join("Contents")
                .join("tools")
                .join("emulator")
                .join("Emulator");

            #[cfg(target_os = "windows")]
            let emulator_path = root.join("tools").join("emulator").join("Emulator.exe");

            #[cfg(target_os = "linux")]
            let emulator_path = root.join("tools").join("emulator").join("Emulator");

            if emulator_path.exists() {
                return Some(emulator_path);
            }
        }

        None
    }

    pub fn codelinter_path(&self) -> Option<PathBuf> {
        // 从 deveco_studio_root 构建 codelinter 路径
        // 例如: /Applications/DevEco-Studio.app/Contents/plugins/codelinter/run/index.js
        if let Some(ref root) = self.deveco_studio_root {
            #[cfg(target_os = "macos")]
            let codelinter_path = root
                .join("Contents")
                .join("plugins")
                .join("codelinter")
                .join("run")
                .join("index.js");

            #[cfg(target_os = "windows")]
            let codelinter_path = root
                .join("plugins")
                .join("codelinter")
                .join("run")
                .join("index.js");

            #[cfg(target_os = "linux")]
            let codelinter_path = root
                .join("plugins")
                .join("codelinter")
                .join("run")
                .join("index.js");

            if codelinter_path.exists() {
                return Some(codelinter_path);
            }
        }

        None
    }

    pub fn get_emulator_instance_path(&self) -> Option<PathBuf> {
        // 如果配置文件中已设置，直接返回
        if let Some(ref path) = self.emulator_instance_path {
            return Some(path.clone());
        }

        // 尝试从默认位置查找（优先使用用户提供的路径）
        let home = std::env::var("HOME").unwrap_or_default();
        let default_paths = vec![PathBuf::from(&home)
            .join(".Huawei")
            .join("Emulator")
            .join("deployed")];

        default_paths.into_iter().find(|path| path.exists())
    }

    pub fn get_emulator_image_root(&self) -> Option<PathBuf> {
        // 如果配置文件中已设置，直接返回
        if let Some(ref path) = self.emulator_image_root {
            return Some(path.clone());
        }

        // 尝试从 SDK 路径查找
        if let Some(sdk_path) = self.sdk_path() {
            let image_path = sdk_path.join("system-images");
            if image_path.exists() {
                return Some(image_path);
            }
        }

        // 尝试从默认位置查找（优先使用用户提供的路径）
        let home = std::env::var("HOME").unwrap_or_default();
        let default_paths = vec![PathBuf::from(&home)
            .join("Library")
            .join("Huawei")
            .join("Sdk")];

        for path in default_paths {
            if path.exists() {
                // 如果是 Sdk 根目录，检查是否有 system-images 子目录
                let image_path = path.join("system-image");
                if image_path.exists() {
                    return Some(path);
                }
            }
        }
        None
    }
}
