use tauri::State;
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
