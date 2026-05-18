use crate::adapters::{PackageManagerAdapter, RefreshAction};
use crate::config::{backup_dir, Config};
use crate::mirror::Mirror;
use crate::utils::command::{http_head, load_backup_text, save_backup_text};
use crate::utils::os::has_executable;
use crate::utils::paths::expand_tilde;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct GoAdapter {
    mirrors: Vec<Mirror>,
}

impl GoAdapter {
    pub fn new(config: &Config) -> Self {
        Self {
            mirrors: config.goenv.mirrors.clone(),
        }
    }

    fn backup_path() -> PathBuf {
        backup_dir().join("Go").join("goproxy.txt")
    }

    /// shell profile 路径
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

    /// 从 shell profile 中移除 MirroMan 的 GOPROXY 行
    fn remove_goproxy_lines(content: &str) -> String {
        let filtered: Vec<&str> = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("export GOPROXY=")
                    && !trimmed.starts_with("# MirroMan: Go")
            })
            .collect();

        let mut result = filtered.join("\n");
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result
    }
}

impl PackageManagerAdapter for GoAdapter {
    fn name(&self) -> &'static str {
        "Go"
    }

    fn list_mirrors(&self) -> Vec<Mirror> {
        self.mirrors.clone()
    }

    fn switch_mirror(&self, mirror: &Mirror) -> Result<()> {
        self.backup()?;

        let env_line = if mirror.name.contains("官方") {
            "# MirroMan: Go official (no mirror)\n".to_string()
        } else {
            format!(
                "# MirroMan: Go mirror\nexport GOPROXY={},direct\n",
                mirror.url
            )
        };

        for profile_path in Self::shell_profile_paths() {
            let content = if profile_path.exists() {
                fs::read_to_string(&profile_path)
                    .with_context(|| format!("无法读取: {}", profile_path.display()))?
            } else {
                String::new()
            };

            let cleaned = Self::remove_goproxy_lines(&content);

            let mut new_content = cleaned;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str(&env_line);

            fs::write(&profile_path, &new_content)
                .with_context(|| format!("无法写入: {}", profile_path.display()))?;
        }

        Ok(())
    }

    fn test_mirror(&self, mirror: &Mirror) -> Result<u64> {
        // 测试 goproxy 协议端点
        let test_url = format!(
            "{}/github.com/golang/example/@v/list",
            mirror.url.trim_end_matches('/')
        );
        http_head(&test_url)
    }

    fn is_available(&self) -> bool {
        has_executable("go")
    }

    fn supported_platforms(&self) -> &'static str {
        "Win / Mac / Linux"
    }

    fn current_mirror_name(&self) -> Option<String> {
        let goproxy = std::env::var("GOPROXY").ok()?;
        // GOPROXY 可能是 "https://goproxy.cn,direct" 格式
        let base = goproxy.split(',').next()?.to_string();
        if base.is_empty() || base == "direct" || base == "off" {
            return None;
        }
        self.mirrors
            .iter()
            .find(|m| base.starts_with(&m.url))
            .map(|m| m.name.clone())
    }

    // ── 备份/还原/重置 ──

    fn backup(&self) -> Result<()> {
        let goproxy = std::env::var("GOPROXY").unwrap_or_default();
        save_backup_text(&Self::backup_path(), &goproxy)
    }

    fn restore(&self) -> Result<()> {
        if let Some(goproxy) = load_backup_text(&Self::backup_path()) {
            // 恢复 GOPROXY 环境变量到 shell profile
            let env_line = if goproxy.is_empty() {
                "# MirroMan: Go restored (empty)\n".to_string()
            } else {
                format!("# MirroMan: Go restored\nexport GOPROXY={}\n", goproxy)
            };

            for profile_path in Self::shell_profile_paths() {
                if profile_path.exists() {
                    let content = fs::read_to_string(&profile_path).unwrap_or_default();
                    let cleaned = Self::remove_goproxy_lines(&content);
                    let mut new_content = cleaned;
                    if !new_content.ends_with('\n') {
                        new_content.push('\n');
                    }
                    new_content.push_str(&env_line);
                    fs::write(&profile_path, &new_content).ok();
                }
            }
        }
        Ok(())
    }

    fn reset_to_default(&self) -> Result<()> {
        for profile_path in Self::shell_profile_paths() {
            if profile_path.exists() {
                let content = fs::read_to_string(&profile_path)
                    .with_context(|| format!("无法读取: {}", profile_path.display()))?;
                let cleaned = Self::remove_goproxy_lines(&content);
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
            description: "重载 shell 环境使 GOPROXY 生效",
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
    fn test_go_adapter_basics() {
        let adapter = GoAdapter {
            mirrors: vec![Mirror::new("goproxy.cn", "https://goproxy.cn")],
        };
        assert_eq!(adapter.name(), "Go");
        assert_eq!(adapter.list_mirrors().len(), 1);
    }

    #[test]
    fn test_remove_goproxy_lines() {
        let input = "export PATH=/usr/bin\n# MirroMan: Go mirror\nexport GOPROXY=https://goproxy.cn,direct\nexport OTHER=1\n";
        let cleaned = GoAdapter::remove_goproxy_lines(input);
        assert!(cleaned.contains("export PATH=/usr/bin"));
        assert!(cleaned.contains("export OTHER=1"));
        assert!(!cleaned.contains("GOPROXY"));
        assert!(!cleaned.contains("# MirroMan: Go"));
    }
}
