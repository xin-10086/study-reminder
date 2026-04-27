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

            // 创建悬浮窗（仅图标，点击打开主窗口）
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
            commands::get_all_due_date_tasks,
            commands::start_drag_floating,
        ])
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}

/// 设置悬浮窗（仅图标，点击打开主窗口）
fn setup_floating_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::WebviewWindowBuilder;

    // 获取屏幕尺寸（默认 1920x1080）
    let (screen_width, screen_height) = if let Some(main_window) = app.get_webview_window("main") {
        if let Some(monitor) = main_window.current_monitor().ok().flatten() {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            (size.width as f64 / scale, size.height as f64 / scale)
        } else {
            (1920.0, 1080.0)
        }
    } else {
        (1920.0, 1080.0)
    };

    // 创建悬浮窗 - 使用独立的 floating.html，不依赖 Svelte 应用
    let float_window = WebviewWindowBuilder::new(
        app,
        "floating",
        tauri::WebviewUrl::App("floating.html".into()),
    )
    .title("")
    .inner_size(50.0, 50.0)
    .position(screen_width - 56.0, screen_height / 2.0 - 25.0)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(true)
    .build()?;

    log::info!("悬浮窗已创建: floating");

    // 注入悬浮窗 UI（使用封面图片作为图标）
    let fw = float_window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(3000));
        
        // 使用 include_bytes! 在编译时嵌入封面图片
        let icon_bg = {
            let bytes = include_bytes!("../../frontend/public/floating-icon.jpg");
            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
            log::info!("封面图片已加载（编译时嵌入），大小: {} bytes, base64: {} chars", bytes.len(), b64.len());
            format!("data:image/jpeg;base64,{}", b64)
        };
        
        // 使用原始字符串构建 HTML，避免转义问题
        let html = format!(
            r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>
            *{{margin:0;padding:0;box-sizing:border-box}}
            body{{width:50px;height:50px;overflow:hidden;background:transparent}}
            #float-icon{{width:44px;height:44px;background-image:url('{icon_bg}');background-size:cover;background-position:center;border-radius:14px;box-shadow:0 3px 10px rgba(0,0,0,0.25);position:absolute;top:3px;left:3px;cursor:grab;user-select:none}}
            #float-icon:active{{cursor:grabbing}}
            </style></head><body>
            <div id="float-icon"></div>
            <script>
            (function(){{
                var i=document.getElementById("float-icon"),dragStarted=!1;
                i.addEventListener("mousedown",function(e){{
                    e.preventDefault();dragStarted=!1;
                    var startX=e.clientX,startY=e.clientY;
                    function onMove(e){{
                        Math.abs(e.clientX-startX)>3||Math.abs(e.clientY-startY)>3?
                            (dragStarted=!0,document.removeEventListener("mousemove",onMove),
                            document.removeEventListener("mouseup",onUp),
                            window.__TAURI_INTERNALS__&&window.__TAURI_INTERNALS__.invoke&&
                            window.__TAURI_INTERNALS__.invoke("start_drag_floating")):void 0
                    }}
                    function onUp(e){{
                        document.removeEventListener("mousemove",onMove),
                        document.removeEventListener("mouseup",onUp),
                        dragStarted||
                            (window.__TAURI_INTERNALS__&&window.__TAURI_INTERNALS__.invoke&&
                            window.__TAURI_INTERNALS__.invoke("toggle_main_window"))
                    }}
                    document.addEventListener("mousemove",onMove),
                    document.addEventListener("mouseup",onUp)
                }});
            }})();
            </script></body></html>"#
        );
        
        // 压缩 HTML：移除换行和多余空格
        let html_compressed = html
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("");
        
        let js = format!(
            r#"document.open();
            document.write('{}');
            document.close();
            console.log("悬浮窗图标已设置（封面图片）");"#,
            html_compressed.replace("'", "\\'")
        );
        
        match fw.eval(&js) {
            Ok(_) => log::info!("悬浮窗图标注入成功（封面图片）"),
            Err(e) => log::error!("悬浮窗图标注入失败: {:?}", e),
        }
    });




    Ok(())
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
        .tooltip("Kill the DDL - 学习待办提醒")
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
            // 仅处理左键点击，右键点击由菜单处理
            if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
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
            }
        })
        .build(app)?;

    Ok(())
}

