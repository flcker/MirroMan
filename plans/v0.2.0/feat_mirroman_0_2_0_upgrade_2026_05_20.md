# MirroMan v0.2.0 升级计划

## 背景

v0.1.0 支持 Cargo / npm / Homebrew 三个包管理器。v0.2.0 目标是扩大覆盖范围至 6 个包管理器，增强安全性和可定制性。

## 目标

- 新增 pip / Go / Pacman 适配器
- 切换前自动备份，支持还原和重置
- sudo 操作支持（Pacman 需要）
- 配置文件拆分 settings.toml + mirrors.toml
- 主题系统：支持 dark/light 预设和自定义
- 消除 app.rs 硬编码，统一通过 Config::mirrors_for() 分发
- 批量测速、源验证、切换后刷新操作
- reflector 冲突检测

## 方案

### 适配器

| 适配器 | 配置方式 | 特殊处理 |
|--------|---------|---------|
| pip | pip config set global.index-url | 自动检测 pip3/pip/python3 -m pip |
| Go | GOPROXY 环境变量 (shell profile) | 链式回退 `,direct` |
| Pacman | /etc/pacman.d/mirrorlist | 插入最前 + 保留回退，sudo 交互式 |

### 备份还原

PackageManagerAdapter trait 新增：
- `backup()` — 切换前自动备份
- `restore()` — 还原到备份
- `reset_to_default()` — 清除修改

TUI 快捷键：`r` 还原，`R` 重置

### 配置文件拆分

```
config.toml → settings.toml + mirrors.toml
```

- settings.toml：全局设置 + 主题
- mirrors.toml：6 个包管理器镜像数据
- 自动迁移旧 config.toml

### 主题系统

- `[themes.dark]` / `[themes.light]` 内置预设
- 支持颜色名和 `#RRGGBB` 十六进制
- 未设置字段回退到内置默认值

### sudo 交互

- `run_sudo()` 统一入口：静默模式（凭据已缓存）或交互模式（临时退出 raw mode）
- 不操作 alternate screen，避免破坏 ratatui 状态
- `terminal.clear()` 强制重绘

### app.rs 重构

- `current_mirrors()` / `add/delete/edit` 全部改为通过 `Config::mirrors_for()` 分发
- 新增 `PostSwitchRefresh` / `ConfirmRestore` / `ConfirmReset` / `BatchTestResults` 弹窗
- `needs_full_redraw` 标志处理 sudo 后渲染恢复

## 影响范围

| 文件 | 变更 |
|------|------|
| src/config.rs | 拆分 Settings + Mirrors，迁移逻辑，mirrors_for/mut，fill_defaults + ThemeConfig |
| src/adapters/mod.rs | trait 新增 8 个方法 + RefreshAction/ValidationResult |
| src/adapters/cargo.rs | backup/restore/reset + 测速改进 (git ls-remote) |
| src/adapters/npm.rs | backup/restore/reset + 测速改进 (/npm) |
| src/adapters/homebrew.rs | backup/restore/reset + refresh_action(source) |
| src/adapters/pip.rs | 新增 |
| src/adapters/goenv.rs | 新增 |
| src/adapters/pacman.rs | 新增 |
| src/app.rs | 6 处 match 去硬编码 + needs_full_redraw + 新快捷键 |
| src/ui.rs | 布局改上下分 + 4 新弹窗 + palette 参数 |
| src/palette.rs | 新增 |
| src/utils/command.rs | sudo_write_file/copy/run_sudo/git_ls_remote/http_head |
| src/main.rs | 新适配器注册 + palette 构建 |

## 步骤

1. Config 重构 — mirrors_for/mirrors_for_mut + 新配置段
2. Trait 新增 — id/backup/restore/reset_to_default/validate_mirror/refresh_action/conflict_warnings
3. command.rs 新增 — sudo 工具函数
4. 现有适配器补充 — backup/restore/reset + 改进 test_mirror
5. app.rs 去硬编码 + PostSwitchRefresh + 批量测速
6. main.rs 初始化备份目录
7. pip / Go / Pacman 适配器
8. TUI 快捷键 r/R/T/v + 刷新弹窗 + 批量测速
9. sudo 交互迭代 (5 次修复)
10. 配置文件拆分 settings.toml + mirrors.toml
11. 主题系统 palette + ThemeConfig
12. 布局调整上下分
13. 当前镜像回退 effective_current_mirror_name

## 进度

全部完成，21 测试通过。

## 验证

- [x] `cargo test` 21 passed
- [x] 旧 config.toml 自动迁移
- [x] 全新安装生成 settings.toml + mirrors.toml
- [x] Pacman sudo 交互正常
- [x] reflector 冲突检测
- [x] 批量测速弹窗显示
- [ ] 用户需手动验证：light theme 显示效果
