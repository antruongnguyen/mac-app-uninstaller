# Tauri Migration

This document tracks the decisions made while migrating App Uninstaller from `egui` to a Tauri + React + shadcn/ui stack. It is a living log — entries are appended as choices are made or revised.

## Reference project

We mirrored the structure of [`teaching-management-system`](../../teaching-management-system) (a sibling Tauri 2 project), adapting it to App Uninstaller's much smaller surface area. Where teaching-management-system uses SQLite + 50+ commands, we have a single domain (installed apps) and ~5 commands, so we keep the layout but trim the moving parts (e.g. no `db/`, no migrations, no i18n).

## Stack decisions

| Concern              | Choice                                                  | Why                                                                                                                                                |
| -------------------- | ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Shell                | Tauri 2.x                                               | Native window, smaller bundle than Electron, Rust backend keeps existing core logic.                                                               |
| Frontend framework   | React 19 + TypeScript                                   | Matches reference project; required by shadcn/ui.                                                                                                  |
| Bundler              | Vite 8 + `@vitejs/plugin-react`                         | Matches reference project. Fast HMR.                                                                                                               |
| CSS                  | Tailwind CSS v4 (`@tailwindcss/vite`)                   | Matches reference project. v4 supports the `@theme inline` token block we copy from shadcn-base-nova.                                              |
| Component library    | shadcn/ui (`base-nova` style, `neutral` base color)     | Goal explicitly says: prefer shadcn over custom UI. We adopt the shadcn defaults (no overrides) so updates stay easy.                              |
| Icon set             | `lucide-react`                                          | shadcn default; matches reference project.                                                                                                         |
| State                | Zustand                                                 | Matches reference project; lighter than Redux for our few stores.                                                                                  |
| Toasts               | `sonner` (re-exported through shadcn)                   | shadcn default; less custom CSS than building our own status banner.                                                                               |
| Theming              | `next-themes` + `oklch()` tokens                        | Same approach as reference project; gives us light/dark with one toggle.                                                                           |
| Font                 | Geist Variable via `@fontsource-variable/geist`         | shadcn's default sans recommendation. Geist is published by Vercel and mirrored on Google Fonts; the `@fontsource-variable` package self-hosts the same files so the desktop app does not depend on `fonts.gstatic.com` at runtime. Net effect: cacheable Google-style font, offline-friendly. |
| Package manager      | Bun                                                     | Matches reference project (`bun.lock`, `bun run …`).                                                                                               |
| Tauri plugins        | `tauri-plugin-dialog`, `tauri-plugin-log`               | We need a confirmation dialog before uninstall and structured logging. We do **not** need `tauri-plugin-fs` because all filesystem access is server-side in Rust. |
| Rust crates retained | `plist`, `trash`, `walkdir`, `sysinfo`, `home`, `anyhow`| Same versions as before. No reason to change them — only the surrounding plumbing has moved.                                                       |
| Rust crates dropped  | `eframe`, `egui`, `cocoa`, `objc`                       | All replaced by Tauri / web UI. The dock-icon code in `osx.rs` is no longer needed because Tauri handles the icon through `tauri.conf.json`.       |

## Open decisions

- **Routing:** the app currently has only one screen. We skip `react-router-dom` entirely; if a settings screen is added later we will introduce it then.
- **i18n:** out of scope for the initial migration. The reference project uses `react-i18next`, but App Uninstaller currently has English-only strings hard-coded. Strings live in components for now; we can extract them later without changing the architecture.

## Behaviour-preserving choices

The following pieces of logic are preserved verbatim from the egui version because they are observable behaviour:

- The list of candidate app directories: `/Applications`, `$HOME/Applications`.
- The plist keys read for app metadata: `CFBundleIdentifier`, `CFBundleName`, falling back to `CFBundleDisplayName`.
- The library locations scanned for related files (`Application Support`, `Caches`, `Preferences`, `Containers`, `Logs`, `LaunchAgents`, plus `/Library/Receipts` and `/private/var/db/receipts`).
- The two-phase uninstall (protected → abort on first failure; unprotected → continue on per-item errors).
- Auto-refresh of the apps list after uninstall completes.

These are tracked by tests in `src-tauri/src/core/` so a future refactor cannot silently regress them.

## Things that intentionally change

- **Visual design.** Goal explicitly allows dropping the AppKit-mimicking palette. We adopt shadcn's `neutral` base colour and the `base-nova` component style.
- **Fonts.** The egui version used the platform default. We now ship Geist Variable.
- **Packaging.** `cargo bundle` is replaced by `bun run tauri build` (uses `cargo-tauri` under the hood). `tauri.conf.json` carries the bundle metadata that previously lived in `[package.metadata.bundle]`.
- **Dock icon plumbing.** The hand-written `osx.rs` (`cocoa`/`objc` calls to `NSApplication setApplicationIconImage:`) is no longer needed; Tauri sets the icon from `icons/` in `tauri.conf.json`.

## Migration checklist

The migration is broken into the following commits/steps so each one can be reviewed in isolation:

1. **Docs first.** Land `docs/ARCHITECTURE.md`, `docs/TAURI_MIGRATION.md`, `docs/UI.md` describing the target.
2. **Scaffold Tauri shell.** Add `package.json`, `vite.config.ts`, `tsconfig*`, `index.html`, `src/main.tsx`, `src/App.tsx`, `src/index.css`, `components.json`, `src-tauri/` with an empty command list. Confirm `bun run tauri dev` boots a blank window.
3. **Port core logic.** Move `core.rs` into `src-tauri/src/core/` (broken up into `apps.rs`, `related.rs`, `running.rs`, `trash.rs`). Add unit tests for the pure functions.
4. **Wire commands.** Add command handlers in `src-tauri/src/commands/` and the progress emitter. Add a typed wrapper in `src/lib/api/uninstaller.ts`.
5. **Build UI.** Compose pages from shadcn primitives — `Card`, `Table`, `Checkbox`, `Button`, `Progress`, `ScrollArea`, `Dialog`, `Sonner`. No bespoke widgets unless shadcn doesn't cover the use case.
6. **Remove legacy.** Delete the egui sources (`src/`, root `Cargo.toml`/`Cargo.lock`, `style.rs`, `osx.rs`, `ui/`, `resources/`, `svg-to-icns.sh`, `target/`). Update `README.md` and `CLAUDE.md`.
7. **Verify.** `bun run build`, `cargo test --manifest-path src-tauri/Cargo.toml`, manual smoke test in `bun run tauri dev`.
