import { useEffect } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
import { Header } from "@/components/header";
import { AppsSidebar } from "@/components/apps-sidebar";
import { DetailPanel } from "@/components/detail-panel";
import { useAppsStore, useRelatedStore, useTaskStore } from "@/stores/uninstaller";
import { tauriListen } from "@/lib/tauri";
import { IDS } from "@/lib/styles";
import type { AppInfo, ProgressEvent } from "@/types/models";

export default function App() {
  const fetchApps = useAppsStore((s) => s.fetchApps);
  const select = useAppsStore((s) => s.select);
  const fetchRelated = useRelatedStore((s) => s.fetchRelated);
  const setProgress = useTaskStore((s) => s.setProgress);
  const loading = useAppsStore((s) => s.loading);

  useEffect(() => {
    fetchApps();
  }, [fetchApps]);

  // Refresh whenever the user returns to the window. Skips while a refresh is
  // already in flight so rapid cmd-tabbing doesn't pile up duplicate scans.
  useEffect(() => {
    const onFocus = () => {
      if (!useAppsStore.getState().loading) {
        fetchApps();
      }
    };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [fetchApps]);

  useEffect(() => {
    const unlisten = tauriListen<ProgressEvent>("progress", setProgress);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [setProgress]);

  function handleSelect(app: AppInfo) {
    select(app.path);
    fetchRelated(app.name, app.bundleId);
  }

  return (
    <TooltipProvider>
      <div
        id={IDS.app}
        className="grid h-screen grid-cols-[minmax(240px,300px)_1fr] grid-rows-[auto_1fr] bg-background text-foreground"
      >
        <div className="row-span-2 h-screen overflow-hidden">
          <AppsSidebar onSelect={handleSelect} />
        </div>
        <Header onRefresh={fetchApps} refreshing={loading} />
        <main className="overflow-hidden">
          <DetailPanel />
        </main>
      </div>
      <Toaster richColors />
    </TooltipProvider>
  );
}
