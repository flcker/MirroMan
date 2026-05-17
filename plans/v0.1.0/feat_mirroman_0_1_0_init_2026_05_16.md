# MirroMan 0.1.0 初始版本

## 背景

需要一个统一的 TUI 工具来管理包括 Homebrew、npm、Rust/Cargo 等包管理器的镜像源。调研版技术方案已确定使用 Rust + ratatui + TOML 配置 + 适配器模式。

## 目标

- 交付可运行的 TUI 镜像源管理工具
- 支持 Cargo、npm、Homebrew 三个包管理器
- 支持镜像源的列表查看、切换、添加、删除、测速
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
├── 核心调度器
├── 包管理器适配器（Adapter 模式）
│   ├── CargoAdapter
│   ├── NpmAdapter
│   └── HomebrewAdapter
├── 镜像源管理模块
├── 配置读写模块（TOML + 跨平台路径）
├── 系统命令执行模块
└── 日志与错误处理
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

- 左面板：包管理器列表（List widget）
- 右面板：镜像源列表（Table widget，名称/URL/状态/延迟）
- 底部栏：快捷键帮助文本
- 模态弹窗：添加/编辑镜像、确认删除、错误提示

### 快捷键

| 按键 | 操作 |
|------|------|
| q / Esc | 退出 |
| ↑↓ | 导航 |
| Tab | 切换面板 |
| Enter | 切换镜像源 |
| t | 测试当前镜像速度 |
| a | 添加镜像 |
| d | 删除镜像 |
| e | 编辑镜像 |

## 影响范围

- 新项目，无存量代码影响
- 配置文件路径：`~/.config/mirroman/config.toml`（Linux）/ `~/Library/Application Support/com.mirroman.mirroman/config.toml`（macOS）
- 不修改系统级配置（如 `/etc/pacman.d/mirrorlist`），仅操作用户级配置

## 步骤

### Phase 1: 项目骨架
- [ ] 创建 plans/v0.1.0/ + plan 文件
- [ ] 创建 docs/v0.1.0/ 骨架目录
- [ ] cargo init 初始化项目
- [ ] 编写 Cargo.toml 依赖声明
- [ ] 创建模块目录结构 + 空模块文件
- [ ] cargo check 编译验证

### Phase 2: 配置系统
- [ ] Mirror 结构体定义
- [ ] Config / Settings 结构体 + serde 派生
- [ ] 配置路径（directories crate）
- [ ] Config::load() / Config::save()
- [ ] 默认配置模板
- [ ] 单元测试

### Phase 3: 核心抽象层
- [ ] PackageManagerAdapter trait 定义
- [ ] 命令执行封装（command.rs）
- [ ] OS 检测工具（os.rs）
- [ ] 路径工具（paths.rs）
- [ ] 单元测试

### Phase 4: 适配器实现
- [ ] CargoAdapter（修改 ~/.cargo/config.toml）
- [ ] NpmAdapter（npm config set registry）
- [ ] HomebrewAdapter（环境变量/shell profile）
- [ ] 单元测试

### Phase 5: TUI 基础框架
- [ ] Terminal 初始化/恢复
- [ ] App 状态结构体
- [ ] 事件循环（crossterm event polling）
- [ ] 按键分发

### Phase 6: TUI 界面渲染
- [ ] 左面板（包管理器列表）
- [ ] 右面板（镜像源表格）
- [ ] 底部帮助栏
- [ ] 模态弹窗（添加/编辑/删除确认/错误）
- [ ] 交互逻辑完善

### Phase 7: 集成测试 & 打包
- [ ] 端到端功能验证
- [ ] README.md
- [ ] CHANGELOG.md
- [ ] docs/v0.1.0/ 伴生文档
- [ ] cargo build --release

## 进度

- 2026-05-16: Phase 1 进行中

## 风险

| 风险 | 缓解 |
|------|------|
| ratatui 异步集成复杂 | Phase 5 先用同步事件循环 |
| Homebrew 镜像切换需修改 shell profile | 0.1.0 仅支持 .zshrc / .bashrc |
| reqwest openssl 编译慢 | 使用 rustls-tls feature |

## 验证

- [ ] `cargo check` 零错误零警告
- [ ] `cargo test` 全部通过
- [ ] `cargo run` TUI 正常启动退出
- [ ] 切换 Cargo 镜像后 `~/.cargo/config.toml` 正确更新
- [ ] 切换 npm 镜像后 `npm config get registry` 返回正确 URL
- [ ] `cargo build --release` 生成可用二进制
