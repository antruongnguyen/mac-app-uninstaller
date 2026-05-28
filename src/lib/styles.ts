/**
 * Stable element IDs and shared Tailwind class fragments used across components.
 *
 * - `IDS`: every interactive/structural element gets a stable id so tests, hooks,
 *   and DOM queries can target it without relying on text content.
 * - `STYLES`: class strings used in more than one place. Local-only classes stay
 *   inline at the call site; we don't pre-extract everything just because we can.
 */

export const REPO_URL = "https://github.com/antruongnguyen/mac-app-uninstaller";

export const IDS = {
  app: "app-root",

  header: "app-header",
  headerTitle: "app-header-title",
  headerActions: "app-header-actions",
  headerRefresh: "app-header-refresh",
  headerGithub: "app-header-github",
  headerThemeToggle: "app-header-theme-toggle",

  sidebar: "app-sidebar",
  sidebarSearch: "app-sidebar-search",
  sidebarList: "app-sidebar-list",
  sidebarFooter: "app-sidebar-footer",
  sidebarAppRow: (path: string) => `app-row-${cssId(path)}`,
  sidebarRunningIcon: (path: string) => `app-row-running-${cssId(path)}`,

  detail: "app-detail",
  detailEmpty: "app-detail-empty",
  detailAppCard: "app-detail-app-card",
  detailRunningWarning: "app-detail-running-warning",
  detailBundleId: "app-detail-bundle-id",
  detailVersion: "app-detail-version",
  detailExecutable: "app-detail-executable",
  detailSize: "app-detail-size",
  detailModified: "app-detail-modified",
  detailPath: "app-detail-path",
  detailReveal: "app-detail-reveal",
  detailRescan: "app-detail-rescan",
  detailQuit: "app-detail-quit",
  detailUninstall: "app-detail-uninstall",

  relatedCard: "app-related-card",
  relatedScanNotice: "app-related-scan-notice",
  relatedSelectAll: "app-related-select-all",
  relatedList: "app-related-list",
  relatedRow: (path: string) => `app-related-row-${cssId(path)}`,
  relatedRowCheckbox: (path: string) => `app-related-row-cb-${cssId(path)}`,
  relatedRowMenu: (path: string) => `app-related-row-menu-${cssId(path)}`,

  confirmDialog: "app-confirm-dialog",
  confirmCancel: "app-confirm-cancel",
  confirmConfirm: "app-confirm-confirm",

  quitDialog: "app-quit-dialog",
  quitCancel: "app-quit-cancel",
  quitConfirm: "app-quit-confirm",
} as const;

/**
 * DOM-safe slug derived from a filesystem path. Not unique across the OS, but
 * unique enough within a single visible app/related-files list.
 */
function cssId(s: string): string {
  return s.replace(/[^a-zA-Z0-9_-]/g, "_");
}

export const STYLES = {
  /** Sidebar / detail row hover state. */
  rowHover: "transition-colors hover:bg-muted",

  /** Label cell of an inline `Label: Value` row in the app card. */
  fieldRowLabel:
    "shrink-0 w-28 text-xs font-medium uppercase tracking-wide text-muted-foreground",

  /** Value cell of an inline `Label: Value` row. */
  fieldRowValue: "flex-1 min-w-0 font-mono text-xs break-all text-foreground",

  /** A subtle warning banner (running, errors). */
  warningBanner:
    "flex items-center gap-2 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive",

  /** A subtle informational banner (advisory text). */
  infoBanner:
    "flex items-center gap-2 rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-700 dark:text-amber-300",

  /** Card that should grow to fill the remaining height of its parent flex column. */
  flexFillCard: "flex flex-1 min-h-0 flex-col",

  /**
   * Full-height scrollable list inside `flexFillCard`. The `min-h-32` floor
   * (128px) guarantees at least ~4 related-file rows are visible at the
   * window's minimum height (820px).
   */
  flexFillList: "flex-1 min-h-32 rounded-md border",
} as const;
