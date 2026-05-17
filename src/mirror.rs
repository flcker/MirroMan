use serde::{Deserialize, Serialize};

/// 镜像源定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    /// 镜像源名称（显示用）
    pub name: String,
    /// 镜像源 URL
    pub url: String,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 测速延迟（毫秒），0 表示未测试
    #[serde(default)]
    pub latency_ms: u64,
}

fn default_enabled() -> bool {
    true
}

impl Mirror {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            enabled: true,
            latency_ms: 0,
        }
    }
}
