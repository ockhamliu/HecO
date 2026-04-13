use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleType {
    Entry,
    Feature,
    Har,
    Shared,
    Unknown,
}

impl ModuleType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "entry" => ModuleType::Entry,
            "feature" => ModuleType::Feature,
            "har" => ModuleType::Har,
            "shared" => ModuleType::Shared,
            _ => ModuleType::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub module_type: ModuleType,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub modules: Vec<Module>,
    pub products: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProjectBuildProfile {
    app: App,
    #[serde(rename = "modules")]
    modules: Vec<ProjectModuleInfo>,
}

#[derive(Debug, Deserialize)]
struct App {
    products: Vec<Product>,
}

#[derive(Debug, Deserialize)]
struct Product {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ProjectModuleInfo {
    name: String,
    #[serde(rename = "srcPath")]
    src_path: String,
    targets: Option<Vec<ModuleTarget>>,
}

#[derive(Debug, Deserialize)]
struct ModuleTarget {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ModuleJson5 {
    module: Option<ModuleInfo>,
}

#[derive(Debug, Deserialize)]
struct ModuleInfo {
    #[serde(rename = "type")]
    module_type: Option<String>,
}

impl Module {
    fn from_project_module_info(root: &Path, info: &ProjectModuleInfo) -> Self {
        let src_path_trimmed = info.src_path.strip_prefix("./").unwrap_or(&info.src_path);
        let module_path = root.join(src_path_trimmed);
        let name = info.name.clone();

        let targets: Vec<String> = info
            .targets
            .as_ref()
            .map(|t| t.iter().map(|target| target.name.clone()).collect())
            .unwrap_or_default();

        let module_type = if module_path.join("src/main/module.json5").exists() {
            if let Ok(content) = std::fs::read_to_string(module_path.join("src/main/module.json5"))
            {
                if let Ok(json) = serde_json5::from_str::<ModuleJson5>(&content) {
                    if let Some(module_info) = json.module {
                        let module_type = module_info
                            .module_type
                            .as_deref()
                            .map(ModuleType::from_str)
                            .unwrap_or(ModuleType::Entry);
                        return Module {
                            name,
                            module_type,
                            targets,
                        };
                    }
                }
            }
            ModuleType::Entry
        } else {
            ModuleType::Unknown
        };

        Module {
            name,
            module_type,
            targets,
        }
    }
}

impl Project {
    pub fn new(root: PathBuf) -> Self {
        Project {
            root,
            modules: Vec::new(),
            products: Vec::new(),
        }
    }

    pub fn discover_modules(&mut self) -> anyhow::Result<()> {
        self.modules.clear();
        self.products.clear();

        let build_profile_path = self.root.join("build-profile.json5");
        if !build_profile_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&build_profile_path)?;
        let project_build: ProjectBuildProfile = serde_json5::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse build-profile.json5: {}", e))?;

        for module_info in &project_build.modules {
            let module = Module::from_project_module_info(&self.root, module_info);
            self.modules.push(module);
        }

        for product in &project_build.app.products {
            self.products.push(product.name.clone());
        }

        Ok(())
    }

    pub fn find_module(&self, name: &str) -> Option<&Module> {
        self.modules.iter().find(|m| m.name == name)
    }

    pub fn validate_target(&self, module_name: &str, target_name: &str) -> anyhow::Result<()> {
        if let Some(module) = self.find_module(module_name) {
            if module.targets.is_empty() {
                anyhow::bail!("module '{}' has no targets defined", module_name);
            }
            if !module.targets.iter().any(|x| x == target_name) {
                anyhow::bail!(
                    "target '{}' not found in module '{}'\n\nAvailable targets:\n  {}",
                    target_name,
                    module_name,
                    module.targets.join("\n  ")
                );
            }
            Ok(())
        } else {
            anyhow::bail!("module '{}' not found", module_name);
        }
    }

    pub fn find_module_by_path(&self, current_dir: &Path) -> Option<&Module> {
        let current_dir = current_dir.canonicalize().ok()?;

        let build_profile_path = self.root.join("build-profile.json5");
        let content = std::fs::read_to_string(&build_profile_path).ok()?;
        let project_build: ProjectBuildProfile = serde_json5::from_str(&content).ok()?;

        for module_info in &project_build.modules {
            let src_path_trimmed = module_info
                .src_path
                .strip_prefix("./")
                .unwrap_or(&module_info.src_path);
            let module_path = self.root.join(src_path_trimmed);
            let module_path = module_path.canonicalize().ok()?;

            if current_dir.starts_with(&module_path) {
                return self.find_module(&module_info.name);
            }
        }

        None
    }

    pub fn validate_product(&self, product_name: &str) -> anyhow::Result<()> {
        if self.products.is_empty() {
            anyhow::bail!("项目中没有定义任何 product");
        }
        if !self.products.iter().any(|p| p == product_name) {
            anyhow::bail!(
                "product '{}' 不存在\n\n可用的 products:\n  {}",
                product_name,
                self.products.join("\n  ")
            );
        }
        Ok(())
    }
}

/// 检查目录是否为有效的项目根目录
/// 条件：同时包含 build-profile.json5 和 oh-package.json5，
/// 且 build-profile.json5 中必须包含 modules 字段
fn check_project_root(path: &std::path::Path) -> Option<PathBuf> {
    let build_profile_path = path.join("build-profile.json5");
    let oh_package_path = path.join("oh-package.json5");

    // 检查两个文件是否都存在
    if !build_profile_path.exists() || !oh_package_path.exists() {
        return None;
    }

    // 解析 build-profile.json5，检查是否包含 modules 字段
    let content = std::fs::read_to_string(&build_profile_path).ok()?;
    let build_profile: serde_json5::Result<ProjectBuildProfile> = serde_json5::from_str(&content);

    match build_profile {
        Ok(profile) => {
            if !profile.modules.is_empty() {
                Some(path.to_path_buf())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn find_project_root() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;

    // 从当前目录向上查找，找到同时满足所有条件的目录
    // 如果有多个满足条件的目录，选择最高层（最靠近根目录）的那个
    let mut best_root: Option<PathBuf> = None;

    for path in current_dir.ancestors() {
        if let Some(root) = check_project_root(path) {
            // 更新 best_root，因为我们想要最高层（最靠近根目录）的目录
            best_root = Some(root);
        }
    }

    best_root.or(Some(current_dir))
}

pub fn load_project() -> anyhow::Result<Project> {
    let root = find_project_root().ok_or_else(|| anyhow::anyhow!("未找到HarmonyOS项目根目录"))?;

    let mut project = Project::new(root);
    project.discover_modules()?;

    Ok(project)
}
