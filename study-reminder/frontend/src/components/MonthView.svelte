<script lang="ts">
  import { tasks, crossMonthTasks, currentYear, currentMonth, currentView, selectedDate } from "../lib/store";
  import { PRIORITY_COLORS } from "../lib/types";

  let { onselect }: { onselect: (date: string) => void } = $props();

  const WEEKDAYS = ["一", "二", "三", "四", "五", "六", "日"];

  function getDaysInMonth(year: number, month: number): number {
    return new Date(year, month, 0).getDate();
  }

  function getFirstDayOfMonth(year: number, month: number): number {
    // 返回 0=周日, 1=周一, ... 6=周六
    return new Date(year, month - 1, 1).getDay();
  }

  function formatDate(year: number, month: number, day: number): string {
    return `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
  }

  function getTasksForDay(day: number) {
    const dateStr = formatDate($currentYear, $currentMonth, day);
    const todayStr = new Date().toISOString().slice(0, 10);
    const date = new Date($currentYear, $currentMonth - 1, day);
    const dayOfWeek = date.getDay(); // 0=周日, 1=周一, ... 6=周六

    return ($tasks || []).filter((t) => {
      // 有截止日期或提醒日期的任务
      if (t.due_date === dateStr || t.remind_date === dateStr) return true;

      // 周期任务：根据类型判断是否显示在今天
      if (t.repeat_type !== "none") {
        // 检查是否超过重复结束日期
        if (t.repeat_end && dateStr > t.repeat_end) return false;

        if (t.repeat_type === "daily") {
          return true; // 每天显示
        }
        if (t.repeat_type === "weekdays") {
          return dayOfWeek >= 1 && dayOfWeek <= 5; // 周一至周五
        }
        if (t.repeat_type === "weekly" && t.repeat_days) {
          // 将 repeat_days "1,3,5" 转为数字数组，注意数据库存的是 1=周一, 7=周日
          const days = t.repeat_days.split(",").map(Number);
          // dayOfWeek: 0=周日, 1=周一, ... 6=周六
          // 需要将 dayOfWeek 转为 1=周一, 7=周日 的格式
          const mappedDay = dayOfWeek === 0 ? 7 : dayOfWeek;
          return days.includes(mappedDay);
        }
        return false;
      }

      // 没有设置任何日期的任务，只显示在今天
      if (!t.due_date && !t.remind_date && dateStr === todayStr) return true;
      return false;
    });
  }

  function isToday(day: number): boolean {
    const today = new Date();
    return (
      today.getFullYear() === $currentYear &&
      today.getMonth() + 1 === $currentMonth &&
      today.getDate() === day
    );
  }

  function isOverdue(dateStr: string): boolean {
    return dateStr < new Date().toISOString().slice(0, 10);
  }

  function isWeekend(day: number): boolean {
    const date = new Date($currentYear, $currentMonth - 1, day);
    const dow = date.getDay();
    return dow === 0 || dow === 6;
  }
</script>

<div class="h-full flex flex-col p-4">
  <!-- 跨月任务横幅 -->
  {#if $crossMonthTasks && $crossMonthTasks.length > 0}
    <div class="mb-3 space-y-1.5">
      {#each $crossMonthTasks as task}
        <div class="px-4 py-2 bg-gradient-to-r from-orange-50 to-amber-50 rounded-xl text-sm text-stone-700 border border-orange-200/60 shadow-sm card-hover">
          <div class="flex items-center gap-2">
            <span>📌</span>
            <span class="font-medium">{task.title}</span>
            <span class="text-xs text-stone-400 ml-auto">
              {task.remind_date} ~ {task.due_date}
            </span>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <!-- 星期表头 -->
  <div class="grid grid-cols-7 mb-2">
    {#each WEEKDAYS as wd, i}
      <div class="text-center text-xs font-medium py-1 {i >= 5 ? 'text-stone-400' : 'text-stone-500'}">{wd}</div>
    {/each}
  </div>

  <!-- 日期网格 - 圆角卡片风格，支持纵向滚动，每行等高 -->
  <div class="flex-1 overflow-y-auto min-h-0">
    <div class="grid grid-cols-7 auto-rows-fr gap-2 p-0.5">
    {#each Array(getDaysInMonth($currentYear, $currentMonth)) as _, i}
      {@const day = i + 1}
      {@const dateStr = formatDate($currentYear, $currentMonth, day)}
      {@const dayTasks = getTasksForDay(day)}
      {@const firstDay = getFirstDayOfMonth($currentYear, $currentMonth)}
      <!-- 用空白占位调整第一行偏移 -->
      {#if i === 0}
        {#each Array(firstDay === 0 ? 6 : firstDay - 1) as _}
          <div></div>
        {/each}
      {/if}

      <button
        onclick={() => onselect(dateStr)}
        class="relative bg-white rounded-xl p-2.5 text-left hover:shadow-lg hover:-translate-y-1.5 transition-all duration-200 min-h-[120px] flex flex-col border border-stone-100 shadow-sm
          {$currentView === 'day' && $selectedDate === dateStr ? 'ring-2 ring-orange-400 shadow-md' : ''}
          {isToday(day) ? 'border-orange-300' : ''}"
      >
        <!-- 日期标头 -->
        <div class="flex items-center justify-between mb-1.5">
          <span
            class="text-xs font-bold w-6 h-6 flex items-center justify-center rounded-full
              {isToday(day) ? 'bg-gradient-to-br from-orange-400 to-orange-600 text-white shadow-sm shadow-orange-200' : isWeekend(day) ? 'text-stone-400' : 'text-stone-700'}"
          >
            {day}
          </span>
          {#if dayTasks.length > 0}
            <span class="text-[10px] font-semibold text-stone-400 bg-stone-100/80 px-1.5 py-0.5 rounded-full">{dayTasks.length}</span>
          {/if}
        </div>
        <!-- 任务列表 -->
        <div class="flex-1 overflow-hidden space-y-1">
          {#each dayTasks.slice(0, 3) as task}
            <div class="flex items-start gap-1.5 group/task">
              <!-- 优先级小圆点 -->
              <span class="mt-0.5 w-1.5 h-1.5 rounded-full flex-shrink-0 {task.priority === 1 ? 'bg-red-400' : task.priority === 2 ? 'bg-amber-400' : 'bg-stone-300'}"></span>
              <!-- 标题 + DDL -->
              <span class="flex-1 text-[10px] leading-tight truncate text-stone-600 font-medium">
                {task.title}
              </span>
              {#if task.due_date === dateStr && task.repeat_type === "none"}
                <span class="text-[7px] font-bold text-red-500 bg-red-50 px-1 rounded flex-shrink-0 mt-0.5">截止</span>
              {/if}
            </div>
          {/each}
          {#if dayTasks.length > 3}
            <div class="text-[10px] text-stone-400 font-medium text-center py-0.5 bg-stone-50 rounded-md">+{dayTasks.length - 3}</div>
          {/if}
        </div>
      </button>
    {/each}
    </div>
  </div>
</div>
