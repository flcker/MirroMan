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

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfig {
    pub mirrors: Vec<Mirror>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NpmConfig {
    pub mirrors: Vec<Mirror>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HomebrewConfig {
    pub mirrors: Vec<Mirror>,
}

/// 顶层配置，所有包管理器的镜像源汇总
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    pub cargo: CargoConfig,
    pub npm: NpmConfig,
    pub homebrew: HomebrewConfig,
}

// ── 配置路径 ──────────────────────────────────────────────

fn config_path() -> PathBuf {
    let proj_dirs =
        ProjectDirs::from("com", "mirroman", "MirroMan").expect("无法确定项目配置目录");
    proj_dirs.config_dir().join("config.toml")
}

// ── 默认配置 ──────────────────────────────────────────────

fn default_config() -> Config {
    Config {
        settings: Settings::default(),
        cargo: CargoConfig {
            mirrors: vec![
                Mirror::new("rsproxy（默认）", "sparse+https://rsproxy.cn/index/"),
                Mirror::new("清华 TUNA", "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"),
                Mirror::new("中科大 USTC", "https://mirrors.ustc.edu.cn/crates.io-index"),
                Mirror::new("crates.io 官方", "https://github.com/rust-lang/crates.io-index"),
            ],
        },
        npm: NpmConfig {
            mirrors: vec![
                Mirror::new("npm 官方", "https://registry.npmjs.org"),
                Mirror::new("淘宝 npmmirror", "https://registry.npmmirror.com"),
                Mirror::new("清华 TUNA", "https://mirrors.tuna.tsinghua.edu.cn/npm/"),
            ],
        },
        homebrew: HomebrewConfig {
            mirrors: vec![
                Mirror::new("Homebrew 官方", "https://github.com/Homebrew/brew"),
                Mirror::new("中科大 USTC", "https://mirrors.ustc.edu.cn/brew.git"),
                Mirror::new("清华 TUNA", "https://mirrors.tuna.tsinghua.edu.cn/git/homebrew/brew.git"),
            ],
        },
    }
}

// ── 读写方法 ──────────────────────────────────────────────

impl Config {
    /// 加载配置：文件存在则读取，不存在则生成默认配置
    pub fn load() -> Result<Self> {
        let path = config_path();

        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("无法读取配置文件: {}", path.display()))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("配置文件解析失败: {}", path.display()))?;
            Ok(config)
        } else {
            let default = default_config();
            default.save()?;
            Ok(default)
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
        assert_eq!(config.homebrew.mirrors.len(), deserialized.homebrew.mirrors.len());
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
}
