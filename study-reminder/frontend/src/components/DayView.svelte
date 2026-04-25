<script lang="ts">
  import { onMount } from "svelte";
  import { selectedDate, showEditor, editingTask, currentView } from "../lib/store";
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

  // 监听编辑弹窗关闭后重新加载
  $effect(() => {
    const editorOpen = $showEditor;
    if (!editorOpen && $selectedDate) {
      setTimeout(() => loadDayData(), 100);
    }
  });

  async function loadDayData() {
    const date = $selectedDate;
    console.log("DayView.loadDayData: date =", date);
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

  function getPriorityDot(priority: number): string {
    switch (priority) {
      case 1: return "bg-red-500";
      case 2: return "bg-yellow-500";
      case 3: return "bg-gray-400";
      default: return "bg-stone-400";
    }
  }

  function getPriorityBorder(priority: number): string {
    switch (priority) {
      case 1: return "border-l-red-400";
      case 2: return "border-l-yellow-400";
      case 3: return "border-l-gray-400";
      default: return "border-l-stone-400";
    }
  }
</script>

<div class="h-full flex flex-col p-5 overflow-y-auto">
  <!-- 全天任务区 -->
  {#if allDayTasks.length > 0}
    <div class="mb-5">
      <div class="flex items-center gap-2 mb-3">
        <span class="text-sm font-semibold text-stone-500">📋 全天任务</span>
        <span class="text-xs text-stone-400 bg-stone-100 px-2 py-0.5 rounded-full">{allDayTasks.length}项</span>
      </div>
      <div class="space-y-2">
        {#each allDayTasks as task}
          <div
            class="flex items-center gap-3 px-4 py-3 rounded-xl border {PRIORITY_COLORS[task.priority]} cursor-pointer card-hover shadow-sm"
            onclick={() => handleEdit(task)}
          >
            <button onclick={(e) => { e.stopPropagation(); handleToggle(task.id); }} class="flex-shrink-0">
              <span class="w-5 h-5 rounded-md border-2 {task.completed ? 'bg-orange-500 border-orange-500 text-white' : 'border-stone-300'} flex items-center justify-center text-xs transition-colors">
                {task.completed ? "✓" : ""}
              </span>
            </button>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium {task.completed ? 'line-through text-stone-400' : ''}">{task.title}</span>
                {#if task.due_date}
                  <span class="text-xs px-1.5 py-0.5 rounded-full {isOverdue(task.due_date) ? 'bg-red-50 text-red-600 border border-red-200 font-medium' : 'bg-stone-100 text-stone-500'}">
                    {isOverdue(task.due_date) ? "逾期!" : `截止 ${task.due_date}`}
                  </span>
                {/if}
              </div>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-xs px-2 py-0.5 rounded-full bg-white/70 text-stone-500 border border-stone-200">{PRIORITY_LABELS[task.priority]}</span>
              <button onclick={(e) => { e.stopPropagation(); handleDelete(task.id); }} class="w-6 h-6 flex items-center justify-center text-stone-400 hover:text-red-500 hover:bg-red-50 rounded-md text-xs transition-colors">
                ✕
              </button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- 时间安排区 -->
  {#if timeSlots.length > 0}
    <div class="mb-5">
      <div class="flex items-center gap-2 mb-3">
        <span class="text-sm font-semibold text-stone-500">⏰ 时间安排</span>
        <span class="text-xs text-stone-400 bg-stone-100 px-2 py-0.5 rounded-full">{timeSlots.length}项</span>
      </div>
      <div class="space-y-1 relative">
        <!-- 时间轴竖线 -->
        <div class="absolute left-[60px] top-0 bottom-0 w-px bg-stone-200"></div>
        {#each timeSlots as task}
          <div
            class="flex items-stretch gap-3 cursor-pointer group"
            onclick={() => handleEdit(task)}
          >
            <div class="w-14 flex-shrink-0 text-right text-xs text-stone-400 pt-3 font-mono">
              {task.time_start || ""}
            </div>
            <div
              class="flex-1 px-4 py-3 rounded-xl border-l-4 {getPriorityBorder(task.priority)} bg-white shadow-sm card-hover ml-3"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full {getPriorityDot(task.priority)}"></span>
                  <span class="text-sm font-medium">{task.title}</span>
                </div>
                <span class="text-xs text-stone-400 bg-stone-50 px-2 py-0.5 rounded-full">{formatTimeRange(task)}</span>
              </div>
              {#if task.note}
                <div class="text-xs text-stone-400 mt-1 ml-4 truncate">{task.note}</div>
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
      <div class="flex items-center gap-2 mb-3">
        <span class="text-sm font-semibold text-stone-500">📝 未安排时间</span>
        <span class="text-xs text-stone-400 bg-stone-100 px-2 py-0.5 rounded-full">{noTimeTasks.length}项</span>
      </div>
      <div class="space-y-2">
        {#each noTimeTasks as task}
          <div
            class="flex items-center gap-3 px-4 py-3 rounded-xl border {PRIORITY_COLORS[task.priority]} cursor-pointer card-hover shadow-sm"
            onclick={() => handleEdit(task)}
          >
            <button onclick={(e) => { e.stopPropagation(); handleToggle(task.id); }} class="flex-shrink-0">
              <span class="w-5 h-5 rounded-md border-2 {task.completed ? 'bg-orange-500 border-orange-500 text-white' : 'border-stone-300'} flex items-center justify-center text-xs transition-colors">
                {task.completed ? "✓" : ""}
              </span>
            </button>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium {task.completed ? 'line-through text-stone-400' : ''}">{task.title}</span>
                {#if task.category}
                  <span class="text-xs text-stone-400 bg-stone-100 px-1.5 py-0.5 rounded">{task.category}</span>
                {/if}
              </div>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-xs px-2 py-0.5 rounded-full bg-white/70 text-stone-500 border border-stone-200">{PRIORITY_LABELS[task.priority]}</span>
              <button onclick={(e) => { e.stopPropagation(); handleDelete(task.id); }} class="w-6 h-6 flex items-center justify-center text-stone-400 hover:text-red-500 hover:bg-red-50 rounded-md text-xs transition-colors">
                ✕
              </button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- 空状态 -->
  {#if dayTasks.length === 0}
    <div class="flex-1 flex items-center justify-center">
      <div class="text-center animate-fade-in">
        <div class="text-5xl mb-3">📭</div>
        <div class="text-sm text-stone-400 mb-1">今天没有任务</div>
        <div class="text-xs text-stone-300">点击右上角「+ 新建」添加任务</div>
      </div>
    </div>
  {/if}
</div>
