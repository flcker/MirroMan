# 配置文件拆分：settings.toml + mirrors.toml

## 背景

当前所有配置集中在 `config.toml`（settings + 6 个包管理器镜像源）。随着包管理器增多和 theme 功能加入，单文件膨胀，关注点混杂。

## 目标

- 拆分为 `settings.toml`（设置 + 主题）和 `mirrors.toml`（镜像数据）
- 自动迁移旧版 `config.toml`
- `Config::mirrors_for()` 接口不变，其余模块无感

## 方案

```
~/.config/mirroman/
├── settings.toml     ← settings + theme
├── mirrors.toml      ← 6 个包管理器的镜像源
└── backups/
```

### 文件结构

**settings.toml**
```toml
[settings]
default_manager = "Cargo"
test_timeout_secs = 5

[theme]
focus_border = "yellow"
```

**mirrors.toml**
```toml
[cargo]
mirrors = [...]

[npm]
mirrors = [...]
...
```

### 迁移逻辑

1. 启动时先尝试读 `settings.toml` + `mirrors.toml`
2. 两文件都不存在 → 检查旧 `config.toml`：
   - 存在 → 读 config.toml，拆分写入两个新文件，删旧文件
   - 不存在 → 用默认值生成两个新文件
3. 只有一个存在 → 错误提示

## 影响范围

- `src/config.rs` — 主要改动
- `src/palette.rs` — Theme 序列化支持

## 步骤

1. 重构 Config 为 Settings + Mirrors 两层
2. 实现 settings.toml / mirrors.toml 读写
3. 实现旧 config.toml 迁移
4. 所有适配器 new() 适配新接口
5. 编译测试

## 进度

## 风险

- 旧用户数据迁移失败导致配置丢失 → 迁移前备份
- `mirrors_for()` 保持兼容

## 验证

- 全新安装生成 settings.toml + mirrors.toml
- 旧 config.toml 自动迁移
- `cargo test` 全部通过
