# UI Guide

App Uninstaller has a single window. The UI is composed entirely from shadcn/ui primitives — we do not author bespoke widgets when a shadcn primitive exists.

## Layout

```
┌──────────────────────────┬─────────────────────────────────────────────┐
│ Sidebar (240–300px)      │ Header                                      │
│  • Search w/ clear button│  • Selected app name (left)                 │
│  • Scrollable Item list  │  • Refresh · GitHub · Theme toggle (right)  │
│    each row has a lock   ├─────────────────────────────────────────────┤
│    icon if running       │ Detail (fills remaining space)              │
│  • Sidebar footer:       │  • App card                                 │
│    "N apps · v1.0.0"     │     ─ Running warning banner (if running)   │
│                          │     ─ Bundle ID, Version, Executable, Size, │
│                          │       Last modified, Path (label/value rows)│
│                          │     ─ Footer: Quit (if running) ·           │
│                          │       Reveal · Scan files · Uninstall       │
│                          │  • Related files card (flex-fills height)   │
│                          │     ─ Name-based-scan info banner           │
│                          │     ─ Select all + N/M counter              │
│                          │     ─ Scrollable checklist                  │
└──────────────────────────┴─────────────────────────────────────────────┘
```

The window has no application footer / no log drawer. Background-task progress is surfaced as transient `sonner` toasts.

## Component map

| Region                        | shadcn primitive(s)                                                                                |
| ----------------------------- | -------------------------------------------------------------------------------------------------- |
| Sidebar app row               | `Item` + `ItemContent` + `ItemActions` for the running lock icon (+ `Tooltip`)                     |
| Sidebar app row title         | `ItemTitle` (name) + inline `<span>` (version, font-mono) + `ItemDescription` (bundle id)          |
| Sidebar search                | `InputGroup` + `InputGroupInput` + `InputGroupAddon` + `InputGroupButton` (clear)                  |
| Sidebar scroll                | `ScrollArea`                                                                                       |
| Header buttons                | `Button` (variant=ghost, size=icon-sm) + `Tooltip`                                                 |
| GitHub button                 | `Button` with `onClick` calling `openUrl(REPO_URL)` from `@tauri-apps/plugin-opener`               |
| App card / Related files card | `Card` + `CardHeader` / `CardContent` / `CardFooter`                                               |
| App card body rows            | Inline `Label : Value` rows, styled via `STYLES.fieldRowLabel` + `STYLES.fieldRowValue`            |
| App card Size field           | Same row pattern, but lazy — shows `LoaderCircleIcon` + "Calculating…" while `useAppSize` resolves |
| Running warning               | Plain `<div>` styled via shared `STYLES.warningBanner` (destructive tint, lock icon)               |
| Name-based-scan notice        | Plain `<div>` styled via shared `STYLES.infoBanner` (amber tint, info icon)                        |
| Path checklist                | `Checkbox` + `<label htmlFor>` + `ScrollArea` + `Tooltip` (full path)                              |
| Per-row reveal action         | `Button` (variant=ghost, size=icon-xs) + `FolderOpenIcon` + `Tooltip`                              |
| Uninstall confirmation        | `AlertDialog` via `<UninstallConfirm>`                                                             |
| Quit confirmation             | Inline `AlertDialog` in `detail-panel.tsx`                                                         |
| Toasts                        | `sonner` Toaster                                                                                   |

## Theming

- shadcn `base-nova` style, `neutral` base color (see `components.json`). No overrides.
- Light/dark via `next-themes`. The header toggle switches the `.dark` class on `<html>`.
- All colour tokens come from the `@theme inline` block in `src/index.css`. New colours go through that block, not as inline values.
- Spacing/radius from `--radius: 0.625rem` and the derived radius scale.

## Typography

- **Font:** Geist Variable, loaded through `@fontsource-variable/geist` (cacheable, self-hosted; equivalent to the Google-Fonts version of Geist).
- The CSS exposes Geist via `--font-sans` / `--font-heading`. shadcn's default classes inherit them.
- Path/bundle-id values use `font-mono` (system monospace stack).

