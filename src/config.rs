use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::mirror::Mirror;
use std::collections::HashMap;

// ── Settings ─────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    /// 默认包管理器（进入 TUI 时自动选中）
    #[serde(default)]
    pub default_manager: Option<String>,
    /// 镜像测速超时（秒）
    #[serde(default = "default_timeout")]
    pub test_timeout_secs: u64,
    /// 当前使用的主题名称
    #[serde(default = "default_theme_name")]
    pub theme: String,
    /// 所有可用主题
    #[serde(default)]
    pub themes: HashMap<String, ThemeConfig>,
}

fn default_timeout() -> u64 {
    5
}

fn default_theme_name() -> String {
    "dark".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_manager: None,
            test_timeout_secs: 5,
            theme: "dark".to_string(),
            themes: HashMap::new(),
        }
    }
}

// ── 主题配置 ──────────────────────────────────────────────

/// 可序列化的主题配色（所有字段可选，未设置时用内置默认值）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_highlight_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_highlight_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_fg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_fg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_fg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_focus_border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_focus_highlight_bg: Option<String>,
}

fn default_dark_theme() -> ThemeConfig {
    ThemeConfig {
        focus_border: Some("yellow".into()),
        focus_highlight_bg: Some("yellow".into()),
        dim_highlight_bg: Some("darkgray".into()),
        highlight_fg: Some("black".into()),
        header_fg: Some("white".into()),
        dim_text: Some("darkgray".into()),
        key_fg: Some("yellow".into()),
        key_desc: Some("gray".into()),
        success: Some("green".into()),
        error: Some("red".into()),
        warning: Some("yellow".into()),
        mirror_focus_border: Some("cyan".into()),
        mirror_focus_highlight_bg: Some("cyan".into()),
    }
}

