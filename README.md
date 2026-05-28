# macOS App Uninstaller (Tauri + React)

A lightweight macOS desktop app that lets you:

- List all `.app` bundles installed in `/Applications` and `~/Applications`.
- Detect running apps (so you don't try to uninstall them).
- Find and pick related files for deletion (LaunchAgents, Logs, Preferences, Receipts, Containers, …).
- Move the app and its related items to **Trash** instead of deleting them outright.
- Stream progress updates and keep a status log while work runs in the background.

The original `egui` implementation was rewritten on top of [Tauri 2](https://tauri.app/) with a [React](https://react.dev) + [shadcn/ui](https://ui.shadcn.com) frontend. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/TAURI_MIGRATION.md](docs/TAURI_MIGRATION.md) for the rationale and design notes.

---

## System requirements

- macOS 10.15 Catalina or later
- [Rust toolchain](https://www.rust-lang.org/tools/install) (Edition 2024 / `rust-version = 1.85`)
- [Bun](https://bun.sh) (or any Node 18+ package manager — substitute `npm` / `pnpm` for `bun`)
- Xcode Command Line Tools: `xcode-select --install`

The Tauri CLI is installed automatically as a dev dependency the first time you run `bun install`.

---

## Project layout

```
mac_uninstaller/
├── docs/                       Architecture, migration log, UI guide
├── src/                        React frontend (TS + Tailwind v4 + shadcn/ui)
│   ├── components/             Page-level components composed from shadcn/ui
│   ├── lib/                    `cn()`, `tauriInvoke`, API wrappers
│   ├── stores/                 Zustand stores
│   └── types/                  TS DTOs mirrored from Rust
├── src-tauri/                  Tauri backend (Rust)
│   ├── src/core/               Pure logic (scan, plist, related, trash)
│   ├── src/commands.rs         #[tauri::command] handlers
│   ├── src/models.rs           Serde DTOs shared with the frontend
│   └── tauri.conf.json         Bundle metadata + window config
├── components.json             shadcn config
├── package.json
└── vite.config.ts
```

---

## Common commands

| Task                       | Command                                                  |
| -------------------------- | -------------------------------------------------------- |
| Install dependencies       | `bun install`                                            |
| Run dev shell (recommended)| `bun run tauri dev`                                      |
| Run frontend only (browser)| `bun run dev` (no Tauri IPC available)                   |
| Build production bundle    | `bun run tauri build` → `src-tauri/target/release/bundle/macos/App Uninstaller.app` |
| Type-check + Vite build    | `bun run build`                                          |
| Lint                       | `bun run lint`                                           |
| Run frontend tests         | `bun run test`                                           |
| Run Rust tests             | `cargo test --manifest-path src-tauri/Cargo.toml`        |

Bundle metadata (name, identifier `day.nhanh.appuninstaller`, icon, window size, macOS minimum version) lives in `src-tauri/tauri.conf.json`.

---

## Permissions

App Uninstaller needs **Full Disk Access** to read system locations like `/private/var/db/receipts` and to move items there to the Trash. Grant it under **System Settings → Privacy & Security → Full Disk Access**, and ideally **App Management** too.

---

## Scanned locations

User Library:
- `~/Library/Application Support/<bundle_id | app_name>`
- `~/Library/Caches/<bundle_id | app_name>`
- `~/Library/Preferences/<bundle_id>.plist`
- `~/Library/Containers/<bundle_id>`
- `~/Library/Logs/<app_name>`
- `~/Library/LaunchAgents/<bundle_id | app_name>*`

System Library:
- `/Library/Application Support/<bundle_id>`
- `/Library/Preferences/<bundle_id>.plist`
- `/Library/Receipts/<app_name>*`
- `/private/var/db/receipts/<bundle_id | app_name>*`

---

## Author

**An Nguyen** — annguyen.apps@gmail.com — https://nhanh.day

## License

MIT — you are free to modify and distribute, but you must include the original author information.
