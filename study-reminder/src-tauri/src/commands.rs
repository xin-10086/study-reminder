use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use crate::db::Database;
use crate::models::{Task, CreateTaskDto, UpdateTaskDto};

#[tauri::command]
pub fn get_tasks_for_date(db: State<Database>, date: String) -> Result<Vec<Task>, String> {
    db.get_tasks_for_date(&date)
}

#[tauri::command]
pub fn get_tasks_for_month(db: State<Database>, year: i32, month: u32) -> Result<Vec<Task>, String> {
    db.get_tasks_for_month(year, month)
}

#[tauri::command]
pub fn get_all_tasks(db: State<Database>, sort_by: String) -> Result<Vec<Task>, String> {
    db.get_all_tasks(&sort_by)
}

#[tauri::command]
pub fn create_task(db: State<Database>, task: CreateTaskDto) -> Result<Task, String> {
    db.create_task(task)
}

#[tauri::command]
pub fn update_task(db: State<Database>, id: i64, task: UpdateTaskDto) -> Result<Task, String> {
    db.update_task(id, task)
}

#[tauri::command]
pub fn delete_task(db: State<Database>, id: i64) -> Result<bool, String> {
    db.delete_task(id)
}

#[tauri::command]
pub fn toggle_complete(db: State<Database>, id: i64) -> Result<Task, String> {
    db.toggle_complete(id)
}

#[tauri::command]
pub fn get_cross_month_tasks(db: State<Database>, year: i32, month: u32) -> Result<Vec<Task>, String> {
    db.get_cross_month_tasks(year, month)
}

#[tauri::command]
pub fn export_tasks(db: State<Database>) -> Result<Vec<Task>, String> {
    db.export_all_tasks()
}

/// 切换主窗口显示/隐藏
#[tauri::command]
pub fn toggle_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
    Ok(())
}

/// 切换开机自启状态
#[tauri::command]
pub fn toggle_autostart(app: AppHandle) -> Result<bool, String> {
    let autostart = app.autostart();
    let is_enabled = autostart.is_enabled().unwrap_or(false);
    if is_enabled {
        autostart.disable().map_err(|e| format!("禁用开机自启失败: {}", e))?;
        Ok(false)
    } else {
        autostart.enable().map_err(|e| format!("启用开机自启失败: {}", e))?;
        Ok(true)
    }
}

/// 获取开机自启状态
#[tauri::command]
pub fn get_autostart_status(app: AppHandle) -> Result<bool, String> {
    let autostart = app.autostart();
    Ok(autostart.is_enabled().unwrap_or(false))
}
