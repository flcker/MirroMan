## **MirroMan** 技术方案

---

**MirroMan** : 基于 Rust + TUI + TOML 配置，支持跨平台（Linux、macOS、Windows）。


# MirroMan 技术方案

## 一、整体架构

```
MirroMan
├── TUI 层 (ratatui + crossterm)
├── 事件循环与用户交互
├── 核心调度器
├── 包管理器适配器（Adapter 模式）
│   ├── HomebrewAdapter
│   ├── PacmanAdapter
│   ├── AURAdapter
│   ├── NpmAdapter
│   ├── CargoAdapter
│   └── (未来可扩展)
├── 镜像源管理模块（增删改查、测试、启用/禁用）
├── 配置读写模块（TOML + 跨平台路径）
├── 系统命令执行模块（异步、超时）
└── 日志与错误处理
```

## 二、技术栈选型

| 类别             | 选择                                                         | 理由                                                         |
| ---------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| **TUI 框架**     | `ratatui` + `crossterm`                                      | 跨平台（终端能力），活跃维护，易于构建响应式界面。crossterm 支持 Windows 终端。 |
| **异步运行时**   | `tokio`（可选）或 `std::thread` + 阻塞命令                   | TUI 中执行网络测试（ping 镜像）或更新配置需避免阻塞界面。建议 `tokio` + `tui` 集成。 |
| **配置解析**     | `toml` + `serde`                                             | 原生支持，稳定高效。                                         |
| **路径处理**     | `directories` 或 `home` + `std::path::PathBuf`               | 跨平台获取配置目录（XDG on Linux, ~/Library/Application Support on macOS, %APPDATA% on Windows）。 |
| **命令执行**     | `std::process::Command` + `tokio::process` (异步版)          | 执行 `brew`, `pacman`, `npm`, `cargo` 等命令并解析输出。     |
| **HTTP 客户端**  | `reqwest`（blocking 或 async）                               | 测试镜像源可用性（可选，发送 HEAD 请求或简单的 GET 检查）。  |
| **错误处理**     | `anyhow` + `thiserror`                                       | 方便错误传播和上下文。                                       |
| **日志**         | `tracing` 或 `log` + `env_logger`                            | 便于调试。                                                   |
| **测试**         | `rstest`, `mockall` 或手动 trait 模拟                        | 对适配器进行单元测试。                                       |
| **打包分发**     | `cargo build --release` + 可选 `cargo-bundle` 或 GitHub Actions 构建二进制 | 提供各平台预编译二进制。                                     |

## 三、跨平台细节

### 1. 配置文件路径（遵循 XDG Base Directory 规范）

使用 `directories` crate:

```rust
use directories::ProjectDirs;

fn config_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "yourname", "mirroman").unwrap();
    proj_dirs.config_dir().join("config.toml")
}
// Linux:   ~/.config/mirroman/config.toml
// macOS:   ~/Library/Application Support/com.yourname.mirroman/config.toml
// Windows: C:\Users\Alice\AppData\Roaming\com\yourname\mirroman\config.toml
```

### 2. 包管理器命令的可执行文件路径

- 需在运行时通过 `which` (Unix) 或 `where` (Windows) 查找，或读取环境变量。
- 推荐在配置中允许用户指定自定义路径。

### 3. 各包管理器镜像切换方法的跨平台兼容性

| 包管理器 | 切换方式（跨平台注意事项）                                   |
| -------- | ------------------------------------------------------------ |
| Homebrew | 仅 macOS/Linux。修改 git remote 或环境变量 `HOMEBREW_API_DOMAIN` 等。使用 `Command` 执行 `brew tap` 或直接重写配置文件。 |
| Pacman   | 仅 Linux (Arch)。修改 `/etc/pacman.d/mirrorlist` 需要 sudo。可提示用户手动执行或使用 `sudo` 包装（需配置无密码 sudo 或 `pkexec`）。 |
| AUR      | 仅 Linux。修改 git remote (aur 仓库)。                       |
| npm      | 跨平台。运行 `npm config set registry <url>`。               |
| Cargo    | 跨平台。修改 `~/.cargo/config.toml` 文件。                   |

> 对于需要提权的操作（如 pacman），可以在 TUI 中检测权限，提示用户以 sudo 重新运行 MirroMan，或提供 sudo 包装命令。

### 4. 终端兼容性

- `crossterm` 支持 Windows 10+ 终端、ConPTY、MSYS2 等。
- 对于 Windows 上缺少 Homebrew/Pacman 的情况，在 TUI 中可自动隐藏不可用的包管理器（根据 OS 或命令是否存在）。

## 四、核心模块设计

### 1. 配置结构（用 Rust 类型定义）

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub settings: Settings,
    pub managers: Vec<ManagerConfig>, // 支持动态扩展
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub default_manager: Option<String>,
    pub test_timeout_secs: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")] // 根据 type 字段反序列化为不同枚举变体
