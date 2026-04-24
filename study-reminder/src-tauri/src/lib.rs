mod models;
mod db;
mod commands;
mod reminder;

use db::Database;
use reminder::ReminderEngine;
use std::sync::Arc;
use tauri::Manager;

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
        ])
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}
