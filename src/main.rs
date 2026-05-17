mod app;
mod adapters;
mod config;
mod mirror;
mod tui;
mod ui;
mod utils;

use anyhow::Result;
use app::App;
use config::Config;

use crate::adapters::cargo::CargoAdapter;
use crate::adapters::homebrew::HomebrewAdapter;
use crate::adapters::npm::NpmAdapter;

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

    // 构建适配器列表
    let adapters: Vec<Box<dyn adapters::PackageManagerAdapter>> = vec![
        Box::new(CargoAdapter::new(&config)),
        Box::new(NpmAdapter::new(&config)),
        Box::new(HomebrewAdapter::new(&config)),
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
        terminal.draw(|frame| ui::draw(frame, &app))?;
        app.handle_event()?;
    }

    // 恢复终端
    tui::restore(terminal)?;

    Ok(())
}
