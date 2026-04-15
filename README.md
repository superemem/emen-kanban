# Emen Kanban

Native macOS kanban app yang baca/tulis task sebagai `.md` file di vault Obsidian.

**Stack:** Tauri 2 + Svelte 5 + Tailwind CSS v4 + Rust

## Features

- 📋 3 kolom kanban (Todo / Doing / Done)
- 🖱️ Drag & drop antar kolom
- ➕ Add / Edit / Delete task via modal
- 🔍 Search bar (title, description, tags)
- 👁️ File watcher — sync 2 arah Obsidian ↔ App
- 📅 Color-coded due date (overdue/soon/normal)
- 🌙 Dark mode (auto/light/dark)
- 🔔 macOS native notifications buat due date
- ⚙️ Settings panel (custom vault path)
- ⌨️ Keyboard shortcuts
- 🚀 Auto-update via GitHub Releases

## Keyboard Shortcuts

| Key | Action |
|---|---|
| `n` | New task |
| `/` | Focus search |
| `r` | Refresh |
| `t` | Toggle theme |
| `?` | Show help |
| `⌘ ,` | Settings |
| `Esc` | Close modal / clear search |
| `⌘ Enter` | Save (di modal) |

## Development

```bash
npm install
npm run tauri dev      # Hot reload
npm run tauri build    # Build .app + .dmg
```

## Release Workflow (Auto-Update)

1. Bump version di `src-tauri/tauri.conf.json`
2. Commit + push
3. Tag baru: `git tag v0.2.0 && git push origin v0.2.0`
4. GitHub Actions auto-build + sign + bikin release
5. User existing dapet popup auto-update on launch
