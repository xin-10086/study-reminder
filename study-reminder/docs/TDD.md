# 技术设计文档 (TDD)

**项目名称**：StudyReminder
**版本**：v1.0
**日期**：2026-04-24
**作者**：AI 助手

---

## 1. 技术选型

| 层          | 技术                      | 版本  | 理由                                  |
| ----------- | ------------------------- | ----- | ------------------------------------- |
| 桌面壳      | Tauri                     | 2.x   | 极小体积 (~3MB)，低内存，原生系统能力 |
| 前端框架    | Svelte                    | 5.x   | 打包最小，学习曲线低，性能优          |
| 样式        | Tailwind CSS              | 4.x   | 快速实现暖色主题，摇树优化            |
| 本地数据库  | SQLite (rusqlite)         | 0.32+ | 零配置，支持复杂查询，本地永久存储    |
| 构建工具    | Vite                      | 6.x   | 快速 HMR                              |
| 包管理      | pnpm                      | 9+    | 速度快，省空间                        |
| 后端语言    | Rust                      | 1.80+ | 稳定，高性能，Tauri 原生              |
| 通知        | tauri-plugin-notification | 2.x   | 原生 Windows Toast                    |
| 开机自启    | tauri-plugin-autostart    | 2.x   | 系统注册表管理                        |
| 托盘/悬浮窗 | tauri 内置 tray + window  | -     | 原生实现                              |

## 2. 项目目录结构

```
study-reminder/
├── src/                          # Rust 后端
│   ├── main.rs                   # 入口：启动 WebView + 托盘 + 定时器
│   ├── db.rs                     # SQLite 初始化、CRUD 操作
│   ├── models.rs                 # 数据结构定义（Task, RepeatType 等）
│   ├── commands.rs               # Tauri 命令（前端调用的接口）
│   └── reminder.rs               # 定时器逻辑（检查任务、触发通知）
├── src-tauri/
│   ├── Cargo.toml                # Rust 依赖清单
│   ├── tauri.conf.json           # Tauri 应用配置（窗口、权限、插件）
│   ├── capabilities/             # Tauri 2.x 权限文件
│   └── icons/                    # 应用图标
├── frontend/                     # Svelte 前端
│   ├── package.json              # 前端依赖
│   ├── vite.config.ts            # Vite 配置
│   ├── svelte.config.js          # Svelte 配置
│   ├── tailwind.config.js        # Tailwind 主题配置
│   ├── index.html                # HTML 入口
│   └── src/
│       ├── main.ts               # 前端入口
│       ├── App.svelte            # 根组件（路由切换）
│       ├── lib/
│       │   ├── api.ts            # 调用 Tauri 命令的统一封装
│       │   ├── store.ts          # Svelte stores（全局状态）
│       │   └── types.ts          # TypeScript 类型定义
│       ├── components/
│       │   ├── FloatingWindow.svelte  # 悬浮半隐藏窗口
│       │   ├── MainWindow.svelte      # 主窗口壳
│       │   ├── MonthView.svelte       # 月视图
│       │   ├── DayView.svelte         # 日视图
│       │   ├── AllTasksView.svelte    # 全部任务视图
│       │   ├── TaskCard.svelte        # 任务卡片（复用）
│       │   └── TaskEditor.svelte      # 新建/编辑弹窗
│       └── styles/
│           └── app.css            # 全局样式 + Tailwind 层
├── docs/
│   ├── PRD.md
│   └── TDD.md
└── README.md
```

## 3. 数据库设计

### 表结构

```sql
CREATE TABLE IF NOT EXISTS tasks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    priority        INTEGER NOT NULL DEFAULT 2,   -- 1=高, 2=中, 3=低
    category        TEXT,                          -- '学习','学校','生活' 等
    due_date        TEXT,                          -- '2026-04-30' 或 NULL
    remind_date     TEXT,                          -- 提醒日期，通常与 due_date 相同或提前
    has_time_slot   INTEGER NOT NULL DEFAULT 0,    -- 0:无时间段, 1:有
    time_start      TEXT,                          -- '14:00' 或 NULL
    time_end        TEXT,                          -- '15:00' 或 NULL
    repeat_type     TEXT NOT NULL DEFAULT 'none',  -- 'none','daily','weekly','weekdays'
    repeat_days     TEXT,                          -- '1,3,5' (周一=1,周日=7) 仅 repeat_type='weekly'
    repeat_end      TEXT,                          -- 重复结束日期 '2026-06-30' 或 NULL
    completed       INTEGER NOT NULL DEFAULT 0,    -- 0 未完成, 1 已完成
    note            TEXT DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    sync_version    INTEGER DEFAULT 0,
    last_synced_at  TEXT
);

CREATE INDEX idx_tasks_due_date ON tasks(due_date);
CREATE INDEX idx_tasks_remind_date ON tasks(remind_date);
CREATE INDEX idx_tasks_completed ON tasks(completed);
```

### 跨月任务表（v1.0 简单实现）

跨月任务也在 tasks 表中，通过 `has_time_slot=0` 和 `due_date` 为区间结束，`remind_date` 为区间开始来标识。前端月视图横幅从 `remind_date` 跨到 `due_date`。

