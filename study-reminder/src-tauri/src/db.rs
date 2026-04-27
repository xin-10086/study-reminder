use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use crate::models::{Task, CreateTaskDto, UpdateTaskDto};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 初始化数据库，创建表并返回 Database 实例
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path).map_err(|e| format!("无法打开数据库: {}", e))?;

        // 创建 tasks 表
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                title           TEXT NOT NULL,
                priority        INTEGER NOT NULL DEFAULT 2,
                category        TEXT,
                due_date        TEXT,
                remind_date     TEXT,
                has_time_slot   INTEGER NOT NULL DEFAULT 0,
                time_start      TEXT,
                time_end        TEXT,
                repeat_type     TEXT NOT NULL DEFAULT 'none',
                repeat_days     TEXT,
                repeat_end      TEXT,
                completed       INTEGER NOT NULL DEFAULT 0,
                note            TEXT DEFAULT '',
                created_at      TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                sync_version    INTEGER DEFAULT 0,
                last_synced_at  TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
            CREATE INDEX IF NOT EXISTS idx_tasks_remind_date ON tasks(remind_date);
            CREATE INDEX IF NOT EXISTS idx_tasks_completed ON tasks(completed);
            "
        ).map_err(|e| format!("创建表失败: {}", e))?;

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 获取指定日期的所有任务（含周期性任务）
    pub fn get_tasks_for_date(&self, date: &str) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;

        // 解析日期获取星期几 (1=周一, 7=周日)
        let weekday = self.parse_weekday(date)?;

        // 查询所有未完成的任务
        let mut stmt = conn.prepare(
            "SELECT * FROM tasks WHERE completed = 0 ORDER BY priority ASC, due_date ASC"
        ).map_err(|e| format!("查询准备失败: {}", e))?;

        let tasks = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                priority: row.get(2)?,
                category: row.get(3)?,
                due_date: row.get(4)?,
                remind_date: row.get(5)?,
                has_time_slot: row.get::<_, i32>(6)? != 0,
                time_start: row.get(7)?,
                time_end: row.get(8)?,
                repeat_type: row.get(9)?,
                repeat_days: row.get(10)?,
                repeat_end: row.get(11)?,
                completed: row.get::<_, i32>(12)? != 0,
                note: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
                sync_version: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        }).map_err(|e| format!("查询执行失败: {}", e))?;

        let mut result = Vec::new();
        for task in tasks {
            let t = task.map_err(|e| format!("行读取失败: {}", e))?;

            // 判断任务是否应该出现在这个日期
            if self.task_matches_date(&t, date, weekday) {
                result.push(t);
            }
        }

        Ok(result)
    }

    /// 判断任务是否匹配指定日期
    fn task_matches_date(&self, task: &Task, date: &str, weekday: i32) -> bool {
        match task.repeat_type.as_str() {
            "none" => {
                // 非周期任务：匹配 due_date 或 remind_date
                task.due_date.as_deref() == Some(date) ||
                task.remind_date.as_deref() == Some(date)
            }
            "daily" => {
                // 每天重复：检查是否在重复结束日期之前
                task.repeat_end.as_deref().map_or(true, |end| end >= date)
            }
            "weekdays" => {
                // 工作日：周一至周五，且在结束日期之前
                weekday >= 1 && weekday <= 5 &&
                task.repeat_end.as_deref().map_or(true, |end| end >= date)
            }
            "weekly" => {
                // 每周指定天：检查星期几匹配
                if let Some(ref days_str) = task.repeat_days {
                    let days: Vec<i32> = days_str.split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                    days.contains(&weekday) &&
                    task.repeat_end.as_deref().map_or(true, |end| end >= date)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// 获取指定月份的所有任务（含周期任务）
    pub fn get_tasks_for_month(&self, year: i32, month: u32) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
        let month_start = format!("{:04}-{:02}-01", year, month);
        let month_end = if month == 12 {
            format!("{:04}-01-01", year + 1)
        } else {
            format!("{:04}-{:02}-01", year, month + 1)
        };

        let mut stmt = conn.prepare(
            "SELECT * FROM tasks WHERE
                completed = 0 AND (
                    (repeat_type = 'none' AND due_date IS NULL AND remind_date IS NULL) OR
                    (repeat_type = 'none' AND (due_date >= ?1 AND due_date < ?2)) OR
                    (repeat_type = 'none' AND (remind_date >= ?1 AND remind_date < ?2)) OR
                    (repeat_type = 'daily' AND (repeat_end IS NULL OR repeat_end >= ?1)) OR
                    (repeat_type = 'weekdays' AND (repeat_end IS NULL OR repeat_end >= ?1)) OR
                    (repeat_type = 'weekly' AND (repeat_end IS NULL OR repeat_end >= ?1))
                )
            ORDER BY priority ASC, due_date ASC"
        ).map_err(|e| format!("查询准备失败: {}", e))?;

        let tasks = stmt.query_map(params![month_start, month_end], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                priority: row.get(2)?,
                category: row.get(3)?,
                due_date: row.get(4)?,
                remind_date: row.get(5)?,
                has_time_slot: row.get::<_, i32>(6)? != 0,
                time_start: row.get(7)?,
                time_end: row.get(8)?,
                repeat_type: row.get(9)?,
                repeat_days: row.get(10)?,
                repeat_end: row.get(11)?,
                completed: row.get::<_, i32>(12)? != 0,
                note: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
                sync_version: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        }).map_err(|e| format!("查询执行失败: {}", e))?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task.map_err(|e| format!("行读取失败: {}", e))?);
        }
        Ok(result)
    }

    /// 获取跨月任务（remind_date <= 月末 AND due_date >= 月初）
    pub fn get_cross_month_tasks(&self, year: i32, month: u32) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
        let month_start = format!("{:04}-{:02}-01", year, month);
        let month_end = if month == 12 {
            format!("{:04}-12-31", year)
        } else {
            // 下个月第一天减一天
            let next_month = if month == 12 { 1 } else { month + 1 };
            let next_year = if month == 12 { year + 1 } else { year };
            format!("{:04}-{:02}-01", next_year, next_month)
        };

        let mut stmt = conn.prepare(
            "SELECT * FROM tasks WHERE
                completed = 0 AND
                repeat_type = 'none' AND
                remind_date IS NOT NULL AND
                due_date IS NOT NULL AND
                remind_date <= ?2 AND
                due_date >= ?1 AND
                remind_date != due_date
            ORDER BY remind_date ASC"
        ).map_err(|e| format!("查询准备失败: {}", e))?;

        let tasks = stmt.query_map(params![month_start, month_end], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                priority: row.get(2)?,
                category: row.get(3)?,
                due_date: row.get(4)?,
                remind_date: row.get(5)?,
                has_time_slot: row.get::<_, i32>(6)? != 0,
                time_start: row.get(7)?,
                time_end: row.get(8)?,
                repeat_type: row.get(9)?,
                repeat_days: row.get(10)?,
                repeat_end: row.get(11)?,
                completed: row.get::<_, i32>(12)? != 0,
                note: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
                sync_version: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        }).map_err(|e| format!("查询执行失败: {}", e))?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task.map_err(|e| format!("行读取失败: {}", e))?);
        }
        Ok(result)
    }

    /// 获取全部未完成任务，按紧迫度+优先级排序
    pub fn get_all_tasks(&self, sort_by: &str) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;

        let order_clause = match sort_by {
            "priority" => "ORDER BY priority ASC, due_date ASC",
            "due_date" => "ORDER BY CASE WHEN due_date IS NULL OR due_date = '' THEN 1 ELSE 0 END, due_date ASC, priority ASC",
            _ => "ORDER BY priority ASC, due_date ASC",
        };

        let sql = format!(
            "SELECT * FROM tasks WHERE completed = 0 {}",
            order_clause
        );

        let mut stmt = conn.prepare(&sql).map_err(|e| format!("查询准备失败: {}", e))?;

        let tasks = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                priority: row.get(2)?,
                category: row.get(3)?,
                due_date: row.get(4)?,
                remind_date: row.get(5)?,
                has_time_slot: row.get::<_, i32>(6)? != 0,
                time_start: row.get(7)?,
                time_end: row.get(8)?,
                repeat_type: row.get(9)?,
                repeat_days: row.get(10)?,
                repeat_end: row.get(11)?,
                completed: row.get::<_, i32>(12)? != 0,
                note: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
                sync_version: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        }).map_err(|e| format!("查询执行失败: {}", e))?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task.map_err(|e| format!("行读取失败: {}", e))?);
        }
        Ok(result)
    }

    /// 创建新任务
    pub fn create_task(&self, dto: CreateTaskDto) -> Result<Task, String> {
        let priority = dto.priority.unwrap_or(2);
        let has_time_slot = dto.has_time_slot.unwrap_or(false);
        let repeat_type = dto.repeat_type.unwrap_or_else(|| "none".to_string());

        let id = {
            let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
            conn.execute(
                "INSERT INTO tasks (title, priority, category, due_date, remind_date, has_time_slot, time_start, time_end, repeat_type, repeat_days, repeat_end, note)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    dto.title,
                    priority,
                    dto.category,
                    dto.due_date,
                    dto.remind_date,
                    has_time_slot as i32,
                    dto.time_start,
                    dto.time_end,
                    repeat_type,
                    dto.repeat_days,
                    dto.repeat_end,
                    dto.note,
                ],
            ).map_err(|e| format!("创建任务失败: {}", e))?;
            conn.last_insert_rowid()
        }; // 锁在这里释放

        self.get_task_by_id(id)
    }

    /// 更新任务
    pub fn update_task(&self, id: i64, dto: UpdateTaskDto) -> Result<Task, String> {
        let existing = {
            let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
            self.get_task_by_id_internal(&conn, id)?
        }; // 锁释放

        let title = dto.title.unwrap_or(existing.title);
        let priority = dto.priority.unwrap_or(existing.priority);
        let category = dto.category.or(existing.category);
        let due_date = dto.due_date.or(existing.due_date);
        let remind_date = dto.remind_date.or(existing.remind_date);
        let has_time_slot = dto.has_time_slot.unwrap_or(existing.has_time_slot);
        let time_start = dto.time_start.or(existing.time_start);
        let time_end = dto.time_end.or(existing.time_end);
        let repeat_type = dto.repeat_type.unwrap_or(existing.repeat_type);
        let repeat_days = dto.repeat_days.or(existing.repeat_days);
        let repeat_end = dto.repeat_end.or(existing.repeat_end);
        let completed = dto.completed.unwrap_or(existing.completed);
        let note = dto.note.or(existing.note);

        {
            let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
            conn.execute(
                "UPDATE tasks SET title=?1, priority=?2, category=?3, due_date=?4, remind_date=?5,
                 has_time_slot=?6, time_start=?7, time_end=?8, repeat_type=?9, repeat_days=?10,
                 repeat_end=?11, completed=?12, note=?13, updated_at=datetime('now','localtime')
                 WHERE id=?14",
                params![
                    title, priority, category, due_date, remind_date,
                    has_time_slot as i32, time_start, time_end, repeat_type, repeat_days,
                    repeat_end, completed as i32, note, id,
                ],
            ).map_err(|e| format!("更新任务失败: {}", e))?;
        } // 锁释放

        self.get_task_by_id(id)
    }

    /// 删除任务
    pub fn delete_task(&self, id: i64) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
        let affected = conn.execute("DELETE FROM tasks WHERE id=?1", params![id])
            .map_err(|e| format!("删除任务失败: {}", e))?;
        Ok(affected > 0)
    }

    /// 切换完成状态
    pub fn toggle_complete(&self, id: i64) -> Result<Task, String> {
        {
            let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
            conn.execute(
                "UPDATE tasks SET completed = CASE WHEN completed=0 THEN 1 ELSE 0 END, updated_at=datetime('now','localtime') WHERE id=?1",
                params![id],
            ).map_err(|e| format!("切换状态失败: {}", e))?;
        } // 锁释放
        self.get_task_by_id(id)
    }

    /// 导出所有任务为 JSON（用于备份）
    pub fn export_all_tasks(&self) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
        let mut stmt = conn.prepare("SELECT * FROM tasks ORDER BY created_at ASC")
            .map_err(|e| format!("查询准备失败: {}", e))?;

        let tasks = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                priority: row.get(2)?,
                category: row.get(3)?,
                due_date: row.get(4)?,
                remind_date: row.get(5)?,
                has_time_slot: row.get::<_, i32>(6)? != 0,
                time_start: row.get(7)?,
                time_end: row.get(8)?,
                repeat_type: row.get(9)?,
                repeat_days: row.get(10)?,
                repeat_end: row.get(11)?,
                completed: row.get::<_, i32>(12)? != 0,
                note: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
                sync_version: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        }).map_err(|e| format!("查询执行失败: {}", e))?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task.map_err(|e| format!("行读取失败: {}", e))?);
        }
        Ok(result)
    }

    // ---- 内部辅助方法 ----

    fn get_task_by_id(&self, id: i64) -> Result<Task, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
        self.get_task_by_id_internal(&conn, id)
    }

    fn get_task_by_id_internal(&self, conn: &Connection, id: i64) -> Result<Task, String> {
        conn.query_row(
            "SELECT * FROM tasks WHERE id=?1",
            params![id],
            |row| {
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    priority: row.get(2)?,
                    category: row.get(3)?,
                    due_date: row.get(4)?,
                    remind_date: row.get(5)?,
                    has_time_slot: row.get::<_, i32>(6)? != 0,
                    time_start: row.get(7)?,
                    time_end: row.get(8)?,
                    repeat_type: row.get(9)?,
                    repeat_days: row.get(10)?,
                    repeat_end: row.get(11)?,
                    completed: row.get::<_, i32>(12)? != 0,
                    note: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                    sync_version: row.get(16)?,
                    last_synced_at: row.get(17)?,
                })
            }
        ).map_err(|e| format!("查询任务失败: {}", e))
    }

    /// 获取指定日期已完成的任务
    pub fn get_completed_tasks_for_date(&self, date: &str) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT * FROM tasks WHERE completed = 1 AND (due_date = ?1 OR remind_date = ?1) ORDER BY updated_at DESC"
        ).map_err(|e| format!("查询准备失败: {}", e))?;

        let tasks = stmt.query_map(params![date], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                priority: row.get(2)?,
                category: row.get(3)?,
                due_date: row.get(4)?,
                remind_date: row.get(5)?,
                has_time_slot: row.get::<_, i32>(6)? != 0,
                time_start: row.get(7)?,
                time_end: row.get(8)?,
                repeat_type: row.get(9)?,
                repeat_days: row.get(10)?,
                repeat_end: row.get(11)?,
                completed: row.get::<_, i32>(12)? != 0,
                note: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
                sync_version: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        }).map_err(|e| format!("查询执行失败: {}", e))?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task.map_err(|e| format!("行读取失败: {}", e))?);
        }
        Ok(result)
    }

    /// 清空所有数据
    pub fn clear_all_tasks(&self) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
        conn.execute("DELETE FROM tasks", [])
            .map_err(|e| format!("清空数据失败: {}", e))?;
        Ok(true)
    }

    /// 清空所有已完成任务
    pub fn clear_completed_tasks(&self) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
        conn.execute("DELETE FROM tasks WHERE completed = 1", [])
            .map_err(|e| format!("清空已完成任务失败: {}", e))?;
        Ok(true)
    }

    /// 获取所有有截止日期的未完成任务，按截止日期早晚排序
    pub fn get_all_due_date_tasks(&self) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().map_err(|e| format!("锁失败: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT * FROM tasks WHERE completed = 0 AND due_date IS NOT NULL AND due_date != '' ORDER BY due_date ASC"
        ).map_err(|e| format!("查询准备失败: {}", e))?;

        let tasks = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                priority: row.get(2)?,
                category: row.get(3)?,
                due_date: row.get(4)?,
                remind_date: row.get(5)?,
                has_time_slot: row.get::<_, i32>(6)? != 0,
                time_start: row.get(7)?,
                time_end: row.get(8)?,
                repeat_type: row.get(9)?,
                repeat_days: row.get(10)?,
                repeat_end: row.get(11)?,
                completed: row.get::<_, i32>(12)? != 0,
                note: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
                sync_version: row.get(16)?,
                last_synced_at: row.get(17)?,
            })
        }).map_err(|e| format!("查询执行失败: {}", e))?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task.map_err(|e| format!("行读取失败: {}", e))?);
        }
        Ok(result)
    }

    /// 解析日期字符串为星期几 (1=周一, 7=周日)
    fn parse_weekday(&self, date: &str) -> Result<i32, String> {
        use chrono::NaiveDate;
        let d = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|e| format!("日期解析失败: {}", e))?;
        // chrono: Mon=1, Sun=7
        let w = d.format("%u").to_string().parse::<i32>().unwrap_or(1);
        Ok(w)
    }
}
