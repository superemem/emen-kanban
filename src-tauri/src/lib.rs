use chrono::Local;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Emitter;
use walkdir::WalkDir;

const DEFAULT_TASKS_FOLDER: &str = "/Users/azwary/Library/Mobile Documents/iCloud~md~obsidian/Documents/Emen Brain/Emen Space/Kanban/Tasks";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    tasks_folder: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tasks_folder: DEFAULT_TASKS_FOLDER.to_string(),
        }
    }
}

fn config_path() -> PathBuf {
    let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("emen-kanban");
    if !p.exists() {
        let _ = fs::create_dir_all(&p);
    }
    p.push("config.json");
    p
}

fn load_config() -> AppConfig {
    let path = config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
            return cfg;
        }
    }
    AppConfig::default()
}

fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let path = config_path();
    let content = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

type SharedConfig = Arc<Mutex<AppConfig>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TaskFrontmatter {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    due_date: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    created: Option<String>,
    #[serde(default)]
    updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    id: String,
    title: String,
    status: String,
    due_date: Option<String>,
    tags: Vec<String>,
    created: Option<String>,
    updated: Option<String>,
    description: String,
    file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTaskInput {
    title: String,
    description: String,
    status: String,
    due_date: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    file_path: String,
    title: String,
    description: String,
    status: String,
    due_date: Option<String>,
    tags: Vec<String>,
}

fn parse_markdown(content: &str, file_path: &str) -> Option<Task> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let after_first = &trimmed[3..];
    let end_idx = after_first.find("\n---")?;
    let yaml_str = &after_first[..end_idx];
    let body = after_first[end_idx + 4..].trim().to_string();

    let fm: TaskFrontmatter = serde_yaml::from_str(yaml_str).ok()?;

    if fm.r#type != "kanban-task" {
        return None;
    }

    Some(Task {
        id: fm.id,
        title: fm.title,
        status: fm.status,
        due_date: fm.due_date,
        tags: fm.tags,
        created: fm.created,
        updated: fm.updated,
        description: body,
        file_path: file_path.to_string(),
    })
}

fn now_iso() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn build_markdown(task: &Task) -> String {
    let mut out = String::from("---\n");
    out.push_str("type: kanban-task\n");
    out.push_str(&format!("id: {}\n", task.id));
    out.push_str(&format!("title: {}\n", escape_yaml(&task.title)));
    out.push_str(&format!("status: {}\n", task.status));
    if let Some(d) = &task.due_date {
        if !d.is_empty() {
            out.push_str(&format!("due_date: {}\n", d));
        }
    }
    if !task.tags.is_empty() {
        let tag_list = task
            .tags
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("tags: [{}]\n", tag_list));
    }
    if let Some(c) = &task.created {
        out.push_str(&format!("created: {}\n", c));
    }
    out.push_str(&format!("updated: {}\n", task.updated.clone().unwrap_or_else(now_iso)));
    out.push_str("---\n\n");
    out.push_str(&task.description);
    if !task.description.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn escape_yaml(s: &str) -> String {
    if s.contains(':') || s.contains('#') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

fn current_folder(state: &SharedConfig) -> PathBuf {
    PathBuf::from(state.lock().unwrap().tasks_folder.clone())
}

#[tauri::command]
fn get_config(state: tauri::State<SharedConfig>) -> AppConfig {
    state.lock().unwrap().clone()
}

#[tauri::command]
fn set_tasks_folder(
    new_path: String,
    state: tauri::State<SharedConfig>,
    app: tauri::AppHandle,
) -> Result<AppConfig, String> {
    let path = PathBuf::from(&new_path);
    if !path.exists() {
        return Err(format!("Folder gak ada: {}", new_path));
    }
    if !path.is_dir() {
        return Err(format!("Path bukan folder: {}", new_path));
    }
    let mut cfg = state.lock().unwrap();
    cfg.tasks_folder = new_path;
    save_config(&cfg)?;
    let _ = app.emit("tasks-changed", ());
    Ok(cfg.clone())
}

#[tauri::command]
fn list_tasks(state: tauri::State<SharedConfig>) -> Result<Vec<Task>, String> {
    let folder = current_folder(&state);
    if !folder.exists() {
        return Err(format!("Tasks folder gak ditemukan: {}", folder.display()));
    }

    let mut tasks = Vec::new();

    for entry in WalkDir::new(&folder).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(path) {
                let path_str = path.to_string_lossy().to_string();
                if let Some(task) = parse_markdown(&content, &path_str) {
                    tasks.push(task);
                }
            }
        }
    }

    Ok(tasks)
}

