# MirroMan

统一的包管理器镜像源管理 TUI 工具。

## 支持

- **Cargo** (Rust) — 跨平台，修改 `~/.cargo/config.toml`
- **npm** — 跨平台，执行 `npm config set registry`
- **Homebrew** — macOS / Linux，写入 shell profile 环境变量

## 安装

### Cargo

```bash
cargo install mirroman
```

### 手动编译

```bash
git clone https://github.com/mirroman/mirroman.git
cd mirroman
cargo build --release
```

二进制文件位于 `target/release/mirroman`。

## 使用

```bash
mirroman
```

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

## 配置

首次运行自动生成默认配置文件：

- **Linux**: `~/.config/mirroman/config.toml`
- **macOS**: `~/Library/Application Support/com.mirroman.MirroMan/config.toml`

默认包含 Cargo、npm、Homebrew 的常用国内镜像源。

## 开发

```bash
cargo check   # 编译检查
cargo test    # 运行测试
cargo run     # 启动 TUI
```

## 许可

MIT
