use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

use crate::adapters::{PackageManagerAdapter, RefreshAction, ValidationResult};
use crate::config::Config;
use crate::mirror::Mirror;

/// TUI 运行模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    /// 正常导航模式
    Normal,
    /// 添加镜像（输入名称和 URL）
    AddMirror {
        input_name: String,
        input_url: String,
        focus_url: bool,
    },
    /// 编辑镜像（预填当前镜像的名称和 URL）
    EditMirror {
        input_name: String,
        input_url: String,
        focus_url: bool,
    },
    /// 确认删除
    ConfirmDelete,
    /// 确认还原到备份
    ConfirmRestore,
    /// 确认重置到默认
    ConfirmReset,
    /// 切换后建议刷新操作
    PostSwitchRefresh {
        action: RefreshAction,
        mirror_name: String,
    },
    /// 批量测速结果
    BatchTestResults(Vec<BatchTestResultItem>),
    /// 显示错误信息
    Error(String),
}

/// 批量测速单项结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchTestResultItem {
    pub name: String,
    pub latency_ms: Option<u64>,
    pub reachable: bool,
}

/// 聚焦面板
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// 左面板：包管理器列表
    Managers,
    /// 右面板：镜像源列表
    Mirrors,
}

/// App 全局状态
pub struct App {
    pub config: Config,
    pub adapters: Vec<Box<dyn PackageManagerAdapter>>,
    pub selected_manager_index: usize,
    pub selected_mirror_index: usize,
    pub focus: Focus,
    pub mode: AppMode,
    /// 当前状态消息（切换成功/失败反馈）
    pub status_message: Option<String>,
    /// 各适配器当前激活的镜像名（None = 未知/未检测）
    pub current_mirror_names: Vec<Option<String>>,
    /// 是否正在运行
    pub running: bool,
    /// 需要全量重绘（sudo 交互后 terminal 内容被破坏）
    pub needs_full_redraw: bool,
}

impl App {
    pub fn new(
        config: Config,
        adapters: Vec<Box<dyn PackageManagerAdapter>>,
        current_mirror_names: Vec<Option<String>>,
    ) -> Self {
        let first_available = adapters
            .iter()
            .position(|a| a.is_available())
            .unwrap_or(0);
        Self {
            config,
            adapters,
            selected_manager_index: first_available,
            selected_mirror_index: 0,
            focus: Focus::Managers,
            mode: AppMode::Normal,
            status_message: None,
            current_mirror_names,
            running: true,
            needs_full_redraw: false,
        }
    }

    /// 当前选中的适配器
    pub fn current_adapter(&self) -> Option<&dyn PackageManagerAdapter> {
        self.adapters
            .get(self.selected_manager_index)
            .map(|a| a.as_ref())
    }

    /// 当前选中适配器的镜像列表（通过 Config 统一访问）
    pub fn current_mirrors(&self) -> Vec<Mirror> {
        let id = self.current_adapter().map(|a| a.id()).unwrap_or("");
        self.config.mirrors_for(id).cloned().unwrap_or_default()
    }

    /// 主事件循环，每次 poll 一个事件并处理
    pub fn handle_event(&mut self) -> Result<()> {
        if !event::poll(Duration::from_millis(100))? {
            return Ok(());
        }

        let ev = event::read()?;
        if let Event::Key(key) = ev {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            match &self.mode.clone() {
                AppMode::Normal => self.handle_normal_key(key.code),
                AppMode::AddMirror { .. } => self.handle_form_key(key.code, false),
                AppMode::EditMirror { .. } => self.handle_form_key(key.code, true),
                AppMode::ConfirmDelete => self.handle_confirm_key(key.code, "delete"),
                AppMode::ConfirmRestore => self.handle_confirm_key(key.code, "restore"),
                AppMode::ConfirmReset => self.handle_confirm_key(key.code, "reset"),
                AppMode::PostSwitchRefresh { action, mirror_name } => {
                    self.handle_refresh_key(key.code, action, mirror_name);
                }
                AppMode::BatchTestResults(_) => {
                    self.handle_batch_test_key(key.code);
                }
                AppMode::Error(_) => {
                    self.handle_error_key(key.code);
                }
            }
        }

        Ok(())
    }