## Shared styles & IDs

Repeated style strings and stable element ids live in `src/lib/styles.ts`:

- **`IDS`** — every interactive/structural element gets a stable id (`app-header`, `app-detail-uninstall`, …). Per-row ids are derived from path (`IDS.sidebarAppRow(path)`). Tests and DOM scripts target these instead of class names or text.
- **`STYLES`** — class strings used in more than one place: `rowHover`, `fieldRowLabel`, `fieldRowValue`, `warningBanner`, `infoBanner`, `flexFillCard`, `flexFillList`. Local-only classes stay inline.
- **`REPO_URL`** — single source of truth for the GitHub link.

## Interaction patterns

- **Selecting an app** updates the header title, immediately fires `find_related` in the background, and triggers `useAppSize` to compute the bundle's recursive size.
- **Refresh** is fired three ways: on initial mount, by clicking the header refresh button, and automatically when the window regains focus (`window.addEventListener("focus", …)`, gated on a not-already-loading flag).
- **Quit** appears in the app-card footer only when `app.running` is true. Click → confirmation `AlertDialog` → `kill_app` SIGKILLs all matching processes and waits for the kernel to reap them, so by the time the call resolves the running indicator is genuinely current. The next `fetchApps()` clears the lock icon, the warning banner, and the Quit button itself.
- **Uninstall** is gated three ways: button is `variant="destructive"`, disabled when the app is running (running warning is shown above), and confirmed via `AlertDialog`.
- **Scan files** lets the user re-walk the Library locations without re-selecting the app.
- **Errors / completion** appear as `sonner` toasts. There is no persistent activity log.
- **GitHub button** opens [github.com/antruongnguyen/mac-app-uninstaller](https://github.com/antruongnguyen/mac-app-uninstaller) in the system browser via `tauri-plugin-opener`'s `openUrl`. A plain `<a target="_blank">` would be silently dropped inside the Tauri webview — see `docs/ARCHITECTURE.md`.

## What we deliberately don't build

- A custom theme switcher — `next-themes` + `lucide` Sun/Moon.
- A custom progress widget — task progress is surfaced as toasts, not a persistent bar.
- A custom modal — `AlertDialog` for destructive prompts (Uninstall, Quit).
- A bespoke list — sidebar rows use shadcn `Item` inside a `ScrollArea`. The heavyweight shadcn `Sidebar` primitive is intentionally not used because the sidebar is non-collapsible and a 723-line component is overkill for that.
- A per-row dropdown menu — the only per-row action is "Reveal in Finder", so a single ghost icon button with a tooltip is clearer than a dropdown wrapping one item. The `dropdown-menu.tsx` primitive was removed when its last call site went away.
- A custom GitHub icon component (we use a single 16×16 inline SVG). Lucide's brand icons were removed in v1, so this is the simplest option that doesn't add a second icon dependency.
- A polling refresh timer — focus-refresh covers the same cases without UI flashing or wasted CPU. See `docs/ARCHITECTURE.md` for the analysis.

## Files

- `src/App.tsx` — top-level CSS Grid (sidebar | header / sidebar | main); window-focus listener.
- `src/components/header.tsx` — Header bar (selected app name + refresh/github/theme).
- `src/components/apps-sidebar.tsx` — Sidebar (search + `Item` list + footer with totals/version).
- `src/components/detail-panel.tsx` — App card (with Quit confirmation `AlertDialog`) + related-files card.
- `src/components/uninstall-confirm.tsx` — Wraps `AlertDialog` for the uninstall flow.
- `src/components/ui/*` — shadcn-generated primitives. Do not hand-edit.
- `src/hooks/use-app-size.ts` — lazy bundle-size lookup with a process-wide cache.
- `src/hooks/use-is-truncated.ts` — `[ref, boolean]`; tooltips only when truncation is actually clipping.
- `src/lib/styles.ts` — `IDS`, `STYLES`, `REPO_URL`.
