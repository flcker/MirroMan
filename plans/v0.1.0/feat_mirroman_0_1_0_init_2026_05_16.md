# MirroMan 0.1.0 初始版本

## 背景

需要一个统一的 TUI 工具来管理包括 Homebrew、npm、Rust/Cargo 等包管理器的镜像源。调研版技术方案已确定使用 Rust + ratatui + TOML 配置 + 适配器模式。

## 目标

- 交付可运行的 TUI 镜像源管理工具
- 支持 Cargo、npm、Homebrew 三个包管理器
- 支持镜像源的列表查看、切换、添加、编辑、删除、测速
- 配置持久化（TOML），跨平台路径（XDG）
- `cargo build --release` 可用

## 方案

### 技术栈

| 类别 | 选择 |
|------|------|
| TUI 框架 | `ratatui` + `crossterm` |
| 异步运行时 | `tokio` |
| 配置解析 | `toml` + `serde` |
| 路径处理 | `directories` |
| HTTP 客户端 | `reqwest`（rustls-tls） |
| 错误处理 | `anyhow` + `thiserror` |
| 日志 | `tracing` + `tracing-subscriber` |

### 架构

```
MirroMan
├── TUI 层 (ratatui + crossterm)
├── 事件循环与用户交互
├── 包管理器适配器（Adapter 模式）
│   ├── CargoAdapter
│   ├── NpmAdapter
│   └── HomebrewAdapter
├── 镜像源管理模块
├── 配置读写模块（TOML + 跨平台路径）
└── 系统命令执行模块
```

### 项目结构

```
src/
├── main.rs
├── app.rs
├── config.rs
├── tui.rs
├── ui.rs
├── mirror.rs
├── adapters/
│   ├── mod.rs
│   ├── cargo.rs
│   ├── npm.rs
│   └── homebrew.rs
└── utils/
    ├── mod.rs
    ├── command.rs
    ├── os.rs
    └── paths.rs
```

### TUI 布局

- 左面板：包管理器列表（含系统平台信息、可用/不可用状态）
- 右面板：镜像源表格（名称/URL/状态/延迟、当前镜像 `*` 标识）
- 底部栏：动态快捷键（根据焦点面板切换）
- 模态弹窗：添加/编辑镜像、确认删除、错误提示

### 快捷键

| 按键 | 面板 | 操作 |
|------|------|------|
| `q` / `Esc` | 通用 | 退出 |
| `↑` `↓` / `j` `k` | 通用 | 上下导航 |
| `Tab` | 通用 | 切换焦点面板 |
| `Enter` | 右面板 | 切换当前选中镜像源 |
| `t` | 右面板 | 测试当前镜像源延迟 |
| `a` | 右面板 | 添加新镜像源 |
| `d` | 右面板 | 删除选中镜像源 |
| `e` | 右面板 | 编辑选中镜像源 |

## 影响范围

- 新项目，无存量代码影响
- 配置文件路径：`~/.config/mirroman/config.toml`（Linux）/ `~/Library/Application Support/com.mirroman.MirroMan/config.toml`（macOS）
- 不修改系统级配置，仅操作用户级配置

## 步骤

### Phase 1: 项目骨架 ✅
- [x] 创建 plans/v0.1.0/ + plan 文件
- [x] 创建 docs/v0.1.0/ 骨架目录
- [x] cargo init 初始化项目
- [x] 编写 Cargo.toml 依赖声明
- [x] 创建模块目录结构 + 空模块文件
- [x] cargo check 编译验证

### Phase 2: 配置系统 ✅
- [x] Mirror 结构体定义
- [x] Config / Settings 结构体 + serde 派生
- [x] 配置路径（directories crate）
- [x] Config::load() / Config::save()
- [x] 默认配置模板（含 rsproxy sparse 协议）
- [x] 单元测试

### Phase 3: 核心抽象层 ✅
- [x] PackageManagerAdapter trait 定义
- [x] 命令执行封装（command.rs）
- [x] OS 检测工具（os.rs）
- [x] 路径工具（paths.rs）
- [x] 单元测试

### Phase 4: 适配器实现 ✅
- [x] CargoAdapter（修改 ~/.cargo/config.toml，sparse 协议，逐行清理旧段）
- [x] NpmAdapter（npm config set registry）
- [x] HomebrewAdapter（环境变量/shell profile，过滤旧 HOMEBREW_* 行）
- [x] 单元测试

### Phase 5: TUI 基础框架 ✅
- [x] Terminal 初始化/恢复
- [x] App 状态结构体（含 current_mirror_names）
- [x] 事件循环（crossterm event polling，matches! 分发）
- [x] 按键分发（焦点门控、导航跳过不可用项）

### Phase 6: TUI 界面渲染 ✅
- [x] 左面板（包管理器列表 + 系统平台 + 可用/不可用样式）
- [x] 右面板（镜像源表格 + 当前镜像 `*` 前缀）
- [x] 底部帮助栏（动态快捷键，按键名黄色加粗）
- [x] 模态弹窗（添加/编辑共用表单 + 确认删除 + 错误提示，带快捷键栏）
- [x] 交互逻辑完善（弹窗方向键/Tab 切换输入行，Enter 保存）

### Phase 7: 集成测试 & 打包 ✅
- [x] 端到端功能验证
- [x] README.md
- [x] CHANGELOG.md
- [x] docs/v0.1.0/ 伴生文档（design / guides / changes / summary）
- [x] cargo build --release

## 进度

- 2026-05-16: Phase 1-6 完成
- 2026-05-17: Phase 7 完成，功能迭代完善

## v0.1.0 功能清单

| 功能 | 状态 |
|------|------|
| Cargo / npm / Homebrew 适配器 | ✅ |
| 镜像列表/切换/添加/编辑/删除 | ✅ |
| 镜像测速（HTTP HEAD） | ✅ |
| 当前镜像标识 `*` | ✅ |
| 系统平台显示 | ✅ |
| 动态快捷键栏 | ✅ |
| 焦点门控 | ✅ |
| 导航跳过不可用适配器 | ✅ |
| TOML 配置持久化 | ✅ |
| Cargo sparse 协议支持 | ✅ |

## 风险

| 风险 | 缓解 |
|------|------|
| ratatui 异步集成复杂 | Phase 5 先用同步事件循环 |
| Homebrew 镜像切换需修改 shell profile | 0.1.0 仅支持 .zshrc / .bashrc |
| reqwest openssl 编译慢 | 使用 rustls-tls feature |
| Cargo config 合并产生重复 source | 逐行遍历清空所有旧段再追加 |

## 验证

- [x] `cargo check` 零错误（4 个预留 API 警告）
- [x] `cargo test` 10/10 通过
- [x] `cargo run` TUI 正常启动退出
- [x] 切换 Cargo 镜像后 `~/.cargo/config.toml` 正确更新，无重复定义
- [x] 切换 npm 镜像后 `npm config get registry` 返回正确 URL
- [x] `cargo build --release` 生成可用二进制（~6.6 MB）