## 4. Rust 后端接口定义

所有 Tauri 命令均以 `#[tauri::command]` 暴露给前端。

| 命令                    | 参数                         | 返回值    | 说明                                                     |
| ----------------------- | ---------------------------- | --------- | -------------------------------------------------------- |
| `get_tasks_for_date`    | date: String                 | Vec<Task> | 获取指定日期所有任务（含周期任务动态生成）               |
| `get_tasks_for_month`   | year: i32, month: u32        | Vec<Task> | 获取指定月份所有任务（含周期任务）                       |
| `get_all_tasks`         | sort_by: String              | Vec<Task> | 获取全部未完成任务，排序（priority,due_date）            |
| `create_task`           | task: CreateTaskDto          | Task      | 创建新任务                                               |
| `update_task`           | id: i64, task: UpdateTaskDto | Task      | 更新任务                                                 |
| `delete_task`           | id: i64                      | bool      | 删除任务                                                 |
| `toggle_complete`       | id: i64                      | Task      | 切换完成状态                                             |
| `get_cross_month_tasks` | year: i32, month: u32        | Vec<Task> | 获取跨月任务（remind_date <= 月末 AND due_date >= 月初） |

## 5. 定时器与通知设计

- Rust 端使用 `tokio::spawn` 一个后台循环，每秒检查一次。
- 检查逻辑：
  - 遍历所有未完成的、有具体时间段的任务，如果当前时间 >= 任务开始时间 - 5分钟 且 尚未通知过，则发送通知并记录已通知（用内存 HashSet 记录 task_id + 日期）。
  - 检查截止日任务：当天是 remind_date 或 due_date 且未完成，发送通知。
  - 逾期任务：每天一次检查，如果逾期且未完成，再次通知（用 HashSet 记录当天已通知的逾期任务）。
- 通知使用 `tauri-plugin-notification` 的 `send_notification`。

## 6. 前端状态管理

- 使用 Svelte stores（`writable`）管理：
  - `currentView`: 'month' | 'day' | 'all'
  - `selectedDate`: string (YYYY-MM-DD)
  - `tasks`: Task[]
  - `crossMonthTasks`: Task[]
- API 调用封装在 `lib/api.ts`，所有命令通过 `@tauri-apps/api/core` 的 `invoke` 调用。

## 7. 视图交互流程

```
悬浮图标 (右下角吸附)
  ├─ 鼠标悬停 → 显示小窗口 (当天任务紧凑列表)
  │    └─ 点击任意处 → 打开主窗口
  └─ 鼠标点击 → 打开主窗口
       ├─ 默认：月视图
       │    ├─ 点击某一天 → 日视图
       │    ├─ 点击月份切换 → 切换月份
       │    └─ 点击“全部任务” → 全部任务视图
       ├─ 日视图：
       │    ├─ 点击“月视图” → 返回月视图
       │    ├─ 点击任务 → 编辑弹窗
       │    └─ 点击“新建任务” → 新建弹窗
       └─ 全部任务视图：
            ├─ 排序切换
            └─ 点击任务 → 编辑弹窗
```

## 8. 渐进式 Milestone

### M1：项目骨架 + 数据库 + 悬浮窗（1次迭代）

- [ ] 初始化 Tauri + Svelte 项目
- [ ] 创建 SQLite 数据库和 CRUD 命令
- [ ] 实现悬浮半隐藏窗口（吸附右侧）
- [ ] 实现当天任务紧凑列表
- [ ] 实现开机自启（插件）

### M2：视图层（2次迭代）

- [ ] 月视图（含跨月横幅）
- [ ] 日视图（时间段 + 无时间任务）
- [ ] 全部任务视图
- [ ] 创建/编辑任务弹窗
- [ ] 任务卡片组件（颜色标记优先级、截止日）

### M3：通知 + 周期任务（1次迭代）

- [ ] 实现定时器检查引擎
- [ ] 发送 Windows 通知（时间点提前5分钟、截止日、逾期）
- [ ] 周期性任务动态生成（daily/weekly/weekdays）

### M4：打磨与发布（1次迭代）

- [ ] 暖色主题完善
- [ ] 数据导出（JSON）
- [ ] 图标、打包测试
- [ ] 修复 Bug，性能优化

---

## 9. 开发环境要求

- Rust (rustc + cargo) 1.80+
- Node.js 20+ (推荐 22)
- pnpm 9+
- Windows 10/11 (开发目标)
- Tauri CLI 2.x (`cargo install tauri-cli --version "^2"`)
- Microsoft Visual C++ Build Tools (Tauri 依赖)

---

## 10. 风险 & 缓解措施

| 风险           | 缓解                                            |
| -------------- | ----------------------------------------------- |
| Rust 编译慢    | 使用 `cargo build` 增量编译，仅修改后端时重建   |
| 前端包体积大   | 使用 `vite` + `rollup` 摇树，Svelte 本身很小    |
| 通知延迟       | 定时器每秒检查，延迟 ≤ 1秒                      |
| 悬浮窗位置不对 | 使用 Tauri 的 `available_monitors` 计算屏幕边缘 |
| 数据迁移       | 数据库版本号 + 迁移脚本（暂不需要）             |

```

```
