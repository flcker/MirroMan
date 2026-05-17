pub mod cargo;
pub mod homebrew;
pub mod npm;

use crate::mirror::Mirror;
use anyhow::Result;

/// 包管理器适配器 trait
///
/// 每个包管理器（Cargo、npm、Homebrew 等）实现此 trait，
/// 提供镜像源列表、切换、测速等能力。
pub trait PackageManagerAdapter: Send + Sync {
    /// 适配器显示名称
    fn name(&self) -> &'static str;

    /// 返回该包管理器的镜像源列表
    fn list_mirrors(&self) -> Vec<Mirror>;

    /// 切换当前镜像源为指定镜像
    fn switch_mirror(&self, mirror: &Mirror) -> Result<()>;

    /// 测试镜像源延迟，返回延迟毫秒数
    fn test_mirror(&self, mirror: &Mirror) -> Result<u64>;

    /// 检测该包管理器在当前系统上是否可用
    fn is_available(&self) -> bool;

    /// 检测当前激活的镜像名称（None = 无法检测或未设置）
    fn current_mirror_name(&self) -> Option<String> {
        None
    }
}
