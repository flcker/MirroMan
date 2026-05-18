use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::mirror::Mirror;

// ── 配置结构体 ────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    /// 默认包管理器（进入 TUI 时自动选中）
    #[serde(default)]
    pub default_manager: Option<String>,
    /// 镜像测速超时（秒）
    #[serde(default = "default_timeout")]
    pub test_timeout_secs: u64,
}

fn default_timeout() -> u64 {
    5
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_manager: None,
            test_timeout_secs: 5,
        }
    }
}

// ── 各包管理器配置 ────────────────────────────────────────

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CargoConfig {
    #[serde(default)]
    pub mirrors: Vec<Mirror>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NpmConfig {
    #[serde(default)]
    pub mirrors: Vec<Mirror>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct HomebrewConfig {
    #[serde(default)]
    pub mirrors: Vec<Mirror>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PipConfig {
    #[serde(default)]
    pub mirrors: Vec<Mirror>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GoConfig {
    #[serde(default)]
    pub mirrors: Vec<Mirror>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PacmanConfig {
    #[serde(default)]
    pub mirrors: Vec<Mirror>,
}

/// 顶层配置，所有包管理器的镜像源汇总
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub cargo: CargoConfig,
    #[serde(default)]
    pub npm: NpmConfig,
    #[serde(default)]
    pub homebrew: HomebrewConfig,
    #[serde(default)]
    pub pip: PipConfig,
    #[serde(default)]
    pub goenv: GoConfig,
    #[serde(default)]
    pub pacman: PacmanConfig,
}

// ── 配置路径 ──────────────────────────────────────────────

fn config_dir() -> PathBuf {
    let proj_dirs =
        ProjectDirs::from("com", "mirroman", "MirroMan").expect("无法确定项目配置目录");
    proj_dirs.config_dir().to_path_buf()
}

fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// 备份根目录
pub fn backup_dir() -> PathBuf {
    config_dir().join("backups")
}

// ── 默认配置 ──────────────────────────────────────────────

fn default_config() -> Config {
    Config {
        settings: Settings::default(),
        cargo: CargoConfig {
            mirrors: vec![
                Mirror::new("rsproxy（默认）", "sparse+https://rsproxy.cn/index/"),
                Mirror::new(
                    "清华 TUNA",
                    "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git",
                ),
                Mirror::new(
                    "中科大 USTC",
                    "https://mirrors.ustc.edu.cn/crates.io-index",
                ),
                Mirror::new(
                    "crates.io 官方",
                    "https://github.com/rust-lang/crates.io-index",
                ),
            ],
        },
        npm: NpmConfig {
            mirrors: vec![
                Mirror::new("npm 官方", "https://registry.npmjs.org"),
                Mirror::new("淘宝 npmmirror", "https://registry.npmmirror.com"),
                Mirror::new(
                    "清华 TUNA",
                    "https://mirrors.tuna.tsinghua.edu.cn/npm/",
                ),
            ],
        },
        homebrew: HomebrewConfig {
            mirrors: vec![
                Mirror::new("Homebrew 官方", "https://github.com/Homebrew/brew"),
                Mirror::new(
                    "中科大 USTC",
                    "https://mirrors.ustc.edu.cn/brew.git",
                ),
                Mirror::new(
                    "清华 TUNA",
                    "https://mirrors.tuna.tsinghua.edu.cn/git/homebrew/brew.git",
                ),
            ],
        },
        pip: PipConfig {
            mirrors: vec![
                Mirror::new("PyPI 官方", "https://pypi.org/simple"),
                Mirror::new(
                    "清华 TUNA",
                    "https://pypi.tuna.tsinghua.edu.cn/simple",
                ),
                Mirror::new(
                    "阿里云",
                    "https://mirrors.aliyun.com/pypi/simple",
                ),
                Mirror::new(
                    "中科大 USTC",
                    "https://pypi.mirrors.ustc.edu.cn/simple",
                ),
                Mirror::new("豆瓣", "https://pypi.douban.com/simple"),
            ],
        },
        goenv: GoConfig {
            mirrors: vec![
                Mirror::new("Go 官方", "https://proxy.golang.org"),
                Mirror::new("七牛 goproxy.cn", "https://goproxy.cn"),
                Mirror::new("goproxy.io", "https://goproxy.io"),
            ],
        },
        pacman: PacmanConfig {
            mirrors: vec![
                Mirror::new(
                    "清华 TUNA",
                    "https://mirrors.tuna.tsinghua.edu.cn/archlinux",
                ),
                Mirror::new(
                    "中科大 USTC",
                    "https://mirrors.ustc.edu.cn/archlinux",
                ),
                Mirror::new(
                    "阿里云",
                    "https://mirrors.aliyun.com/archlinux",
                ),
            ],
        },
    }
}

// ── 统一镜像源访问方法 ────────────────────────────────────

impl Config {
    /// 根据适配器 id 获取镜像列表（只读）
    pub fn mirrors_for(&self, id: &str) -> Option<&Vec<Mirror>> {
        match id {
            "Cargo" => Some(&self.cargo.mirrors),
            "npm" => Some(&self.npm.mirrors),
            "Homebrew" => Some(&self.homebrew.mirrors),
            "pip" => Some(&self.pip.mirrors),
            "Go" => Some(&self.goenv.mirrors),
            "Pacman" => Some(&self.pacman.mirrors),
            _ => None,
        }
    }

    /// 根据适配器 id 获取镜像列表（可变）
    pub fn mirrors_for_mut(&mut self, id: &str) -> Option<&mut Vec<Mirror>> {
        match id {
            "Cargo" => Some(&mut self.cargo.mirrors),
            "npm" => Some(&mut self.npm.mirrors),
            "Homebrew" => Some(&mut self.homebrew.mirrors),
            "pip" => Some(&mut self.pip.mirrors),
            "Go" => Some(&mut self.goenv.mirrors),
            "Pacman" => Some(&mut self.pacman.mirrors),
            _ => None,
        }
    }
}

// ── 读写方法 ──────────────────────────────────────────────

impl Config {
    /// 加载配置：文件存在则读取，不存在则生成默认配置
    /// 加载后对空镜像列表自动补充默认值（兼容旧版配置文件升级）
    pub fn load() -> Result<Self> {
        let path = config_path();

        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("无法读取配置文件: {}", path.display()))?;
            let mut config: Config = toml::from_str(&content)
                .with_context(|| format!("配置文件解析失败: {}", path.display()))?;
            config.fill_defaults();
            Ok(config)
        } else {
            let default = default_config();
            default.save()?;
            Ok(default)
        }
    }

    /// 对空的镜像列表补充默认值（不影响已有数据的段）
    fn fill_defaults(&mut self) {
        let defaults = default_config();
        if self.cargo.mirrors.is_empty() {
            self.cargo.mirrors = defaults.cargo.mirrors;
        }
        if self.npm.mirrors.is_empty() {
            self.npm.mirrors = defaults.npm.mirrors;
        }
        if self.homebrew.mirrors.is_empty() {
            self.homebrew.mirrors = defaults.homebrew.mirrors;
        }
        if self.pip.mirrors.is_empty() {
            self.pip.mirrors = defaults.pip.mirrors;
        }
        if self.goenv.mirrors.is_empty() {
            self.goenv.mirrors = defaults.goenv.mirrors;
        }
        if self.pacman.mirrors.is_empty() {
            self.pacman.mirrors = defaults.pacman.mirrors;
        }
    }

    /// 保存配置到文件（自动创建父目录）
    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建配置目录: {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self).context("配置序列化失败")?;
        fs::write(&path, content)
            .with_context(|| format!("无法写入配置文件: {}", path.display()))?;
        Ok(())
    }

    /// 返回配置文件路径（供外部显示用）
    pub fn path() -> PathBuf {
        config_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config_roundtrip() {
        let config = default_config();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.cargo.mirrors.len(), deserialized.cargo.mirrors.len());
        assert_eq!(config.npm.mirrors.len(), deserialized.npm.mirrors.len());
        assert_eq!(
            config.homebrew.mirrors.len(),
            deserialized.homebrew.mirrors.len()
        );
        // v0.1.1 新增
        assert_eq!(config.pip.mirrors.len(), deserialized.pip.mirrors.len());
        assert_eq!(config.goenv.mirrors.len(), deserialized.goenv.mirrors.len());
        assert_eq!(config.pacman.mirrors.len(), deserialized.pacman.mirrors.len());
    }

    #[test]
    fn test_save_and_load() {
        let tmp = env::temp_dir().join("mirroman_test_config.toml");
        let config = default_config();
        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&tmp, &content).unwrap();

        let loaded: Config = toml::from_str(&std::fs::read_to_string(&tmp).unwrap()).unwrap();
        assert_eq!(loaded.settings.test_timeout_secs, 5);
        assert_eq!(loaded.cargo.mirrors[0].name, "rsproxy（默认）");

        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_mirrors_for() {
        let config = default_config();
        assert_eq!(config.mirrors_for("Cargo").unwrap().len(), 4);
        assert_eq!(config.mirrors_for("pip").unwrap().len(), 5);
        assert_eq!(config.mirrors_for("Go").unwrap().len(), 3);
        assert!(config.mirrors_for("unknown").is_none());
    }

    #[test]
    fn test_mirrors_for_mut() {
        let mut config = default_config();
        let mirrors = config.mirrors_for_mut("npm").unwrap();
        mirrors.push(Mirror::new("test", "https://example.com"));
        assert_eq!(config.mirrors_for("npm").unwrap().len(), 4);
    }

    #[test]
    fn test_fill_defaults_for_empty_mirrors() {
        // 模拟从旧配置加载：某些段不存在，mirrors 为空
        let minimal = r#"
[settings]
test_timeout_secs = 5

[cargo]
mirrors = [{ name = "rsproxy", url = "sparse+https://rsproxy.cn/index/" }]
"#;
        let mut config: Config = toml::from_str(minimal).unwrap();
        // npm/homebrew/pip/go/pacman 都为空
        assert!(config.pip.mirrors.is_empty());
        assert!(config.pacman.mirrors.is_empty());

        config.fill_defaults();

        // 已有的 cargo 不受影响
        assert_eq!(config.cargo.mirrors.len(), 1);
        assert_eq!(config.cargo.mirrors[0].name, "rsproxy");
        // 空的段被补充默认值
        assert!(!config.pip.mirrors.is_empty());
        assert!(!config.pacman.mirrors.is_empty());
        assert!(!config.goenv.mirrors.is_empty());
    }

    #[test]
    fn test_backward_compat_minimal_config() {
        // 旧版最小配置（无 pip/go/pacman 字段）应能反序列化成功
        let minimal = r#"
[settings]
test_timeout_secs = 5

[cargo]
mirrors = [{ name = "rsproxy", url = "sparse+https://rsproxy.cn/index/" }]

[npm]
mirrors = [{ name = "npm", url = "https://registry.npmjs.org" }]

[homebrew]
mirrors = [{ name = "brew", url = "https://github.com/Homebrew/brew" }]
"#;
        let config: Config = toml::from_str(minimal).unwrap();
        assert_eq!(config.cargo.mirrors.len(), 1);
        // 新字段应默认为空
        assert!(config.pip.mirrors.is_empty());
        assert!(config.goenv.mirrors.is_empty());
        assert!(config.pacman.mirrors.is_empty());
    }
}
