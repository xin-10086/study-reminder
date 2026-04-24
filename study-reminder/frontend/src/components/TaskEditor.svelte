<script lang="ts">
  import { editingTask } from "../lib/store";
  import { createTask, updateTask } from "../lib/api";
  import type { CreateTaskDto, UpdateTaskDto } from "../lib/types";

  let { onclose }: { onclose: () => void } = $props();

  // 编辑模式还是新建模式
  let isEdit = $derived($editingTask !== null);

  // 表单数据
  let title = $state("");
  let priority = $state(2);
  let category = $state("学习");
  let hasDueDate = $state(false);
  let dueDate = $state("");
  let remindDate = $state("");
  let hasTimeSlot = $state(false);
  let timeStart = $state("");
  let timeEnd = $state("");
  let hasRepeat = $state(false);
  let repeatType = $state("weekly");
  let repeatDays = $state<string[]>([]);
  let repeatEnd = $state("");
  let note = $state("");
  let saving = $state(false);

  const CATEGORIES = ["学习", "学校", "生活", "工作", "其他"];
  const WEEKDAYS = [
    { label: "一", value: "1" },
    { label: "二", value: "2" },
    { label: "三", value: "3" },
    { label: "四", value: "4" },
    { label: "五", value: "5" },
    { label: "六", value: "6" },
    { label: "日", value: "7" },
  ];

  // 如果是编辑模式，填充表单
  $effect(() => {
    const task = $editingTask;
    if (task) {
      title = task.title;
      priority = task.priority;
      category = task.category || "学习";
      hasDueDate = !!task.due_date;
      dueDate = task.due_date || "";
      remindDate = task.remind_date || "";
      hasTimeSlot = task.has_time_slot;
      timeStart = task.time_start || "";
      timeEnd = task.time_end || "";
      hasRepeat = task.repeat_type !== "none";
      repeatType = task.repeat_type === "none" ? "weekly" : task.repeat_type;
      repeatDays = task.repeat_days ? task.repeat_days.split(",") : [];
      repeatEnd = task.repeat_end || "";
      note = task.note || "";
    }
  });

  function toggleDay(day: string) {
    if (repeatDays.includes(day)) {
      repeatDays = repeatDays.filter((d) => d !== day);
    } else {
      repeatDays = [...repeatDays, day];
    }
  }

  async function handleSave() {
    if (!title.trim()) {
      alert("请输入任务标题");
      return;
    }

    saving = true;

    const baseDto: CreateTaskDto = {
      title: title.trim(),
      priority,
      category: category || null,
      due_date: hasDueDate && dueDate ? dueDate : null,
      remind_date: remindDate || (hasDueDate && dueDate ? dueDate : null),
      has_time_slot: hasTimeSlot,
      time_start: hasTimeSlot && timeStart ? timeStart : null,
      time_end: hasTimeSlot && timeEnd ? timeEnd : null,
      repeat_type: hasRepeat ? repeatType : "none",
      repeat_days: hasRepeat && repeatType === "weekly" && repeatDays.length > 0 ? repeatDays.join(",") : null,
      repeat_end: hasRepeat && repeatEnd ? repeatEnd : null,
      note: note || null,
    };

    try {
      if (isEdit && $editingTask) {
        await updateTask($editingTask.id, baseDto as UpdateTaskDto);
      } else {
        await createTask(baseDto);
      }
      onclose();
    } catch (e) {
      console.error("保存任务失败:", e);
      alert("保存失败，请重试");
    } finally {
      saving = false;
    }
  }
</script>

<!-- 遮罩层 -->
<div
  class="fixed inset-0 bg-black/30 flex items-center justify-center z-50"
  onclick={onclose}