    fn handle_normal_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.running = false;
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Managers => Focus::Mirrors,
                    Focus::Mirrors => Focus::Managers,
                };
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.navigate_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.navigate_down();
            }
            KeyCode::Enter => {
                if self.focus == Focus::Mirrors {
                    self.switch_current_mirror();
                }
            }
            KeyCode::Char('a') => {
                if self.focus == Focus::Mirrors {
                    self.mode = AppMode::AddMirror {
                        input_name: String::new(),
                        input_url: String::new(),
                        focus_url: false,
                    };
                }
            }
            KeyCode::Char('d') => {
                if self.focus == Focus::Mirrors
                    && self.selected_mirror_index < self.current_mirrors().len()
                {
                    self.mode = AppMode::ConfirmDelete;
                }
            }
            KeyCode::Char('e') => {
                if self.focus == Focus::Mirrors {
                    let mirrors = self.current_mirrors();
                    if let Some(mirror) = mirrors.get(self.selected_mirror_index) {
                        self.mode = AppMode::EditMirror {
                            input_name: mirror.name.clone(),
                            input_url: mirror.url.clone(),
                            focus_url: false,
                        };
                    }
                }
            }
            KeyCode::Char('t') => {
                if self.focus == Focus::Mirrors {
                    self.test_current_mirror();
                }
            }
            // ── v0.1.1 新增快捷键 ──
            KeyCode::Char('T') => {
                if self.focus == Focus::Mirrors {
                    self.test_all_mirrors();
                }
            }
            KeyCode::Char('v') => {
                if self.focus == Focus::Mirrors {
                    self.validate_current_mirror();
                }
            }
            KeyCode::Char('r') => {
                if self.focus == Focus::Mirrors {
                    self.mode = AppMode::ConfirmRestore;
                }
            }
            KeyCode::Char('R') => {
                if self.focus == Focus::Mirrors {
                    self.mode = AppMode::ConfirmReset;
                }
            }
            _ => {}
        }
    }

    /// 统一的弹窗表单按键处理（添加 + 编辑）
    fn handle_form_key(&mut self, code: KeyCode, is_edit: bool) {
        let (input_name, input_url, focus_url) = match &mut self.mode {
            AppMode::AddMirror {
                input_name,
                input_url,
                focus_url,
            } => (input_name, input_url, focus_url),
            AppMode::EditMirror {
                input_name,
                input_url,
                focus_url,
            } => (input_name, input_url, focus_url),
            _ => return,
        };

        match code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                let name = input_name.clone();
                let url = input_url.clone();
                if is_edit {
                    self.edit_current_mirror(&name, &url);
                } else {
                    self.add_mirror_to_current(&name, &url);
                }
                self.mode = AppMode::Normal;
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                *focus_url = !*focus_url;
            }
            KeyCode::Char(c) => {
                if *focus_url {
                    input_url.push(c);
                } else {
                    input_name.push(c);
                }
            }
            KeyCode::Backspace => {
                if *focus_url {
                    input_url.pop();
                } else {
                    input_name.pop();
                }
            }
            _ => {}
        }
    }

    /// 统一确认弹窗（删除/还原/重置）
    fn handle_confirm_key(&mut self, code: KeyCode, action: &str) {
        match code {
            KeyCode::Char('y') => match action {
                "delete" => {
                    self.delete_current_mirror();
                }
                "restore" => {
                    self.restore_current();
                }
                "reset" => {
                    self.reset_current();
                }
                _ => {}
            },
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {}
            _ => return,
        }
        self.mode = AppMode::Normal;
    }

    /// 刷新操作弹窗
    fn handle_refresh_key(
        &mut self,
        code: KeyCode,
        action: &RefreshAction,
        mirror_name: &str,
    ) {
        match code {
            KeyCode::Enter => {
                // 清除弹窗，写执行中提示到屏幕
                use crossterm::execute;
                execute!(
                    std::io::stdout(),
                    crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                    crossterm::cursor::MoveTo(0, 0),
                )
                .ok();
                println!("正在执行: {} {} ...", action.command, action.args.join(" "));

                // 执行刷新命令
                let args_str: Vec<&str> = action.args.iter().map(|s| s.as_str()).collect();
                let result = if action.requires_sudo {
                    // 使用 run_sudo 自动处理密码交互
                    let mut all_args = vec![action.command.as_str()];
                    all_args.extend(&args_str);
                    crate::utils::command::run_sudo(&all_args)
                        .map(|_| String::new())
                } else {
                    crate::utils::command::run_command(&action.command, &args_str)
                };
                match result {
                    Ok(_) => {
                        self.needs_full_redraw = true;
                        self.status_message = Some(format!(
                            "已切换到: {}，刷新完成",
                            mirror_name
                        ));
                    }
                    Err(e) => {
                        self.mode = AppMode::Error(format!(
                            "刷新失败: {e}\n请手动执行: {} {}",
                            action.command,
                            action.args.join(" ")
                        ));
                        return;
                    }
                }
                self.mode = AppMode::Normal;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.needs_full_redraw = true;
                self.status_message =
                    Some(format!("已切换到镜像: {}", mirror_name));
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    /// 批量测速结果弹窗
    fn handle_batch_test_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_error_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Enter => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    // ── 导航 ──────────────────────────────────────────────

    fn navigate_up(&mut self) {
        match self.focus {
            Focus::Managers => {
                let mut idx = self.selected_manager_index;
                while idx > 0 {
                    idx -= 1;
                    if self.adapters[idx].is_available() {
                        self.selected_manager_index = idx;
                        break;
                    }
                }
                self.selected_mirror_index = 0;
            }
            Focus::Mirrors => {
                if self.selected_mirror_index > 0 {
                    self.selected_mirror_index -= 1;
                }
            }
        }
    }

    fn navigate_down(&mut self) {
        match self.focus {
            Focus::Managers => {
                let mut idx = self.selected_manager_index;
                while idx + 1 < self.adapters.len() {
                    idx += 1;
                    if self.adapters[idx].is_available() {
                        self.selected_manager_index = idx;
                        break;
                    }
                }
                self.selected_mirror_index = 0;
            }
            Focus::Mirrors => {
                let len = self.current_mirrors().len();
                if len > 0 && self.selected_mirror_index + 1 < len {
                    self.selected_mirror_index += 1;
                }
            }
        }
    }

    // ── 操作 ──────────────────────────────────────────────

    fn switch_current_mirror(&mut self) {
        let mirrors = self.current_mirrors();
        let mirror = match mirrors.get(self.selected_mirror_index) {
            Some(m) => m.clone(),
            None => return,
        };

        // 先执行切换，再处理副作用（避免借用冲突）
        let switch_result = self
            .current_adapter()
            .map(|a| (a.switch_mirror(&mirror), a.refresh_action()));
        let (result, refresh_action) = match switch_result {
            Some((Ok(()), action)) => (Ok(()), action),
            Some((Err(e), _)) => (Err(e), None),
            None => {
                self.mode = AppMode::Error("没有可用的适配器".into());
                return;
            }
        };

        match result {
            Ok(()) => {
                self.needs_full_redraw = true;
                if self.selected_manager_index < self.current_mirror_names.len() {
                    self.current_mirror_names[self.selected_manager_index] =
                        Some(mirror.name.clone());
                }
                self.config.save().ok();

                if let Some(action) = refresh_action {
                    self.mode = AppMode::PostSwitchRefresh {
                        action,
                        mirror_name: mirror.name.clone(),
                    };
                } else {
                    self.status_message = Some(format!("已切换到镜像: {}", mirror.name));
                }
            }
            Err(e) => {
                self.mode = AppMode::Error(format!("切换失败: {e}"));
            }
        }
    }

    fn add_mirror_to_current(&mut self, name: &str, url: &str) {
        if name.is_empty() || url.is_empty() {
            self.mode = AppMode::Error("名称和 URL 不能为空".into());
            return;
        }
        let mirror = Mirror::new(name, url);
        let adapter_id = self.current_adapter().map(|a| a.id()).unwrap_or("");

        if let Some(mirrors) = self.config.mirrors_for_mut(adapter_id) {
            mirrors.push(mirror);
            self.config.save().ok();
            self.status_message = Some(format!("已添加镜像: {}", name));
        }
    }

    fn edit_current_mirror(&mut self, new_name: &str, new_url: &str) {
        if new_name.is_empty() || new_url.is_empty() {
            self.mode = AppMode::Error("名称和 URL 不能为空".into());
            return;
        }
        let adapter_id = self.current_adapter().map(|a| a.id()).unwrap_or("");

        if let Some(mirrors) = self.config.mirrors_for_mut(adapter_id) {
            if let Some(mirror) = mirrors.get_mut(self.selected_mirror_index) {
                mirror.name = new_name.to_string();
                mirror.url = new_url.to_string();
            }
            self.config.save().ok();
            self.status_message = Some(format!("已更新镜像: {}", new_name));
        }
    }

    fn delete_current_mirror(&mut self) {
        let adapter_id = self.current_adapter().map(|a| a.id()).unwrap_or("");
        let current_mirrors = self.current_mirrors();
        let mirror_name = match current_mirrors.get(self.selected_mirror_index) {
            Some(m) => m.name.clone(),
            None => return,
        };

        if let Some(mirrors) = self.config.mirrors_for_mut(adapter_id) {
            mirrors.retain(|m| m.name != mirror_name);
        }

        if self.selected_mirror_index >= self.current_mirrors().len()
            && self.selected_mirror_index > 0
        {
            self.selected_mirror_index -= 1;
        }
        self.config.save().ok();
        self.status_message = Some(format!("已删除镜像: {}", mirror_name));
    }

    fn test_current_mirror(&mut self) {
        let mirrors = self.current_mirrors();
        let mirror = match mirrors.get(self.selected_mirror_index) {
            Some(m) => m.clone(),
            None => return,
        };
        let adapter_id = self.current_adapter().map(|a| a.id()).unwrap_or("");
        let result = self.current_adapter().and_then(|a| a.test_mirror(&mirror).ok());

        match result {
            Some(latency) => {
                self.status_message = Some(format!("{} 延迟: {}ms", mirror.name, latency));
                // 更新配置中的延迟值
                if let Some(mirrors) = self.config.mirrors_for_mut(adapter_id) {
                    if let Some(m) = mirrors.iter_mut().find(|m| m.name == mirror.name) {
                        m.latency_ms = latency;
                    }
                }
                self.config.save().ok();
            }
            None => {
                self.mode = AppMode::Error(format!("测速失败: {} 不可达", mirror.name));
            }
        }
    }

    // ── v0.1.1 新增操作 ──────────────────────────────────

    /// 批量测速所有镜像源
    fn test_all_mirrors(&mut self) {
        let mirrors = self.current_mirrors();
        if mirrors.is_empty() {
            self.mode = AppMode::Error("没有可用镜像源".into());
            return;
        }

        // 克隆必要数据以避免借用冲突
        let mirror_list: Vec<Mirror> = mirrors.clone();
        let adapter_id = self
            .current_adapter()
            .map(|a| a.id().to_string())
            .unwrap_or_default();

        let mut results: Vec<BatchTestResultItem> = Vec::new();
        for mirror in &mirror_list {
            let test_result = self
                .current_adapter()
                .and_then(|a| a.test_mirror(mirror).ok());
            match test_result {
                Some(latency) => {
                    results.push(BatchTestResultItem {
                        name: mirror.name.clone(),
                        latency_ms: Some(latency),
                        reachable: true,
                    });
                    if let Some(config_mirrors) = self.config.mirrors_for_mut(&adapter_id) {
                        if let Some(m) =
                            config_mirrors.iter_mut().find(|m| m.name == mirror.name)
                        {
                            m.latency_ms = latency;
                        }
                    }
                }
                None => {
                    results.push(BatchTestResultItem {
                        name: mirror.name.clone(),
                        latency_ms: None,
                        reachable: false,
                    });
                }
            }
        }

        results.sort_by_key(|r| r.latency_ms.unwrap_or(u64::MAX));

        self.config.save().ok();
        self.mode = AppMode::BatchTestResults(results);
    }

    /// 验证当前选中镜像源
    fn validate_current_mirror(&mut self) {
        let mirrors = self.current_mirrors();
        let mirror = match mirrors.get(self.selected_mirror_index) {
            Some(m) => m.clone(),
            None => return,
        };

        match self.current_adapter() {
            Some(adapter) => match adapter.validate_mirror(&mirror) {
                Ok(ValidationResult {
                    reachable,
                    latency_ms: _,
                    content_valid,
                    detail,
                }) => {
                    let status = if reachable && content_valid {
                        "✓ 有效"
                    } else if reachable {
                        "⚠ 可达但内容异常"
                    } else {
                        "✗ 不可达"
                    };
                    self.status_message = Some(format!("{} {}: {}", mirror.name, status, detail));
                }
                Err(e) => {
                    self.mode = AppMode::Error(format!("验证失败: {e}"));
                }
            },
            None => {
                self.mode = AppMode::Error("没有可用的适配器".into());
            }
        }
    }

    /// 还原到备份
    fn restore_current(&mut self) {
        let result = self
            .current_adapter()
            .map(|a| (a.restore(), a.name(), a.current_mirror_name()));

        match result {
            Some((Ok(()), name, current_name)) => {
                self.needs_full_redraw = true;
                self.status_message = Some(format!("{} 已还原到备份状态", name));
                if self.selected_manager_index < self.current_mirror_names.len() {
                    self.current_mirror_names[self.selected_manager_index] = current_name;
                }
            }
            Some((Err(e), _, _)) => {
                self.mode = AppMode::Error(format!("还原失败: {e}"));
            }
            None => {
                self.mode = AppMode::Error("没有可用的适配器".into());
            }
        }
    }

    /// 重置到系统默认
    fn reset_current(&mut self) {
        let result = self
            .current_adapter()
            .map(|a| (a.reset_to_default(), a.name()));

        match result {
            Some((Ok(()), name)) => {
                self.needs_full_redraw = true;
                self.status_message = Some(format!("{} 已重置到系统默认", name));
                if self.selected_manager_index < self.current_mirror_names.len() {
                    self.current_mirror_names[self.selected_manager_index] = None;
                }
            }
            Some((Err(e), _)) => {
                self.mode = AppMode::Error(format!("重置失败: {e}"));
            }
            None => {
                self.mode = AppMode::Error("没有可用的适配器".into());
            }
        }
    }
}
