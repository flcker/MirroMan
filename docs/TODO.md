# 后续需求与优化

## 高优先级

- [ ] **setting 热加载**：监控 `settings.toml` 变更，自动重建 Palette + 重绘（`inotify`/`notify` crate）
- [ ] **apt 适配器**：Debian/Ubuntu sources.list 管理，需检测发行版代号
- [ ] **dnf 适配器**：Fedora/RHEL yum.repos.d 管理
- [ ] **操作历史栈**：undo/redo 链，记录每次切换操作，支持多步回退
- [ ] **完整卸载**：一键清理所有包管理器的 MirroMan 修改痕迹

## 中优先级

- [ ] **帮助系统**：`?` 呼出全屏帮助弹窗，各操作详细说明 + 包管理器特定说明
- [ ] **man page / --help**：命令行帮助文档，`mirroman --help` 输出使用说明

- [ ] **Maven 适配器**：`~/.m2/settings.xml` mirror 配置
- [ ] **RubyGems 适配器**：`gem sources` 管理
- [ ] **Composer 适配器**：PHP `composer config repos.packagist`
- [ ] **NuGet 适配器**：.NET `dotnet nuget` / `nuget.config`
- [ ] **Docker 适配器**：`/etc/docker/daemon.json` registry-mirrors（需 sudo）
- [ ] **Conda 适配器**：`.condarc` channels 管理
- [ ] **快捷键自定义**：`settings.toml` 中配置键位映射
- [ ] **镜像源导入/导出**：分享 `mirrors.toml` 或 JSON 格式
- [ ] **多语言源自动化配置**：根据地理位置推荐最佳镜像

## 低优先级

- [ ] **winget 适配器**：评估可行性（镜像生态薄弱）
- [ ] **异步测速**：批量测速改用 tokio 异步，不阻塞 UI 更新
- [ ] **测速进度条**：批量测速时逐项更新弹窗内容
- [ ] **包管理器状态缓存**：启动时并行检测可用性，减少启动时间
- [ ] **终端 resize 自适应**：窗口大小变化时自动重算布局
- [ ] **CLI 模式**：不启动 TUI，直接 `mirroman set cargo --mirror tuna`
