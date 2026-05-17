use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

use crate::adapters::PackageManagerAdapter;
use crate::config::Config;
use crate::mirror::Mirror;

/// TUI 运行模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    /// 正常导航模式
    Normal,
    /// 添加镜像（输入名称和 URL）
    AddMirror { input_name: String, input_url: String, focus_url: bool },
    /// 编辑镜像（预填当前镜像的名称和 URL）
    EditMirror { input_name: String, input_url: String, focus_url: bool },
    /// 确认删除
    ConfirmDelete,
    /// 显示错误信息
    Error(String),
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
}

impl App {
    pub fn new(
        config: Config,
        adapters: Vec<Box<dyn PackageManagerAdapter>>,
        current_mirror_names: Vec<Option<String>>,
    ) -> Self {
        Self {
            config,
            adapters,
            selected_manager_index: 0,
            selected_mirror_index: 0,
            focus: Focus::Managers,
            mode: AppMode::Normal,
            status_message: None,
            current_mirror_names,
            running: true,
        }
    }

    /// 当前选中的适配器
    pub fn current_adapter(&self) -> Option<&dyn PackageManagerAdapter> {
        self.adapters.get(self.selected_manager_index).map(|a| a.as_ref())
    }

    /// 当前选中适配器的镜像列表（直接从 Config 读取，保证数据实时）
    pub fn current_mirrors(&self) -> Vec<Mirror> {
        let name = self.current_adapter().map(|a| a.name()).unwrap_or("");
        match name {
            "Cargo" => self.config.cargo.mirrors.clone(),
            "npm" => self.config.npm.mirrors.clone(),
            "Homebrew" => self.config.homebrew.mirrors.clone(),
            _ => vec![],
        }
    }

    /// 获取指定适配器的镜像列表（可变引用）
    pub fn get_mirrors_for(&self, name: &str) -> Vec<Mirror> {
        match name {
            "Cargo" => self.config.cargo.mirrors.clone(),
            "npm" => self.config.npm.mirrors.clone(),
            "Homebrew" => self.config.homebrew.mirrors.clone(),
            _ => vec![],
        }
    }