fn default_light_theme() -> ThemeConfig {
    ThemeConfig {
        focus_border: Some("blue".into()),
        focus_highlight_bg: Some("blue".into()),
        dim_highlight_bg: Some("darkgray".into()),
        highlight_fg: Some("white".into()),
        header_fg: Some("black".into()),
        dim_text: Some("darkgray".into()),
        key_fg: Some("blue".into()),
        key_desc: Some("gray".into()),
        success: Some("green".into()),
        error: Some("red".into()),
        warning: Some("yellow".into()),
        mirror_focus_border: Some("magenta".into()),
        mirror_focus_highlight_bg: Some("magenta".into()),
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

// ── 镜像数据（mirrors.toml） ─────────────────────────────

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Mirrors {
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

// ── 顶层配置（内存中合并） ──────────────────────────────

pub struct Config {
    pub settings: Settings,
    pub mirrors: Mirrors,
}

// ── 配置路径 ──────────────────────────────────────────────

fn config_dir() -> PathBuf {
    let proj_dirs =
        ProjectDirs::from("com", "mirroman", "MirroMan").expect("无法确定项目配置目录");
    proj_dirs.config_dir().to_path_buf()
}

fn settings_path() -> PathBuf {
    config_dir().join("settings.toml")
}

fn mirrors_path() -> PathBuf {
    config_dir().join("mirrors.toml")
}

/// 旧版配置文件路径（v0.1.0 / v0.1.1）
fn old_config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// 备份根目录
pub fn backup_dir() -> PathBuf {
    config_dir().join("backups")
}

// ── 默认值 ────────────────────────────────────────────────

fn default_settings() -> Settings {
    let mut themes = HashMap::new();
    themes.insert("dark".to_string(), default_dark_theme());
    themes.insert("light".to_string(), default_light_theme());
    Settings {
        themes,
        ..Settings::default()
    }
}

fn default_mirrors() -> Mirrors {
    Mirrors {
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
        pip: PipConfig {
            mirrors: vec![
                Mirror::new("PyPI 官方", "https://pypi.org/simple"),
                Mirror::new("清华 TUNA", "https://pypi.tuna.tsinghua.edu.cn/simple"),
                Mirror::new("阿里云", "https://mirrors.aliyun.com/pypi/simple"),
                Mirror::new("中科大 USTC", "https://pypi.mirrors.ustc.edu.cn/simple"),
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
                Mirror::new("清华 TUNA", "https://mirrors.tuna.tsinghua.edu.cn/archlinux"),
                Mirror::new("中科大 USTC", "https://mirrors.ustc.edu.cn/archlinux"),
                Mirror::new("阿里云", "https://mirrors.aliyun.com/archlinux"),
            ],
        },
    }
}

// ── 旧版配置（仅用于迁移） ──────────────────────────────

#[derive(Debug, Deserialize)]
struct OldConfig {
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

// ── 读写方法 ──────────────────────────────────────────────

impl Config {
    /// 加载配置：
    /// - settings.toml + mirrors.toml 存在 → 直接加载
    /// - 两个都不存在但有旧 config.toml → 迁移
    /// - 都不存在 → 生成默认
    pub fn load() -> Result<Self> {
        let sp = settings_path();
        let mp = mirrors_path();

        if sp.exists() && mp.exists() {
            Self::load_new(&sp, &mp)
        } else if !sp.exists() && !mp.exists() && old_config_path().exists() {
            Self::migrate_from_old()
        } else {
            Self::create_default()
        }
    }

    /// 从新格式加载
    fn load_new(sp: &PathBuf, mp: &PathBuf) -> Result<Self> {
        let mut settings: Settings = toml::from_str(
            &fs::read_to_string(sp)
                .with_context(|| format!("无法读取: {}", sp.display()))?,
        )
        .context("settings.toml 解析失败")?;

        let mut mirrors: Mirrors = toml::from_str(
            &fs::read_to_string(mp)
                .with_context(|| format!("无法读取: {}", mp.display()))?,
        )
        .context("mirrors.toml 解析失败")?;

        let themes_was_empty = settings.themes.is_empty();
        settings.fill_defaults();
        mirrors.fill_defaults();

        let config = Self { settings, mirrors };

        // 如果补充了默认 themes，立即写回 settings.toml
        if themes_was_empty {
            config.save_settings()?;
        }

        Ok(config)
    }

    /// 从旧 config.toml 迁移
    fn migrate_from_old() -> Result<Self> {
        let old_path = old_config_path();
        let old: OldConfig = toml::from_str(
            &fs::read_to_string(&old_path)
                .with_context(|| format!("无法读取旧配置: {}", old_path.display()))?,
        )
        .context("旧 config.toml 解析失败")?;

        let mut settings = old.settings;
        settings.fill_defaults();

        let mut mirrors = Mirrors {
            cargo: old.cargo,
            npm: old.npm,
            homebrew: old.homebrew,
            pip: old.pip,
            goenv: old.goenv,
            pacman: old.pacman,
        };
        mirrors.fill_defaults();

        let config = Config {
            settings,
            mirrors,
        };

        // 写入新文件
        config.save()?;

        // 重命名旧文件做备份
        let bak_path = old_path.with_extension("toml.bak");
        fs::rename(&old_path, &bak_path)
            .with_context(|| format!("无法重命名旧配置文件: {}", old_path.display()))?;

        tracing::info!("已从 config.toml 迁移到 settings.toml + mirrors.toml");
        Ok(config)
    }

    /// 生成默认配置
    fn create_default() -> Result<Self> {
        let config = Config {
            settings: default_settings(),
            mirrors: default_mirrors(),
        };
        config.save()?;
        Ok(config)
    }

    /// 保存全部配置到两个文件
    pub fn save(&self) -> Result<()> {
        self.save_settings()?;
        self.save_mirrors()?;
        Ok(())
    }

    /// 仅保存 settings.toml
    fn save_settings(&self) -> Result<()> {
        let dir = config_dir();
        fs::create_dir_all(&dir)
            .with_context(|| format!("无法创建目录: {}", dir.display()))?;
        let content = toml::to_string_pretty(&self.settings).context("设置序列化失败")?;
        fs::write(&settings_path(), &content)
            .with_context(|| format!("无法写入: {}", settings_path().display()))?;
        Ok(())
    }

    /// 仅保存 mirrors.toml
    pub fn save_mirrors(&self) -> Result<()> {
        let dir = config_dir();
        fs::create_dir_all(&dir)
            .with_context(|| format!("无法创建目录: {}", dir.display()))?;
        let content = toml::to_string_pretty(&self.mirrors).context("镜像配置序列化失败")?;
        fs::write(&mirrors_path(), &content)
            .with_context(|| format!("无法写入: {}", mirrors_path().display()))?;
        Ok(())
    }
}

// ── 默认值填充 ──────────────────────────────────────────

impl Settings {
    fn fill_defaults(&mut self) {
        if self.themes.is_empty() {
            self.themes
                .insert("dark".to_string(), default_dark_theme());
            self.themes
                .insert("light".to_string(), default_light_theme());
        }
        if self.theme.is_empty() || !self.themes.contains_key(&self.theme) {
            self.theme = "dark".to_string();
        }
    }
}

// ── 镜像默认填充 ──────────────────────────────────────────

impl Mirrors {
    /// 对空的镜像列表补充默认值
    fn fill_defaults(&mut self) {
        let defaults = default_mirrors();
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
}

// ── 统一镜像源访问方法 ────────────────────────────────────

impl Config {
    /// 根据适配器 id 获取镜像列表（只读）
    pub fn mirrors_for(&self, id: &str) -> Option<&Vec<Mirror>> {
        match id {
            "Cargo" => Some(&self.mirrors.cargo.mirrors),
            "npm" => Some(&self.mirrors.npm.mirrors),
            "Homebrew" => Some(&self.mirrors.homebrew.mirrors),
            "pip" => Some(&self.mirrors.pip.mirrors),
            "Go" => Some(&self.mirrors.goenv.mirrors),
            "Pacman" => Some(&self.mirrors.pacman.mirrors),
            _ => None,
        }
    }

    /// 根据适配器 id 获取镜像列表（可变）
    pub fn mirrors_for_mut(&mut self, id: &str) -> Option<&mut Vec<Mirror>> {
        match id {
            "Cargo" => Some(&mut self.mirrors.cargo.mirrors),
            "npm" => Some(&mut self.mirrors.npm.mirrors),
            "Homebrew" => Some(&mut self.mirrors.homebrew.mirrors),
            "pip" => Some(&mut self.mirrors.pip.mirrors),
            "Go" => Some(&mut self.mirrors.goenv.mirrors),
            "Pacman" => Some(&mut self.mirrors.pacman.mirrors),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mirrors_roundtrip() {
        let mirrors = default_mirrors();
        let serialized = toml::to_string_pretty(&mirrors).unwrap();
        let deserialized: Mirrors = toml::from_str(&serialized).unwrap();

        assert_eq!(mirrors.cargo.mirrors.len(), deserialized.cargo.mirrors.len());
        assert_eq!(mirrors.npm.mirrors.len(), deserialized.npm.mirrors.len());
        assert_eq!(mirrors.pip.mirrors.len(), deserialized.pip.mirrors.len());
        assert_eq!(mirrors.goenv.mirrors.len(), deserialized.goenv.mirrors.len());
        assert_eq!(mirrors.pacman.mirrors.len(), deserialized.pacman.mirrors.len());
    }

    #[test]
    fn test_mirrors_for() {
        let config = Config {
            settings: default_settings(),
            mirrors: default_mirrors(),
        };
        assert_eq!(config.mirrors_for("Cargo").unwrap().len(), 4);
        assert_eq!(config.mirrors_for("pip").unwrap().len(), 5);
        assert_eq!(config.mirrors_for("Go").unwrap().len(), 3);
        assert!(config.mirrors_for("unknown").is_none());
    }

    #[test]
    fn test_mirrors_for_mut() {
        let mut config = Config {
            settings: default_settings(),
            mirrors: default_mirrors(),
        };
        let mirrors = config.mirrors_for_mut("npm").unwrap();
        mirrors.push(Mirror::new("test", "https://example.com"));
        assert_eq!(config.mirrors_for("npm").unwrap().len(), 4);
    }

    #[test]
    fn test_fill_defaults_for_empty_mirrors() {
        let mut mirrors = Mirrors::default();
        assert!(mirrors.pip.mirrors.is_empty());
        assert!(mirrors.pacman.mirrors.is_empty());

        mirrors.fill_defaults();

        assert!(!mirrors.cargo.mirrors.is_empty());
        assert!(!mirrors.pip.mirrors.is_empty());
        assert!(!mirrors.pacman.mirrors.is_empty());
        assert!(!mirrors.goenv.mirrors.is_empty());
    }

    #[test]
    fn test_fill_defaults_preserves_existing() {
        let mut mirrors = Mirrors::default();
        mirrors.cargo.mirrors.push(Mirror::new("custom", "https://custom.example.com"));

        mirrors.fill_defaults();

        assert_eq!(mirrors.cargo.mirrors.len(), 1);
        assert_eq!(mirrors.cargo.mirrors[0].name, "custom");
        assert!(!mirrors.pip.mirrors.is_empty());
    }

    #[test]
    fn test_backward_compat_minimal_mirrors() {
        let minimal = r#"
[cargo]
mirrors = [{ name = "rsproxy", url = "sparse+https://rsproxy.cn/index/" }]
"#;
        let mut mirrors: Mirrors = toml::from_str(minimal).unwrap();
        assert_eq!(mirrors.cargo.mirrors.len(), 1);
        assert!(mirrors.pip.mirrors.is_empty());

        mirrors.fill_defaults();

        assert_eq!(mirrors.cargo.mirrors.len(), 1);
        assert!(!mirrors.pip.mirrors.is_empty());
    }

    #[test]
    fn test_old_config_migration_format() {
        // 验证 old config.toml 格式仍可反序列化
        let old_toml = r#"
[settings]
test_timeout_secs = 5

[cargo]
mirrors = [{ name = "rsproxy", url = "sparse+https://rsproxy.cn/index/" }]

[npm]
mirrors = [{ name = "npm", url = "https://registry.npmjs.org" }]

[homebrew]
mirrors = [{ name = "brew", url = "https://github.com/Homebrew/brew" }]
"#;
        let old: OldConfig = toml::from_str(old_toml).unwrap();
        assert_eq!(old.cargo.mirrors.len(), 1);
        assert_eq!(old.settings.test_timeout_secs, 5);
        // v0.1.1 新增的段应该为空
        assert!(old.pip.mirrors.is_empty());
        assert!(old.goenv.mirrors.is_empty());
        assert!(old.pacman.mirrors.is_empty());
    }
}
