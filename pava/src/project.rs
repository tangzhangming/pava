use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const PROJECT_TOML_NAME: &str = "project.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectSection,
    #[serde(default)]
    pub paths: PathsSection,
    #[serde(default)]
    pub vendor: VendorSection,
    #[serde(default)]
    pub build: BuildSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSection {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathsSection {
    #[serde(default = "default_source_dirs")]
    pub source_dirs: Vec<String>,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

fn default_source_dirs() -> Vec<String> {
    vec!["src".to_string()]
}

fn default_output_dir() -> String {
    "target".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VendorSection {
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSection {
    pub main_class: String,
    #[serde(default)]
    pub fat_jar: bool,
}

impl Default for BuildSection {
    fn default() -> Self {
        Self {
            main_class: "Main".to_string(),
            fat_jar: false,
        }
    }
}

impl ProjectConfig {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn find_from_entry(entry_file: &Path) -> anyhow::Result<(Self, PathBuf)> {
        let mut current = entry_file.canonicalize()?;
        loop {
            let toml_path = current.join(PROJECT_TOML_NAME);
            if toml_path.exists() {
                let config = Self::from_file(&toml_path)?;
                return Ok((config, current));
            }
            if !current.pop() {
                break;
            }
        }
        anyhow::bail!("未找到 {} 文件", PROJECT_TOML_NAME)
    }

    pub fn is_project_class(&self, class_name: &str) -> bool {
        let root_package = self.project.name.replace(".", "/");
        class_name.starts_with(&root_package)
    }

    pub fn resolve_source_path(&self, class_name: &str, project_root: &Path) -> Option<PathBuf> {
        if !self.is_project_class(class_name) {
            return None;
        }
        let root_package = self.project.name.replace(".", "/");
        let relative_path = class_name.strip_prefix(&root_package)?;
        let relative_path = relative_path.trim_start_matches("/");
        for source_dir in &self.paths.source_dirs {
            let source_path = project_root
                .join(source_dir)
                .join(format!("{}.pava", relative_path));
            if source_path.exists() {
                return Some(source_path);
            }
        }
        None
    }

    pub fn resolve_vendor_jar(&self, class_name: &str, project_root: &Path) -> Option<PathBuf> {
        for (prefix, jar_path) in &self.vendor.dependencies {
            let class_prefix = prefix.replace(".", "/");
            if class_name.starts_with(&class_prefix) {
                let jar_full_path = project_root.join(jar_path);
                if jar_full_path.exists() {
                    return Some(jar_full_path);
                }
            }
        }
        None
    }

    pub fn is_java_stdlib(&self, class_name: &str) -> bool {
        class_name.starts_with("java/")
            || class_name.starts_with("javax/")
            || class_name.starts_with("sun/")
            || class_name.starts_with("com/sun/")
    }

    pub fn get_output_dir(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.paths.output_dir)
    }

    pub fn get_classes_dir(&self, project_root: &Path) -> PathBuf {
        self.get_output_dir(project_root).join("classes")
    }
}

pub fn generate_project_toml(package_name: &str) -> String {
    let config = ProjectConfig {
        project: ProjectSection {
            name: package_name.to_string(),
            version: "1.0.0".to_string(),
        },
        paths: PathsSection {
            source_dirs: vec!["src".to_string()],
            output_dir: "target".to_string(),
        },
        vendor: VendorSection {
            dependencies: HashMap::new(),
        },
        build: BuildSection {
            main_class: format!("{}.Main", package_name),
            fat_jar: false,
        },
    };
    toml::to_string_pretty(&config).unwrap_or_default()
}
