<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  let { onClose, onChanged }: { onClose: () => void; onChanged: () => void } = $props();

  let folder = $state("");
  let saving = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const cfg = await invoke<{ tasks_folder: string }>("get_config");
      folder = cfg.tasks_folder;
    } catch (e) {
      error = String(e);
    }
  });

  async function browse() {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: folder,
    });
    if (typeof selected === "string") {
      folder = selected;
    }
  }

  async function save() {
    if (!folder.trim()) {
      error = "Folder gak boleh kosong";
      return;
    }
    saving = true;
    error = null;
    try {
      await invoke("set_tasks_folder", { newPath: folder.trim() });
      onChanged();
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
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
    class="bg-white dark:bg-neutral-900 rounded-xl shadow-2xl w-full max-w-lg border border-transparent dark:border-neutral-800"
    onclick={(e) => e.stopPropagation()}
    onkeydown={() => {}}
    role="presentation"
  >
    <div class="px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between">
      <h2 class="text-base font-semibold text-neutral-900 dark:text-neutral-100">Settings</h2>
      <button
        type="button"
        onclick={onClose}
        class="text-neutral-400 hover:text-neutral-700 text-xl leading-none"
        aria-label="Close"
      >
        ×
      </button>
    </div>

    <div class="px-6 py-4 space-y-4">
      <div>
        <label for="folder" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1.5">
          Tasks Folder
        </label>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 mb-2">
          Folder berisi file <code>.md</code> dengan frontmatter <code>type: kanban-task</code>.
        </p>
        <div class="flex gap-2">
          <input
            id="folder"
            type="text"
            bind:value={folder}
            class="flex-1 px-3 py-2 text-xs border border-neutral-300 dark:border-neutral-700 rounded-md focus:outline-none focus:ring-2 focus:ring-neutral-900 dark:focus:ring-neutral-100 focus:border-transparent bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 font-mono"
          />
          <button
            type="button"
            onclick={browse}
            class="text-xs font-medium text-neutral-700 dark:text-neutral-300 px-3 py-2 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 transition-colors"
          >
            Browse...
          </button>
        </div>
      </div>

      {#if error}
        <div class="bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-900 rounded-md p-3">
          <p class="text-xs text-red-700 dark:text-red-400">{error}</p>
        </div>
      {/if}

      <div class="bg-amber-50 dark:bg-amber-950 border border-amber-200 dark:border-amber-900 rounded-md p-3">
        <p class="text-xs text-amber-800 dark:text-amber-300">
          ⚠️ Restart app supaya file watcher track folder baru.
        </p>
      </div>
    </div>

    <div class="px-6 py-4 border-t border-neutral-200 dark:border-neutral-800 flex items-center justify-end gap-2">
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
        disabled={saving}
        class="text-xs font-medium text-white dark:text-neutral-900 bg-neutral-900 dark:bg-neutral-100 hover:bg-neutral-800 dark:hover:bg-neutral-200 px-4 py-1.5 rounded-md transition-colors disabled:opacity-50"
      >
        {saving ? "Menyimpan..." : "Simpan"}
      </button>
    </div>
  </div>
</div>
