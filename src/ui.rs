use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::{App, AppMode, Focus};

/// 渲染整个 TUI 界面
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // 主布局：左侧面板 + 右侧面板
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let left_area = main_chunks[0];
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(main_chunks[1]);

    let table_area = right_chunks[0];
    let help_area = right_chunks[1];

    render_managers(frame, app, left_area);
    render_mirrors(frame, app, table_area);
    render_help(frame, app, help_area);

    // 弹窗层（覆盖在最上层）
    match &app.mode {
        AppMode::AddMirror { input_name, input_url, focus_url } => {
            render_mirror_form_popup(frame, "添加镜像", input_name, input_url, *focus_url);
        }
        AppMode::EditMirror { input_name, input_url, focus_url } => {
            render_mirror_form_popup(frame, "编辑镜像", input_name, input_url, *focus_url);
        }
        AppMode::ConfirmDelete => {
            render_confirm_delete_popup(frame, app);
        }
        AppMode::Error(msg) => {
            render_error_popup(frame, msg);
        }
        AppMode::Normal => {}
    }
}

// ── 左侧：包管理器列表 ──────────────────────────────────

fn render_managers(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .adapters
        .iter()
        .map(|a| {
            let label = if a.is_available() {
                format!(" ✓ {}", a.name())
            } else {
                format!(" ✗ {} (不可用)", a.name())
            };
            ListItem::new(label)
        })
        .collect();

    let is_focused = app.focus == Focus::Managers;
    let block = Block::default()
        .title("包管理器")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(if is_focused { Color::Yellow } else { Color::DarkGray }),
        );

    frame.render_stateful_widget(list, area, &mut ratatui::widgets::ListState::default().with_selected(Some(app.selected_manager_index)));
}

// ── 右侧：镜像源表格 ────────────────────────────────────

fn render_mirrors(frame: &mut Frame, app: &App, area: Rect) {
    let mirrors = app.current_mirrors();
    let is_focused = app.focus == Focus::Mirrors;

    let header = Row::new(vec!["名称", "URL", "状态", "延迟"])
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    let current_name = app.current_mirror_names
        .get(app.selected_manager_index)
        .and_then(|n| n.as_deref());

    let rows: Vec<Row> = mirrors
        .iter()
        .map(|m| {
            let display_name = if Some(m.name.as_str()) == current_name {
                format!("* {}", m.name)
            } else {
                format!("  {}", m.name)
            };
            let status = if m.enabled { "启用" } else { "禁用" };
            let latency = if m.latency_ms > 0 {
                format!("{}ms", m.latency_ms)
            } else {
                "-".to_string()
            };
            Row::new(vec![
                display_name,
                m.url.clone(),
                status.to_string(),
                latency,
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(45),
        Constraint::Percentage(10),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title("镜像源")
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                }),
        )
        .row_highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(if is_focused { Color::Cyan } else { Color::DarkGray }),
        );

    frame.render_stateful_widget(
        table,
        area,
        &mut ratatui::widgets::TableState::default().with_selected(Some(app.selected_mirror_index)),
    );
}

// ── 底部帮助栏 ───────────────────────────────────────────

fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let shortcuts: &[(&str, &str)] = if app.focus == Focus::Mirrors {
        &[
            ("q/Esc", "退出"),
            ("↑↓/jk", "导航"),
            ("Tab", "→管理器"),
            ("Enter", "切换"),
            ("t", "测速"),
            ("a", "添加"),
            ("d", "删除"),
            ("e", "编辑"),
        ]
    } else {
        &[
            ("q/Esc", "退出"),
            ("↑↓/jk", "导航"),
            ("Tab", "→镜像源"),
            ("Enter", "切换"),
        ]
    };

    let mut spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(*key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {}  ", desc), Style::default().fg(Color::Gray)),
            ]
        })
        .collect();

    // 追加当前镜像标识说明
    spans.push(Span::styled(
        "  *当前镜像",
        Style::default().fg(Color::DarkGray),
    ));

    let status = app
        .status_message
        .as_deref()
        .unwrap_or("");

    let mut text = vec![Line::from(spans)];
    if !status.is_empty() {
        text.push(Line::from(Span::styled(status, Style::default().fg(Color::Green))));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(paragraph, area);
}

// ── 弹窗：添加镜像 ──────────────────────────────────────

fn render_mirror_form_popup(frame: &mut Frame, title: &str, name: &str, url: &str, focus_url: bool) {
    let r = frame.area();

    // 弹窗：60% 宽，最少 10 行高度
    let width_pct = 60u16;
    let popup_height = ((r.height as u16 * 35 / 100).max(10)).min(r.height as u16);

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(r);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(vert[1]);

    let area = horiz[1];
    frame.render_widget(ratatui::widgets::Clear, area);

    // 弹窗内部上下分栏
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    let form_area = chunks[0];
    let help_area = chunks[1];

    // ── 输入区 ──
    let content = vec![
        Line::from(""),
        Line::from(Span::styled("名称:", Style::default().fg(Color::Yellow))),
        {
            let display_name = if focus_url { name.to_string() } else { format!("{}▎", name) };
            Line::from(Span::styled(display_name, Style::default()))
        },
        Line::from(""),
        Line::from(Span::styled("URL:", Style::default().fg(Color::Yellow))),
        {
            let display_url = if focus_url { format!("{}▎", url) } else { url.to_string() };
            Line::from(Span::styled(display_url, Style::default()))
        },
    ];

    let form_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default());

    let paragraph = Paragraph::new(content).block(form_block);
    frame.render_widget(paragraph, form_area);

    // ── 快捷键栏 ──
    let shortcuts = vec![
        ("Enter", "保存"),
        ("↑↓←→/Tab", "切换行"),
        ("Esc", "取消"),
    ];
    let mut spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(*key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {}  ", desc), Style::default().fg(Color::Gray)),
            ]
        })
        .collect();

    // 追加当前镜像标识说明
    spans.push(Span::styled(
        "  *当前镜像",
        Style::default().fg(Color::DarkGray),
    ));

    let help_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let help_paragraph = Paragraph::new(Line::from(spans)).block(help_block);
    frame.render_widget(help_paragraph, help_area);
}

// ── 弹窗：确认删除 ──────────────────────────────────────

fn render_confirm_delete_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 20, frame.area());
    let mirrors = app.current_mirrors();
    let mirror_name = mirrors
        .get(app.selected_mirror_index)
        .map(|m| m.name.as_str())
        .unwrap_or("未知");

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("确认删除镜像 \"{}\" ?", mirror_name),
            Style::default(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "y 确认 | n/Esc 取消",
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Block::default()
        .title("删除确认")
        .borders(Borders::ALL)
        .style(Style::default());

    let paragraph = Paragraph::new(content).block(block).wrap(Wrap { trim: false });
    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(paragraph, area);
}

// ── 弹窗：错误提示 ──────────────────────────────────────

fn render_error_popup(frame: &mut Frame, msg: &str) {
    let area = centered_rect(50, 30, frame.area());

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(msg, Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from(Span::styled(
            "Esc / Enter 关闭",
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Block::default()
        .title("错误")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(content).block(block).wrap(Wrap { trim: false });
    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(paragraph, area);
}

// ── 工具函数 ────────────────────────────────────────────

/// 计算居中的矩形区域
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
