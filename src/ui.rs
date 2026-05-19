use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::{App, AppMode, BatchTestResultItem, Focus};
use crate::palette::Palette;

/// 渲染整个 TUI 界面
pub fn draw(frame: &mut Frame, app: &App, p: &Palette) {
    let area = frame.area();

    // 布局：上下分 —— 上部（管理器+镜像源） | 底部（快捷键+信息）
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let top_area = main_chunks[0];
    let help_area = main_chunks[1];

    // 上部分左右分
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(top_area);

    let left_area = top_chunks[0];
    let table_area = top_chunks[1];

    render_managers(frame, app, left_area, p);
    render_mirrors(frame, app, table_area, p);
    render_help(frame, app, help_area, p);

    // 弹窗层（覆盖在最上层）
    match &app.mode {
        AppMode::AddMirror { input_name, input_url, focus_url } => {
            render_mirror_form_popup(frame, "添加镜像", input_name, input_url, *focus_url, p);
        }
        AppMode::EditMirror { input_name, input_url, focus_url } => {
            render_mirror_form_popup(frame, "编辑镜像", input_name, input_url, *focus_url, p);
        }
        AppMode::ConfirmDelete => {
            render_confirm_popup(frame, app, "删除", p, |a| {
                a.current_mirrors()
                    .get(a.selected_mirror_index)
                    .map(|m| m.name.as_str())
                    .unwrap_or("未知").to_string()
            });
        }
        AppMode::ConfirmRestore => {
            let adapter_name = app.current_adapter().map(|a| a.name()).unwrap_or("未知");
            render_confirm_popup(frame, app, "还原", p, |_| {
                format!("{} 的配置", adapter_name)
            });
        }
        AppMode::ConfirmReset => {
            let adapter_name = app.current_adapter().map(|a| a.name()).unwrap_or("未知");
            render_confirm_popup(frame, app, "重置", p, |_| {
                format!("{} 到系统默认", adapter_name)
            });
        }
        AppMode::PostSwitchRefresh { action, mirror_name } => {
            render_refresh_popup(frame, action, mirror_name, p);
        }
        AppMode::BatchTestResults(results) => {
            render_batch_test_popup(frame, results, p);
        }
        AppMode::Error(msg) => {
            render_error_popup(frame, msg, p);
        }
        AppMode::Normal => {}
    }
}

// ── 左侧：包管理器列表 ──────────────────────────────────

