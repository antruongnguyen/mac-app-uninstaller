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

A macOS-only desktop app written in Rust + egui that lists installed `.app` bundles, finds their related support/cache/preference files, and moves selected items to the Trash. The binary crate is `app_uninstaller` (Rust edition 2024).

### Common Commands

- Run (dev): `cargo run`
- Build (debug / release): `cargo build` / `cargo build --release`
- Format / Lint: `cargo fmt` / `cargo clippy`
- Tests: `cargo test` (single test: `cargo test <name>`; with output: `cargo test -- --nocapture`)
- Build `.app` bundle: `cargo bundle --release` → `target/release/bundle/osx/App Uninstaller.app` (requires `cargo install cargo-bundle`)
- Regenerate `.icns` icons from SVG: `./svg-to-icns.sh` (requires `brew install librsvg`)

Bundle metadata (name, identifier `day.nhanh.appuninstaller`, icon) lives in `Cargo.toml` under `[package.metadata.bundle]`.

### Architecture

The app is a single-window egui frontend that delegates all filesystem and process work to the `core` module via background threads. Understanding the threading boundary is key:

- **`main.rs`** — entry point. Sets the Dock icon (macOS) and launches `eframe::run_native` with `MacUninstallerApp`.
- **`ui/mod.rs`** — `MacUninstallerApp` (eframe `App`) and `GuiState`. State is held in `Arc<Mutex<GuiState>>` and shared between the UI thread and worker threads. Communication from workers → UI happens through an `mpsc` channel of `ProgressUpdate` messages drained each frame in `update()`. Worker threads also write directly into `GuiState` (apps list, related paths, status messages) under the mutex.
- **`ui/tasks.rs`** — the only place that spawns threads. Three entry points: `spawn_refresh_apps`, `spawn_refresh_related_for_selected`, `spawn_uninstall_selected`. Uninstall logic partitions related paths into "protected" (system locations like `/Library`, `/private`, `/System/...`) and "unprotected"; protected items are processed first and abort on first failure, unprotected items continue on per-item errors. After an uninstall the apps list is auto-refreshed.
- **`core.rs`** — pure business logic (no UI deps): `find_app_bundles_progress`, `read_info_from_app` (parses `Contents/Info.plist`), `is_app_running` / `is_app_running_simple` (sysinfo-based heuristics across process name, exe path, and cmdline), `find_related_paths` (combines `common_paths_for_bundle_id` with a `walkdir` scan of user/system Library locations), and `move_to_trash_or_remove` (prefers `trash` crate, falls back to `fs::remove_*`).
- **`ui/panels/`** — `top`, `side` (app list), `central` (selected app + related files checklist), `bottom` (progress + status log).
- **`style.rs`** + **`ui/color.rs`** — centralized AppKit-like theming. `set_appkit_style(ctx)` is called every frame.
- **`osx.rs`** — macOS-only AppKit calls (set Dock icon via `cocoa`/`objc`, open System Settings).
- **`types.rs`** — `AppInfo`, `TaskKind` (`Idle | RefreshApps | RefreshRelated(idx) | Uninstall(idx)`), `ProgressUpdate`, `StateColors`.

When adding a new background operation, follow the existing pattern: spawn a thread, clone the `progress_tx` sender, send a starting `ProgressUpdate { finished: false }`, do work, mutate `GuiState` under the lock, send a final `ProgressUpdate { finished: true }`. Do not block the UI thread on filesystem or `sysinfo` work.

### Conventions

- Error handling: `anyhow::Result<T>` with `.context("...")?` for filesystem/plist operations.
- Paths: always `PathBuf` / `Path`; check `exists()` before operations.
- Dependencies pinned in `Cargo.toml`: eframe/egui 0.32, sysinfo 0.37, plist 1.7, trash 5.2, walkdir 2.5, anyhow 1.0, home 0.5; macOS-only: cocoa 0.26, objc 0.2.
- macOS-specific code is gated by `#[cfg(target_os = "macos")]` (see `osx.rs` and `main.rs`).

### Code Style Guidelines

#### Imports
- Group imports: std → external crates → local modules
- Use explicit imports, avoid glob imports (`use crate::*`)
- Example:
```rust
use std::{fs, path::PathBuf};
use anyhow::{Context, Result};
use walkdir::WalkDir;
use crate::types::AppInfo;
```

#### Naming Conventions
- **Functions/structs**: snake_case (e.g., `find_app_bundles`)
- **Types**: PascalCase (e.g., `AppInfo`, `TaskKind`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Modules**: snake_case

#### Error Handling
- Use `anyhow::Result<T>` for return types
- Use `Context` for error messages: `.context("descriptive message")?`
- Prefer early returns with `?` operator
- Example: `let data = fs::read_to_string(path).context("Failed to read file")?`

#### Documentation
- Use `//!` for module-level documentation
- Use `///` for public functions/structs
- Keep documentation concise and descriptive

#### Code Structure
- Use `#[derive(Clone, Debug)]` for data structures
- Use `#[allow(dead_code)]` for enums with potentially unused variants
- Prefer explicit types over inference for public APIs
- Use meaningful variable names (avoid single letters except in loops)

#### Safety & Best Practices
- Use `PathBuf` for path handling
- Validate file existence before operations
- Handle permissions gracefully with proper error messages
- Use `trash` crate for safe file deletion (moves to Trash)
- Check running processes before attempting uninstallation

#### Dependencies
- eframe/egui: GUI framework
- sysinfo: System information and process checking
- plist: macOS property list parsing
- trash: Safe file deletion
- walkdir: Directory traversal
- anyhow: Error handling

### Documentation & Version Lookup

**IMPORTANT**: Before making changes to any framework or library usage, always use MCP server Context7 to lookup the latest documentation and versions:

#### Core Dependencies to Check:
- **eframe** (current: 0.32) - GUI framework
- **egui** (current: 0.32) - Immediate mode GUI
- **sysinfo** (current: 0.37) - System information and process checking
- **plist** (current: 1.7) - macOS property list parsing
- **trash** (current: 5.2) - Safe file deletion
- **home** (current: 0.5) - Cross-platform home directory detection
- **anyhow** (current: 1.0) - Error handling
- **walkdir** (current: 2.5) - Directory traversal

#### macOS-Specific Dependencies:
- **cocoa** (current: 0.26) - macOS Cocoa framework bindings
- **objc** (current: 0.2) - Objective-C runtime bindings

#### Usage Instructions:
1. Lookup latest docs/versions via Context7 before changing framework usage
2. Review breaking changes and migration guides
3. Update Cargo.toml with latest compatible versions
4. Test thoroughly after dependency updates
5. Update this document with new version numbers after successful upgrades