#[tauri::command]
fn update_task_status(file_path: String, new_status: String) -> Result<Task, String> {
    let path = PathBuf::from(&file_path);
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let mut task = parse_markdown(&content, &file_path)
        .ok_or_else(|| "Failed to parse task".to_string())?;

    task.status = new_status;
    task.updated = Some(now_iso());

    let new_content = build_markdown(&task);
    fs::write(&path, new_content).map_err(|e| e.to_string())?;

    Ok(task)
}

#[tauri::command]
fn create_task(
    input: NewTaskInput,
    state: tauri::State<SharedConfig>,
) -> Result<Task, String> {
    let folder = current_folder(&state);
    if !folder.exists() {
        fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    }

    let date = today_iso();
    let slug_title = slug::slugify(&input.title);
    let id = format!("{}-{}", date, slug_title);
    let filename = format!("{}.md", id);
    let file_path = folder.join(&filename);

    let task = Task {
        id: id.clone(),
        title: input.title,
        status: input.status,
        due_date: input.due_date,
        tags: input.tags,
        created: Some(now_iso()),
        updated: Some(now_iso()),
        description: input.description,
        file_path: file_path.to_string_lossy().to_string(),
    };

    let content = build_markdown(&task);
    fs::write(&file_path, content).map_err(|e| e.to_string())?;

    Ok(task)
}

#[tauri::command]
fn update_task(input: UpdateTaskInput) -> Result<Task, String> {
    let path = PathBuf::from(&input.file_path);
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let existing = parse_markdown(&content, &input.file_path)
        .ok_or_else(|| "Failed to parse task".to_string())?;

    let task = Task {
        id: existing.id,
        title: input.title,
        status: input.status,
        due_date: input.due_date,
        tags: input.tags,
        created: existing.created,
        updated: Some(now_iso()),
        description: input.description,
        file_path: input.file_path.clone(),
    };

    let new_content = build_markdown(&task);
    fs::write(&path, new_content).map_err(|e| e.to_string())?;

    Ok(task)
}

#[tauri::command]
fn delete_task(file_path: String) -> Result<(), String> {
    let path = PathBuf::from(&file_path);
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(())
}

fn start_file_watcher(app_handle: tauri::AppHandle, folder: PathBuf) {
    thread::spawn(move || {
        let (tx, rx) = channel();

        let mut watcher = match notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create file watcher: {}", e);
                return;
            }
        };

        if !folder.exists() {
            eprintln!("Watch folder gak ada: {}", folder.display());
            return;
        }

        if let Err(e) = watcher.watch(&folder, RecursiveMode::NonRecursive) {
            eprintln!("Failed to watch folder: {}", e);
            return;
        }

        let mut last_emit = std::time::Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(event) => {
                    if matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) {
                        if last_emit.elapsed() > Duration::from_millis(300) {
                            let _ = app_handle.emit("tasks-changed", ());
                            last_emit = std::time::Instant::now();
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = Arc::new(Mutex::new(load_config()));
    let cfg_for_setup = cfg.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(cfg)
        .setup(move |app| {
            let folder = PathBuf::from(cfg_for_setup.lock().unwrap().tasks_folder.clone());
            start_file_watcher(app.handle().clone(), folder);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_tasks,
            update_task_status,
            create_task,
            update_task,
            delete_task,
            get_config,
            set_tasks_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[allow(dead_code)]
fn _unused(_: &Path) {}
