use std::path::PathBuf;

/// 获取用户 HOME 目录
pub fn home_dir() -> Option<PathBuf> {
    dirs_next()
}

/// 展开 `~` 为 HOME 目录
pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs_next() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| {
            std::env::var("USERPROFILE")
        })
        .ok()
        .map(PathBuf::from)
}
