use crate::adapters::PackageManagerAdapter;
use crate::config::{backup_dir, Config};
use crate::mirror::Mirror;
use crate::utils::command::{http_head, load_backup_text, run_command, save_backup_text};
use crate::utils::os::has_executable;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::OnceLock;

pub struct PipAdapter {
    mirrors: Vec<Mirror>,
    /// is_available 结果缓存（避免重复 fork 进程）
    available: OnceLock<bool>,
}

impl PipAdapter {
    pub fn new(config: &Config) -> Self {
        Self {
            mirrors: config.mirrors.pip.mirrors.clone(),
            available: OnceLock::new(),
        }
    }

    fn backup_path() -> PathBuf {
        backup_dir().join("pip").join("index-url.txt")
    }

    /// 查找可用的 pip 调用方式
    /// 返回 (命令, 是否为 python -m 模式)
    fn pip_cmd() -> (String, Vec<String>) {
        if has_executable("pip3") {
            ("pip3".to_string(), vec![])
        } else if has_executable("pip") {
            ("pip".to_string(), vec![])
        } else if has_executable("python3") {
            // 回退到 python3 -m pip
            ("python3".to_string(), vec!["-m".to_string(), "pip".to_string()])
        } else {
            ("python".to_string(), vec!["-m".to_string(), "pip".to_string()])
        }
    }

    /// 构建完整的 pip 命令参数
    fn pip_args(subcmd: &[&str]) -> (String, Vec<String>) {
        let (cmd, prefix) = Self::pip_cmd();
        let mut args = prefix;
        for s in subcmd {
            args.push(s.to_string());
        }
        (cmd, args)
    }
}

impl PackageManagerAdapter for PipAdapter {
    fn name(&self) -> &'static str {
        "pip"
    }

    fn list_mirrors(&self) -> Vec<Mirror> {
        self.mirrors.clone()
    }

    fn switch_mirror(&self, mirror: &Mirror) -> Result<()> {
        self.backup()?;

        let (cmd, args) = Self::pip_args(&["config", "set", "global.index-url", &mirror.url]);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        run_command(&cmd, &args_ref)
            .with_context(|| {
                format!("pip config set global.index-url 失败: {}", mirror.url)
            })?;
        Ok(())
    }

    fn test_mirror(&self, mirror: &Mirror) -> Result<u64> {
        // 测试 /simple/ 端点
        let test_url = format!("{}/simple/", mirror.url.trim_end_matches('/'));
        http_head(&test_url)
    }

    fn is_available(&self) -> bool {
        *self.available.get_or_init(|| {
            // 独立 pip 命令可直接使用
            if has_executable("pip3") || has_executable("pip") {
                return true;
            }
            // 回退到 python -m pip，需验证 pip 模块实际存在
            let python = if has_executable("python3") {
                "python3"
            } else if has_executable("python") {
                "python"
            } else {
                return false;
            };
            std::process::Command::new(python)
                .args(["-m", "pip", "--version"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        })
    }

    fn supported_platforms(&self) -> &'static str {
        "Win / Mac / Linux"
    }

    fn current_mirror_name(&self) -> Option<String> {
        let (cmd, args) = Self::pip_args(&["config", "get", "global.index-url"]);
        let output = std::process::Command::new(&cmd)
            .args(&args)
            .output()
            .ok()?;
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if url.is_empty() {
            return None;
        }
        self.mirrors
            .iter()
            .find(|m| m.url.trim_end_matches('/') == url.trim_end_matches('/'))
            .map(|m| m.name.clone())
    }

    // ── 备份/还原/重置 ──

    fn backup(&self) -> Result<()> {
        let (cmd, args) = Self::pip_args(&["config", "get", "global.index-url"]);
        let output = std::process::Command::new(&cmd)
            .args(&args)
            .output()
            .context("无法获取当前 pip index-url")?;
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        save_backup_text(&Self::backup_path(), &url)
    }

    fn restore(&self) -> Result<()> {
        if let Some(url) = load_backup_text(&Self::backup_path()) {
            if !url.is_empty() {
                let (cmd, args) = Self::pip_args(&["config", "set", "global.index-url", &url]);
                let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                run_command(&cmd, &args_ref)
                    .context("pip config set 还原失败")?;
            }
        }
        Ok(())
    }

    fn reset_to_default(&self) -> Result<()> {
        let (cmd, args) = Self::pip_args(&["config", "unset", "global.index-url"]);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        run_command(&cmd, &args_ref)
            .context("pip config unset global.index-url 失败")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pip_adapter_basics() {
        let adapter = PipAdapter {
            mirrors: vec![Mirror::new("PyPI", "https://pypi.org/simple")],
            available: OnceLock::new(),
        };
        assert_eq!(adapter.name(), "pip");
        assert_eq!(adapter.list_mirrors().len(), 1);
    }
}
