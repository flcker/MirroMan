# 设计模式与设计原则

## 核心原则

### 1. 适配器语义匹配

实现 `PackageManagerAdapter` 时，配置处理方式必须贴合包管理器的原生语义，而不是统一模式。

| 包管理器 | 原生配置语义 | 处理方式 |
|----------|-------------|---------|
| Cargo | 单 registry | 覆写 `[source.*]` 段 |
| npm | 单 registry | `npm config set` (覆写) |
| pip | 单 index-url | `pip config set` (覆写) |
| Homebrew | 单组环境变量 | 删旧行写新行 (覆写) |
| Go | GOPROXY 链式多源 | 写 `GOPROXY=<url>,direct` |
| Pacman | mirrorlist 多源容错 | **插入最前，保留回退** |
| apt (规划) | sources.list 多源 | 每条 `deb` 行独立存在 |
| dnf (规划) | repo 文件独立管理 | 每 repo 独立文件 |

**判断方法**：新增适配器前先回答——
- 配置是单值还是多值？
- 多值之间是互斥还是互补？
- 改动是破坏性（覆写）还是增值性（插入/追加）？

### 2. 适配器模式

所有包管理器通过 `PackageManagerAdapter` trait 接入，核心调度逻辑 (`app.rs`) 不直接依赖具体类型。

```rust
pub trait PackageManagerAdapter: Send + Sync {
    fn id(&self) -> &'static str;       // Config 段标识
    fn name(&self) -> &'static str;     // 显示名称
    fn switch_mirror(&self, m: &Mirror) -> Result<()>;
    fn backup(&self) -> Result<()>;
    fn restore(&self) -> Result<()>;
    fn reset_to_default(&self) -> Result<()>;
    // ...
}
```

### 3. 单一职责 (SRP)

- `config.rs` — 配置读写与统一访问 (`mirrors_for`)
- `adapters/` — 各自包管理器的镜像切换逻辑
- `app.rs` — TUI 事件调度与状态管理
- `ui.rs` — 纯渲染，不包含业务逻辑
- `utils/` — 跨模块通用工具（命令执行、文件操作、OS 检测）

### 4. 开闭原则 (OCP)

新增包管理器时，改动点应局限在：
1. `src/adapters/<name>.rs` — 新增适配器
2. `src/adapters/mod.rs` — 注册模块
3. `src/config.rs` — 新增配置段 + `mirrors_for` 匹配
4. `src/main.rs` — 注册实例

**不应修改 `app.rs` 和 `ui.rs`**（2025-05 v0.1.1 已消除 `app.rs` 中的硬编码 match）。

## 常见反模式

### ❌ 反模式：统一覆写

所有适配器都做"删旧→写新"，忽略包管理器配置语义差异。

```rust
// 错误：Pacman mirrorlist 是多源容错列表，不应覆写
fn switch_mirror(&self, mirror: &Mirror) -> Result<()> {
    write_file(path, format!("Server = {}\n", mirror.url))
}
```

### ✅ 正确：语义匹配

```rust
// 正确：把选中镜像插入最前，保留其余条目作为回退
fn switch_mirror(&self, mirror: &Mirror) -> Result<()> {
    let new_line = format!("Server = {}/$repo/os/$arch", mirror.url);
    let existing = read_file(path);
    // 保留原有 Server 行，选中镜像放最前
    let content = prepend_line(&existing, &new_line);
    write_file(path, &content)
}
```

### ❌ 反模式：硬编码类型分发

```rust
// 错误：每加一个适配器要改 6 处
match adapter_name {
    "Cargo" => self.config.cargo.mirrors.push(m),
    "npm"   => self.config.npm.mirrors.push(m),
    // ...
}
```

### ✅ 正确：通过 trait 方法抽象

```rust
// 正确：新增适配器无需修改此处
if let Some(mirrors) = self.config.mirrors_for_mut(adapter_id) {
    mirrors.push(m);
}
```

## 代码审查清单

新增适配器时确认：

- [ ] 配置处理方式匹配该包管理器的原生语义
- [ ] `id()` 与 `Config::mirrors_for` 中的 key 一致
- [ ] `backup()` 在 `switch_mirror()` 开头被调用
- [ ] `restore()` 可正常工作
- [ ] `reset_to_default()` 正确还原到系统/官方默认
- [ ] 测速端点 (`test_mirror`) 使用该包管理器专有路径（而非泛泛的 URL）
- [ ] 未在 `app.rs` / `ui.rs` 中引入新的具体适配器依赖
