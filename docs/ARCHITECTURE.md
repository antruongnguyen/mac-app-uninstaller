# Architecture

App Uninstaller is a macOS-only desktop application that lists installed `.app` bundles, finds related support/cache/preference files, and moves selected items to the Trash.

After the migration from egui to Tauri, the application is split into two halves that talk to each other via Tauri's IPC bridge:

```
┌────────────────────────────────────────────────────────────────────┐
│ Frontend (React + TypeScript + Vite + Tailwind v4 + shadcn/ui)     │
│  ─ Pages/components for app list, related files, progress, log    │
│  ─ Zustand stores (apps, related, task)                            │
│  ─ src/lib/api/uninstaller.ts wraps `invoke()` calls               │
│  ─ Listens to `progress` events for streaming updates              │
└──────────────────────────▲──────────────────┬──────────────────────┘
                           │ events           │ invoke
                           │                  ▼
┌────────────────────────────────────────────────────────────────────┐
│ Backend (Rust, src-tauri/)                                         │
│  ─ commands/        Tauri command handlers (#[tauri::command])     │
│  ─ core/            Pure business logic (scan, plist, trash, …)   │
│  ─ models.rs        Serde-serialisable types shared with frontend  │
│  ─ progress.rs      Helper that emits typed `progress` events      │
└────────────────────────────────────────────────────────────────────┘
```

## Goals of the migration

- Keep the same user-visible feature set (list apps, pick related files, move to Trash, status log, progress bar).
- Drop egui rendering in favour of standard web UI, so we can reuse community-maintained shadcn/ui components rather than hand-styled custom widgets.
- Keep all heavy lifting in Rust — the frontend should not call `walkdir`/`sysinfo`/the trash. It only renders state.
- Only depend on cross-platform Tauri primitives so we are not blocked by future macOS API changes.

## Directory layout

```
mac_uninstaller/
├── docs/                       Architecture, migration notes, UI guide
├── src/                        React frontend
│   ├── components/             Page-level components composed from shadcn/ui
│   │   └── ui/                 shadcn-generated primitives (button, card, …)
│   ├── lib/
│   │   ├── api/uninstaller.ts  Typed wrapper around `invoke()`
│   │   ├── tauri.ts            isInsideTauri + dev-mode bypass for `bun run dev`
│   │   └── utils.ts            `cn` helper (clsx + tailwind-merge)
│   ├── stores/                 Zustand stores
│   ├── types/                  TS mirror of the Rust models
│   ├── App.tsx                 Top-level layout
│   ├── main.tsx                React entry point
│   └── index.css               Tailwind + theme tokens
├── src-tauri/                  Tauri backend
│   ├── src/
│   │   ├── core/               (mod) scan, plist, related-paths, trash helpers
│   │   ├── commands/           (mod) one file per command group
│   │   ├── models.rs           Serde DTOs shared with the frontend
│   │   ├── progress.rs         Progress event emitter
│   │   ├── lib.rs              `pub fn run()` registers all commands
│   │   └── main.rs             Entry — calls `lib::run()`
│   ├── capabilities/default.json
│   ├── tauri.conf.json
│   └── Cargo.toml
├── components.json             shadcn config (mirrors teaching-management)
├── package.json
├── vite.config.ts
└── README.md
```

The legacy `src/` (egui Rust sources), `Cargo.toml` and `Cargo.lock` at the repo root, the `style.rs`/`osx.rs`/`ui/` modules, the top-level `target/`, and the `svg-to-icns.sh` helper script were removed as part of the migration. Bundle icons live in `src-tauri/icons/` instead of `resources/`.

## Frontend ↔ backend contract

All cross-process communication happens through three things:

1. **Commands** (request/response) — declared with `#[tauri::command]`. The frontend calls them through `tauriInvoke<T>(name, args)`.
2. **Events** (push) — `app_handle.emit("progress", payload)`. The frontend subscribes via `listen("progress", …)`.
3. **DTOs** — defined in `src-tauri/src/models.rs` with `#[derive(Serialize, Deserialize)]`. The same shape is mirrored in `src/types/models.ts`.

### Commands

