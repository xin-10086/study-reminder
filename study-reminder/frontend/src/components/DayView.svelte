<script lang="ts">
  import { onMount } from "svelte";
  import { selectedDate, showEditor, editingTask } from "../lib/store";
  import { getTasksForDate, toggleComplete, deleteTask } from "../lib/api";
  import { PRIORITY_COLORS, PRIORITY_LABELS } from "../lib/types";
  import type { Task } from "../lib/types";

  let dayTasks = $state<Task[]>([]);
  let timeSlots = $state<Task[]>([]);
  let noTimeTasks = $state<Task[]>([]);
  let allDayTasks = $state<Task[]>([]);

  onMount(() => {
    loadDayData();
  });

  // 监听日期变化
  $effect(() => {
    const date = $selectedDate;
    if (date) loadDayData();
  });

  async function loadDayData() {
    const date = $selectedDate;
    if (!date) return;
    try {
      const tasks = await getTasksForDate(date);
      dayTasks = tasks;
      // 分类
      timeSlots = tasks.filter((t) => t.has_time_slot).sort((a, b) => {
        if (!a.time_start) return 1;
        if (!b.time_start) return -1;
        return a.time_start.localeCompare(b.time_start);
      });
      noTimeTasks = tasks.filter((t) => !t.has_time_slot).sort((a, b) => a.priority - b.priority);
      allDayTasks = tasks.filter((t) => t.due_date === date && !t.has_time_slot);
    } catch (e) {
      console.error("加载日数据失败:", e);
    }
  }

  async function handleToggle(id: number) {
    await toggleComplete(id);
    loadDayData();
  }

  async function handleDelete(id: number) {
    if (confirm("确定删除此任务？")) {
      await deleteTask(id);
      loadDayData();
    }
  }

  function handleEdit(task: Task) {
    editingTask.set(task);
    showEditor.set(true);
  }

  function isOverdue(dateStr: string | null): boolean {
    if (!dateStr) return false;
    return dateStr < new Date().toISOString().slice(0, 10);
  }

  function formatTimeRange(task: Task): string {
    if (task.time_start && task.time_end) {
      return `${task.time_start} - ${task.time_end}`;
    }
    if (task.time_start) return task.time_start;
    return "";
  }
</script>

<div class="h-full flex flex-col p-4 overflow-y-auto">
  <!-- 全天任务区 -->
  {#if allDayTasks.length > 0}
    <div class="mb-4">
      <h3 class="text-sm font-semibold text-stone-500 mb-2">📋 全天任务</h3>
      <div class="space-y-2">
        {#each allDayTasks as task}
          <div
            class="flex items-center gap-2 px-3 py-2 rounded border {PRIORITY_COLORS[task.priority]} cursor-pointer hover:shadow-sm transition-shadow"
            onclick={() => handleEdit(task)}
          >
            <button onclick={(e) => { e.stopPropagation(); handleToggle(task.id); }} class="flex-shrink-0">
              <span class="w-5 h-5 rounded border-2 border-stone-300 flex items-center justify-center text-xs">
                {task.completed ? "✓" : ""}
              </span>
            </button>
            <div class="flex-1 min-w-0">
              <span class="text-sm font-medium">{task.title}</span>
              {#if task.due_date}
                <span class="text-xs ml-2 {isOverdue(task.due_date) ? 'text-red-600 font-bold' : 'text-stone-400'}">
                  {isOverdue(task.due_date) ? "逾期!" : `截止:${task.due_date}`}
                </span>
              {/if}
            </div>
            <span class="text-xs px-1.5 py-0.5 rounded bg-white/50">{PRIORITY_LABELS[task.priority]}</span>
            <button onclick={(e) => { e.stopPropagation(); handleDelete(task.id); }} class="text-stone-400 hover:text-red-500 text-xs">
              ✕
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- 时间安排区 -->
  {#if timeSlots.length > 0}
    <div class="mb-4">
      <h3 class="text-sm font-semibold text-stone-500 mb-2">⏰ 时间安排</h3>
      <div class="space-y-1">
        {#each timeSlots as task}
          <div
            class="flex items-stretch gap-2 cursor-pointer hover:opacity-80"
            onclick={() => handleEdit(task)}
          >
            <div class="w-14 flex-shrink-0 text-right text-xs text-stone-400 pt-1">
              {task.time_start || ""}
            </div>
            <div
              class="flex-1 px-3 py-2 rounded border-l-4 {task.priority === 1 ? 'border-l-red-400' : task.priority === 2 ? 'border-l-yellow-400' : 'border-l-gray-400'} bg-white shadow-sm"
            >
              <div class="flex items-center justify-between">
                <span class="text-sm font-medium">{task.title}</span>
                <span class="text-xs text-stone-400">{formatTimeRange(task)}</span>
              </div>
              {#if task.note}
                <div class="text-xs text-stone-400 mt-0.5 truncate">{task.note}</div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- 未安排时间区 -->
  {#if noTimeTasks.length > 0}
    <div>
      <h3 class="text-sm font-semibold text-stone-500 mb-2">📝 未安排时间</h3>
      <div class="space-y-2">
        {#each noTimeTasks as task}
          <div
            class="flex items-center gap-2 px-3 py-2 rounded border {PRIORITY_COLORS[task.priority]} cursor-pointer hover:shadow-sm transition-shadow"
            onclick={() => handleEdit(task)}
          >
            <button onclick={(e) => { e.stopPropagation(); handleToggle(task.id); }} class="flex-shrink-0">
              <span class="w-5 h-5 rounded border-2 border-stone-300 flex items-center justify-center text-xs">
                {task.completed ? "✓" : ""}
              </span>
            </button>
            <div class="flex-1 min-w-0">
              <span class="text-sm font-medium">{task.title}</span>
              {#if task.category}
                <span class="text-xs text-stone-400 ml-1">({task.category})</span>
              {/if}
            </div>
            <span class="text-xs px-1.5 py-0.5 rounded bg-white/50">{PRIORITY_LABELS[task.priority]}</span>
            <button onclick={(e) => { e.stopPropagation(); handleDelete(task.id); }} class="text-stone-400 hover:text-red-500 text-xs">
              ✕
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- 空状态 -->
  {#if dayTasks.length === 0}
    <div class="flex-1 flex items-center justify-center text-stone-400">
      <div class="text-center">
        <div class="text-4xl mb-2">📭</div>
        <div class="text-sm">今天没有任务</div>
      </div>
    </div>
  {/if}
</div>
