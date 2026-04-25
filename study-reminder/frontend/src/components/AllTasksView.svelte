<script lang="ts">
  import { onMount } from "svelte";
  import { showEditor, editingTask } from "../lib/store";
  import { getAllTasks, toggleComplete, deleteTask } from "../lib/api";
  import { PRIORITY_COLORS, PRIORITY_LABELS } from "../lib/types";
  import type { Task } from "../lib/types";

  let allTasks = $state<Task[]>([]);
  let sortBy = $state("priority");

  onMount(() => {
    loadAllTasks();
  });

  async function loadAllTasks() {
    try {
      allTasks = await getAllTasks(sortBy);
    } catch (e) {
      console.error("加载全部任务失败:", e);
    }
  }

  async function handleToggle(id: number) {
    await toggleComplete(id);
    loadAllTasks();
  }

  async function handleDelete(id: number) {
    if (confirm("确定删除此任务？")) {
      await deleteTask(id);
      loadAllTasks();
    }
  }

  function handleEdit(task: Task) {
    editingTask.set(task);
    showEditor.set(true);
  }

  function changeSort(sort: string) {
    sortBy = sort;
    loadAllTasks();
  }

  function isOverdue(dateStr: string | null): boolean {
    if (!dateStr) return false;
    return dateStr < new Date().toISOString().slice(0, 10);
  }

  function getUrgency(task: Task): string {
    if (task.due_date && isOverdue(task.due_date)) return "逾期";
    if (task.due_date === new Date().toISOString().slice(0, 10)) return "今天截止";
    return "";
  }

  function getRepeatLabel(repeatType: string): string {
    switch (repeatType) {
      case "daily": return "每天";
      case "weekly": return "每周";
      case "weekdays": return "工作日";
      default: return "";
    }
  }
</script>

<div class="h-full flex flex-col p-5">
  <!-- 排序切换 -->
  <div class="flex items-center gap-2 mb-4">
    <span class="text-xs text-stone-500 font-medium">排序：</span>
    <div class="flex gap-1 bg-stone-100 rounded-lg p-0.5">
      <button
        onclick={() => changeSort("priority")}
        class="px-3 py-1 text-xs rounded-md transition-all {sortBy === 'priority' ? 'bg-white text-orange-700 shadow-sm' : 'text-stone-500 hover:text-stone-700'}"
      >
        优先级
      </button>
      <button
        onclick={() => changeSort("due_date")}
        class="px-3 py-1 text-xs rounded-md transition-all {sortBy === 'due_date' ? 'bg-white text-orange-700 shadow-sm' : 'text-stone-500 hover:text-stone-700'}"
      >
        截止日期
      </button>
    </div>
    <span class="text-xs text-stone-400 ml-auto">{allTasks.length}项未完成</span>
  </div>

  <!-- 任务列表 -->
  <div class="flex-1 overflow-y-auto space-y-2">
    {#each allTasks as task}
      <div
        class="flex items-center gap-3 px-4 py-3 rounded-xl border {PRIORITY_COLORS[task.priority]} cursor-pointer card-hover shadow-sm hover:shadow-md transition-all"
        onclick={() => handleEdit(task)}
      >
        <button onclick={(e) => { e.stopPropagation(); handleToggle(task.id); }} class="flex-shrink-0">
          <span class="w-5 h-5 rounded-md border-2 {task.completed ? 'bg-orange-500 border-orange-500 text-white' : 'border-stone-300'} flex items-center justify-center text-xs transition-colors hover:border-orange-400">
            {task.completed ? "✓" : ""}
          </span>
        </button>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium {task.completed ? 'line-through text-stone-400' : 'text-stone-800'}">{task.title}</span>
            {#if task.category}
              <span class="text-xs text-stone-400 bg-stone-100 px-1.5 py-0.5 rounded">{task.category}</span>
            {/if}
          </div>
          <div class="flex items-center gap-2 mt-1 flex-wrap">
            {#if task.due_date}
              <span class="text-xs {isOverdue(task.due_date) ? 'text-red-600 font-medium' : 'text-stone-400'}">
                📅 截止 {task.due_date}
              </span>
            {/if}
            {#if task.has_time_slot && task.time_start}
              <span class="text-xs text-stone-400 bg-stone-50 px-1.5 py-0.5 rounded border border-stone-100">
                ⏰ {task.time_start}{task.time_end ? `-${task.time_end}` : ''}
              </span>
            {/if}
            {#if task.repeat_type !== "none"}
              <span class="text-xs text-blue-500 bg-blue-50 px-1.5 py-0.5 rounded border border-blue-100">🔄 {getRepeatLabel(task.repeat_type)}</span>
            {/if}
          </div>
        </div>
        <div class="flex items-center gap-2 flex-shrink-0">
          {#if getUrgency(task)}
            <span class="text-xs px-2 py-0.5 rounded-full bg-red-50 text-red-600 border border-red-200 font-medium animate-pulse-slow">
              {getUrgency(task)}
            </span>
          {/if}
          <span class="text-xs px-2 py-0.5 rounded-full bg-white/70 text-stone-500 border border-stone-200">{PRIORITY_LABELS[task.priority]}</span>
          <button onclick={(e) => { e.stopPropagation(); handleDelete(task.id); }} class="w-6 h-6 flex items-center justify-center text-stone-400 hover:text-red-500 hover:bg-red-50 rounded-md text-xs transition-colors">
            ✕
          </button>
        </div>
      </div>
    {/each}

    {#if allTasks.length === 0}
      <div class="flex-1 flex items-center justify-center pt-20">
        <div class="text-center animate-fade-in">
          <div class="text-5xl mb-3">🎉</div>
          <div class="text-sm text-stone-400 mb-1">所有任务已完成！</div>
          <div class="text-xs text-stone-300">点击右上角「+ 新建」添加新任务</div>
        </div>
      </div>
    {/if}
  </div>
</div>
