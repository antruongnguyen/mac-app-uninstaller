# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Behavioral Guidelines

Behavioral guidelines to reduce common LLM coding mistakes. **Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

### 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

### 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

## Project Overview

A macOS-only desktop app that lists installed `.app` bundles, finds their related support/cache/preference files, and moves selected items to the Trash. Built on **Tauri 2** with a **React + TypeScript + Tailwind v4 + shadcn/ui** frontend and a **Rust** backend.

The original implementation used `eframe`/`egui`. See `docs/TAURI_MIGRATION.md` for the migration log and design decisions.

### Common Commands

| Task                        | Command                                                  |
| --------------------------- | -------------------------------------------------------- |
| Install dependencies        | `bun install`                                            |
| Run dev shell               | `bun run tauri dev`                                      |
| Run frontend only (browser) | `bun run dev` (no Tauri IPC; useful for UI iteration)    |
| Build app bundle            | `bun run tauri build` → `src-tauri/target/release/bundle/macos/App Uninstaller.app` |
| Vite build + tsc            | `bun run build`                                          |
| Lint                        | `bun run lint`                                           |
| Frontend tests              | `bun run test`                                           |
| Rust tests                  | `cargo test --manifest-path src-tauri/Cargo.toml`        |
| Format Rust                 | `cargo fmt --manifest-path src-tauri/Cargo.toml`         |
| Clippy                      | `cargo clippy --manifest-path src-tauri/Cargo.toml`      |
| Add a shadcn primitive      | `npx shadcn@latest add @shadcn/<name>` (e.g. `item`, `input-group`) |
| Search shadcn registry      | `npx shadcn@latest search @shadcn -q "<query>"`          |

Bundle metadata (name, identifier `day.nhanh.appuninstaller`, icon, window size) lives in `src-tauri/tauri.conf.json`.

### Architecture

The app has two halves that talk through Tauri's IPC:

- **`src/`** — React frontend. Single window. State held in Zustand stores; long-running work is dispatched as Tauri commands and streamed back as `progress` events.
- **`src-tauri/`** — Rust backend. `core/` holds pure logic (scan, plist, related-paths, trash, running-process detection). `commands.rs` exposes it through `#[tauri::command]` handlers. `progress.rs` defines the typed `progress` event channel.

Front-to-back contract:

| Command            | Args                                                  | Returns           | Emits `progress` events |
| ------------------ | ----------------------------------------------------- | ----------------- | ----------------------- |
| `list_apps`        | —                                                     | `AppInfo[]`       | yes (`refresh_apps`)    |
| `find_related`     | `bundle_id?`, `app_name`                              | `string[]`        | yes (`find_related`)    |
| `is_app_running`   | `bundle_id?`, `app_name?`                             | `boolean`         | no                      |
| `kill_app`         | `bundle_id?`, `app_name?`                             | `number`          | no                      |
| `get_app_size`     | `path`                                                | `number \| null`  | no                      |
| `uninstall`        | `app_path`, `app_name`, `bundle_id?`, `related_paths` | `UninstallReport` | yes (`uninstall`)       |
| `reveal_in_finder` | `path`                                                | `()`              | no                      |

Long-running commands (`list_apps`, `find_related`, `uninstall`) run in `tauri::async_runtime::spawn_blocking` and emit `ProgressEvent { kind, progress, message, finished, error }` while they run. Short commands (`is_app_running`, `kill_app`, `get_app_size`, `reveal_in_finder`) also use `spawn_blocking` to keep the IPC thread free, but do not emit progress. The frontend subscribes once in `App.tsx` and routes events into `useTaskStore`.

When adding a new background operation:
1. Add a pure function in `src-tauri/src/core/` (no Tauri deps).
2. Add a `#[tauri::command] async fn` in `src-tauri/src/commands.rs` that calls it inside `spawn_blocking`. Emit typed `progress` events only if the work is long enough that the user benefits from intermediate feedback.
3. Register it in the `invoke_handler!` macro in `src-tauri/src/lib.rs`.
4. Add a typed wrapper in `src/lib/api/uninstaller.ts` and the matching DTO in `src/types/models.ts` (Rust uses `#[serde(rename_all = "camelCase")]`).
5. Compose the UI from existing shadcn primitives — only build a new component if shadcn doesn't have one. Add new primitives via `npx shadcn@latest add @shadcn/<name>` rather than hand-writing them.
6. Give every interactive/structural element a stable id from `IDS` in `src/lib/styles.ts`. Repeated class strings go into `STYLES` in the same file.

