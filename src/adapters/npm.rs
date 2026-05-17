use crate::adapters::PackageManagerAdapter;
use crate::config::Config;
use crate::mirror::Mirror;
use crate::utils::command::run_command;
use crate::utils::os::has_executable;
use anyhow::{Context, Result};
use std::time::Instant;

pub struct NpmAdapter {
    mirrors: Vec<Mirror>,
}

impl NpmAdapter {
    pub fn new(config: &Config) -> Self {
        Self {
            mirrors: config.npm.mirrors.clone(),
        }
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
        run_command("npm", &["config", "set", "registry", &mirror.url])
            .with_context(|| format!("npm config set registry 失败: {}", mirror.url))?;
        Ok(())
    }

    fn test_mirror(&self, mirror: &Mirror) -> Result<u64> {
        let start = Instant::now();

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .context("无法创建 HTTP 客户端")?;

        client.head(&mirror.url).send().context("镜像不可达")?;
        Ok(start.elapsed().as_millis() as u64)
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
        self.mirrors.iter().find(|m| m.url == registry).map(|m| m.name.clone())
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
        // npm 可能不可用，但结构体测试应该通过
        assert_eq!(adapter.name(), "npm");
        assert_eq!(adapter.list_mirrors().len(), 1);
    }
}
