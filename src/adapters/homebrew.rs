use crate::adapters::{PackageManagerAdapter, RefreshAction};
use crate::config::{backup_dir, Config};
use crate::mirror::Mirror;
use crate::utils::command::{backup_file, http_head, restore_file};
use crate::utils::os::{current_os, has_executable, Os};
use crate::utils::paths::expand_tilde;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

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
    fn shell_profile_paths() -> Vec<PathBuf> {
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
        if paths.is_empty() {
            paths.push(home.join(".zshrc"));
        }
        paths
    }

    fn backup_path(profile: &PathBuf) -> PathBuf {
        // 用文件名作为备份标识
        let fname = profile
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("profile");
        backup_dir().join("Homebrew").join(fname)
    }

    /// 构建 Homebrew 环境变量行
    fn build_env_lines(mirror: &Mirror) -> String {
        let is_official = mirror.name.contains("官方");

        if is_official {
            "# MirroMan: Homebrew official (no mirror)\n".to_string()
        } else {
            let api_domain = if mirror.url.contains("tuna") {
                "https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles/api"
            } else if mirror.url.contains("ustc") {
                "https://mirrors.ustc.edu.cn/homebrew-bottles/api"
            } else {
                &mirror.url
            };

            format!(
                "# MirroMan: Homebrew mirror\nexport HOMEBREW_API_DOMAIN=\"{}\"\nexport HOMEBREW_BOTTLE_DOMAIN=\"{}\"\nexport HOMEBREW_BREW_GIT_REMOTE=\"{}\"\nexport HOMEBREW_CORE_GIT_REMOTE=\"{}\"\n",
                api_domain, api_domain, mirror.url, mirror.url
            )
        }
    }

    /// 从 shell profile 中移除 MirroMan 的 HOMEBREW_* 行
    fn remove_homebrew_lines(content: &str) -> String {
        let filtered: Vec<&str> = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("export HOMEBREW_API_DOMAIN=")
                    && !trimmed.starts_with("export HOMEBREW_BOTTLE_DOMAIN=")
                    && !trimmed.starts_with("export HOMEBREW_BREW_GIT_REMOTE=")
                    && !trimmed.starts_with("export HOMEBREW_CORE_GIT_REMOTE=")
                    && !trimmed.starts_with("# MirroMan: Homebrew")
            })
            .collect();

        let mut result = filtered.join("\n");
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result
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
        // 切换前自动备份
        self.backup()?;

        for profile_path in Self::shell_profile_paths() {
            let content = if profile_path.exists() {
                fs::read_to_string(&profile_path)
                    .with_context(|| format!("无法读取: {}", profile_path.display()))?
            } else {
                String::new()
            };

            let cleaned = Self::remove_homebrew_lines(&content);
            let env_lines = Self::build_env_lines(mirror);

            let mut new_content = cleaned;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str(&env_lines);

            fs::write(&profile_path, &new_content)
                .with_context(|| format!("无法写入: {}", profile_path.display()))?;
        }

        Ok(())
    }

    fn test_mirror(&self, mirror: &Mirror) -> Result<u64> {
        // 改进：测试 API 端点而非 git URL
        let test_url = if mirror.url.contains("tuna") {
            "https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles/api"
        } else if mirror.url.contains("ustc") {
            "https://mirrors.ustc.edu.cn/homebrew-bottles/api"
        } else if mirror.url.starts_with("https://github.com") {
            // 官方源：测试 GitHub API
            "https://api.github.com"
        } else {
            &mirror.url
        };
        http_head(test_url)
    }

    fn is_available(&self) -> bool {
        let os = current_os();
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
        self.mirrors
            .iter()
            .find(|m| {
                api_domain.contains("ustc") && m.url.contains("ustc")
                    || api_domain.contains("tuna") && m.url.contains("tuna")
                    || m.url.contains(&api_domain)
            })
            .map(|m| m.name.clone())
    }

    // ── v0.1.1 备份/还原/重置 ──

    fn backup(&self) -> Result<()> {
        for profile_path in Self::shell_profile_paths() {
            if profile_path.exists() {
                let dst = Self::backup_path(&profile_path);
                backup_file(&profile_path, &dst)?;
            }
        }
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        for profile_path in Self::shell_profile_paths() {
            let src = Self::backup_path(&profile_path);
            if src.exists() {
                restore_file(&src, &profile_path)?;
            }
        }
        Ok(())
    }

    fn reset_to_default(&self) -> Result<()> {
        for profile_path in Self::shell_profile_paths() {
            if profile_path.exists() {
                let content = fs::read_to_string(&profile_path)
                    .with_context(|| format!("无法读取: {}", profile_path.display()))?;
                let cleaned = Self::remove_homebrew_lines(&content);
                fs::write(&profile_path, &cleaned)
                    .with_context(|| format!("无法写入: {}", profile_path.display()))?;
            }
        }
        Ok(())
    }

    // ── 切换后提示 ──

    fn refresh_action(&self) -> Option<RefreshAction> {
        let shell = std::env::var("SHELL").unwrap_or_default();
        let profile = if shell.contains("zsh") {
            "~/.zshrc"
        } else if shell.contains("bash") {
            "~/.bashrc"
        } else {
            "shell profile"
        };
        Some(RefreshAction {
            description: "重载 shell 环境使环境变量生效",
            command: "source".to_string(),
            args: vec![profile.to_string()],
            requires_sudo: false,
        })
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
        let _ = adapter.is_available();
    }

    #[test]
    fn test_remove_homebrew_lines() {
        let input = "export PATH=/usr/bin\n# MirroMan: Homebrew mirror\nexport HOMEBREW_API_DOMAIN=\"https://...\"\nexport HOMEBREW_BOTTLE_DOMAIN=\"https://...\"\nexport OTHER_VAR=1\n";
        let cleaned = HomebrewAdapter::remove_homebrew_lines(input);
        assert!(cleaned.contains("export PATH=/usr/bin"));
        assert!(cleaned.contains("export OTHER_VAR=1"));
        assert!(!cleaned.contains("HOMEBREW_API_DOMAIN"));
        assert!(!cleaned.contains("# MirroMan: Homebrew"));
    }
}
