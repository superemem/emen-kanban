<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import { dndzone, type DndEvent } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import type { Task } from "$lib/types";
  import { COLUMNS } from "$lib/types";
  import TaskCard from "$lib/TaskCard.svelte";
  import TaskModal from "$lib/TaskModal.svelte";
  import SettingsModal from "$lib/SettingsModal.svelte";
  import { checkAndNotifyDueTasks } from "$lib/notifications";
  import { applyTheme, getStoredTheme, nextTheme, setTheme, type Theme } from "$lib/theme";
  import { checkForUpdate, downloadAndInstall } from "$lib/updater";

  let tasks = $state<Task[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let modalOpen = $state(false);
  let editingTask = $state<Task | null>(null);
  let unlistenFn: UnlistenFn | null = null;
  let searchQuery = $state("");
  let searchInput: HTMLInputElement | null = $state(null);
  let theme = $state<Theme>("system");
  let showHelp = $state(false);
  let showSettings = $state(false);
  let updateBanner = $state<{ version: string; notes?: string } | null>(null);
  let installing = $state(false);

  async function manualCheckUpdate() {
    const result = await checkForUpdate(false);
    if (result.available && result.version) {
      updateBanner = { version: result.version, notes: result.notes };
    } else if (result.message) {
      alert(result.message);
    }
  }

  async function installUpdate() {
    if (!updateBanner) return;
    installing = true;
    try {
      await downloadAndInstall();
    } catch (e) {
      alert(`Gagal install: ${e}`);
      installing = false;
    }
  }

  async function loadTasks() {
    error = null;
    try {
      tasks = await invoke<Task[]>("list_tasks");
      checkAndNotifyDueTasks(tasks);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function filteredTasks(): Task[] {
    if (!searchQuery.trim()) return tasks;
    const q = searchQuery.toLowerCase();
    return tasks.filter(
      (t) =>
        t.title.toLowerCase().includes(q) ||
        t.description.toLowerCase().includes(q) ||
        t.tags.some((tag) => tag.toLowerCase().includes(q))
    );
  }

  function tasksByStatus(status: string): Task[] {
    return filteredTasks().filter((t) => t.status === status);
  }

  function openCreate(presetStatus?: string) {
    editingTask = presetStatus
      ? ({
          id: "",
          title: "",
          status: presetStatus,
          due_date: null,
          tags: [],
          created: null,
          updated: null,
          description: "",
          file_path: "",
        } as unknown as Task)
      : null;
    if (presetStatus) {
      editingTask = null; // create modal handles default
    }
    modalOpen = true;
  }

  function openEdit(task: Task) {
    editingTask = task;
    modalOpen = true;
  }

  function closeModal() {
    modalOpen = false;
    editingTask = null;
  }

  function cycleTheme() {
    theme = nextTheme(theme);
    setTheme(theme);
  }

  function handleDndConsider(columnId: string, e: CustomEvent<DndEvent<Task>>) {
    const newItems = e.detail.items;
    const otherTasks = tasks.filter((t) => t.status !== columnId);
    const updatedColumnTasks = newItems.map((t) => ({ ...t, status: columnId }));
    tasks = [...otherTasks, ...updatedColumnTasks];
  }

  async function handleDndFinalize(columnId: string, e: CustomEvent<DndEvent<Task>>) {
    const newItems = e.detail.items;
    const otherTasks = tasks.filter((t) => t.status !== columnId);
    const updatedColumnTasks = newItems.map((t) => ({ ...t, status: columnId }));
    tasks = [...otherTasks, ...updatedColumnTasks];

    for (const t of newItems) {
      try {
        await invoke("update_task_status", {
          filePath: t.file_path,
          newStatus: columnId,
        });
      } catch (err) {
        error = `Gagal update status: ${err}`;
      }
    }
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    // Ignore when typing in inputs/textarea, or modal open
    const target = e.target as HTMLElement;
    const isTyping =
      target?.tagName === "INPUT" ||
      target?.tagName === "TEXTAREA" ||
      target?.isContentEditable;

    if (modalOpen) return;
    if (isTyping && e.key !== "Escape") return;

    switch (e.key) {
      case "n":
        e.preventDefault();
        openCreate();
        break;
      case "/":
        e.preventDefault();
        searchInput?.focus();
        break;
      case "r":
        e.preventDefault();
        loadTasks();
        break;
      case "?":
      case "h":
        e.preventDefault();
        showHelp = !showHelp;
        break;
      case "t":
        e.preventDefault();
        cycleTheme();
        break;
      case ",":
        if (e.metaKey || e.ctrlKey) {
          e.preventDefault();
          showSettings = true;
        }
        break;
      case "Escape":
        if (showHelp) showHelp = false;
        if (searchQuery && document.activeElement === searchInput) {
          searchQuery = "";
          (document.activeElement as HTMLElement)?.blur();
        }
        break;
    }
  }

  onMount(async () => {
    theme = getStoredTheme();
    applyTheme(theme);

    // React to system theme change when in 'system' mode
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    mq.addEventListener("change", () => {
      if (theme === "system") applyTheme("system");
    });

    await loadTasks();

    unlistenFn = await listen("tasks-changed", () => {
      loadTasks();
    });

    // Re-check notifications every 5 minutes
    setInterval(() => checkAndNotifyDueTasks(tasks), 5 * 60 * 1000);

    // Silent update check on launch
    const result = await checkForUpdate(true);
    if (result.available && result.version) {
      updateBanner = { version: result.version, notes: result.notes };
    }
  });

  onDestroy(() => {
    if (unlistenFn) unlistenFn();
  });
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="h-screen flex flex-col bg-neutral-50 dark:bg-neutral-950">

{#if updateBanner}
  <div class="bg-emerald-50 dark:bg-emerald-950 border-b border-emerald-200 dark:border-emerald-900 px-6 py-2.5 flex items-center justify-between gap-3">
    <div class="text-xs text-emerald-900 dark:text-emerald-300">
      🚀 Versi baru tersedia: <strong>v{updateBanner.version}</strong>
      {#if updateBanner.notes}
        <span class="text-emerald-700 dark:text-emerald-400">— {updateBanner.notes}</span>
      {/if}
    </div>
    <div class="flex gap-2">
      <button
        type="button"
        onclick={() => (updateBanner = null)}
        class="text-xs text-emerald-700 dark:text-emerald-400 hover:text-emerald-900 dark:hover:text-emerald-200 px-2"
      >
        Nanti
      </button>
      <button
        type="button"
        onclick={installUpdate}
        disabled={installing}
        class="text-xs font-semibold text-white bg-emerald-700 hover:bg-emerald-800 dark:bg-emerald-600 dark:hover:bg-emerald-500 px-3 py-1 rounded-md transition-colors disabled:opacity-50"
      >
        {installing ? "Installing..." : "Update Sekarang"}
      </button>
    </div>
  </div>
{/if}

  <header
    class="px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 flex items-center justify-between gap-4"
  >
    <div class="flex items-center gap-3">
      <span class="text-xl">⚡</span>
      <h1 class="text-lg font-semibold text-neutral-900 dark:text-neutral-100">Emen Kanban</h1>
      <span class="text-xs text-neutral-400 dark:text-neutral-500 font-medium">
        {tasks.length} task{tasks.length !== 1 ? "s" : ""}
      </span>
    </div>

    <div class="flex items-center gap-2 flex-1 max-w-md">
      <input
        bind:this={searchInput}
        type="text"
        bind:value={searchQuery}
        placeholder="Cari task, tag, deskripsi... (/)"
        class="flex-1 px-3 py-1.5 text-xs border border-neutral-200 dark:border-neutral-700 rounded-md focus:outline-none focus:ring-2 focus:ring-neutral-900 dark:focus:ring-neutral-100 focus:border-transparent bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100"
      />
    </div>

    <div class="flex items-center gap-2">
      <button
        type="button"
        onclick={cycleTheme}
        class="text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 px-2 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        title="Toggle theme (t)"
      >
        {theme === "light" ? "☀️" : theme === "dark" ? "🌙" : "🖥️"}
      </button>
      <button
        type="button"
        onclick={manualCheckUpdate}
        class="text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 px-2 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        title="Cek update"
      >
        ⬇
      </button>
      <button
        type="button"
        onclick={() => (showSettings = true)}
        class="text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 px-2 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        title="Settings (⌘,)"
      >
        ⚙️
      </button>
      <button
        type="button"
        onclick={() => (showHelp = !showHelp)}
        class="text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 px-2 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        title="Shortcuts (?)"
      >
        ?
      </button>
      <button
        type="button"
        onclick={loadTasks}
        class="text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 px-2 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        title="Refresh (r)"
      >
        ↻
      </button>
      <button
        type="button"
        onclick={() => openCreate()}
        class="text-xs font-medium text-white dark:text-neutral-900 bg-neutral-900 dark:bg-neutral-100 hover:bg-neutral-800 dark:hover:bg-neutral-200 px-3 py-1.5 rounded-md transition-colors"
        title="New task (n)"
      >
        + Task Baru
      </button>
    </div>
  </header>

  <main class="flex-1 overflow-hidden p-4">
    {#if loading}
      <div class="flex items-center justify-center h-full text-neutral-400 dark:text-neutral-500 text-sm">
        Loading tasks...
      </div>
    {:else if error}
      <div class="flex items-center justify-center h-full">
        <div class="bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-900 rounded-lg p-4 max-w-md">
          <h3 class="font-semibold text-red-900 dark:text-red-300 mb-1">Gagal load tasks</h3>
          <p class="text-sm text-red-700 dark:text-red-400">{error}</p>
        </div>
      </div>
    {:else}
      <div class="grid grid-cols-3 gap-4 h-full">
        {#each COLUMNS as column}
          {@const colTasks = tasksByStatus(column.id)}
          <div class="flex flex-col bg-neutral-100/60 dark:bg-neutral-900/60 rounded-xl overflow-hidden">
            <div class="px-4 py-3 flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span class="text-base">{column.icon}</span>
                <h2 class="text-sm font-semibold text-neutral-700 dark:text-neutral-300">
                  {column.label}
                </h2>
                <span class="text-xs text-neutral-400 dark:text-neutral-500 font-medium">
                  {colTasks.length}
                </span>
              </div>
            </div>

            <section
              class="flex-1 overflow-y-auto px-3 pb-3 min-h-0"
              use:dndzone={{
                items: colTasks,
                flipDurationMs: 200,
                dropTargetStyle: { outline: "2px dashed rgb(115 115 115)" },
              }}
              onconsider={(e) => handleDndConsider(column.id, e)}
              onfinalize={(e) => handleDndFinalize(column.id, e)}
            >
              {#each colTasks as task (task.id)}
                <div animate:flip={{ duration: 200 }}>
                  <TaskCard {task} onclick={() => openEdit(task)} />
                </div>
              {/each}

              {#if colTasks.length === 0}
                <div class="text-xs text-neutral-400 dark:text-neutral-600 text-center py-8 italic pointer-events-none">
                  Drop di sini
                </div>
              {/if}
            </section>
          </div>
        {/each}
      </div>
    {/if}
  </main>
</div>

{#if modalOpen}
  <TaskModal task={editingTask} onClose={closeModal} onSaved={loadTasks} />
{/if}

{#if showSettings}
  <SettingsModal
    onClose={() => (showSettings = false)}
    onChanged={loadTasks}
  />
{/if}

{#if showHelp}
  <div
    class="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50 p-4"
    onclick={() => (showHelp = false)}
    onkeydown={() => {}}
    role="presentation"
  >
    <div
      class="bg-white dark:bg-neutral-900 rounded-xl shadow-2xl w-full max-w-sm border border-transparent dark:border-neutral-800"
      onclick={(e) => e.stopPropagation()}
      onkeydown={() => {}}
      role="presentation"
    >
      <div class="px-6 py-4 border-b border-neutral-200 dark:border-neutral-800">
        <h2 class="text-base font-semibold text-neutral-900 dark:text-neutral-100">
          Keyboard Shortcuts
        </h2>
      </div>
      <div class="px-6 py-4 space-y-2.5 text-sm">
        {#each [
          ["n", "Task baru"],
          ["/", "Focus search"],
          ["r", "Refresh"],
          ["t", "Toggle theme"],
          ["⌘ + ,", "Settings"],
          ["?", "Show / hide help"],
          ["Esc", "Close modal / clear search"],
          ["⌘ + Enter", "Save (di modal)"],
        ] as [key, label]}
          <div class="flex justify-between items-center">
            <span class="text-neutral-600 dark:text-neutral-400">{label}</span>
            <kbd
              class="text-xs font-mono px-2 py-0.5 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded text-neutral-700 dark:text-neutral-300"
            >
              {key}
            </kbd>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
