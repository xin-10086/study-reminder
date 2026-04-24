use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use chrono::Local;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;
use crate::db::Database;

/// 通知管理器，跟踪已发送的通知避免重复
pub struct ReminderEngine {
    /// 已通知的任务ID+日期组合 (task_id:date)
    notified: Arc<Mutex<HashSet<String>>>,
}

impl ReminderEngine {
    pub fn new() -> Self {
        ReminderEngine {
            notified: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// 启动后台检查循环
    pub fn start(&self, app: AppHandle, db: Arc<Database>) {
        let notified = self.notified.clone();
        let app_clone = app.clone();

        std::thread::spawn(move || {
            loop {
                if let Err(e) = Self::check_reminders(&app_clone, &db, &notified) {
                    log::error!("提醒检查失败: {}", e);
                }
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        });
    }

    fn check_reminders(
        app: &AppHandle,
        db: &Arc<Database>,
        notified: &Mutex<HashSet<String>>,
    ) -> Result<(), String> {
        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();
        let current_time = now.format("%H:%M").to_string();

        let tasks = db.get_tasks_for_date(&today)?;

        for task in &tasks {
            if task.completed {
                continue;
            }

            // 1. 具体时间任务：提前5分钟通知
            if task.has_time_slot {
                if let Some(ref start) = task.time_start {
                    if let Ok(notify_time) = Self::time_minus_minutes(start, 5) {
                        if current_time == notify_time {
                            let key = format!("time:{}:{}", task.id, today);
                            let mut set = notified.lock().map_err(|e| format!("锁失败: {}", e))?;
                            if !set.contains(&key) {
                                Self::send_notification(app, &task.title, &format!("任务即将开始: {}", task.title));
                                set.insert(key);
                            }
                        }
                    }
                }
            }

            // 2. 截止日任务：当天通知
            if let Some(ref due) = task.due_date {
                if due == &today {
                    let key = format!("due:{}:{}", task.id, today);
                    let mut set = notified.lock().map_err(|e| format!("锁失败: {}", e))?;
                    if !set.contains(&key) {
                        Self::send_notification(app, "截止日提醒", &format!("任务「{}」今天截止！", task.title));
                        set.insert(key);
                    }
                }
            }

            // 3. 提醒日期任务
            if let Some(ref remind) = task.remind_date {
                if remind == &today && task.due_date.as_deref() != Some(&today) {
                    let key = format!("remind:{}:{}", task.id, today);
                    let mut set = notified.lock().map_err(|e| format!("锁失败: {}", e))?;
                    if !set.contains(&key) {
                        Self::send_notification(app, "任务提醒", &format!("任务「{}」需要关注", task.title));
                        set.insert(key);
                    }
                }
            }

            // 4. 逾期任务：每天通知一次
            if let Some(ref due) = task.due_date {
                if due < &today {
                    let key = format!("overdue:{}:{}", task.id, today);
                    let mut set = notified.lock().map_err(|e| format!("锁失败: {}", e))?;
                    if !set.contains(&key) {
                        Self::send_notification(app, "逾期提醒", &format!("任务「{}」已逾期！请尽快处理", task.title));
                        set.insert(key);
                    }
                }
            }
        }

        Ok(())
    }

    fn send_notification(app: &AppHandle, title: &str, body: &str) {
        if let Err(e) = app.notification()
            .builder()
            .title(title)
            .body(body)
            .show()
        {
            log::error!("发送通知失败: {}", e);
        }
    }

    /// 将时间减去指定分钟数，返回 "HH:MM" 格式
    fn time_minus_minutes(time: &str, minutes: i64) -> Result<String, String> {
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() != 2 {
            return Err("时间格式错误".to_string());
        }
        let hour: i64 = parts[0].parse().map_err(|_| "小时解析失败")?;
        let min: i64 = parts[1].parse().map_err(|_| "分钟解析失败")?;

        let total_minutes = hour * 60 + min - minutes;
        let new_hour = ((total_minutes % 1440 + 1440) % 1440) / 60;
        let new_min = (total_minutes % 1440 + 1440) % 1440 % 60;

        Ok(format!("{:02}:{:02}", new_hour, new_min))
    }
}
