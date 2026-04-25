<script lang="ts">
  import { onMount } from "svelte";
  import {
    currentView,
    selectedDate,
    currentYear,
    currentMonth,
    showEditor,
    editingTask,
    tasks,
    crossMonthTasks,
  } from "./lib/store";
  import type { ViewType, Task } from "./lib/types";
  import { getTasksForMonth, getCrossMonthTasks, getTasksForDate, exportTasks, getAutostartStatus, toggleAutostart } from "./lib/api";
  import MonthView from "./components/MonthView.svelte";
  import DayView from "./components/DayView.svelte";
  import AllTasksView from "./components/AllTasksView.svelte";
  import TaskEditor from "./components/TaskEditor.svelte";

  let today = $state(new Date().toISOString().slice(0, 10));
  let notificationGranted = $state(false);
  let showSettings = $state(false);
  let autostartEnabled = $state(false);

  // 悬浮窗相关
  let isFloating = $state(false);
  let floatingTasks = $state<Task[]>([]);

  onMount(async () => {
    // 检测是否是悬浮窗模式
    const isFloatingUrl = window.location.search.includes("floating=true");
    const isFloatingName = window.name === "floating";
    let isFloatingLabel = false;
    try {
      const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const currentWindow = getCurrentWebviewWindow();
      isFloatingLabel = currentWindow.label === "floating";
    } catch (e) {
      // 忽略
    }
    isFloating = isFloatingUrl || isFloatingName || isFloatingLabel;

    if (isFloating) {
      // 悬浮窗模式：只显示图标，点击打开主窗口
      return;
    }

    // 主窗口模式 - 每次显示都重置为日视图（今日任务）
    selectedDate.set(today);
    currentView.set("day");

    // 监听窗口可见性变化，每次显示时重置为日视图
    try {
      const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const mainWindow = getCurrentWebviewWindow();
      
      // 监听窗口显示事件
      mainWindow.onFocusChanged(({ payload: focused }) => {
        if (focused) {
          selectedDate.set(new Date().toISOString().slice(0, 10));
          currentView.set("day");
        }
      });
    } catch (e) {
      console.warn("窗口事件监听失败:", e);
    }

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
      const exportedTasks = await exportTasks();
      const json = JSON.stringify(exportedTasks, null, 2);
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

  let _year = $state(new Date().getFullYear());
  let _month = $state(new Date().getMonth() + 1);

  // 同步 store 到本地 state
  currentYear.subscribe(v => _year = v);
  currentMonth.subscribe(v => _month = v);

  async function loadMonthData() {
    try {
      console.log("loadMonthData: year=", _year, "month=", _month);
      const [monthTasks, crossTasks] = await Promise.all([
        getTasksForMonth(_year, _month),
        getCrossMonthTasks(_year, _month),
      ]);
      console.log("loadMonthData: tasks=", monthTasks.length, "cross=", crossTasks.length);
      tasks.set(monthTasks);
      crossMonthTasks.set(crossTasks);
    } catch (e) {
      console.error("加载月份数据失败:", e);
    }
  }

  function switchView(view: ViewType) {
    currentView.set(view);
    if (view === "month") {
      loadMonthData();
    }
  }

  function goToPrevMonth() {
    if (_month === 1) {
      currentYear.set(_year - 1);
      currentMonth.set(12);
    } else {
      currentMonth.set(_month - 1);
    }
    loadMonthData();
  }

  function goToNextMonth() {
    if (_month === 12) {
      currentYear.set(_year + 1);
      currentMonth.set(1);
    } else {
      currentMonth.set(_month + 1);
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
    let view: ViewType;
    currentView.subscribe(v => view = v)();
    if (view! === "month") {
      loadMonthData();
    }
  }

  // 悬浮窗点击打开主窗口
  async function toggleMainWindow() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("toggle_main_window");
    } catch (e) {
      console.error("切换主窗口失败:", e);
    }
  }

  function getPriorityColor(priority: number): string {
    switch (priority) {
      case 1: return "bg-red-100 text-red-700";
      case 2: return "bg-yellow-100 text-yellow-700";
      case 3: return "bg-gray-100 text-gray-500";
      default: return "bg-stone-100 text-stone-500";
    }
  }

  function formatDateDisplay(dateStr: string): string {
    const d = new Date(dateStr + "T00:00:00");
    const weekdays = ["日", "一", "二", "三", "四", "五", "六"];
    return `${dateStr} 周${weekdays[d.getDay()]}`;
  }
</script>

<!-- 悬浮窗模式：仅显示图标，点击打开主窗口 -->
{#if isFloating}
  <div class="floating-wrapper" onclick={toggleMainWindow} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && toggleMainWindow()}>
    <div class="floating-icon">
      <span class="icon-text">📋</span>
    </div>
  </div>
{:else}
  <!-- 主窗口模式 -->
  <div class="h-screen w-screen flex flex-col bg-gradient-to-br from-orange-50 via-amber-50 to-stone-50">
    <!-- 顶部导航栏 -->
    <header class="flex items-center justify-between px-5 py-3 bg-white/90 backdrop-blur-md border-b border-orange-200/40 shadow-sm">
      <div class="flex items-center gap-3">
        {#if $currentView === "month"}
          <div class="flex items-center gap-1 bg-stone-100 rounded-lg p-0.5">
            <button onclick={goToPrevMonth} class="px-2 py-1 text-orange-600 hover:bg-white rounded-md text-sm transition-colors btn-press">
              ◀
            </button>
            <span class="px-3 text-sm font-semibold text-stone-800 min-w-[100px] text-center">
              {$currentYear}年{$currentMonth}月
            </span>
            <button onclick={goToNextMonth} class="px-2 py-1 text-orange-600 hover:bg-white rounded-md text-sm transition-colors btn-press">
              ▶
            </button>
          </div>
        {:else if $currentView === "day"}
          <button onclick={() => switchView("month")} class="flex items-center gap-1 px-3 py-1.5 text-sm bg-orange-50 text-orange-700 rounded-lg hover:bg-orange-100 transition-colors btn-press">
            <span>◀</span>
            <span>月视图</span>
          </button>
          <div class="flex items-center gap-2 ml-1">
            <span class="text-base font-semibold text-stone-800">{$selectedDate}</span>
            <span class="text-xs text-stone-400 bg-stone-100 px-2 py-0.5 rounded-full">
              {formatDateDisplay($selectedDate).split(" ")[1]}
            </span>
          </div>
        {:else}
          <button onclick={() => switchView("month")} class="flex items-center gap-1 px-3 py-1.5 text-sm bg-orange-50 text-orange-700 rounded-lg hover:bg-orange-100 transition-colors btn-press">
            <span>◀</span>
            <span>月视图</span>
          </button>
          <span class="text-base font-semibold text-stone-800 ml-1">全部任务</span>
        {/if}
      </div>

      <div class="flex items-center gap-1.5">
        <button
          onclick={() => switchView("all")}
          class="px-3 py-1.5 text-sm rounded-lg transition-all btn-press {$currentView === 'all' ? 'bg-orange-500 text-white shadow-sm shadow-orange-200' : 'text-stone-500 hover:bg-stone-100'}"
        >
          全部任务
        </button>
        <button
          onclick={handleExport}
          class="px-3 py-1.5 text-sm rounded-lg text-stone-500 hover:bg-stone-100 transition-colors btn-press"
          title="导出为 JSON"
        >
          📤
        </button>
        <button
          onclick={() => showSettings = !showSettings}
          class="px-3 py-1.5 text-sm rounded-lg text-stone-500 hover:bg-stone-100 transition-colors btn-press"
          title="设置"
        >
          ⚙️
        </button>
        <div class="w-px h-5 bg-stone-200 mx-1"></div>
        <button
          onclick={openNewTask}
          class="flex items-center gap-1 px-4 py-1.5 text-sm bg-gradient-to-r from-orange-500 to-amber-500 text-white rounded-lg hover:from-orange-600 hover:to-amber-600 shadow-sm shadow-orange-200 transition-all btn-press"
        >
          <span class="text-base leading-none">+</span>
          <span>新建</span>
        </button>
      </div>
    </header>

    <!-- 设置面板 -->
    {#if showSettings}
      <div class="bg-white/95 backdrop-blur-md border-b border-orange-200/40 px-5 py-3 animate-slide-up">
        <div class="max-w-2xl mx-auto flex items-center gap-6">
          <h3 class="text-sm font-semibold text-stone-700 flex items-center gap-1.5">
            <span>⚙️</span>
            <span>设置</span>
          </h3>

          <!-- 开机自启 -->
          <label class="flex items-center gap-2 cursor-pointer group">
            <button
              onclick={handleToggleAutostart}
              class="relative w-10 h-5 rounded-full transition-colors {autostartEnabled ? 'bg-orange-500' : 'bg-stone-300'} group-hover:shadow-sm"
            >
              <span
                class="absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-all {autostartEnabled ? 'translate-x-5' : 'translate-x-0.5'}"
              ></span>
            </button>
            <span class="text-sm text-stone-600">开机自启</span>
          </label>

          <!-- 通知状态 -->
          <div class="flex items-center gap-2">
            <span class="text-sm text-stone-600">通知：</span>
            <span class="text-xs px-2 py-0.5 rounded-full {notificationGranted ? 'bg-green-50 text-green-700 border border-green-200' : 'bg-red-50 text-red-700 border border-red-200'}">
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
{/if}

<!-- 新建/编辑弹窗 -->
{#if $showEditor}
  <TaskEditor onclose={onEditorClose} />
{/if}

<style>
  /* 悬浮窗样式 */
  :global(.floating-wrapper) {
    position: relative;
    width: 50px;
    height: 50px;
    cursor: pointer;
  }

  :global(.floating-icon) {
    width: 44px;
    height: 44px;
    background: linear-gradient(135deg, #f97316, #ea580c);
    border-radius: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 3px 10px rgba(0, 0, 0, 0.25);
    transition: transform 0.15s ease, box-shadow 0.15s ease;
    position: absolute;
    top: 3px;
    left: 3px;
    z-index: 10;
  }

  :global(.floating-icon:hover) {
    transform: scale(1.1);
    box-shadow: 0 4px 14px rgba(249, 115, 22, 0.4);
  }

  :global(.floating-icon:active) {
    transform: scale(0.95);
  }

  :global(.icon-text) {
    color: white;
    font-size: 20px;
    font-weight: bold;
  }
</style>
