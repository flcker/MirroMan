use crate::adapters::PackageManagerAdapter;
use crate::config::Config;
use crate::mirror::Mirror;
use crate::utils::os::{current_os, has_executable, Os};
use crate::utils::paths::expand_tilde;
use anyhow::{Context, Result};
use std::fs;
use std::time::Instant;

pub struct HomebrewAdapter {
    mirrors: Vec<Mirror>,
}

impl HomebrewAdapter {
    pub fn new(config: &Config) -> Self {
        Self {
            mirrors: config.homebrew.mirrors.clone(),
        }
    }

    /// 检测当前 shell，返回 shell profile 路径列表
    fn shell_profile_paths() -> Vec<std::path::PathBuf> {
        let shell = std::env::var("SHELL").unwrap_or_default();
        let home = expand_tilde("~");

        let mut paths = Vec::new();
        if shell.contains("zsh") {
            paths.push(home.join(".zshrc"));
        }
        if shell.contains("bash") {
            paths.push(home.join(".bashrc"));
            paths.push(home.join(".bash_profile"));
        }
        // 兜底
        if paths.is_empty() {
            paths.push(home.join(".zshrc"));
        }
        paths
    }
}

impl PackageManagerAdapter for HomebrewAdapter {
    fn name(&self) -> &'static str {
        "Homebrew"
    }

    fn list_mirrors(&self) -> Vec<Mirror> {
        self.mirrors.clone()
    }

    fn switch_mirror(&self, mirror: &Mirror) -> Result<()> {
        // Homebrew 镜像切换通过设置环境变量实现
        // 将环境变量写入 shell profile 文件
        let is_official = mirror.name.contains("官方");

        for profile_path in Self::shell_profile_paths() {
            // 移除旧的 HOMEBREW_* 环境变量
            if profile_path.exists() {
                let content = fs::read_to_string(&profile_path)
                    .with_context(|| format!("无法读取: {}", profile_path.display()))?;

                let filtered: Vec<&str> = content
                    .lines()
                    .filter(|line| {
                        let trimmed = line.trim();
                        !trimmed.starts_with("export HOMEBREW_API_DOMAIN=")
                            && !trimmed.starts_with("export HOMEBREW_BOTTLE_DOMAIN=")
                            && !trimmed.starts_with("export HOMEBREW_BREW_GIT_REMOTE=")
                            && !trimmed.starts_with("export HOMEBREW_CORE_GIT_REMOTE=")
                    })
                    .collect();

                let mut new_content = filtered.join("\n");
                if !new_content.ends_with('\n') {
                    new_content.push('\n');
                }

                if !is_official {
                    let api_domain = if mirror.url.contains("tuna") {
                        "https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles/api"
                    } else if mirror.url.contains("ustc") {
                        "https://mirrors.ustc.edu.cn/homebrew-bottles/api"
                    } else {
                        &mirror.url
                    };

                    new_content.push_str(&format!(
                        "\n# MirroMan: Homebrew mirror\nexport HOMEBREW_API_DOMAIN=\"{}\"\nexport HOMEBREW_BOTTLE_DOMAIN=\"{}\"\nexport HOMEBREW_BREW_GIT_REMOTE=\"{}\"\nexport HOMEBREW_CORE_GIT_REMOTE=\"{}\"\n",
                        api_domain, api_domain, mirror.url, mirror.url
                    ));
                } else {
                    new_content.push_str("\n# MirroMan: Homebrew official (no mirror)\n");
                }

                fs::write(&profile_path, &new_content)
                    .with_context(|| format!("无法写入: {}", profile_path.display()))?;
            }
        }

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
        let os = current_os();
        // Homebrew 仅在 macOS 和 Linux 上可用
        matches!(os, Os::MacOS | Os::Linux) && has_executable("brew")
    }

    fn supported_platforms(&self) -> &'static str {
        "macOS / Linux"
    }

    fn current_mirror_name(&self) -> Option<String> {
        let api_domain = std::env::var("HOMEBREW_API_DOMAIN").ok()?;
        if api_domain.is_empty() {
            return None;
        }
        // 在 mirrors 中查找包含该 api_domain 关键字的镜像
        self.mirrors
            .iter()
            .find(|m| api_domain.contains("ustc") && m.url.contains("ustc")
                  || api_domain.contains("tuna") && m.url.contains("tuna")
                  || m.url.contains(&api_domain))
            .map(|m| m.name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homebrew_adapter_basics() {
        let adapter = HomebrewAdapter {
            mirrors: vec![Mirror::new("test", "https://mirrors.ustc.edu.cn/brew.git")],
        };
        assert_eq!(adapter.name(), "Homebrew");
        assert_eq!(adapter.list_mirrors().len(), 1);
    }

    #[test]
    fn test_is_available_on_linux() {
        let adapter = HomebrewAdapter {
            mirrors: vec![],
        };
        // 在 CI/Linux 环境下 brew 大概率不可用，只验证函数不 panic
        let _ = adapter.is_available();
    }
}
