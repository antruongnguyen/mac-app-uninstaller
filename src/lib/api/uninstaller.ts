import { tauriInvoke } from "@/lib/tauri";
import type { AppInfo, UninstallReport } from "@/types/models";

export const uninstallerApi = {
  listApps: () => tauriInvoke<AppInfo[]>("list_apps"),

  findRelated: (appName: string, bundleId: string | null) =>
    tauriInvoke<string[]>("find_related", { bundleId, appName }),

  isAppRunning: (appName: string | null, bundleId: string | null) =>
    tauriInvoke<boolean>("is_app_running", { bundleId, appName }),

  killApp: (appName: string, bundleId: string | null) =>
    tauriInvoke<number>("kill_app", { bundleId, appName }),

  getAppSize: (path: string) =>
    tauriInvoke<number | null>("get_app_size", { path }),

  uninstall: (
    appPath: string,
    appName: string,
    bundleId: string | null,
    relatedPaths: string[],
  ) =>
    tauriInvoke<UninstallReport>("uninstall", {
      appPath,
      appName,
      bundleId,
      relatedPaths,
    }),

  revealInFinder: (path: string) => tauriInvoke<void>("reveal_in_finder", { path }),
};