pub enum ManagerConfig {
    Homebrew(HomebrewConfig),
    Pacman(PacmanConfig),
    AUR(AurConfig),
    Npm(NpmConfig),
    Cargo(CargoConfig),
}

// 各配置结构体包含镜像源列表
#[derive(Debug, Deserialize, Serialize)]
pub struct HomebrewConfig {
    pub mirrors: Vec<Mirror>,
    // ... 其他特有字段
}
```

### 2. 适配器 Trait

```rust
pub trait PackageManagerAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn list_mirrors(&self) -> Vec<Mirror>;
    fn switch_mirror(&self, mirror: &Mirror) -> Result<(), anyhow::Error>;
    fn test_mirror(&self, mirror: &Mirror) -> Result<u64, anyhow::Error>; // 返回延迟 ms
    fn is_available(&self) -> bool; // 检测命令是否存在、OS 是否支持
}
```

### 3. 镜像源测试

- 简单方案：`reqwest::blocking::Client` 发送 HEAD 请求，测量时间（注意超时）。
- 对于 git 镜像（Homebrew, AUR, Cargo），可尝试 `git ls-remote --heads <url>` 并测量时间，但可能较慢；备选为 HTTP 检测（通常镜像也提供 HTTPS 访问）。

### 4. TUI 布局建议

- 左面板：包管理器列表（用 `ratatui::widgets::List`）
- 右面板：当前选中包管理器的镜像源列表，显示名称、URL、是否启用、测试延迟
- 底部：帮助文本（快捷键：切换、激活、测试、添加镜像、删除、保存等）
- 模态框（popup）：添加/编辑镜像源，确认删除

## 五、关键实现流程

### 1. 启动
- 读取配置文件，若不存在则创建默认模板。
- 根据 OS 和可用命令，实例化适配器。
- 初始化 TUI，进入事件循环。

### 2. 切换镜像
- 用户选择镜像 → 调用适配器的 `switch_mirror` → 执行相应系统命令或写文件 → 显示结果反馈。

### 3. 添加/删除镜像
- 用户输入新镜像名称、URL → 写入配置的 `managers` 对应项 → 刷新 TUI 显示。
- 注意：修改配置后需调用适配器的更新逻辑（或提醒用户手动生效，如 `npm config set` 立即生效）。

### 4. 测试镜像
- 在后台异步执行（不阻塞 UI），显示进度，最终更新延迟值。

### 5. 错误处理
- 所有可能失败的操作（命令执行失败、配置写入失败）都通过 `anyhow::Result` 传播，在 TUI 中显示错误弹出框，不影响主循环。

## 六、项目结构示例

```
mirroman/
├── Cargo.toml
├── src/
│   ├── main.rs               # 入口，初始化
│   ├── app.rs                # App 状态及事件循环
│   ├── config.rs             # 配置读取/写入
│   ├── tui.rs                # TUI 初始化和清理
│   ├── ui.rs                 # 界面渲染逻辑
│   ├── adapters/
│   │   ├── mod.rs            # 适配器 trait 定义
│   │   ├── homebrew.rs
│   │   ├── pacman.rs
│   │   ├── aur.rs
│   │   ├── npm.rs
│   │   └── cargo.rs
│   ├── mirror.rs             # Mirror 结构体定义
│   └── utils/
│       ├── command.rs        # 执行命令封装
│       ├── os.rs             # OS 检测
│       └── paths.rs          # 跨平台路径
└── assets/
    └── default_config.toml   # 默认配置模板
```

## 七、构建与分发

### 1. 编译
```bash
cargo build --release
```
目标文件位于 `target/release/mirroman`

### 2. 跨平台构建
- 本地构建：在目标 OS 上直接 `cargo build --release`
- CI（GitHub Actions）：使用 `cross` 或矩阵构建

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
```

### 3. 打包
- 提供单一二进制文件，无需额外运行时。
- 可打包为 `.deb`, `.rpm`, `.app` (macOS), `.msi` (Windows) 使用 `cargo-bundle` 或 `wix`。

### 4. 安装脚本
- `cargo install mirroman`（发布到 crates.io）
- 使用 Homebrew tap（macOS/Linux）或 scoop（Windows）

## 八、未来扩展

- 插件化支持：允许用户自定义包管理器配置（通过 TOML 定义命令模板）。
- 镜像源自动测速排序。
- 支持代理设置。
- 支持导入/导出镜像源列表。

## 九、总结

- **语言**：Rust
- **配置**：TOML，跨平台路径（directories）
- **TUI**：ratatui + crossterm
- **适配器**：trait + 枚举，支持动态加载
- **分发**：单一二进制，支持三大平台

此方案可确保 MirroMan 既轻量又可靠，且易于维护和扩展。需要的话我可以进一步提供核心代码骨架或配置读写实现。
