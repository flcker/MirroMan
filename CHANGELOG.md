# Changelog

## v0.1.1 (2026-05-19)

### 新增

- **pip 适配器**：支持 PyPI 镜像切换、测速、备份还原（自动检测 pip3/pip/python3 -m pip）
- **Go 适配器**：GOPROXY 镜像管理（shell profile 环境变量模式）
- **Pacman 适配器**：Arch Linux mirrorlist 管理（插入最前 + 保留回退）
- **备份/还原机制**：切换前自动备份，`r` 还原到备份，`R` 重置到系统默认
- **批量测速**：`T` 一键测速所有镜像源，弹窗排序显示
- **源验证**：`v` 验证镜像源内容正确性
- **反射器冲突检测**：Pacman 适配器检测 reflector.service 并在 TUI 中警告
- **切换后刷新操作**：弹窗建议并一键执行（如 `pacman -Syy`、`source ~/.zshrc`）
- **交互式 sudo**：临时退出 raw 模式让用户输入密码，自动恢复 TUI
- **旧配置兼容**：`fill_defaults()` 自动补充新包管理器的默认镜像源
- **设计模式规则**：`.agent/rules/design-patterns.md`

### 重构

- **Config 统一访问**：`mirrors_for()` / `mirrors_for_mut()` 消除 `app.rs` 6 处硬编码 match
- **Trait 扩展**：`id` / `backup` / `restore` / `reset_to_default` / `validate_mirror` / `refresh_action` / `conflict_warnings`
- `command.rs` 新增：`sudo_write_file` / `sudo_copy` / `run_sudo` / `git_ls_remote` / `http_head` / 备份文件工具函数
- 测速端点改为各适配器专用路径（Cargo: git ls-remote / npm: /npm / pip: /simple/ / Pacman: core.db）

### 修复

- Sudo 交互导致 TUI 渲染破坏（见下文）

## v0.1.0 (2026-05-17)

### 新增

- 基础 TUI 框架（ratatui + crossterm）
- TOML 配置文件读写，跨平台路径（XDG）
- Cargo 适配器：镜像列表、切换、测速（sparse 协议）
- npm 适配器：镜像列表、切换、测速
- Homebrew 适配器：镜像列表、切换、测速
- TUI 界面：左侧包管理器列表 + 右侧镜像源表格 + 底部帮助栏
- 弹窗：添加镜像、编辑镜像、确认删除、错误提示
- 默认配置内置常用国内镜像
- 当前镜像标识（`*` 前缀）
- 系统平台显示（Win / Mac / Linux、macOS / Linux）
- 动态快捷键栏（根据焦点面板切换）
- 焦点门控（镜像操作仅在右面板生效）
- 导航跳过不可用包管理器

### 修复

- 延迟列数据源从 adapter 副本改为 Config 直读
- Cargo config 合并时重复 source 定义
- 弹窗尺寸过小导致遮挡
- 弹窗 Tab/方向键切换输入行不生效
