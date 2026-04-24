/** 任务数据结构（对应 Rust 后端 Task） */
export interface Task {
  id: number;
  title: string;
  priority: number; // 1=高, 2=中, 3=低
  category: string | null;
  due_date: string | null;
  remind_date: string | null;
  has_time_slot: boolean;
  time_start: string | null;
  time_end: string | null;
  repeat_type: string;
  repeat_days: string | null;
  repeat_end: string | null;
  completed: boolean;
  note: string | null;
  created_at: string;
  updated_at: string;
  sync_version: number;
  last_synced_at: string | null;
}

/** 创建任务时的输入 */
export interface CreateTaskDto {
  title: string;
  priority?: number;
  category?: string;
  due_date?: string;
  remind_date?: string;
  has_time_slot?: boolean;
  time_start?: string;
  time_end?: string;
  repeat_type?: string;
  repeat_days?: string;
  repeat_end?: string;
  note?: string;
}

/** 更新任务时的输入 */
export interface UpdateTaskDto {
  title?: string;
  priority?: number;
  category?: string;
  due_date?: string;
  remind_date?: string;
  has_time_slot?: boolean;
  time_start?: string;
  time_end?: string;
  repeat_type?: string;
  repeat_days?: string;
  repeat_end?: string;
  completed?: boolean;
  note?: string;
}

/** 视图类型 */
export type ViewType = "month" | "day" | "all";

/** 优先级对应的颜色 */
export const PRIORITY_COLORS: Record<number, string> = {
  1: "bg-red-100 text-red-800 border-red-200",   // 高
  2: "bg-yellow-100 text-yellow-800 border-yellow-200", // 中
  3: "bg-gray-100 text-gray-600 border-gray-200",  // 低
};

/** 优先级对应的标签 */
export const PRIORITY_LABELS: Record<number, string> = {
  1: "高",
  2: "中",
  3: "低",
};
