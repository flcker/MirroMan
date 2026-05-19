use crate::adapters::PackageManagerAdapter;
use crate::config::{backup_dir, Config};
use crate::mirror::Mirror;
use crate::utils::command::{http_head, load_backup_text, run_command, save_backup_text};
use crate::utils::os::has_executable;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct NpmAdapter {
    mirrors: Vec<Mirror>,
}

impl NpmAdapter {
    pub fn new(config: &Config) -> Self {
        Self {
            mirrors: config.mirrors.npm.mirrors.clone(),
        }
    }

    fn backup_path() -> PathBuf {
        backup_dir().join("npm").join("registry.txt")
    }
}

impl PackageManagerAdapter for NpmAdapter {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn list_mirrors(&self) -> Vec<Mirror> {
        self.mirrors.clone()
    }

    fn switch_mirror(&self, mirror: &Mirror) -> Result<()> {
        // 切换前自动备份
        self.backup()?;

        run_command("npm", &["config", "set", "registry", &mirror.url])
            .with_context(|| format!("npm config set registry 失败: {}", mirror.url))?;
        Ok(())
    }

    fn test_mirror(&self, mirror: &Mirror) -> Result<u64> {
        // 测试 /npm 端点（轻量包）
        let test_url = format!("{}/npm", mirror.url.trim_end_matches('/'));
        http_head(&test_url)
    }

    fn is_available(&self) -> bool {
        has_executable("npm")
    }

    fn supported_platforms(&self) -> &'static str {
        "Win / Mac / Linux"
    }

    fn current_mirror_name(&self) -> Option<String> {
        let output = std::process::Command::new("npm")
            .args(["config", "get", "registry"])
            .output()
            .ok()?;
        let registry = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if registry.is_empty() {
            return None;
        }
        self.mirrors
            .iter()
            .find(|m| m.url == registry)
            .map(|m| m.name.clone())
    }

    // ── v0.1.1 备份/还原/重置 ──

    fn backup(&self) -> Result<()> {
        let output = std::process::Command::new("npm")
            .args(["config", "get", "registry"])
            .output()
            .context("无法获取当前 npm registry")?;
        let registry = String::from_utf8_lossy(&output.stdout).trim().to_string();
        save_backup_text(&Self::backup_path(), &registry)
    }

    fn restore(&self) -> Result<()> {
        if let Some(registry) = load_backup_text(&Self::backup_path()) {
            if !registry.is_empty() {
                run_command("npm", &["config", "set", "registry", &registry])
                    .context("npm config set registry 还原失败")?;
            }
        }
        Ok(())
    }

    fn reset_to_default(&self) -> Result<()> {
        run_command("npm", &["config", "delete", "registry"])
            .context("npm config delete registry 失败")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npm_adapter_basics() {
        let adapter = NpmAdapter {
            mirrors: vec![Mirror::new("test", "https://registry.npmjs.org")],
        };
        assert_eq!(adapter.name(), "npm");
        assert_eq!(adapter.list_mirrors().len(), 1);
    }
}
