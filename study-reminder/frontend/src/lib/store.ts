import { writable } from "svelte/store";
import type { Task, ViewType } from "./types";

/** 当前视图 */
export const currentView = writable<ViewType>("month");

/** 选中的日期 (YYYY-MM-DD) */
export const selectedDate = writable<string>("");

/** 当前月份的任务列表 */
export const tasks = writable<Task[]>([]);

/** 跨月任务列表 */
export const crossMonthTasks = writable<Task[]>([]);

/** 当前查看的年份 */
export const currentYear = writable<number>(new Date().getFullYear());

/** 当前查看的月份 (1-12) */
export const currentMonth = writable<number>(new Date().getMonth() + 1);

/** 是否显示新建/编辑弹窗 */
export const showEditor = writable<boolean>(false);

/** 正在编辑的任务（null 表示新建） */
export const editingTask = writable<Task | null>(null);

/** 加载状态 */
export const isLoading = writable<boolean>(false);
