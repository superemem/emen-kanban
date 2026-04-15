import { Circle, CircleDot, CircleCheck } from "@lucide/svelte";
import type { Component } from "svelte";

export interface Task {
  id: string;
  title: string;
  status: string;
  due_date: string | null;
  tags: string[];
  created: string | null;
  updated: string | null;
  description: string;
  file_path: string;
}

export interface NewTaskInput {
  title: string;
  description: string;
  status: string;
  due_date: string | null;
  tags: string[];
}

export interface UpdateTaskInput {
  file_path: string;
  title: string;
  description: string;
  status: string;
  due_date: string | null;
  tags: string[];
}

export interface Column {
  id: string;
  label: string;
  icon: Component<any>;
}

export const COLUMNS: Column[] = [
  { id: "todo", label: "Todo", icon: Circle as Component<any> },
  { id: "doing", label: "Doing", icon: CircleDot as Component<any> },
  { id: "done", label: "Done", icon: CircleCheck as Component<any> },
];
