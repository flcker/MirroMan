use crate::adapters::PackageManagerAdapter;
use crate::config::{backup_dir, Config};
use crate::mirror::Mirror;
use crate::utils::command::{backup_file, git_ls_remote, http_head, restore_file};
use crate::utils::os::has_executable;
use crate::utils::paths::expand_tilde;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct CargoAdapter {
    mirrors: Vec<Mirror>,
}

impl CargoAdapter {
    pub fn new(config: &Config) -> Self {
        Self {
            mirrors: config.cargo.mirrors.clone(),
        }
    }

    /// Cargo 配置文件路径
    fn config_path() -> PathBuf {
        expand_tilde("~/.cargo/config.toml")
    }

    /// 备份文件路径
    fn backup_path() -> PathBuf {
        backup_dir().join("Cargo").join("config.toml")
    }

    /// 构建 cargo config 格式
    fn build_cargo_config(mirror_name: &str, mirror_url: &str) -> String {
        let index_url = if mirror_name.contains("官方") || mirror_name.contains("crates.io") {
            "https://github.com/rust-lang/crates.io-index".to_string()
        } else if mirror_url.contains(".git") {
            mirror_url.to_string()
        } else {
            if mirror_url.contains("sparse+") {
                mirror_url.to_string()
            } else if mirror_url.contains("rsproxy") {
                "sparse+https://rsproxy.cn/index/".to_string()
            } else {
                format!("{}/crates.io-index", mirror_url.trim_end_matches('/'))
            }
        };

        format!(
            r#"[source.crates-io]
replace-with = 'mirror'

[source.mirror]
registry = "{index_url}"

[registries.mirror]
index = "{index_url}"
"#
        )
    }
}

impl PackageManagerAdapter for CargoAdapter {
    fn name(&self) -> &'static str {
        "Cargo"
    }

    fn list_mirrors(&self) -> Vec<Mirror> {
        self.mirrors.clone()
    }

    fn switch_mirror(&self, mirror: &Mirror) -> Result<()> {
        // 切换前自动备份
        self.backup()?;

        let path = Self::config_path();

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建目录: {}", parent.display()))?;
        }

        let new_content = Self::build_cargo_config(&mirror.name, &mirror.url);

        let final_content = if path.exists() {
            let existing = fs::read_to_string(&path)
                .with_context(|| format!("无法读取: {}", path.display()))?;
            merge_cargo_config(&existing, &new_content)
        } else {
            new_content
        };

        fs::write(&path, &final_content)
            .with_context(|| format!("无法写入: {}", path.display()))?;

        Ok(())
    }

    fn test_mirror(&self, mirror: &Mirror) -> Result<u64> {
        // 改进的测试端点选择
        if mirror.url.contains("rsproxy") {
            http_head("https://rsproxy.cn")
        } else if mirror.url.contains("sparse+") {
            let url = mirror.url.trim_start_matches("sparse+");
            http_head(url)
        } else if mirror.url.contains(".git") {
            // git 仓库：用 git ls-remote 替代 HTTP HEAD
            git_ls_remote(&mirror.url)
        } else {
            http_head(&mirror.url)
        }
    }

    fn is_available(&self) -> bool {
        has_executable("cargo")
    }

    fn supported_platforms(&self) -> &'static str {
        "Win / Mac / Linux"
    }

    fn current_mirror_name(&self) -> Option<String> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(&path).ok()?;

        let registry_url = content
            .lines()
            .find(|line| line.trim().starts_with("registry ="))
            .and_then(|line| {
                let v = line.split('"').nth(1).or_else(|| line.split('\'').nth(1));
                v.map(|s| s.to_string())
            })?;

        self.mirrors
            .iter()
            .find(|m| m.url == registry_url)
            .map(|m| m.name.clone())
    }

    // ── v0.1.1 备份/还原/重置 ──

    fn backup(&self) -> Result<()> {
        let src = Self::config_path();
        let dst = Self::backup_path();
        backup_file(&src, &dst)
    }

    fn restore(&self) -> Result<()> {
        let src = Self::backup_path();
        let dst = Self::config_path();
        restore_file(&src, &dst)
    }

    fn reset_to_default(&self) -> Result<()> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("无法读取: {}", path.display()))?;
            // 移除 [source.*] 和 [registries.*] 段
            let cleaned = remove_sections(&content);
            fs::write(&path, &cleaned)
                .with_context(|| format!("无法写入: {}", path.display()))?;
        }
        Ok(())
    }
}

// ── Cargo 配置处理辅助函数 ───────────────────────────────

/// 清理所有 [source.*] 和 [registries.*] 段，追加新的 source 配置
fn merge_cargo_config(existing: &str, new_source_section: &str) -> String {
    let mut result = String::new();
    let mut skip = false;

    for line in existing.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("[source.") || trimmed.starts_with("[registries.") {
            skip = true;
            continue;
        }

        if skip && trimmed.starts_with('[') {
            skip = false;
        }

        if !skip {
            result.push_str(line);
            result.push('\n');
        }
    }

    if !result.ends_with('\n') {
        result.push('\n');
    }

    result.push('\n');
    result.push_str(new_source_section);
    result
}

/// 移除 [source.*] 和 [registries.*] 段（用于 reset_to_default）
fn remove_sections(existing: &str) -> String {
    let mut result = String::new();
    let mut skip = false;

    for line in existing.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("[source.") || trimmed.starts_with("[registries.") {
            skip = true;
            continue;
        }

        if skip && trimmed.starts_with('[') {
            skip = false;
        }

        if !skip {
            result.push_str(line);
            result.push('\n');
        }
    }

    // 去除尾部多余空行
    while result.ends_with("\n\n") {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_cargo_config_no_existing_sources() {
        let existing = "[build]\nrustflags = [\"-D\", \"warnings\"]\n";
        let new_source = "[source.crates-io]\nreplace-with = 'mirror'\n";
        let merged = merge_cargo_config(existing, new_source);
        assert!(merged.contains("[build]"));
        assert!(merged.contains("[source.crates-io]"));
    }

    #[test]
    fn test_merge_removes_old_source_sections() {
        let existing = "[build]\nrustflags = [\"-D\"]\n\n[source.rsproxy]\nregistry = \"old\"\n\n[registries.rsproxy]\nindex = \"old\"\n\n[other]\nkeep = true\n";
        let new_source = "[source.crates-io]\nreplace-with = 'mirror'\n";
        let merged = merge_cargo_config(existing, new_source);
        assert!(merged.contains("[build]"));
        assert!(merged.contains("[other]"));
        assert!(merged.contains("[source.crates-io]"));
        assert!(!merged.contains("[source.rsproxy]"));
        assert!(!merged.contains("[registries.rsproxy]"));
    }

    #[test]
    fn test_cargo_adapter_basics() {
        let adapter = CargoAdapter {
            mirrors: vec![Mirror::new("test", "sparse+https://rsproxy.cn/index/")],
        };
        assert_eq!(adapter.name(), "Cargo");
        assert!(adapter.supported_platforms().contains("Win"));
        assert_eq!(adapter.list_mirrors().len(), 1);
    }

    #[test]
    fn test_remove_sections() {
        let existing = "[build]\nr = true\n\n[source.crates-io]\nreplace-with = 'm'\n\n[other]\nx = 1\n";
        let cleaned = remove_sections(existing);
        assert!(cleaned.contains("[build]"));
        assert!(cleaned.contains("[other]"));
        assert!(!cleaned.contains("[source.crates-io]"));
    }
}