    /// 主事件循环，每次 poll 一个事件并处理
    pub fn handle_event(&mut self) -> Result<()> {
        if !event::poll(Duration::from_millis(100))? {
            return Ok(());
        }

        let ev = event::read()?;
        // 只处理按下事件，忽略重复和释放
        if let Event::Key(key) = ev {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            // 先取出 mode 分发，避免 match &self.mode 和 handler 中 &mut self 的潜在借用冲突
            if matches!(self.mode, AppMode::Normal) {
                self.handle_normal_key(key.code)
            } else if matches!(self.mode, AppMode::AddMirror { .. }) {
                self.handle_form_key(key.code, false)
            } else if matches!(self.mode, AppMode::EditMirror { .. }) {
                self.handle_form_key(key.code, true)
            } else if matches!(self.mode, AppMode::ConfirmDelete) {
                self.handle_confirm_delete_key(key.code)
            } else if matches!(self.mode, AppMode::Error(_)) {
                self.handle_error_key(key.code)
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
            _ => {}
        }
    }

    /// 统一的弹窗表单按键处理（添加 + 编辑）
    fn handle_form_key(&mut self, code: KeyCode, is_edit: bool) {
        let (input_name, input_url, focus_url) = match &mut self.mode {
            AppMode::AddMirror { input_name, input_url, focus_url } => (input_name, input_url, focus_url),
            AppMode::EditMirror { input_name, input_url, focus_url } => (input_name, input_url, focus_url),
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
                    let adapter_name = self.current_adapter().map(|a| a.name()).unwrap_or("");
                    self.edit_current_mirror(adapter_name, &name, &url);
                } else {
                    self.add_mirror_to_current(&name, &url);
                }
                self.mode = AppMode::Normal;
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Tab
            | KeyCode::Left | KeyCode::Right => {
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

    fn handle_confirm_delete_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') => {
                self.delete_current_mirror();
                self.mode = AppMode::Normal;
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {
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
                if self.selected_manager_index > 0 {
                    self.selected_manager_index -= 1;
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
                if self.selected_manager_index + 1 < self.adapters.len() {
                    self.selected_manager_index += 1;
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
        if let Some(mirror) = mirrors.get(self.selected_mirror_index) {
            match self.current_adapter() {
                Some(adapter) => match adapter.switch_mirror(mirror) {
                    Ok(()) => {
                        if self.selected_manager_index < self.current_mirror_names.len() {
                            self.current_mirror_names[self.selected_manager_index] =
                                Some(mirror.name.clone());
                        }
                        self.status_message = Some(format!("已切换到镜像: {}", mirror.name));
                        self.config.save().ok();
                    }
                    Err(e) => {
                        self.mode = AppMode::Error(format!("切换失败: {e}"));
                    }
                },
                None => {
                    self.mode = AppMode::Error("没有可用的适配器".into());
                }
            }
        }
    }

    fn add_mirror_to_current(&mut self, name: &str, url: &str) {
        if name.is_empty() || url.is_empty() {
            self.mode = AppMode::Error("名称和 URL 不能为空".into());
            return;
        }
        let mirror = Mirror::new(name, url);
        let adapter_name = self.current_adapter().map(|a| a.name()).unwrap_or("");
        match adapter_name {
            "Cargo" => self.config.cargo.mirrors.push(mirror),
            "npm" => self.config.npm.mirrors.push(mirror),
            "Homebrew" => self.config.homebrew.mirrors.push(mirror),
            _ => {}
        }
        self.config.save().ok();
        self.status_message = Some(format!("已添加镜像: {}", name));
    }

    fn edit_current_mirror(&mut self, adapter_name: &str, new_name: &str, new_url: &str) {
        if new_name.is_empty() || new_url.is_empty() {
            self.mode = AppMode::Error("名称和 URL 不能为空".into());
            return;
        }
        let mirrors = match adapter_name {
            "Cargo" => &mut self.config.cargo.mirrors,
            "npm" => &mut self.config.npm.mirrors,
            "Homebrew" => &mut self.config.homebrew.mirrors,
            _ => return,
        };
        if let Some(mirror) = mirrors.get_mut(self.selected_mirror_index) {
            mirror.name = new_name.to_string();
            mirror.url = new_url.to_string();
        }
        self.config.save().ok();
        self.status_message = Some(format!("已更新镜像: {}", new_name));
    }

    fn delete_current_mirror(&mut self) {
        let adapter_name = self.current_adapter().map(|a| a.name()).unwrap_or("");
        let mirrors = self.get_mirrors_for(adapter_name);
        if let Some(mirror) = mirrors.get(self.selected_mirror_index) {
            let mirror_name = mirror.name.clone();
            match adapter_name {
                "Cargo" => { self.config.cargo.mirrors.retain(|m| m.name != mirror_name); }
                "npm" => { self.config.npm.mirrors.retain(|m| m.name != mirror_name); }
                "Homebrew" => { self.config.homebrew.mirrors.retain(|m| m.name != mirror_name); }
                _ => {}
            }
            if self.selected_mirror_index >= self.current_mirrors().len() && self.selected_mirror_index > 0 {
                self.selected_mirror_index -= 1;
            }
            self.config.save().ok();
            self.status_message = Some(format!("已删除镜像: {}", mirror_name));
        }
    }

    fn test_current_mirror(&mut self) {
        let mirrors = self.current_mirrors();
        let mirror = match mirrors.get(self.selected_mirror_index) {
            Some(m) => m.clone(),
            None => return,
        };
        let adapter_name = self.current_adapter().map(|a| a.name()).unwrap_or("");
        let result = self.current_adapter().and_then(|a| a.test_mirror(&mirror).ok());

        match result {
            Some(latency) => {
                self.status_message = Some(format!("{} 延迟: {}ms", mirror.name, latency));
                // 更新配置中的延迟值
                match adapter_name {
                    "Cargo" => {
                        if let Some(m) = self.config.cargo.mirrors.iter_mut().find(|m| m.name == mirror.name) {
                            m.latency_ms = latency;
                        }
                    }
                    "npm" => {
                        if let Some(m) = self.config.npm.mirrors.iter_mut().find(|m| m.name == mirror.name) {
                            m.latency_ms = latency;
                        }
                    }
                    "Homebrew" => {
                        if let Some(m) = self.config.homebrew.mirrors.iter_mut().find(|m| m.name == mirror.name) {
                            m.latency_ms = latency;
                        }
                    }
                    _ => {}
                }
                self.config.save().ok();
            }
            None => {
                self.mode = AppMode::Error(format!("测速失败: {} 不可达", mirror.name));
            }
        }
    }
}
