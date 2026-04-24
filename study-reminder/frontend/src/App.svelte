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
  import { getTasksForMonth, getCrossMonthTasks, exportTasks, getAutostartStatus, toggleAutostart } from "./lib/api";
  import MonthView from "./components/MonthView.svelte";
  import DayView from "./components/DayView.svelte";
  import AllTasksView from "./components/AllTasksView.svelte";
  import TaskEditor from "./components/TaskEditor.svelte";

  let today = $state(new Date().toISOString().slice(0, 10));
  let notificationGranted = $state(false);
  let showSettings = $state(false);
  let autostartEnabled = $state(false);

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

    // 获取开机自启状态
    try {
      autostartEnabled = await getAutostartStatus();
    } catch (e) {
      console.warn("获取开机自启状态失败:", e);
    }
  });

  async function handleExport() {
    try {
      const tasks = await exportTasks();
      const json = JSON.stringify(tasks, null, 2);
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

  async function handleToggleAutostart() {
    try {
      autostartEnabled = await toggleAutostart();
    } catch (e) {
      console.error("切换开机自启失败:", e);
    }
  }

  function getStoreValue<T>(store: { subscribe: (run: (value: T) => void) => () => void }): T {
    let value: T;
    store.subscribe((v: T) => { value = v; })();
    return value!;
  }

  async function loadMonthData() {
    const year = getStoreValue(currentYear);
    const month = getStoreValue(currentMonth);

    try {
      const [monthTasks, crossTasks] = await Promise.all([
        getTasksForMonth(year, month),
        getCrossMonthTasks(year, month),
      ]);
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
    const curYear = getStoreValue(currentYear);
    const curMonth = getStoreValue(currentMonth);
    if (curMonth === 1) {
      currentYear.set(curYear - 1);
      currentMonth.set(12);
    } else {
      currentMonth.set(curMonth - 1);
    }
    loadMonthData();
  }

  function goToNextMonth() {
    const curYear = getStoreValue(currentYear);
    const curMonth = getStoreValue(currentMonth);
    if (curMonth === 12) {
      currentYear.set(curYear + 1);
      currentMonth.set(1);
    } else {
      currentMonth.set(curMonth + 1);
    }
    loadMonthData();
  }

  function openNewTask() {
    editingTask.set(null);
    showEditor.set(true);
  }

  function onEditorClose() {
    showEditor.set(false);
    editingTask.set(null);
    // 根据当前视图重新加载数据
    const view = getStoreValue(currentView);
    if (view === "month") {
      loadMonthData();
    }
    // 日视图和全部任务视图通过 $effect 自动重新加载
  }
</script>

<div class="h-screen w-screen flex flex-col bg-gradient-to-br from-orange-50 to-amber-50">
  <!-- 顶部导航栏 -->
  <header class="flex items-center justify-between px-4 py-2.5 bg-white/80 backdrop-blur-sm border-b border-orange-200/60 shadow-sm">
    <div class="flex items-center gap-2">
      {#if $currentView === "month"}
        <button onclick={goToPrevMonth} class="px-2 py-1 text-orange-600 hover:bg-orange-100 rounded text-lg transition-colors">
          ◀
        </button>
        <span class="text-lg font-semibold text-stone-800">
          {$currentYear}年{$currentMonth}月
        </span>
        <button onclick={goToNextMonth} class="px-2 py-1 text-orange-600 hover:bg-orange-100 rounded text-lg transition-colors">
          ▶
        </button>
      {:else if $currentView === "day"}
        <button onclick={() => switchView("month")} class="px-3 py-1 text-sm bg-orange-100 text-orange-700 rounded-lg hover:bg-orange-200 transition-colors">
          ← 月视图
        </button>
        <span class="text-lg font-semibold text-stone-800 ml-2">{$selectedDate}</span>
      {:else}
        <button onclick={() => switchView("month")} class="px-3 py-1 text-sm bg-orange-100 text-orange-700 rounded-lg hover:bg-orange-200 transition-colors">
          ← 月视图
        </button>
        <span class="text-lg font-semibold text-stone-800 ml-2">全部任务</span>
      {/if}
    </div>

    <div class="flex items-center gap-2">
      <button
        onclick={() => switchView("all")}
        class="px-3 py-1 text-sm rounded-lg {$currentView === 'all' ? 'bg-orange-500 text-white shadow-sm' : 'bg-stone-100 text-stone-600 hover:bg-stone-200'} transition-colors"
      >
        全部任务
      </button>
      <button
        onclick={handleExport}
        class="px-3 py-1 text-sm rounded-lg bg-stone-100 text-stone-600 hover:bg-stone-200 transition-colors"
        title="导出为 JSON"
      >
        📤 导出
      </button>
      <button
        onclick={() => showSettings = !showSettings}
        class="px-3 py-1 text-sm rounded-lg bg-stone-100 text-stone-600 hover:bg-stone-200 transition-colors"
        title="设置"
      >
        ⚙️
      </button>
      <button
        onclick={openNewTask}
        class="px-3 py-1 text-sm bg-gradient-to-r from-orange-500 to-amber-500 text-white rounded-lg hover:from-orange-600 hover:to-amber-600 shadow-sm transition-all"
      >
        + 新建
      </button>
    </div>
  </header>

  <!-- 设置面板 -->
  {#if showSettings}
    <div class="bg-white/90 backdrop-blur-sm border-b border-orange-200/60 px-4 py-3">
      <div class="max-w-2xl mx-auto flex items-center gap-6">
        <h3 class="text-sm font-semibold text-stone-700">⚙️ 设置</h3>

        <!-- 开机自启 -->
        <label class="flex items-center gap-2 cursor-pointer">
          <button
            onclick={handleToggleAutostart}
            class="relative w-10 h-5 rounded-full transition-colors {autostartEnabled ? 'bg-orange-500' : 'bg-stone-300'}"
          >
            <span
              class="absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-transform {autostartEnabled ? 'translate-x-5' : 'translate-x-0.5'}"
            ></span>
          </button>
          <span class="text-sm text-stone-600">开机自启</span>
        </label>

        <!-- 通知状态 -->
        <div class="flex items-center gap-2">
          <span class="text-sm text-stone-600">通知：</span>
          <span class="text-xs px-2 py-0.5 rounded {notificationGranted ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'}">
            {notificationGranted ? '已开启' : '未授权'}
          </span>
        </div>

        <!-- 版本信息 -->
        <span class="text-xs text-stone-400 ml-auto">v0.1.0</span>
      </div>
    </div>
  {/if}

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
