<script lang="ts">
  import { onMount } from "svelte";
  import {
    currentView,
    selectedDate,
    currentYear,
    currentMonth,
    showEditor,
    editingTask,
  } from "./lib/store";
  import type { ViewType, Task } from "./lib/types";
  import { getTasksForMonth, getCrossMonthTasks, exportTasks } from "./lib/api";
  import MonthView from "./components/MonthView.svelte";
  import DayView from "./components/DayView.svelte";
  import AllTasksView from "./components/AllTasksView.svelte";
  import TaskEditor from "./components/TaskEditor.svelte";

  let today = $state(new Date().toISOString().slice(0, 10));
  let notificationGranted = $state(false);

  onMount(async () => {
    // 初始化选中日期为今天
    selectedDate.set(today);
    loadMonthData();

    // 请求通知权限
    try {
      const { isPermissionGranted, requestPermission } = await import("@tauri-apps/plugin-notification");
      let granted = await isPermissionGranted();
      if (!granted) {
        const permission = await requestPermission();
        granted = permission === "granted";
      }
      notificationGranted = granted;
      console.log(`通知权限: ${granted ? "已授权" : "未授权"}`);
    } catch (e) {
      console.warn("通知权限请求失败（开发环境正常）:", e);
    }
  });

  async function handleExport() {
    try {
      const tasks = await exportTasks();
      const json = JSON.stringify(tasks, null, 2);
      // 使用 Blob 下载
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `study-reminder-backup-${today}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error("导出失败:", e);
    }
  }

  async function loadMonthData() {
    let year: number, month: number;
    currentYear.subscribe((v) => (year = v))();
    currentMonth.subscribe((v) => (month = v))();

    try {
      const [monthTasks, crossTasks] = await Promise.all([
        getTasksForMonth(year, month),
        getCrossMonthTasks(year, month),
      ]);
      // 通过 store 更新
      const { tasks, crossMonthTasks } = await import("./lib/store");
      tasks.set(monthTasks);
      crossMonthTasks.set(crossTasks);
    } catch (e) {
      console.error("加载月份数据失败:", e);
    }
  }

  function switchView(view: ViewType) {
    currentView.set(view);
  }

  function goToPrevMonth() {
    currentYear.update((y) => {
      let m: number;
      currentMonth.update((v) => {
        m = v;
        return v;
      })();
      if (m === 1) {
        currentMonth.set(12);
        return y - 1;
      } else {
        currentMonth.set(m - 1);
        return y;
      }
    });
    loadMonthData();
  }

  function goToNextMonth() {
    currentYear.update((y) => {
      let m: number;
      currentMonth.update((v) => {
        m = v;
        return v;
      })();
      if (m === 12) {
        currentMonth.set(1);
        return y + 1;
      } else {
        currentMonth.set(m + 1);
        return y;
      }
    });
    loadMonthData();
  }

  function openNewTask() {
    editingTask.set(null);
    showEditor.set(true);
  }

  function onEditorClose() {
    showEditor.set(false);
    editingTask.set(null);
    loadMonthData();
  }
</script>

<div class="h-screen w-screen flex flex-col bg-orange-50">
  <!-- 顶部导航栏 -->
  <header class="flex items-center justify-between px-4 py-2 bg-white border-b border-orange-200 shadow-sm">
    <div class="flex items-center gap-2">
      {#if $currentView === "month"}
        <button onclick={goToPrevMonth} class="px-2 py-1 text-orange-600 hover:bg-orange-100 rounded text-lg">
          ◀
        </button>
        <span class="text-lg font-semibold text-stone-800">
          {$currentYear}年{$currentMonth}月
        </span>
        <button onclick={goToNextMonth} class="px-2 py-1 text-orange-600 hover:bg-orange-100 rounded text-lg">
          ▶
        </button>
      {:else if $currentView === "day"}
        <button onclick={() => switchView("month")} class="px-3 py-1 text-sm bg-orange-100 text-orange-700 rounded hover:bg-orange-200">
          ← 月视图
        </button>
        <span class="text-lg font-semibold text-stone-800 ml-2">{$selectedDate}</span>
      {:else}
        <button onclick={() => switchView("month")} class="px-3 py-1 text-sm bg-orange-100 text-orange-700 rounded hover:bg-orange-200">
          ← 月视图
        </button>
        <span class="text-lg font-semibold text-stone-800 ml-2">全部任务</span>
      {/if}
    </div>

    <div class="flex items-center gap-2">
      <button
        onclick={() => switchView("all")}
        class="px-3 py-1 text-sm rounded {$currentView === 'all' ? 'bg-orange-500 text-white' : 'bg-stone-100 text-stone-600 hover:bg-stone-200'}"
      >
        全部任务
      </button>
      <button
        onclick={handleExport}
        class="px-3 py-1 text-sm rounded bg-stone-100 text-stone-600 hover:bg-stone-200"
        title="导出为 JSON"
      >
        📤 导出
      </button>
      <button
        onclick={openNewTask}
        class="px-3 py-1 text-sm bg-orange-500 text-white rounded hover:bg-orange-600"
      >
        + 新建
      </button>
    </div>
  </header>

  <!-- 主内容区 -->
  <main class="flex-1 overflow-hidden">
    {#if $currentView === "month"}
      <MonthView onselect={(date: string) => { selectedDate.set(date); switchView("day"); }} />
    {:else if $currentView === "day"}
      <DayView />
    {:else if $currentView === "all"}
      <AllTasksView />
    {/if}
  </main>
</div>

<!-- 新建/编辑弹窗 -->
{#if $showEditor}
  <TaskEditor onclose={onEditorClose} />
{/if}
