mod app;
mod adapters;
mod config;
mod mirror;
mod palette;
mod tui;
mod ui;
mod utils;

use anyhow::Result;
use app::App;
use config::Config;

use crate::adapters::cargo::CargoAdapter;
use crate::adapters::goenv::GoAdapter;
use crate::adapters::homebrew::HomebrewAdapter;
use crate::adapters::npm::NpmAdapter;
use crate::adapters::pacman::PacmanAdapter;
use crate::adapters::pip::PipAdapter;

fn main() -> Result<()> {
    // 初始化日志（可通过 RUST_LOG 环境变量控制）
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mirroman=info".into()),
        )
        .init();

    // 加载配置
    let config = Config::load()?;

    // 初始化备份目录
    let backup_root = config::backup_dir();
    if !backup_root.exists() {
        std::fs::create_dir_all(&backup_root)?;
        tracing::info!("备份目录已创建: {}", backup_root.display());
    }

    // 主题调色板（在 config move 前提取）
    let fallback_theme = crate::config::ThemeConfig {
        focus_border: None, focus_highlight_bg: None, dim_highlight_bg: None,
        highlight_fg: None, header_fg: None, dim_text: None,
        key_fg: None, key_desc: None, success: None, error: None,
        warning: None, mirror_focus_border: None, mirror_focus_highlight_bg: None,
    };
    let palette = palette::Palette::from_theme(
        config.settings.themes
            .get(&config.settings.theme)
            .unwrap_or(&fallback_theme),
    );

    // 构建适配器列表
    // 注意：顺序即 TUI 左侧面板中显示的顺序
    let adapters: Vec<Box<dyn adapters::PackageManagerAdapter>> = vec![
        Box::new(CargoAdapter::new(&config)),
        Box::new(NpmAdapter::new(&config)),
        Box::new(PipAdapter::new(&config)),
        Box::new(GoAdapter::new(&config)),
        Box::new(HomebrewAdapter::new(&config)),
        Box::new(PacmanAdapter::new(&config)),
    ];

    // 检测各适配器当前使用的镜像
    let current_mirror_names: Vec<Option<String>> = adapters
        .iter()
        .map(|a| a.current_mirror_name())
        .collect();

    // 创建 App 状态
    let mut app = App::new(config, adapters, current_mirror_names);

    // 初始化 TUI
    let mut terminal = tui::init()?;

    // 主事件循环
    while app.running {
        if app.needs_full_redraw {
            terminal.clear()?;
            app.needs_full_redraw = false;
        }
        terminal.draw(|frame| ui::draw(frame, &app, &palette))?;
        app.handle_event()?;
    }

    // 恢复终端
    tui::restore(terminal)?;

    Ok(())
}
