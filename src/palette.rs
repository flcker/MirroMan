use ratatui::style::Color;

use crate::config::ThemeConfig;

/// 调色板：集中管理所有 UI 颜色，方便主题切换
#[derive(Debug, Clone)]
pub struct Palette {
    // ── 面板边框 ──
    /// 获得焦点的面板边框色
    pub focus_border: Color,
    /// 未获得焦点的面板边框色
    pub border: Color,

    // ── 高亮 ──
    /// 焦点面板的选中行背景色
    pub focus_highlight_bg: Color,
    /// 非焦点面板的选中行背景色
    pub dim_highlight_bg: Color,
    /// 高亮行的文字色
    pub highlight_fg: Color,

    // ── 文字 ──
    /// 表头文字色
    pub header_fg: Color,
    /// 普通文字色
    pub text: Color,
    /// 暗色文字（注释、占位）
    pub dim_text: Color,
    /// 快捷键键名色
    pub key_fg: Color,
    /// 快捷键描述色
    pub key_desc: Color,

    // ── 状态 ──
    /// 成功 / 状态消息
    pub success: Color,
    /// 错误消息
    pub error: Color,
    /// 警告消息
    pub warning: Color,
    /// 可用标识
    pub available: Color,
    /// 不可用标识
    pub unavailable: Color,

    // ── 镜像面板专色 ──
    pub mirror_focus_border_color: Color,
    pub mirror_focus_highlight_bg_color: Color,
}

impl Palette {
    /// 内置 dark 主题默认值（用于 fallback）
    fn dark_defaults() -> Self {
        Self {
            focus_border: Color::Yellow,
            border: Color::Reset,
            focus_highlight_bg: Color::Yellow,
            dim_highlight_bg: Color::DarkGray,
            highlight_fg: Color::Black,
            header_fg: Color::White,
            text: Color::Reset,
            dim_text: Color::DarkGray,
            key_fg: Color::Yellow,
            key_desc: Color::Gray,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            available: Color::Reset,
            unavailable: Color::DarkGray,
            mirror_focus_border_color: Color::Cyan,
            mirror_focus_highlight_bg_color: Color::Cyan,
        }
    }

    /// 从 ThemeConfig 构建 Palette，未设置的字段回退到 dark 默认值
    pub fn from_theme(theme: &ThemeConfig) -> Self {
        let defaults = Self::dark_defaults();
        Self {
            focus_border: parse_color(theme.focus_border.as_deref())
                .unwrap_or(defaults.focus_border),
            border: defaults.border,
            focus_highlight_bg: parse_color(theme.focus_highlight_bg.as_deref())
                .unwrap_or(defaults.focus_highlight_bg),
            dim_highlight_bg: parse_color(theme.dim_highlight_bg.as_deref())
                .unwrap_or(defaults.dim_highlight_bg),
            highlight_fg: parse_color(theme.highlight_fg.as_deref())
                .unwrap_or(defaults.highlight_fg),
            header_fg: parse_color(theme.header_fg.as_deref())
                .unwrap_or(defaults.header_fg),
            text: defaults.text,
            dim_text: parse_color(theme.dim_text.as_deref())
                .unwrap_or(defaults.dim_text),
            key_fg: parse_color(theme.key_fg.as_deref())
                .unwrap_or(defaults.key_fg),
            key_desc: parse_color(theme.key_desc.as_deref())
                .unwrap_or(defaults.key_desc),
            success: parse_color(theme.success.as_deref())
                .unwrap_or(defaults.success),
            error: parse_color(theme.error.as_deref())
                .unwrap_or(defaults.error),
            warning: parse_color(theme.warning.as_deref())
                .unwrap_or(defaults.warning),
            available: defaults.available,
            unavailable: defaults.unavailable,
            mirror_focus_border_color: parse_color(theme.mirror_focus_border.as_deref())
                .unwrap_or(defaults.mirror_focus_border_color),
            mirror_focus_highlight_bg_color: parse_color(
                theme.mirror_focus_highlight_bg.as_deref(),
            )
            .unwrap_or(defaults.mirror_focus_highlight_bg_color),
        }
    }

    /// 镜像面板专色方法（保持 ui.rs 兼容）
    pub fn mirror_focus_border(&self) -> Color {
        self.mirror_focus_border_color
    }

    pub fn mirror_focus_highlight_bg(&self) -> Color {
        self.mirror_focus_highlight_bg_color
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::dark_defaults()
    }
}

/// 解析颜色字符串为 ratatui Color
fn parse_color(s: Option<&str>) -> Option<Color> {
    match s? {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" => Some(Color::Gray),
        "darkgray" => Some(Color::DarkGray),
        "white" => Some(Color::White),
        "reset" => Some(Color::Reset),
        s if s.starts_with('#') => parse_hex(s),
        _ => None,
    }
}

fn parse_hex(s: &str) -> Option<Color> {
    let hex = s.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}