>
  <!-- 弹窗 -->
  <div
    class="bg-white rounded-xl shadow-xl w-[480px] max-h-[90vh] overflow-y-auto animate-fade-in"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- 标题 -->
    <div class="flex items-center justify-between px-5 py-3 border-b border-stone-200">
      <h2 class="text-base font-semibold text-stone-800">
        {isEdit ? "编辑任务" : "新建任务"}
      </h2>
      <button onclick={onclose} class="text-stone-400 hover:text-stone-600 text-lg">✕</button>
    </div>

    <!-- 表单 -->
    <div class="p-5 space-y-4">
      <!-- 标题 -->
      <div>
        <label class="block text-xs font-medium text-stone-500 mb-1">标题</label>
        <input
          bind:value={title}
          type="text"
          placeholder="输入任务标题"
          class="w-full px-3 py-2 border border-stone-300 rounded text-sm focus:outline-none focus:ring-2 focus:ring-orange-400"
        />
      </div>

      <!-- 优先级 + 类别 -->
      <div class="flex gap-3">
        <div class="flex-1">
          <label class="block text-xs font-medium text-stone-500 mb-1">优先级</label>
          <select bind:value={priority} class="w-full px-3 py-2 border border-stone-300 rounded text-sm">
            <option value={1}>高</option>
            <option value={2}>中</option>
            <option value={3}>低</option>
          </select>
        </div>
        <div class="flex-1">
          <label class="block text-xs font-medium text-stone-500 mb-1">类别</label>
          <select bind:value={category} class="w-full px-3 py-2 border border-stone-300 rounded text-sm">
            {#each CATEGORIES as cat}
              <option value={cat}>{cat}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- 截止日期 -->
      <div>
        <label class="flex items-center gap-2 text-sm">
          <input type="checkbox" bind:checked={hasDueDate} class="rounded" />
          <span class="text-xs font-medium text-stone-500">有截止日期</span>
        </label>
        {#if hasDueDate}
          <div class="flex gap-3 mt-2">
            <div class="flex-1">
              <label class="block text-xs text-stone-400 mb-0.5">截止日期</label>
              <input bind:value={dueDate} type="date" class="w-full px-3 py-2 border border-stone-300 rounded text-sm" />
            </div>
            <div class="flex-1">
              <label class="block text-xs text-stone-400 mb-0.5">提醒日期</label>
              <input bind:value={remindDate} type="date" class="w-full px-3 py-2 border border-stone-300 rounded text-sm" />
            </div>
          </div>
        {/if}
      </div>

      <!-- 时间段 -->
      <div>
        <label class="flex items-center gap-2 text-sm">
          <input type="checkbox" bind:checked={hasTimeSlot} class="rounded" />
          <span class="text-xs font-medium text-stone-500">安排时间段</span>
        </label>
        {#if hasTimeSlot}
          <div class="flex gap-3 mt-2">
            <div class="flex-1">
              <label class="block text-xs text-stone-400 mb-0.5">开始</label>
              <input bind:value={timeStart} type="time" class="w-full px-3 py-2 border border-stone-300 rounded text-sm" />
            </div>
            <div class="flex-1">
              <label class="block text-xs text-stone-400 mb-0.5">结束</label>
              <input bind:value={timeEnd} type="time" class="w-full px-3 py-2 border border-stone-300 rounded text-sm" />
            </div>
          </div>
        {/if}
      </div>

      <!-- 周期性 -->
      <div>
        <label class="flex items-center gap-2 text-sm">
          <input type="checkbox" bind:checked={hasRepeat} class="rounded" />
          <span class="text-xs font-medium text-stone-500">周期性任务</span>
        </label>
        {#if hasRepeat}
          <div class="mt-2 space-y-2">
            <select bind:value={repeatType} class="w-full px-3 py-2 border border-stone-300 rounded text-sm">
              <option value="daily">每天</option>
              <option value="weekly">每周（选择星期）</option>
              <option value="weekdays">工作日（周一至周五）</option>
            </select>
            {#if repeatType === "weekly"}
              <div class="flex gap-1">
                {#each WEEKDAYS as wd}
                  <button
                    onclick={() => toggleDay(wd.value)}
                    class="w-8 h-8 text-xs rounded {repeatDays.includes(wd.value) ? 'bg-orange-500 text-white' : 'bg-stone-100 text-stone-600'} hover:bg-orange-200"
                  >
                    {wd.label}
                  </button>
                {/each}
              </div>
            {/if}
            <div>
              <label class="block text-xs text-stone-400 mb-0.5">结束日期（可选）</label>
              <input bind:value={repeatEnd} type="date" class="w-full px-3 py-2 border border-stone-300 rounded text-sm" />
            </div>
          </div>
        {/if}
      </div>

      <!-- 备注 -->
      <div>
        <label class="block text-xs font-medium text-stone-500 mb-1">备注（可选）</label>
        <textarea
          bind:value={note}
          rows={2}
          placeholder="添加备注..."
          class="w-full px-3 py-2 border border-stone-300 rounded text-sm resize-none"
        ></textarea>
      </div>
    </div>

    <!-- 按钮 -->
    <div class="flex justify-end gap-2 px-5 py-3 border-t border-stone-200 bg-stone-50 rounded-b-xl">
      <button onclick={onclose} class="px-4 py-2 text-sm text-stone-600 hover:bg-stone-100 rounded">
        取消
      </button>
      <button
        onclick={handleSave}
        disabled={saving}
        class="px-4 py-2 text-sm bg-orange-500 text-white rounded hover:bg-orange-600 disabled:opacity-50"
      >
        {saving ? "保存中..." : "保存"}
      </button>
    </div>
  </div>
</div>
