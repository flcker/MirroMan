pub mod cargo;
pub mod homebrew;
pub mod npm;
pub mod pip;
pub mod goenv;
pub mod pacman;

use crate::mirror::Mirror;
use anyhow::Result;

// ── 辅助结构体 ────────────────────────────────────────────

/// 刷新操作定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshAction {
    /// 操作描述（如 "刷新包数据库"、"重载 shell 环境"）
    pub description: &'static str,
    /// 命令
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 是否需要 sudo
    pub requires_sudo: bool,
}

/// 镜像源验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否可达
    pub reachable: bool,
    /// 延迟（毫秒）
    pub latency_ms: u64,
    /// 内容是否有效
    pub content_valid: bool,
    /// 详情描述
    pub detail: String,
}

// ── 适配器 trait ──────────────────────────────────────────

/// 包管理器适配器 trait
///
/// 每个包管理器（Cargo、npm、Homebrew、pip、Go、Pacman 等）实现此 trait，
/// 提供镜像源列表、切换、测速、备份还原等能力。
pub trait PackageManagerAdapter: Send + Sync {
    // ── 基础标识 ──

    /// 唯一标识符，对应 Config 中的配置段（如 "Cargo"、"pip"）
    /// 默认等于 name()，子类型可覆盖
    fn id(&self) -> &'static str {
        self.name()
    }

    /// 适配器显示名称
    fn name(&self) -> &'static str;

    /// 支持的操作系统描述
    fn supported_platforms(&self) -> &'static str;

    /// 检测该包管理器在当前系统上是否可用
    fn is_available(&self) -> bool;

    // ── 镜像管理 ──

    /// 返回该包管理器的镜像源列表
    fn list_mirrors(&self) -> Vec<Mirror>;

    /// 检测当前激活的镜像名称（None = 无法检测或未设置）
    fn current_mirror_name(&self) -> Option<String> {
        None
    }

    /// 切换当前镜像源为指定镜像
    fn switch_mirror(&self, mirror: &Mirror) -> Result<()>;

    // ── 测试验证 ──

    /// 测试镜像源延迟，返回延迟毫秒数
    fn test_mirror(&self, mirror: &Mirror) -> Result<u64>;

    /// 验证镜像源是否提供正确的仓库服务
    /// 默认实现：回退到 test_mirror 做连通性检查
    fn validate_mirror(&self, mirror: &Mirror) -> Result<ValidationResult> {
        match self.test_mirror(mirror) {
            Ok(latency) => Ok(ValidationResult {
                reachable: true,
                latency_ms: latency,
                content_valid: true,
                detail: format!("延迟 {}ms", latency),
            }),
            Err(e) => Ok(ValidationResult {
                reachable: false,
                latency_ms: 0,
                content_valid: false,
                detail: format!("不可达: {e}"),
            }),
        }
    }

    // ── 备份还原 ──

    /// 切换前备份当前配置（默认空实现，子类覆盖）
    fn backup(&self) -> Result<()> {
        Ok(())
    }

    /// 从备份还原配置（默认空实现，子类覆盖）
    fn restore(&self) -> Result<()> {
        Ok(())
    }

    /// 清除 MirroMan 修改，恢复到系统/官方默认
    fn reset_to_default(&self) -> Result<()>;

    // ── 切换后操作 ──

    /// 切换镜像后建议执行的刷新操作，返回 None 表示无需额外操作
    fn refresh_action(&self) -> Option<RefreshAction> {
        None
    }

    /// 可能与该包管理器冲突的系统服务/配置警告
    /// 例如 Pacman 与 reflector.service 的冲突
    fn conflict_warnings(&self) -> Vec<String> {
        vec![]
    }
}
