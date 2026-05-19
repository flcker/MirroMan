# 项目说明

此文件为 AI 助手提供项目上下文参考。

## 项目类型

TUI 包管理器镜像源管理工具。基于 Rust + ratatui + crossterm，跨平台（Linux / macOS / Windows）。

## 构建与测试

```bash
cargo check          # 编译检查
cargo test           # 运行测试
cargo run            # 启动 TUI
cargo build --release # 发布构建
```

## 架构

```
src/
├── main.rs              # 入口，初始化 TUI + 适配器列表 + 事件循环
├── app.rs               # App 状态机、事件分发、所有操作逻辑
├── config.rs            # TOML 配置读写、mirrors_for() 统一访问
├── mirror.rs            # Mirror 结构体（name/url/enabled/latency_ms）
├── tui.rs               # 终端 init/restore
├── ui.rs                # 纯渲染（左面板+右表格+帮助栏+弹窗）
├── adapters/
│   ├── mod.rs           # PackageManagerAdapter trait + 辅助结构体
│   ├── cargo.rs          # Cargo (~/.cargo/config.toml)
│   ├── npm.rs            # npm (npm config set registry)
│   ├── pip.rs            # pip (pip/pip3/python3 -m pip)
│   ├── goenv.rs          # Go GOPROXY (shell profile)
│   ├── homebrew.rs       # Homebrew (shell profile 环境变量)
│   └── pacman.rs         # Pacman (/etc/pacman.d/mirrorlist)
└── utils/
    ├── command.rs        # 命令执行、sudo 操作、HTTP 测速
    ├── os.rs             # OS 检测、可执行文件查找
    └── paths.rs          # 跨平台路径（tilde expansion）
```

## 设计约束

- **app.rs 不依赖具体适配器类型**，通过 `PackageManagerAdapter` trait 和 `Config::mirrors_for()` 分发
- **新增包管理器**改动局限在 4 个文件：`adapters/<name>.rs` + `adapters/mod.rs` + `config.rs` + `main.rs`
- **ui.rs 纯渲染**，不含业务逻辑
- **command.rs** 所有 sudo 操作通过 `run_sudo()` 统一入口（sudo -n 静默 / 交互式 raw mode 切换）
- **ratatui 渲染**：不要在适配器或 command 层操作 `alternate screen`，仅 raw mode 切换配合 `terminal.clear()` 全量重绘

## 快捷键

| 键 | 操作 |
|----|------|
| `q`/`Esc` | 退出 |
| `↑↓`/`jk` | 导航 |
| `Tab` | 切换焦点面板 |
| `Enter` | 切换当前选中镜像源 |
| `t` | 测速当前镜像 |
| `T` | 批量测速所有镜像 |
| `v` | 验证镜像源内容 |
| `a` | 添加新镜像源 |
| `d` | 删除选中镜像源 |
| `e` | 编辑选中镜像源 |
| `r` | 还原到备份 |
| `R` | 重置到系统默认 |

## 待办事项

- [后续需求与优化](docs/TODO.md)

## 规则文件

项目规则按模块拆分存放于 `rules/` 目录，团队共享，纳入版本控制：

- [模型与执行策略](.agent/rules/model-strategy.md)
- [上下文管理策略](.agent/rules/context-strategy.md)
- [Plan 文件规范](.agent/rules/plan-convention.md)
- [文档规范](.agent/rules/doc-convention.md)
- [设计模式与设计原则](.agent/rules/design-patterns.md)
