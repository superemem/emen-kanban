<script lang="ts">
  import type { Task } from "./types";
  import { Calendar, Hash } from "@lucide/svelte";

  let { task, onclick }: { task: Task; onclick?: () => void } = $props();

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return "";
    const d = new Date(dateStr);
    if (isNaN(d.getTime())) return dateStr;
    return d.toLocaleDateString("id-ID", {
      day: "numeric",
      month: "short",
    });
  }

  function isDueSoon(dateStr: string | null): boolean {
    if (!dateStr) return false;
    const d = new Date(dateStr);
    const now = new Date();
    const diff = (d.getTime() - now.getTime()) / (1000 * 60 * 60 * 24);
    return diff <= 3 && diff >= 0;
  }

  function isOverdue(dateStr: string | null): boolean {
    if (!dateStr) return false;
    const d = new Date(dateStr);
    const now = new Date();
    return d.getTime() < now.getTime();
  }
</script>

<button
  type="button"
  {onclick}
  class="w-full text-left bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-3 mb-2 hover:border-neutral-300 dark:hover:border-neutral-700 hover:shadow-sm transition-all cursor-pointer block"
>
  <h3 class="text-sm font-medium text-neutral-900 dark:text-neutral-100 leading-snug mb-2">
    {task.title}
  </h3>

  {#if task.description}
    <p class="text-xs text-neutral-500 dark:text-neutral-400 line-clamp-2 mb-3 whitespace-pre-line">
      {task.description}
    </p>
  {/if}

  <div class="flex items-center justify-between gap-2 flex-wrap">
    {#if task.tags && task.tags.length > 0}
      <div class="flex gap-1 flex-wrap">
        {#each task.tags as tag}
          <span
            class="text-[10px] px-1.5 py-0.5 rounded bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400 font-medium inline-flex items-center gap-0.5"
          >
            <Hash size={10} strokeWidth={2.5} />{tag}
          </span>
        {/each}
      </div>
    {/if}

    {#if task.due_date}
      <span
        class="text-[10px] font-medium px-1.5 py-0.5 rounded ml-auto inline-flex items-center gap-1"
        class:bg-red-50={isOverdue(task.due_date) && task.status !== "done"}
        class:dark:bg-red-950={isOverdue(task.due_date) && task.status !== "done"}
        class:text-red-600={isOverdue(task.due_date) && task.status !== "done"}
        class:dark:text-red-400={isOverdue(task.due_date) && task.status !== "done"}
        class:bg-amber-50={isDueSoon(task.due_date) && !isOverdue(task.due_date)}
        class:dark:bg-amber-950={isDueSoon(task.due_date) && !isOverdue(task.due_date)}
        class:text-amber-600={isDueSoon(task.due_date) && !isOverdue(task.due_date)}
        class:dark:text-amber-400={isDueSoon(task.due_date) && !isOverdue(task.due_date)}
        class:bg-neutral-100={!isDueSoon(task.due_date) && !isOverdue(task.due_date)}
        class:dark:bg-neutral-800={!isDueSoon(task.due_date) && !isOverdue(task.due_date)}
        class:text-neutral-500={!isDueSoon(task.due_date) && !isOverdue(task.due_date)}
        class:dark:text-neutral-400={!isDueSoon(task.due_date) && !isOverdue(task.due_date)}
      >
        <Calendar size={10} strokeWidth={2.5} />
        {formatDate(task.due_date)}
      </span>
    {/if}
  </div>
</button>