fn render_managers(frame: &mut Frame, app: &App, area: Rect, p: &Palette) {
    let items: Vec<ListItem> = app
        .adapters
        .iter()
        .map(|a| {
            let status = if a.is_available() { "✓" } else { "✗" };
            let name_style = if a.is_available() {
                Style::default()
            } else {
                Style::default().fg(p.dim_text)
            };
            let line = Line::from(vec![
                Span::styled(format!(" {} {}", status, a.name()), name_style),
                Span::styled(
                    if a.is_available() {
                        format!("  [{}]", a.supported_platforms())
                    } else {
                        "  [不可用]".to_string()
                    },
                    Style::default().fg(p.dim_text),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let is_focused = app.focus == Focus::Managers;
    let block = Block::default()
        .title("包管理器")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Style::default().fg(p.focus_border)
        } else {
            Style::default()
        });

    let list = List::new(items).block(block).highlight_style(
        Style::default().fg(p.highlight_fg).bg(if is_focused {
            p.focus_highlight_bg
        } else {
            p.dim_highlight_bg
        }),
    );

    frame.render_stateful_widget(
        list,
        area,
        &mut ratatui::widgets::ListState::default()
            .with_selected(Some(app.selected_manager_index)),
    );
}

// ── 右侧：镜像源表格 ────────────────────────────────────

fn render_mirrors(frame: &mut Frame, app: &App, area: Rect, p: &Palette) {
    let mirrors = app.current_mirrors();
    let is_focused = app.focus == Focus::Mirrors;

    let header = Row::new(vec!["名称", "URL", "状态", "延迟"])
        .style(Style::default().fg(p.header_fg).add_modifier(Modifier::BOLD));

    let current_name = app.effective_current_mirror_name();

    let rows: Vec<Row> = mirrors
        .iter()
        .map(|m| {
            let display_name = if Some(m.name.as_str()) == current_name.as_deref() {
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
            Row::new(vec![display_name, m.url.clone(), status.to_string(), latency])
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
                    Style::default().fg(p.mirror_focus_border())
                } else {
                    Style::default()
                }),
        )
        .row_highlight_style(
            Style::default().fg(p.highlight_fg).bg(if is_focused {
                p.mirror_focus_highlight_bg()
            } else {
                p.dim_highlight_bg
            }),
        );

    frame.render_stateful_widget(
        table,
        area,
        &mut ratatui::widgets::TableState::default()
            .with_selected(Some(app.selected_mirror_index)),
    );
}

// ── 底部帮助栏 ───────────────────────────────────────────

fn render_help(frame: &mut Frame, app: &App, area: Rect, p: &Palette) {
    let shortcuts: &[(&str, &str)] = if app.focus == Focus::Mirrors {
        &[
            ("q/Esc", "退出"),
            ("↑↓/jk", "导航"),
            ("Tab", "→管理器"),
            ("Enter", "切换"),
            ("t", "测速"),
            ("T", "批量测速"),
            ("v", "验证"),
            ("a", "添加"),
            ("d", "删除"),
            ("e", "编辑"),
            ("r", "还原"),
            ("R", "重置"),
        ]
    } else {
        &[
            ("q/Esc", "退出"),
            ("↑↓/jk", "导航"),
            ("Tab", "→镜像源"),
            ("t", "测速"),
            ("T", "批量测速"),
            ("v", "验证"),
            ("a", "添加"),
            ("d", "删除"),
            ("e", "编辑"),
            ("r", "还原"),
            ("R", "重置"),
        ]
    };

    let mut spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(*key, Style::default().fg(p.key_fg).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {}  ", desc), Style::default().fg(p.key_desc)),
            ]
        })
        .collect();

    spans.push(Span::styled("  *当前镜像", Style::default().fg(p.dim_text)));

    let status = app.status_message.as_deref().unwrap_or("");

    let mut text = vec![Line::from(spans)];
    if !status.is_empty() {
        text.push(Line::from(Span::styled(status, Style::default().fg(p.success))));
    }

    // 冲突警告
    if let Some(adapter) = app.current_adapter() {
        for warning in adapter.conflict_warnings() {
            text.push(Line::from(Span::styled(
                format!("⚠ {}", warning),
                Style::default().fg(p.warning).add_modifier(Modifier::BOLD),
            )));
        }
    }

    let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::NONE));

    frame.render_widget(paragraph, area);
}

// ── 弹窗：添加/编辑镜像 ──────────────────────────────────

fn render_mirror_form_popup(
    frame: &mut Frame,
    title: &str,
    name: &str,
    url: &str,
    focus_url: bool,
    p: &Palette,
) {
    let r = frame.area();

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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    let form_area = chunks[0];
    let help_area = chunks[1];

    let content = vec![
        Line::from(""),
        Line::from(Span::styled("名称:", Style::default().fg(p.key_fg))),
        {
            let display_name = if focus_url { name.to_string() } else { format!("{}▎", name) };
            Line::from(Span::styled(display_name, Style::default()))
        },
        Line::from(""),
        Line::from(Span::styled("URL:", Style::default().fg(p.key_fg))),
        {
            let display_url = if focus_url { format!("{}▎", url) } else { url.to_string() };
            Line::from(Span::styled(display_url, Style::default()))
        },
    ];

    let form_block = Block::default().title(title).borders(Borders::ALL).style(Style::default());
    let paragraph = Paragraph::new(content).block(form_block);
    frame.render_widget(paragraph, form_area);

    let shortcuts = vec![("Enter", "保存"), ("↑↓←→/Tab", "切换行"), ("Esc", "取消")];
    let mut spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(*key, Style::default().fg(p.key_fg).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {}  ", desc), Style::default().fg(p.key_desc)),
            ]
        })
        .collect();
    spans.push(Span::styled("  *当前镜像", Style::default().fg(p.dim_text)));

    let help_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(p.dim_text));
    let help_paragraph = Paragraph::new(Line::from(spans)).block(help_block);
    frame.render_widget(help_paragraph, help_area);
}

