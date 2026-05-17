use anyhow::{Context, Result};
use std::process::Command;

/// 执行系统命令，返回标准输出（UTF-8 字符串）
pub fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("执行命令失败: {} {:?}", cmd, args))?;

    if output.status.success() {
        String::from_utf8(output.stdout).context("命令输出非 UTF-8")
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("命令执行失败: {} {:?}\n{}", cmd, args, stderr)
    }
}

/// 执行系统命令，忽略失败（返回 None）
pub fn run_command_opt(cmd: &str, args: &[&str]) -> Option<String> {
    run_command(cmd, args).ok()
}