### Conventions

**Rust**
- Error handling: `anyhow::Result<T>` with `.context("...")?` for filesystem/plist operations; commands return `Result<T, String>` (Tauri requires `Serialize` on the error).
- Paths: always `PathBuf` / `Path`; check `exists()` before operations.
- Pure logic stays in `src-tauri/src/core/`. The `commands` and `progress` modules may depend on `tauri`; `core` may not.
- macOS-specific code is gated by `#[cfg(target_os = "macos")]`. The trash + plist + sysinfo crates we use are cross-platform; only `reveal_in_finder` calls `open -R` directly.
- `list_apps` must stay cheap — the per-app scan is bounded to `read_dir` + `Info.plist` parse + a sysinfo string match. Anything that walks the bundle interior (e.g. `compute_size` via `WalkDir`) is exposed as its own command and called lazily by the frontend when an app is selected, never during the scan. Re-adding such work to `scan_one_dir` produces multi-second freezes on machines with Xcode-class bundles.

**Frontend**
- Component library: shadcn/ui (`base-nova` style, `neutral` base color, lucide icons). Use shadcn primitives whenever possible — only build a custom component if a primitive does not exist. Add new primitives via the CLI (`npx shadcn@latest add @shadcn/<name>`); don't hand-write them.
- Path alias: `@/*` → `./src/*`.
- State: Zustand for cross-component state; local `useState` otherwise.
- Tauri IPC: always go through `tauriInvoke` / `tauriListen` in `src/lib/tauri.ts`, never `invoke` / `listen` directly. The wrapper gives a clearer error when running outside the Tauri shell.
- Toast notifications: `sonner` (via `@/components/ui/sonner`).
- DTOs: defined in Rust (`src-tauri/src/models.rs`) and mirrored verbatim in `src/types/models.ts`. Keep field names in sync. Rust uses `#[serde(rename_all = "camelCase")]` so the TS interface uses camelCase.
- IDs and shared styles: `src/lib/styles.ts` exports `IDS` (stable element ids — every interactive/structural element gets one) and `STYLES` (class strings used in more than one place). Per-row ids are generated via helpers (e.g. `IDS.sidebarAppRow(path)`). Local-only classes stay inline at the call site.
- Hooks: cross-component hooks live in `src/hooks/`. Examples: `useIsTruncated` (mounts a tooltip only when `truncate` is actually clipping), `useAppSize` (lazy per-path size lookup with a process-wide cache).

#### Imports
- Group imports: std → external crates → local modules (Rust); React → external → `@/` (TS).
- Use explicit imports, avoid glob imports.

#### Naming
- Rust: `snake_case` functions, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants.
- TypeScript: `camelCase` variables/functions, `PascalCase` components/types, `SCREAMING_SNAKE_CASE` constants.

#### Theming & fonts
- Light/dark themes via `next-themes` and `oklch()` tokens in `src/index.css`. Do not introduce new colour variables — extend the existing token block.
- Font: Geist Variable, loaded via `@fontsource-variable/geist` (self-hosted, cacheable). Don't link to Google Fonts CDN at runtime.

### Documentation & Version Lookup

**IMPORTANT**: Before making changes to any framework or library usage, use MCP server Context7 to look up the latest documentation and versions.

#### Core dependencies to verify before changing usage

- **Tauri** (current: 2.10.x) — desktop shell + IPC
- **Tauri plugins** — `tauri-plugin-dialog`, `tauri-plugin-log`
- **React 19** + **React DOM 19**
- **Vite 8** + **@vitejs/plugin-react 6**
- **Tailwind CSS 4** (`@tailwindcss/vite`)
- **shadcn/ui** (style: `base-nova`, base color: `neutral`)
- **@base-ui/react** — primitives the shadcn components wrap
- **lucide-react** — icon set
- **next-themes** — light/dark switching
- **sonner** — toasts
- **zustand** — state management

#### Rust crates retained from the original implementation

- **plist 1.7** — `Info.plist` parsing
- **trash 5.2** — safe file deletion (moves to Trash)
- **walkdir 2.5** — directory traversal
- **sysinfo 0.37** — process listing
- **home 0.5** — home directory detection
- **anyhow 1.0** — error handling

#### Usage instructions

1. Look up latest docs/versions via Context7 before changing framework usage.
2. Review breaking changes and migration guides.
3. Update `package.json` / `src-tauri/Cargo.toml` with latest compatible versions.
4. Test thoroughly after dependency updates.
5. Update this document with new version numbers after successful upgrades.
