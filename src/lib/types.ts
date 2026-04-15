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
  icon: string;
}

export const COLUMNS: Column[] = [
  { id: "todo", label: "Todo", icon: "📋" },
  { id: "doing", label: "Doing", icon: "🔄" },
  { id: "done", label: "Done", icon: "✅" },
];
