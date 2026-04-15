<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { X, Trash2 } from "@lucide/svelte";
  import type { Task, NewTaskInput, UpdateTaskInput } from "./types";
  import { COLUMNS } from "./types";

  let {
    task = null,
    onClose,
    onSaved,
  }: {
    task?: Task | null;
    onClose: () => void;
    onSaved: () => void;
  } = $props();

  let title = $state(task?.title ?? "");
  let description = $state(task?.description ?? "");
  let status = $state(task?.status ?? "todo");
  let dueDate = $state(task?.due_date ?? "");
  let tagsStr = $state(task?.tags?.join(", ") ?? "");
  let saving = $state(false);
  let deleting = $state(false);
  let error = $state<string | null>(null);

  const isEditing = $derived(task !== null);

  async function save() {
    if (!title.trim()) {
      error = "Title gak boleh kosong";
      return;
    }
    saving = true;
    error = null;
    try {
      const tags = tagsStr
        .split(",")
        .map((t) => t.trim())
        .filter((t) => t.length > 0);

      if (isEditing && task) {
        const input: UpdateTaskInput = {
          file_path: task.file_path,
          title: title.trim(),
          description: description.trim(),
          status,
          due_date: dueDate || null,
          tags,
        };
        await invoke("update_task", { input });
      } else {
        const input: NewTaskInput = {
          title: title.trim(),
          description: description.trim(),
          status,
          due_date: dueDate || null,
          tags,
        };
        await invoke("create_task", { input });
      }
      onSaved();
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function deleteTask() {
    if (!task) return;
    if (!confirm(`Hapus task "${task.title}"? Gak bisa di-undo.`)) return;
    deleting = true;
    error = null;
    try {
      await invoke("delete_task", { filePath: task.file_path });
      onSaved();
      onClose();
    } catch (e) {
      error = String(e);
      deleting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) save();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50 p-4"
  onclick={onClose}
  onkeydown={() => {}}
  role="presentation"
>
  <div
    class="bg-white dark:bg-neutral-900 rounded-xl shadow-2xl w-full max-w-lg max-h-[90vh] overflow-y-auto border border-transparent dark:border-neutral-800"
    onclick={(e) => e.stopPropagation()}
    onkeydown={() => {}}
    role="presentation"
  >
    <div class="px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between">
      <h2 class="text-base font-semibold text-neutral-900 dark:text-neutral-100">
        {isEditing ? "Edit Task" : "Task Baru"}
      </h2>
      <button
        type="button"
        onclick={onClose}
        class="text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 transition-colors"
        aria-label="Close"
      >
        <X size={18} strokeWidth={2} />
      </button>
    </div>

    <div class="px-6 py-4 space-y-4">
      <div>
        <label for="title" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1.5">
          Title
        </label>
        <input
          id="title"
          type="text"
          bind:value={title}
          placeholder="Apa yang mau dikerjain..."
          class="w-full px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-700 rounded-md focus:outline-none focus:ring-2 focus:ring-neutral-900 dark:focus:ring-neutral-100 focus:border-transparent bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100"
        />
      </div>

      <div>
        <label for="description" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1.5">
          Description
        </label>
        <textarea
          id="description"
          bind:value={description}
          rows="4"
          placeholder="Detail task (opsional)..."
          class="w-full px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-700 rounded-md focus:outline-none focus:ring-2 focus:ring-neutral-900 dark:focus:ring-neutral-100 focus:border-transparent bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 resize-none"
        ></textarea>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label for="status" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1.5">
            Status
          </label>
          <select
            id="status"
            bind:value={status}
            class="w-full px-3 py-2 text-sm border border-neutral-300 rounded-md focus:outline-none focus:ring-2 focus:ring-neutral-900 focus:border-transparent bg-white"
          >
            {#each COLUMNS as col}
              <option value={col.id}>{col.icon} {col.label}</option>
            {/each}
          </select>
        </div>

        <div>
          <label for="due-date" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1.5">
            Due Date
          </label>
          <input
            id="due-date"
            type="date"
            bind:value={dueDate}
            class="w-full px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-700 rounded-md focus:outline-none focus:ring-2 focus:ring-neutral-900 dark:focus:ring-neutral-100 focus:border-transparent bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100"
          />
        </div>
      </div>

      <div>
        <label for="tags" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1.5">
          Tags <span class="text-neutral-400 font-normal">(pisah dengan koma)</span>
        </label>
        <input
          id="tags"
          type="text"
          bind:value={tagsStr}
          placeholder="content, urgent, rust"
          class="w-full px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-700 rounded-md focus:outline-none focus:ring-2 focus:ring-neutral-900 dark:focus:ring-neutral-100 focus:border-transparent bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100"
        />
      </div>

      {#if error}
        <div class="bg-red-50 border border-red-200 rounded-md p-3">
          <p class="text-xs text-red-700">{error}</p>
        </div>
      {/if}
    </div>

    <div class="px-6 py-4 border-t border-neutral-200 dark:border-neutral-800 flex items-center justify-between">
      {#if isEditing}
        <button
          type="button"
          onclick={deleteTask}
          disabled={deleting || saving}
          class="text-xs font-medium text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300 px-3 py-1.5 rounded-md hover:bg-red-50 dark:hover:bg-red-950 transition-colors disabled:opacity-50 inline-flex items-center gap-1.5"
        >
          <Trash2 size={14} strokeWidth={2} />
          {deleting ? "Menghapus..." : "Hapus"}
        </button>
      {:else}
        <span class="text-xs text-neutral-400 dark:text-neutral-500">⌘ + Enter buat save</span>
      {/if}
      <div class="flex gap-2">
        <button
          type="button"
          onclick={onClose}
          class="text-xs font-medium text-neutral-700 dark:text-neutral-300 px-4 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        >
          Batal
        </button>
        <button
          type="button"
          onclick={save}
          disabled={saving || deleting}
          class="text-xs font-medium text-white dark:text-neutral-900 bg-neutral-900 dark:bg-neutral-100 hover:bg-neutral-800 dark:hover:bg-neutral-200 px-4 py-1.5 rounded-md transition-colors disabled:opacity-50"
        >
          {saving ? "Menyimpan..." : "Simpan"}
        </button>
      </div>
    </div>
  </div>
</div>