| Command            | Args                                 | Returns          | Purpose                                          |
| ------------------ | ------------------------------------ | ---------------- | ------------------------------------------------ |
| `list_apps`        | none                                 | `Vec<AppInfo>`   | Scan `/Applications` and `~/Applications`        |
| `find_related`     | `bundle_id?`, `app_name`             | `Vec<String>`    | Walk Library locations, return related paths     |
| `is_app_running`   | `bundle_id?`, `app_name?`            | `bool`           | Re-check before uninstall                        |
| `uninstall`        | `app_path`, `paths: Vec<String>`     | `UninstallReport`| Trash the app and the user-selected related items|
| `reveal_in_finder` | `path: String`                       | `()`             | Run `open -R <path>`                             |

Long-running commands (`list_apps`, `find_related`, `uninstall`) are async and emit `progress` events while they run. They take an `AppHandle` parameter so they can call `app.emit(...)`.

### Progress events

A single channel is used for all task progress; the payload is tagged so the frontend can filter:

```rust
// progress.rs
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProgressEvent {
    RefreshApps   { progress: f32, message: String, finished: bool, error: Option<String> },
    FindRelated   { progress: f32, message: String, finished: bool, error: Option<String> },
    Uninstall     { progress: f32, message: String, finished: bool, error: Option<String> },
}
```

The frontend keeps a single `useTaskStore` (Zustand) that mirrors the current task — `kind | progress | message | running` — and is updated by a global listener registered in `main.tsx`.

### Why Tauri commands instead of WebSockets / a separate HTTP server

Tauri's IPC bridge is in-process, has no port to negotiate, runs entirely over OS pipes, and integrates with the capabilities system. Adding an HTTP layer would only add deployment friction.

## Threading and progress

Each long-running command uses `tauri::async_runtime::spawn_blocking` for the `walkdir`/`sysinfo`/`trash` work and emits progress from the spawned task. The command itself awaits the task and returns the final value. This keeps the UI responsive without us having to write our own thread-pool code (the egui version did this manually in `ui/tasks.rs`).

## Uninstall semantics (preserved from the egui version)

1. Check whether the app is running (`is_app_running`). If yes → abort.
2. Move the app bundle itself to Trash.
3. Partition the user-selected related items into `protected` (paths under `/Library`, `/private`, `/System`, `/usr`, `/var`, `/opt`, `/etc`, `/Applications`) and `unprotected`.
4. Process protected items first; abort on the first failure (so the OS auth prompt fires once at the start).
5. Process unprotected items; continue past per-item errors and report each one in the status log.
6. Finalise with a `finished: true` event; the frontend then re-runs `list_apps`.

This is identical to the previous behaviour in `src/ui/tasks.rs::spawn_uninstall_selected`.

## Theming, fonts, design system

See [UI.md](./UI.md) for the visual design and component conventions. In short:

- **Component library:** shadcn/ui (`base-nova` style, `neutral` base color, lucide icons). We do not build custom widgets when a shadcn primitive exists.
- **CSS:** Tailwind v4 via `@tailwindcss/vite`, with the same `@theme inline` token block used by the reference project. Light/dark themes via `next-themes` and `oklch()` tokens.
- **Font:** Geist Variable, served as cacheable static assets through `@fontsource-variable/geist`. shadcn's default sans recommendation is Geist, so we reuse the same default rather than introducing a Google-Fonts CDN dependency.
  - Decision: `@fontsource-variable/geist` ships the same files Google would, but bundled and self-hosted, which is friendlier to offline desktop usage and avoids a runtime dependency on `fonts.gstatic.com`. We still reach the goal of using a "cacheable Google font" — Geist is published by Vercel and is mirrored on Google Fonts.

## Why drop egui

- `egui` immediate-mode rendering meant every visual element was hand-drawn. Adding shadcn-quality dialogs, dropdowns, scroll areas, and toasts would have meant re-implementing each.
- We had three Rust files (`style.rs`, `ui/color.rs`, `ui/list.rs`) of pure presentation code that can be replaced by community components.
- The egui worker-thread pattern (`Arc<Mutex<GuiState>>` + `mpsc::channel<ProgressUpdate>`) maps cleanly to `spawn_blocking` + `app.emit`, so no functionality is lost.

## Testing

- Backend: `cargo test --manifest-path src-tauri/Cargo.toml`. Pure logic in `core/` is testable without a Tauri runtime.
- Frontend: `bun run test` (vitest) for store/util tests; `bun run test:e2e` for Playwright (set up in a follow-up commit).
- Manual: `bun run tauri dev` opens the dev shell with hot-reload.
