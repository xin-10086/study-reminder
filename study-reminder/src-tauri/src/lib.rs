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

            // 主窗口关闭时隐藏到托盘（不退出）
            if let Some(window) = app.get_webview_window("main") {
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let _ = handle.get_webview_window("main").map(|w| w.hide());
                        api.prevent_close();
                    }
                });
            }

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

/// 设置系统托盘（作为悬浮入口）
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
        .tooltip("StudyReminder - 学习待办提醒")
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
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// 设置悬浮窗（使用 eval 注入脚本）
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

        // 创建悬浮窗 - 加载 Svelte 应用，通过 URL 参数区分
        let float_window = WebviewWindowBuilder::new(
            app,
            "floating",
            tauri::WebviewUrl::App("?floating=true".into()),
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

        log::info!("悬浮窗已创建: floating");

        // 等待窗口加载完成后注入脚本
        let fw = float_window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(2000));
            // 注入脚本将 Svelte 应用转换为悬浮窗模式
            let js = r#"
                (function() {
                    // 检查是否是悬浮窗窗口（通过窗口标签名判断）
                    var isFloating = false;
                    try {
                        // 尝试通过 Tauri API 获取当前窗口标签
                        if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.metadata) {
                            // 如果无法判断，通过 URL 参数判断
                            isFloating = window.location.search.includes('floating') || window.name === 'floating';
                        }
                    } catch(e) {}
                    
                    // 如果不是悬浮窗窗口，不执行任何操作
                    if (!isFloating) {
                        console.log('主窗口模式，跳过悬浮窗脚本');
                        return;
                    }
                    
                    console.log('悬浮窗模式已激活');
                    
                    // 隐藏主应用内容，显示悬浮图标
                    var style = document.createElement('style');
                    style.textContent = `
                        body { background: transparent !important; margin: 0; padding: 0; overflow: hidden; }
                        #app { display: none !important; }
                        #floatContainer { 
                            display: flex !important; 
                            width: 40px; height: 40px;
                            background: linear-gradient(135deg, #f97316, #ea580c);
                            border-radius: 12px;
                            align-items: center; justify-content: center;
                            cursor: pointer;
                            box-shadow: 0 2px 8px rgba(0,0,0,0.2);
                        }
                        #floatContainer:hover { transform: scale(1.1); }
                        #floatContainer .icon-text { color: white; font-size: 18px; font-weight: bold; }
                    `;
                    document.head.appendChild(style);
                    
                    // 创建悬浮容器
                    var container = document.createElement('div');
                    container.id = 'floatContainer';
                    container.innerHTML = '<span class="icon-text">📋</span>';
                    document.body.appendChild(container);
                    document.body.style.margin = '0';
                    document.body.style.padding = '0';
                    
                    // 点击事件
                    container.addEventListener('click', function() {
                        if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                            window.__TAURI_INTERNALS__.invoke('toggle_main_window');
                        }
                    });
                })();
            "#;
            match fw.eval(js) {
                Ok(_) => log::info!("悬浮窗脚本注入成功"),
                Err(e) => log::error!("悬浮窗脚本注入失败: {:?}", e),
            }
        });
    }

    Ok(())
}
