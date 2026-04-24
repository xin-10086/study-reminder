use serde::{Deserialize, Serialize};

/// 任务优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    High = 1,
    Medium = 2,
    Low = 3,
}

impl Priority {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Priority::High,
            2 => Priority::Medium,
            _ => Priority::Low,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Priority::High => 1,
            Priority::Medium => 2,
            Priority::Low => 3,
        }
    }
}

/// 重复类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepeatType {
    None,
    Daily,
    Weekly,
    Weekdays,
}

impl RepeatType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "daily" => RepeatType::Daily,
            "weekly" => RepeatType::Weekly,
            "weekdays" => RepeatType::Weekdays,
            _ => RepeatType::None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            RepeatType::None => "none",
            RepeatType::Daily => "daily",
            RepeatType::Weekly => "weekly",
            RepeatType::Weekdays => "weekdays",
        }
    }
}

/// 任务数据结构（对应数据库 tasks 表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub priority: i32,          // 1=高, 2=中, 3=低
    pub category: Option<String>,
    pub due_date: Option<String>,     // '2026-04-30'
    pub remind_date: Option<String>,  // 提醒日期
    pub has_time_slot: bool,
    pub time_start: Option<String>,   // '14:00'
    pub time_end: Option<String>,     // '15:00'
    pub repeat_type: String,          // 'none','daily','weekly','weekdays'
    pub repeat_days: Option<String>,  // '1,3,5' (周一=1, 周日=7)
    pub repeat_end: Option<String>,   // 重复结束日期
    pub completed: bool,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub sync_version: i32,
    pub last_synced_at: Option<String>,
}

/// 创建任务时的输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskDto {
    pub title: String,
    pub priority: Option<i32>,
    pub category: Option<String>,
    pub due_date: Option<String>,
    pub remind_date: Option<String>,
    pub has_time_slot: Option<bool>,
    pub time_start: Option<String>,
    pub time_end: Option<String>,
    pub repeat_type: Option<String>,
    pub repeat_days: Option<String>,
    pub repeat_end: Option<String>,
    pub note: Option<String>,
}

/// 更新任务时的输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskDto {
    pub title: Option<String>,
    pub priority: Option<i32>,
    pub category: Option<String>,
    pub due_date: Option<String>,
    pub remind_date: Option<String>,
    pub has_time_slot: Option<bool>,
    pub time_start: Option<String>,
    pub time_end: Option<String>,
    pub repeat_type: Option<String>,
    pub repeat_days: Option<String>,
    pub repeat_end: Option<String>,
    pub completed: Option<bool>,
    pub note: Option<String>,
}
