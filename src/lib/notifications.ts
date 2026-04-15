import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import type { Task } from "./types";

const NOTIFIED_KEY = "emen-kanban-notified";

function getNotified(): Set<string> {
  try {
    const raw = localStorage.getItem(NOTIFIED_KEY);
    if (!raw) return new Set();
    return new Set(JSON.parse(raw));
  } catch {
    return new Set();
  }
}

function saveNotified(set: Set<string>) {
  localStorage.setItem(NOTIFIED_KEY, JSON.stringify([...set]));
}

export async function ensurePermission(): Promise<boolean> {
  let granted = await isPermissionGranted();
  if (!granted) {
    const result = await requestPermission();
    granted = result === "granted";
  }
  return granted;
}

export async function checkAndNotifyDueTasks(tasks: Task[]) {
  const granted = await ensurePermission();
  if (!granted) return;

  const notified = getNotified();
  const now = new Date();

  for (const task of tasks) {
    if (task.status === "done" || !task.due_date) continue;

    const due = new Date(task.due_date);
    if (isNaN(due.getTime())) continue;

    const diffDays = (due.getTime() - now.getTime()) / (1000 * 60 * 60 * 24);

    // Today key (so we notify max once per day per task)
    const todayKey = `${task.id}-${now.toISOString().slice(0, 10)}`;
    if (notified.has(todayKey)) continue;

    let title = "";
    let body = "";

    if (diffDays < 0) {
      title = "⚠️ Task Overdue";
      body = `${task.title}\nDeadline kelewat ${Math.abs(Math.floor(diffDays))} hari`;
    } else if (diffDays < 1) {
      title = "🔔 Due Hari Ini";
      body = task.title;
    } else if (diffDays < 2) {
      title = "📅 Due Besok";
      body = task.title;
    } else {
      continue;
    }

    sendNotification({ title, body });
    notified.add(todayKey);
  }

  saveNotified(notified);
}
