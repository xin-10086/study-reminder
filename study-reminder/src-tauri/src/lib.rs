mod models;
mod db;
mod commands;
mod reminder;

use db::Database;
use reminder::ReminderEngine;
use std::sync::Arc;
use tauri::{
    Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle, Runtime,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--flag1"]),
        ))
        .setup(|app| {
            // 获取应用数据目录存放数据库
            let app_data_dir = app.path().app_data_dir().expect("获取应用数据目录失败");
            std::fs::create_dir_all(&app_data_dir).expect("创建数据目录失败");
            let db_path = app_data_dir.join("study_reminder.db");
            let db_path_str = db_path.to_str().expect("路径转换失败");

            log::info!("数据库路径: {}", db_path_str);

            // 初始化数据库
            let database = Database::new(db_path_str).expect("数据库初始化失败");
            let db_arc = Arc::new(database);

            // 注册数据库到 Tauri 状态管理
            app.manage::<Database>((*db_arc).clone());

            // 启动提醒引擎
            let reminder = ReminderEngine::new();
            reminder.start(app.handle().clone(), db_arc);

            // 创建系统托盘
            setup_tray(app.handle())?;

            // 创建悬浮窗（第二个窗口）
            setup_floating_window(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks_for_date,
            commands::get_tasks_for_month,
            commands::get_all_tasks,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::toggle_complete,
            commands::get_cross_month_tasks,
            commands::export_tasks,
            commands::toggle_main_window,
            commands::toggle_autostart,
            commands::get_autostart_status,
        ])
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}

/// 设置系统托盘
fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

/// 设置悬浮窗（吸附在屏幕右侧的小窗口）
fn setup_floating_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::WebviewWindowBuilder;

    // 获取主窗口
    let main_window = app.get_webview_window("main").unwrap();

    // 获取屏幕尺寸
    let monitor = main_window.current_monitor().ok().flatten();
    if let Some(monitor) = monitor {
        let size = monitor.size();
        let scale = monitor.scale_factor();

        let screen_width = size.width as f64 / scale;
        let screen_height = size.height as f64 / scale;

        // 创建悬浮窗 - 40x40 在屏幕右侧中间
        let float_window = WebviewWindowBuilder::new(
            app,
            "floating",
            tauri::WebviewUrl::App("floating.html".into()),
        )
        .title("")
        .inner_size(40.0, 40.0)
        .position(screen_width - 50.0, screen_height / 2.0 - 20.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .build()?;

        // 设置窗口为圆角
        let _ = float_window.set_shadow(false);
    }

    Ok(())
}
