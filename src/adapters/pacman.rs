use crate::adapters::{PackageManagerAdapter, RefreshAction};
use crate::config::{backup_dir, Config};
use crate::mirror::Mirror;
use crate::utils::command::{http_head, sudo_copy, sudo_write_file};
use crate::utils::os::{current_os, has_executable, Os};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct PacmanAdapter {
    mirrors: Vec<Mirror>,
    /// 冲突警告（懒加载，避免每帧执行 systemctl）
    warnings: OnceLock<Vec<String>>,
}

impl PacmanAdapter {
    pub fn new(config: &Config) -> Self {
        Self {
            mirrors: config.mirrors.pacman.mirrors.clone(),
            warnings: OnceLock::new(),
        }
    }

    /// mirrorlist 文件路径
    fn mirrorlist_path() -> &'static Path {
        Path::new("/etc/pacman.d/mirrorlist")
    }

    /// 备份文件路径
    fn backup_path() -> PathBuf {
        backup_dir().join("Pacman").join("mirrorlist")
    }
}

impl PackageManagerAdapter for PacmanAdapter {
    fn name(&self) -> &'static str {
        "Pacman"
    }

    fn list_mirrors(&self) -> Vec<Mirror> {
        self.mirrors.clone()
    }

    fn switch_mirror(&self, mirror: &Mirror) -> Result<()> {
        self.backup()?;

        let path = Self::mirrorlist_path();
        let new_line = format!(
            "Server = {}/$repo/os/$arch",
            mirror.url.trim_end_matches('/')
        );

        // 读取现有 mirrorlist，保留所有 Server 行作为回退
        let existing = if path.exists() {
            std::fs::read_to_string(path).unwrap_or_default()
        } else {
            String::new()
        };

        // 收集所有 Server 行（去掉已有镜像重复项），
        // 把选中的镜像插入到最前面
        let existing_servers: Vec<&str> = existing
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("Server =") && trimmed != new_line.as_str()
            })
            .collect();

        let mut content = String::from("## MirroMan: Pacman mirror\n");
        content.push_str(&new_line);
        content.push('\n');
        for s in &existing_servers {
            content.push_str(s);
            content.push('\n');
        }

        sudo_write_file(path, &content)
            .with_context(|| "写入 /etc/pacman.d/mirrorlist 失败，需要 sudo 权限")?;
        Ok(())
    }

    fn test_mirror(&self, mirror: &Mirror) -> Result<u64> {
        // 测试 core.db 端点
        let test_url = format!("{}/core/os/x86_64/core.db", mirror.url.trim_end_matches('/'));
        http_head(&test_url)
    }

    fn is_available(&self) -> bool {
        let os = current_os();
        matches!(os, Os::Linux) && has_executable("pacman")
    }

    fn supported_platforms(&self) -> &'static str {
        "Linux (Arch)"
    }

    fn current_mirror_name(&self) -> Option<String> {
        let content = std::fs::read_to_string(Self::mirrorlist_path()).ok()?;
        let server_line = content
            .lines()
            .find(|line| line.trim().starts_with("Server ="))?;

        let url = server_line
            .trim_start_matches("Server = ")
            .trim()
            .split("/$repo")
            .next()?
            .to_string();

        self.mirrors
            .iter()
            .find(|m| url.starts_with(&m.url.trim_end_matches('/')))
            .map(|m| m.name.clone())
    }

    // ── 备份/还原/重置 ──

    fn backup(&self) -> Result<()> {
        let src = Self::mirrorlist_path();
        let dst = Self::backup_path();
        if src.exists() {
            // 确保备份目录存在
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            sudo_copy(src, &dst)
                .with_context(|| "备份 mirrorlist 失败，需要 sudo 权限")?;
        }
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        let src = Self::backup_path();
        let dst = Self::mirrorlist_path();
        if src.exists() {
            sudo_copy(&src, dst)
                .with_context(|| "还原 mirrorlist 失败，需要 sudo 权限")?;
        }
        Ok(())
    }

    fn reset_to_default(&self) -> Result<()> {
        // pacman 默认就是之前备份的版本
        self.restore()
    }

    // ── 冲突检测 ──

    fn conflict_warnings(&self) -> Vec<String> {
        self.warnings
            .get_or_init(|| {
                let mut warnings = Vec::new();

                if let Ok(output) = std::process::Command::new("systemctl")
                    .args(["is-active", "reflector"])
                    .output()
                {
                    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if status == "active" {
                        warnings.push(
                            "reflector.service 正在运行，会定时覆写 mirrorlist，\n与 MirroMan 冲突。建议: sudo systemctl disable --now reflector"
                                .to_string(),
                        );
                    }
                }

                warnings
            })
            .clone()
    }

    // ── 切换后提示 ──

    fn refresh_action(&self) -> Option<RefreshAction> {
        Some(RefreshAction {
            description: "刷新包数据库",
            command: "pacman".to_string(),
            args: vec!["-Syy".to_string()],
            requires_sudo: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacman_adapter_basics() {
        let adapter = PacmanAdapter {
            mirrors: vec![Mirror::new(
                "清华 TUNA",
                "https://mirrors.tuna.tsinghua.edu.cn/archlinux",
            )],
            warnings: OnceLock::new(),
        };
        assert_eq!(adapter.name(), "Pacman");
        assert_eq!(adapter.list_mirrors().len(), 1);
    }
}
