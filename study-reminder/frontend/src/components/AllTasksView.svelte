m<script lang="ts">
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
</script>

<div class="h-full flex flex-col p-4">
  <!-- 排序切换 -->
  <div class="flex items-center gap-2 mb-3">
    <span class="text-xs text-stone-500">排序：</span>
    <button
      onclick={() => changeSort("priority")}
      class="px-2 py-1 text-xs rounded {sortBy === 'priority' ? 'bg-orange-500 text-white' : 'bg-stone-100 text-stone-600'}"
    >
      优先级
    </button>
    <button
      onclick={() => changeSort("due_date")}
      class="px-2 py-1 text-xs rounded {sortBy === 'due_date' ? 'bg-orange-500 text-white' : 'bg-stone-100 text-stone-600'}"
    >
      截止日期
    </button>
  </div>

  <!-- 任务列表 -->
  <div class="flex-1 overflow-y-auto space-y-2">
    {#each allTasks as task}
      <div
        class="flex items-center gap-2 px-3 py-2.5 rounded border {PRIORITY_COLORS[task.priority]} cursor-pointer hover:shadow-sm transition-shadow"
        onclick={() => handleEdit(task)}
      >
        <button onclick|stopPropagation={() => handleToggle(task.id)} class="flex-shrink-0">
          <span class="w-5 h-5 rounded border-2 border-stone-300 flex items-center justify-center text-xs">
            {task.completed ? "✓" : ""}
          </span>
        </button>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium">{task.title}</span>
            {#if task.category}
              <span class="text-xs text-stone-400">({task.category})</span>
            {/if}
          </div>
          <div class="flex items-center gap-2 mt-0.5">
            {#if task.due_date}
              <span class="text-xs {isOverdue(task.due_date) ? 'text-red-600 font-bold' : 'text-stone-400'}">
                截止: {task.due_date}
              </span>
            {/if}
            {#if task.has_time_slot && task.time_start}
              <span class="text-xs text-stone-400">{task.time_start}{task.time_end ? `-${task.time_end}` : ''}</span>
            {/if}
            {#if task.repeat_type !== "none"}
              <span class="text-xs text-blue-500">🔄 {task.repeat_type}</span>
            {/if}
          </div>
        </div>
        <div class="flex items-center gap-2">
          {#if getUrgency(task)}
            <span class="text-xs px-1.5 py-0.5 rounded bg-red-100 text-red-600 font-bold">
              {getUrgency(task)}
            </span>
          {/if}
          <span class="text-xs px-1.5 py-0.5 rounded bg-white/50">{PRIORITY_LABELS[task.priority]}</span>
          <button onclick|stopPropagation={() => handleDelete(task.id)} class="text-stone-400 hover:text-red-500 text-xs">
            ✕
          </button>
        </div>
      </div>
    {/each}

    {#if allTasks.length === 0}
      <div class="flex-1 flex items-center justify-center text-stone-400 pt-20">
        <div class="text-center">
          <div class="text-4xl mb-2">🎉</div>
          <div class="text-sm">所有任务已完成！</div>
        </div>
      </div>
    {/if}
  </div>
</div>
