/**
 * DTOs mirrored from `src-tauri/src/models.rs`. Keep field names in sync.
 */

export interface AppInfo {
  path: string;
  name: string;
  bundleId: string | null;
  version: string | null;
  executable: string | null;
  /** Last-modified time as a Unix timestamp (seconds); `null` when unreadable. */
  modifiedAt: number | null;
  running: boolean;
}

export interface UninstallFailure {
  path: string;
  error: string;
}

export interface UninstallReport {
  appPath: string;
  removed: string[];
  failed: UninstallFailure[];
  aborted: boolean;
}

export type ProgressKind = "refresh_apps" | "find_related" | "uninstall";

export interface ProgressEvent {
  kind: ProgressKind;
  progress: number;
  message: string;
  finished: boolean;
  error: string | null;
}
