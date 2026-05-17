use std::path::PathBuf;

/// 操作系统类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    MacOS,
    Windows,
}

/// 获取当前操作系统
pub fn current_os() -> Os {
    if cfg!(target_os = "linux") {
        Os::Linux
    } else if cfg!(target_os = "macos") {
        Os::MacOS
    } else if cfg!(target_os = "windows") {
        Os::Windows
    } else {
        // 默认按 Linux 处理
        Os::Linux
    }
}

/// 查找可执行文件路径（模拟 `which`）
pub fn find_executable(name: &str) -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        // Windows: where
        let output = std::process::Command::new("where")
            .arg(name)
            .output()
            .ok()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().next().map(PathBuf::from)
        } else {
            None
        }
    } else {
        // Unix: which
        let output = std::process::Command::new("which")
            .arg(name)
            .output()
            .ok()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Some(PathBuf::from(stdout.trim()))
        } else {
            None
        }
    }
}

/// 检测指定名称的可执行文件是否存在
pub fn has_executable(name: &str) -> bool {
    find_executable(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_os() {
        // 至少能调用不 panic
        let os = current_os();
        assert!(matches!(os, Os::Linux | Os::MacOS | Os::Windows));
    }

    #[test]
    fn test_has_executable() {
        // cargo 和 ls/sh 应该基本都存在
        assert!(has_executable("cargo"));
    }
}
