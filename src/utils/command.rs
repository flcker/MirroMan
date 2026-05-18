use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
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

// ── sudo 操作 ─────────────────────────────────────────────

/// 检测当前进程是否以 root 运行
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// 检测 sudo 凭据是否已缓存（sudo -n 不要求密码交互）
/// 返回 true 表示可以无交互执行 sudo，false 表示需要先 sudo -v
pub fn sudo_available() -> bool {
    std::process::Command::new("sudo")
        .args(["-n", "true"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// 执行 sudo 命令（自动选择静默或交互模式）
/// - 凭据已缓存：sudo（静默执行）
/// - 凭据未缓存：临时退出 TUI cooked 模式让用户输入密码，完成后恢复
pub fn run_sudo(args: &[&str]) -> Result<()> {
    if sudo_available() {
        run_command("sudo", args)?;
    } else {
        run_sudo_interactive(args)?;
    }
    Ok(())
}

/// 交互式 sudo：临时退出 raw 模式让用户输入密码，
/// 完成后恢复 raw 模式（清屏由下一帧 ratatui draw 完成）
fn run_sudo_interactive(args: &[&str]) -> Result<()> {
    // 临时退出 raw 模式，sudo 密码提示在 cooked 模式下正常显示
    crossterm::terminal::disable_raw_mode().ok();

    // 执行 sudo（sudo 自身会显示密码提示）
    let status = std::process::Command::new("sudo")
        .args(args)
        .status()
        .context("sudo 执行失败")?;

    // 恢复 raw 模式，清屏消除 sudo 密码提示残留
    crossterm::terminal::enable_raw_mode().ok();
    use crossterm::execute;
    execute!(
        std::io::stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
    )
    .ok();

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("sudo 命令执行失败 (exit code: {:?})", status.code())
    }
}

/// 以 sudo 权限写入文件
/// - 如果是 root：直接写入
/// - 如果不是 root：写临时文件 + run_sudo（自动选择静默或交互模式）
pub fn sudo_write_file(path: &Path, content: &str) -> Result<()> {
    if is_root() {
        fs::write(path, content)
            .with_context(|| format!("无法写入: {}", path.display()))?;
    } else {
        let tmp = tempfile::NamedTempFile::new().context("无法创建临时文件")?;
        fs::write(tmp.path(), content)
            .with_context(|| format!("无法写入临时文件: {}", tmp.path().display()))?;
        run_sudo(&[
            "cp",
            tmp.path().to_str().unwrap_or(""),
            path.to_str().unwrap_or(""),
        ])
        .with_context(|| format!("sudo cp 到 {} 失败", path.display()))?;
    }
    Ok(())
}

/// 以 sudo 权限复制文件
/// - 如果是 root：直接复制
/// - 如果不是 root：run_sudo cp（自动选择静默或交互模式）
pub fn sudo_copy(src: &Path, dst: &Path) -> Result<()> {
    if is_root() {
        fs::copy(src, dst)
            .with_context(|| format!("复制失败: {} → {}", src.display(), dst.display()))?;
    } else {
        run_sudo(&[
            "cp",
            src.to_str().unwrap_or(""),
            dst.to_str().unwrap_or(""),
        ])
        .with_context(|| format!("sudo cp {} 失败", src.display()))?;
    }
    Ok(())
}

/// 备份文件到目标路径（普通用户文件，不需要 sudo）
pub fn backup_file(src: &Path, dst: &Path) -> Result<()> {
    if src.exists() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建目录: {}", parent.display()))?;
        }
        fs::copy(src, dst)
            .with_context(|| format!("备份失败: {} → {}", src.display(), dst.display()))?;
    }
    Ok(())
}

/// 从备份还原文件
pub fn restore_file(src: &Path, dst: &Path) -> Result<()> {
    if src.exists() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建目录: {}", parent.display()))?;
        }
        fs::copy(src, dst)
            .with_context(|| format!("还原失败: {} → {}", src.display(), dst.display()))?;
    }
    Ok(())
}

/// 保存文本内容到备份文件
pub fn save_backup_text(dst: &Path, content: &str) -> Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {}", parent.display()))?;
    }
    fs::write(dst, content)
        .with_context(|| format!("写入备份失败: {}", dst.display()))?;
    Ok(())
}

/// 读取备份文本内容
pub fn load_backup_text(src: &Path) -> Option<String> {
    fs::read_to_string(src).ok()
}

/// 执行 git ls-remote 测试 git 仓库可达性，返回延迟毫秒数
pub fn git_ls_remote(url: &str) -> Result<u64> {
    let start = std::time::Instant::now();
    run_command("git", &["ls-remote", "--heads", url, "HEAD"])
        .context("git ls-remote 失败，镜像 git 仓库不可达")?;
    Ok(start.elapsed().as_millis() as u64)
}

/// 发送 HTTP HEAD 请求，返回延迟毫秒数
pub fn http_head(url: &str) -> Result<u64> {
    let start = std::time::Instant::now();
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .context("无法创建 HTTP 客户端")?;
    client.head(url).send().context("HTTP HEAD 请求失败")?;
    Ok(start.elapsed().as_millis() as u64)
}