// ── 弹窗：通用确认（删除/还原/重置） ──────────────────────

fn render_confirm_popup<F>(frame: &mut Frame, app: &App, action: &str, p: &Palette, name_fn: F)
where
    F: FnOnce(&App) -> String,
{
    let area = centered_rect(50, 20, frame.area());
    let name = name_fn(app);

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(format!("确认{} \"{}\" ?", action, name), Style::default())),
        Line::from(""),
        Line::from(Span::styled("y 确认 | n/Esc 取消", Style::default().fg(p.dim_text))),
    ];

    let block = Block::default()
        .title(format!("{}确认", action))
        .borders(Borders::ALL)
        .style(Style::default());

    let paragraph = Paragraph::new(content).block(block).wrap(Wrap { trim: false });
    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(paragraph, area);
}

// ── 弹窗：切换后刷新操作 ──────────────────────────────────

fn render_refresh_popup(
    frame: &mut Frame,
    action: &crate::adapters::RefreshAction,
    mirror_name: &str,
    p: &Palette,
) {
    let area = centered_rect(60, 30, frame.area());

    let sudo_note = if action.requires_sudo {
        "\n(需要 sudo 权限)"
    } else {
        ""
    };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(format!("已切换到镜像: {}", mirror_name), Style::default().fg(p.success))),
        Line::from(""),
        Line::from(Span::styled(
            format!("⚡ 建议{}:", action.description),
            Style::default().fg(p.warning),
        )),
        Line::from(Span::styled(
            format!("   {} {}{}", action.command, action.args.join(" "), sudo_note),
            Style::default(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Enter 执行（可能需要数秒）| Esc 跳过",
            Style::default().fg(p.dim_text),
        )),
    ];

    let block = Block::default()
        .title("刷新建议")
        .borders(Borders::ALL)
        .style(Style::default());

    let paragraph = Paragraph::new(content).block(block).wrap(Wrap { trim: false });
    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(paragraph, area);
}

// ── 弹窗：批量测速结果 ──────────────────────────────────

fn render_batch_test_popup(frame: &mut Frame, results: &[BatchTestResultItem], p: &Palette) {
    let area = centered_rect(70, 50, frame.area());

    let header = Row::new(vec!["镜像名称", "延迟", "状态"])
        .style(Style::default().fg(p.header_fg).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = results
        .iter()
        .map(|r| {
            let latency = r.latency_ms.map(|l| format!("{}ms", l)).unwrap_or_else(|| "-".to_string());
            let status = if r.reachable { "✓" } else { "✗" };
            let status_style = if r.reachable {
                Style::default().fg(p.success)
            } else {
                Style::default().fg(p.error)
            };
            Row::new(vec![r.name.clone(), latency, status.to_string()]).style(status_style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title("批量测速结果（按延迟排序）").borders(Borders::ALL));

    let help = Line::from(Span::styled("Esc / Enter / q 关闭", Style::default().fg(p.dim_text)));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(table, chunks[0]);
    frame.render_widget(Paragraph::new(help), chunks[1]);
}

// ── 弹窗：错误提示 ──────────────────────────────────────

fn render_error_popup(frame: &mut Frame, msg: &str, p: &Palette) {
    let area = centered_rect(50, 30, frame.area());

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(msg, Style::default().fg(p.error))),
        Line::from(""),
        Line::from(Span::styled("Esc / Enter 关闭", Style::default().fg(p.dim_text))),
    ];

    let block = Block::default()
        .title("错误")
        .borders(Borders::ALL)
        .style(Style::default().fg(p.error));

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
