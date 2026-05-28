# UI Guide

App Uninstaller has a single window. The UI is composed entirely from shadcn/ui primitives — we do not author bespoke widgets when a shadcn primitive exists.

## Layout

```
┌──────────────────────────┬─────────────────────────────────────────────┐
│ Sidebar (240–300px)      │ Header                                      │
│  • Search                │  • Selected app name (left)                 │
│  • Scrollable app list   │  • Refresh · GitHub · Theme toggle (right)  │
│  • Sidebar footer:       ├─────────────────────────────────────────────┤
│    "N apps · v1.0.0"     │ Detail (fills remaining space)              │
│                          │  • App card                                 │
│                          │     ─ Bundle ID, Path (label/value rows)    │
│                          │     ─ Running warning banner (if running)   │
│                          │     ─ Footer: Reveal · Rescan · Uninstall   │
│                          │  • Related files card (flex-fills height)   │
│                          │     ─ Select all + N/M counter              │
│                          │     ─ Scrollable checklist                  │
└──────────────────────────┴─────────────────────────────────────────────┘
```

The window has no application footer / no log drawer. Background-task progress is surfaced as transient `sonner` toasts.

## Component map

| Region                            | shadcn primitive(s)                                    |
| --------------------------------- | ------------------------------------------------------ |
| Sidebar app row                   | `Item` + `ItemContent` + `ItemMedia` + `Tooltip` for the running lock icon |
| Sidebar search                    | `InputGroup` + `InputGroupInput` + `InputGroupAddon` + `InputGroupButton` (clear) |
| Sidebar scroll                    | `ScrollArea`                                           |
| Header buttons                    | `Button` (variant=ghost, size=icon-sm) + `Tooltip`     |
| GitHub link                       | `Button` rendered as `<a>` (via base-ui `render` prop) |
| App card / Related files card     | `Card` + `CardHeader` / `CardContent` / `CardFooter`   |
| Bundle ID / Version / Path / etc. | Inline `Label : Value` rows, styled via `STYLES.fieldRowLabel` + `STYLES.fieldRowValue` |
| Running warning                   | Plain `<div>` styled via shared `STYLES.warningBanner` (destructive tint, lock icon) |
| Name-based-scan notice            | Plain `<div>` styled via shared `STYLES.infoBanner` (amber tint, info icon) |
| Path checklist                    | `Checkbox` + `<label htmlFor>` + `ScrollArea` + `Tooltip` (full path) |
| Per-row actions                   | `DropdownMenu`                                         |
| Uninstall confirmation            | `AlertDialog`                                          |
| Toasts                            | `sonner` Toaster                                       |

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

- **Selecting an app** updates the header title and immediately fires `find_related` in the background.
- **Uninstall** is gated three ways: button is `variant="destructive"`, disabled when the app is running (running warning is shown above), and confirmed via `AlertDialog`.
- **Rescan related files** lets the user re-walk the Library locations without re-selecting the app.
- **Errors / completion** appear as `sonner` toasts. There is no persistent activity log.
- **GitHub button** opens [github.com/antruongnguyen/mac-app-uninstaller](https://github.com/antruongnguyen/mac-app-uninstaller) in the system browser via `<a target="_blank">`.

## What we deliberately don't build

- A custom theme switcher — `next-themes` + `lucide` Sun/Moon.
- A custom progress widget — task progress is surfaced as toasts, not a persistent bar.
- A custom modal — `AlertDialog` for destructive prompts.
- A bespoke list — sidebar rows are plain `Button` ghosts inside a `ScrollArea`.
- A custom GitHub icon component (we use a single 16×16 inline SVG). Lucide's brand icons were removed in v1, so this is the simplest option that doesn't add a second icon dependency.

## Files

- `src/App.tsx` — top-level CSS Grid (sidebar | header / sidebar | main).
- `src/components/header.tsx` — Header bar (selected app name + refresh/github/theme).
- `src/components/apps-sidebar.tsx` — Sidebar (search + list + footer with totals/version).
- `src/components/detail-panel.tsx` — App card + related-files card.
- `src/components/uninstall-confirm.tsx` — Wraps `AlertDialog`.
- `src/components/ui/*` — shadcn-generated primitives. Do not hand-edit.
- `src/lib/styles.ts` — `IDS`, `STYLES`, `REPO_URL`.
