# Sudo 交互与 TUI 渲染冲突

## 问题

Pacman 适配器切换镜像需要 sudo 写入 `/etc/pacman.d/mirrorlist`。当 sudo 凭据未缓存时，用户需要输入密码，此时 TUI 事件循环被阻塞，且密码提示与 TUI 界面渲染冲突。

## 症状

1. sudo 密码提示显示在 TUI 界面底部，不明显
2. 用户输入密码期间，q/Esc 等按键无法退出（事件循环阻塞在 sudo 子进程）
3. sudo 完成后，TUI 界面渲染不完整——包管理器列表和镜像表空白，仅底部状态栏可见

## 尝试方案

### 方案 A：提前缓存凭据

用户在另一终端执行 `sudo -v`，缓存凭据后再操作 MirroMan。sudo 凭据未缓存时，MirroMan 弹窗提示。

- 结果：**用户反馈交互不友好**，需要频繁切换终端

### 方案 B：LeaveAlternateScreen + 交互 + EnterAlternateScreen

在 `run_sudo_interactive` 中：
1. `LeaveAlternateScreen` 退出 TUI 的 alternate screen
2. `disable_raw_mode` 回到 cooked 模式
3. 执行 `sudo`（密码提示正常）
4. `enable_raw_mode` + `EnterAlternateScreen` 恢复

- 结果：**TUI 渲染不完整**（症状 3）

### 方案 C：B + Clear(All) 清屏

在方案 B 的 EnterAlternateScreen 后追加 `Clear(All)`，期望下一帧完整渲染。

- 结果：**仍然不完整**。原因：ratatui 使用 diff 渲染——内部缓冲区保留旧帧内容，`Clear(All)` 清屏后缓冲区未同步，下一帧 diff 无变化则不输出任何内容

### 方案 D：不操作 alternate screen，仅切换 raw mode

只 `disable_raw_mode` / `enable_raw_mode`，不触碰 alternate screen。sudo 密码提示覆盖在 TUI 内容之上。

- 结果：**ratatui 状态不被破坏**，但 sudo 密码提示残留可能不完全清除

### 方案 E（最终采用）：D + terminal.clear() 强制重绘

在方案 D 基础上：
1. `run_sudo_interactive` 只切换 raw mode + `Clear(All)` 清屏
2. `App` 新增 `needs_full_redraw` 标志，sudo 相关操作成功后置 `true`
3. `main.rs` 事件循环检测到标志后调用 `terminal.clear()` 重置 ratatui 内部缓冲区
4. 下一帧 `terminal.draw()` 全量重绘

## 最终方案详解

### command.rs

```rust
fn run_sudo_interactive(args: &[&str]) -> Result<()> {
    crossterm::terminal::disable_raw_mode().ok();
    let status = Command::new("sudo").args(args).status()?;
    crossterm::terminal::enable_raw_mode().ok();
    execute!(stdout, Clear(All)).ok();
    // ...
}
```

### app.rs

```rust
pub struct App {
    // ...
    pub needs_full_redraw: bool,  // sudo 操作后设置为 true
}
```

`switch_mirror` / `restore_current` / `reset_current` / 刷新操作成功后均设置 `self.needs_full_redraw = true`。

### main.rs

```rust
while app.running {
    if app.needs_full_redraw {
        terminal.clear()?;          // 重置 ratatui 内部缓冲区
        app.needs_full_redraw = false;
    }
    terminal.draw(|frame| ui::draw(frame, &app))?;
    app.handle_event()?;
}
```

## 经验教训

1. **ratatui 的 diff 渲染**与手动终端操作不兼容。任何对终端的直接写入（`Clear`、`Print`）都会破坏 ratatui 的内部缓冲区一致性，导致后续 diff 输出不完整
2. 需要手动操作终端时（如 sudo 交互），必须配合 `terminal.clear()` 强制重置
3. 不要操作 `LeaveAlternateScreen` / `EnterAlternateScreen`——这些状态由 ratatui terminal 管理，手动介入会导致双重切换
4. 只切换 `raw_mode` 是安全的——不影响 ratatui 的帧缓冲区
