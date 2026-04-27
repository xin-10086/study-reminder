import { invoke } from "@tauri-apps/api/core";
import type { Task, CreateTaskDto, UpdateTaskDto } from "./types";

/** 获取指定日期的所有任务 */
export async function getTasksForDate(date: string): Promise<Task[]> {
  return invoke("get_tasks_for_date", { date });
}

/** 获取指定月份的所有任务 */
export async function getTasksForMonth(year: number, month: number): Promise<Task[]> {
  return invoke("get_tasks_for_month", { year, month });
}

/** 获取全部未完成任务 */
export async function getAllTasks(sortBy: string = "priority"): Promise<Task[]> {
  return invoke("get_all_tasks", { sortBy });
}

/** 创建新任务 */
export async function createTask(task: CreateTaskDto): Promise<Task> {
  return invoke("create_task", { task });
}

/** 更新任务 */
export async function updateTask(id: number, task: UpdateTaskDto): Promise<Task> {
  return invoke("update_task", { id, task });
}

/** 删除任务 */
export async function deleteTask(id: number): Promise<boolean> {
  return invoke("delete_task", { id });
}

/** 切换完成状态 */
export async function toggleComplete(id: number): Promise<Task> {
  return invoke("toggle_complete", { id });
}

/** 获取跨月任务 */
export async function getCrossMonthTasks(year: number, month: number): Promise<Task[]> {
  return invoke("get_cross_month_tasks", { year, month });
}

/** 导出所有任务 */
export async function exportTasks(): Promise<Task[]> {
  return invoke("export_tasks");
}

/** 切换主窗口显示/隐藏 */
export async function toggleMainWindow(): Promise<void> {
  return invoke("toggle_main_window");
}

/** 切换开机自启 */
export async function toggleAutostart(): Promise<boolean> {
  return invoke("toggle_autostart");
}

/** 获取开机自启状态 */
export async function getAutostartStatus(): Promise<boolean> {
  return invoke("get_autostart_status");
}

/** 获取所有有截止日期的未完成任务，按截止日期早晚排序 */
export async function getAllDueDateTasks(): Promise<Task[]> {
  return invoke("get_all_due_date_tasks");
}

/** 获取指定日期已完成的任务 */
export async function getCompletedTasksForDate(date: string): Promise<Task[]> {
  return invoke("get_completed_tasks_for_date", { date });
}
