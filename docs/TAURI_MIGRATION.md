# Tauri Migration

This document tracks the decisions made while migrating App Uninstaller from `egui` to a Tauri + React + shadcn/ui stack. It is a living log — entries are appended as choices are made or revised.

## Reference project

We mirrored the structure of [`teaching-management-system`](../../teaching-management-system) (a sibling Tauri 2 project), adapting it to App Uninstaller's much smaller surface area. Where teaching-management-system uses SQLite + 50+ commands, we have a single domain (installed apps) and ~5 commands, so we keep the layout but trim the moving parts (e.g. no `db/`, no migrations, no i18n).

## Stack decisions

| Concern              | Choice                                                           | Why                                                                                                                                                                                                                                                                                            |
| -------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Shell                | Tauri 2.x                                                        | Native window, smaller bundle than Electron, Rust backend keeps existing core logic.                                                                                                                                                                                                           |
| Frontend framework   | React 19 + TypeScript                                            | Matches reference project; required by shadcn/ui.                                                                                                                                                                                                                                              |
| Bundler              | Vite 8 + `@vitejs/plugin-react`                                  | Matches reference project. Fast HMR.                                                                                                                                                                                                                                                           |
| CSS                  | Tailwind CSS v4 (`@tailwindcss/vite`)                            | Matches reference project. v4 supports the `@theme inline` token block we copy from shadcn-base-nova.                                                                                                                                                                                          |
| Component library    | shadcn/ui (`base-nova` style, `neutral` base color)              | Goal explicitly says: prefer shadcn over custom UI. We adopt the shadcn defaults (no overrides) so updates stay easy.                                                                                                                                                                          |
| Icon set             | `lucide-react`                                                   | shadcn default; matches reference project.                                                                                                                                                                                                                                                     |
| State                | Zustand                                                          | Matches reference project; lighter than Redux for our few stores.                                                                                                                                                                                                                              |
| Toasts               | `sonner` (re-exported through shadcn)                            | shadcn default; less custom CSS than building our own status banner.                                                                                                                                                                                                                           |
| Theming              | `next-themes` + `oklch()` tokens                                 | Same approach as reference project; gives us light/dark with one toggle.                                                                                                                                                                                                                       |
| Font                 | Geist Variable via `@fontsource-variable/geist`                  | shadcn's default sans recommendation. Geist is published by Vercel and mirrored on Google Fonts; the `@fontsource-variable` package self-hosts the same files so the desktop app does not depend on `fonts.gstatic.com` at runtime. Net effect: cacheable Google-style font, offline-friendly. |
| Package manager      | Bun                                                              | Matches reference project (`bun.lock`, `bun run …`).                                                                                                                                                                                                                                           |
| Tauri plugins        | `tauri-plugin-dialog`, `tauri-plugin-log`, `tauri-plugin-opener` | Confirmation dialogs, structured logging, and OS-level URL opening for the GitHub link. We do **not** need `tauri-plugin-fs` because all filesystem access is server-side in Rust. The opener plugin was added later — see "Post-migration changes" below.                                     |
| Rust crates retained | `plist`, `trash`, `walkdir`, `sysinfo`, `home`, `anyhow`         | Same versions as before. No reason to change them — only the surrounding plumbing has moved.                                                                                                                                                                                                   |
| Rust crates dropped  | `eframe`, `egui`, `cocoa`, `objc`                                | All replaced by Tauri / web UI. The dock-icon code in `osx.rs` is no longer needed because Tauri handles the icon through `tauri.conf.json`.                                                                                                                                                   |

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

## Post-migration changes

These entries document architectural decisions made after the initial migration shipped, in the order they were made.

### Lazy bundle-size lookup

The first version eagerly computed each bundle's recursive size during `list_apps` so the detail card could show it without an extra round-trip. This regressed scan time from ~200 ms to many seconds: macOS bundles range from a few MB (Safari) to ~5 GB (Xcode, ~700 000 files), and walking every interior file across 50+ apps blocks the UI. The fix split bundle-size into its own `get_app_size` Tauri command, dropped `size_bytes` from `AppInfo`, and added a `useAppSize` hook that fires on selection and caches by path. The Size field shows a "Calculating…" spinner while the walk is in flight, then the formatted bytes; re-selecting a previously inspected app is instant.

The general rule — encoded in `CLAUDE.md` so a future session doesn't reintroduce it — is that `list_apps` must stay cheap: per-item cost is bounded to `read_dir` + `Info.plist` parse + sysinfo match. Anything that walks bundle interiors becomes its own lazy command.

### Quit (SIGKILL) flow

A user-requested **Quit** button was added to the app card footer (visible only when the selected app is running). It surfaces a confirmation `AlertDialog` and on confirm calls a new `kill_app` command that sends SIGKILL to all processes matched by the same heuristics `is_app_running` uses. To avoid a UX bug where the running indicator stayed lit for a beat after the kill, `kill_app` polls a fresh `System::new_all()` every 50 ms (capped at 2 s) until the targeted PIDs disappear from the snapshot before returning. The user-visible label is "Quit" though the underlying mechanism is SIGKILL — the user explicitly requested this naming.

### Window-focus refresh, no overlay

An earlier iteration added a full-screen loading overlay tied to the apps store's `loading` flag, plus support for a polling refresh timer. Both were dropped:

- A polling refresh would flash the overlay every cycle and waste CPU re-scanning state that doesn't change between user actions. The only field that actually changes between refreshes is `running: bool`, which doesn't justify the cost or the visual disruption.
- Focus-refresh (`window.addEventListener("focus", fetchApps)`, gated on a not-already-loading flag) covers the same use case — the user comes back from another app and sees fresh state — at zero cost while the window is backgrounded.

The loading overlay was removed entirely; the header refresh button still spins its icon during loading, and the sidebar shows skeleton rows on the very first load.

### `tauri-plugin-opener` for the GitHub button

The header's GitHub button was first implemented as a `<Button render={<a href={REPO_URL} target="_blank" />}>`. This silently does nothing inside the Tauri webview: there are no tabs and `window.open` returns `null`. Switched to `tauri-plugin-opener`'s `openUrl(REPO_URL)` from a plain `onClick`, which hands the URL to `NSWorkspace -openURL:` and opens the user's default browser. The capability is scoped tightly: `opener:allow-open-url` only allows `https://github.com/*` so the same permission can't be exploited to open arbitrary URIs.

### IDs and shared styles

Per a user instruction ("every UI element will have an ID and the similar style will be extracted into the common style"), `src/lib/styles.ts` was added as the single source of truth for `IDS` (stable element ids — every interactive/structural element gets one) and `STYLES` (class strings used in more than one place). Per-row ids use a slugify helper (`IDS.sidebarAppRow(path)`) so paths don't end up in DOM `id` attributes verbatim. This convention is documented in `CLAUDE.md` and is expected to apply to all future UI work.

### Stronger running detection

The original port mirrored the egui heuristic for matching processes to apps: `proc.name()` against `CFBundleName` or the bundle id's last segment, plus a few substring fallbacks against the executable path and cmdline. Two failure modes showed up:

- **False negatives on Electron/Chromium apps.** VS Code's process name is `Electron`, not "Visual Studio Code"; the bundle id's last segment (`vscode`) didn't match either. Detection only fired through the `/<name>.app/` path check on `proc.exe()`, which sysinfo can't always read.
- **False positives on substring matches.** `cmdline.contains(bundle_id)` would attribute `log show --predicate 'subsystem == "com.apple.mail"'` to Mail. Bare-token name matches (`" mail "`) had the same problem for short app names.

Two changes addressed both: `MatchKeys` gained an `executable` field (lowercased `CFBundleExecutable` from `Info.plist`) which is checked first against `proc.name()` — this is the highest-confidence key on macOS and catches the Electron case directly. Bundle-id substring checks were replaced with a `matches_bundle_id_boundary` helper that requires the id to appear as a path component (`/<bid>/`, `/<bid>.app`, `=<bid>`) rather than anywhere in the string. The bare-token cmdline name match was dropped entirely; the path-bounded `/<name>.app` check still covers legitimate `open -a` invocations.

The asymmetry between the scan-time call (which has all three keys) and the on-demand `is_app_running` Tauri command (which only carries bundle id + name from the frontend) is intentional — keeping the public command signature stable was worth more than threading the executable through every IPC boundary. The frontend never needed it: the scan reports `running: bool` directly, and the on-demand checks are pre-uninstall guards where the slightly weaker match is acceptable. See `docs/ARCHITECTURE.md` "Running detection" for the matching order.

A native source of truth (`NSWorkspace.runningApplications` via `objc2`) was considered and deferred. It's the right long-term answer — Activity Monitor uses it — but adds an Objective-C dependency for an asymmetric win: it would resolve all bundle-app cases cleanly but stops detecting non-bundled CLI processes that are still useful to flag. Reach for it if specific apps still slip through the current heuristics.

### Path-based running detection

The previous iteration still relied on string keys derived from `Info.plist`. A user reported a concrete failure mode that the keys could not fix: with the `claude` CLI running in a terminal and `Claude.app` _not_ running, the scanner reported the app as running, and clicking Quit killed every CLI session. Cause: `Claude.app`'s `CFBundleExecutable` is `Claude`, the CLI's `proc.name()` is `claude`, and the case-insensitive equality check fired on the first comparison. Any short or generic executable name has the same shape (the `code` CLI for VS Code, the `slack` CLI for Slack, etc).

The fix replaces all string heuristics with **`proc.exe()?.starts_with(bundle_path)`** as the authoritative match. macOS spawns app processes from `<bundle>/Contents/MacOS/`, so a process whose absolute exe path is inside the bundle belongs to the bundle, full stop — no name collisions, no substring tricks. The `Info.plist`-keyed heuristics still exist but are only consulted when `proc.exe()` is unreadable (rare, mostly kernel/system processes). Crucially, when the exe path _is_ readable and the path doesn't match, no fallback can override that — the previous design's mistake was letting weaker keys vote against a clear path signal.

This required threading `app_path` through every entry point: the scan-time call already had it, but `is_app_running_simple`, `kill_app`, and the corresponding Tauri commands all gained an `app_path: Option<PathBuf>` parameter. The frontend wrapper signatures changed (`isAppRunning(appPath, appName, bundleId)`, `killApp(appPath, appName, bundleId)`) and the one call site (`detail-panel.tsx::handleQuit`) was updated to pass `app.path`. The pre-uninstall guard already had `app_path` in scope and just needed to forward it.

This makes the asymmetry from the previous "Stronger running detection" entry obsolete: scan-time, on-demand, and kill all use the same `process_matches` predicate with the same authoritative path key, so the running indicator, the uninstall guard, and SIGKILL targeting cannot disagree.
