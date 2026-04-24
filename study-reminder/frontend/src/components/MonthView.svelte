<script lang="ts">
  import { tasks, crossMonthTasks, currentYear, currentMonth } from "../lib/store";
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
    return ($tasks || []).filter((t) => {
      if (t.due_date === dateStr || t.remind_date === dateStr) return true;
      // 周期任务显示在每一天
      if (t.repeat_type !== "none") return true;
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
</script>

<div class="h-full flex flex-col p-4">
  <!-- 跨月任务横幅 -->
  {#if $crossMonthTasks && $crossMonthTasks.length > 0}
    <div class="mb-3 space-y-1">
      {#each $crossMonthTasks as task}
        <div class="px-3 py-1.5 bg-gradient-to-r from-orange-100 to-amber-100 rounded text-sm text-stone-700 border border-orange-200">
          📌 {task.title}
          <span class="text-xs text-stone-500 ml-2">
            {task.remind_date} ~ {task.due_date}
          </span>
        </div>
      {/each}
    </div>
  {/if}

  <!-- 星期表头 -->
  <div class="grid grid-cols-7 mb-1">
    {#each WEEKDAYS as wd}
      <div class="text-center text-xs font-medium text-stone-500 py-1">{wd}</div>
    {/each}
  </div>

  <!-- 日期网格 -->
  <div class="grid grid-cols-7 flex-1 gap-px bg-stone-200 rounded overflow-hidden">
    {#each Array(getDaysInMonth($currentYear, $currentMonth)) as _, i}
      {@const day = i + 1}
      {@const dateStr = formatDate($currentYear, $currentMonth, day)}
      {@const dayTasks = getTasksForDay(day)}
      {@const firstDay = getFirstDayOfMonth($currentYear, $currentMonth)}
      <!-- 用空白占位调整第一行偏移 -->
      {#if i === 0}
        {#each Array(firstDay === 0 ? 6 : firstDay - 1) as _}
          <div class="bg-stone-50"></div>
        {/each}
      {/if}

      <button
        onclick={() => onselect(dateStr)}
        class="bg-white p-1 text-left hover:bg-orange-50 transition-colors min-h-[70px] flex flex-col {$currentView === 'day' && $selectedDate === dateStr ? 'ring-2 ring-orange-400' : ''}"
      >
        <span
          class="text-xs font-medium px-1 rounded inline-block w-fit
            {isToday(day) ? 'bg-orange-500 text-white' : 'text-stone-600'}"
        >
          {day}
        </span>
        <div class="flex-1 overflow-hidden mt-0.5 space-y-0.5">
          {#each dayTasks.slice(0, 3) as task}
            <div
              class="text-[10px] leading-tight px-1 rounded truncate {$PRIORITY_COLORS[task.priority] || 'bg-stone-100 text-stone-600'}"
            >
              {task.title}
              {#if task.due_date === dateStr && task.repeat_type === "none"}
                <span class="text-red-500 font-bold"> DDL</span>
              {/if}
            </div>
          {/each}
          {#if dayTasks.length > 3}
            <div class="text-[10px] text-stone-400 px-1">+{dayTasks.length - 3} 更多</div>
          {/if}
        </div>
      </button>
    {/each}
  </div>
</div>
