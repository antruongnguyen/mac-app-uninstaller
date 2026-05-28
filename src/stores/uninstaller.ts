import { create } from "zustand";
import type { AppInfo, ProgressEvent } from "@/types/models";
import { uninstallerApi } from "@/lib/api/uninstaller";

interface AppsState {
  apps: AppInfo[];
  selectedPath: string | null;
  loading: boolean;
  error: string | null;

  fetchApps: () => Promise<void>;
  select: (path: string | null) => void;
}

export const useAppsStore = create<AppsState>((set) => ({
  apps: [],
  selectedPath: null,
  loading: false,
  error: null,

  fetchApps: async () => {
    set({ loading: true, error: null });
    try {
      const apps = await uninstallerApi.listApps();
      set({ apps, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  select: (path) => set({ selectedPath: path }),
}));

interface RelatedState {
  paths: string[];
  selected: Set<string>;
  loading: boolean;
  error: string | null;

  fetchRelated: (appName: string, bundleId: string | null) => Promise<void>;
  toggle: (path: string) => void;
  toggleAll: (checked: boolean) => void;
  clear: () => void;
}

export const useRelatedStore = create<RelatedState>((set, get) => ({
  paths: [],
  selected: new Set(),
  loading: false,
  error: null,

  fetchRelated: async (appName, bundleId) => {
    set({ loading: true, error: null, paths: [], selected: new Set() });
    try {
      const paths = await uninstallerApi.findRelated(appName, bundleId);
      set({ paths, selected: new Set(paths), loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  toggle: (path) => {
    const next = new Set(get().selected);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    set({ selected: next });
  },

  toggleAll: (checked) => {
    set({ selected: checked ? new Set(get().paths) : new Set() });
  },

  clear: () => set({ paths: [], selected: new Set(), error: null }),
}));

interface TaskState {
  current: ProgressEvent | null;
  setProgress: (event: ProgressEvent) => void;
}

export const useTaskStore = create<TaskState>((set) => ({
  current: null,
  setProgress: (event) => set({ current: event }),
}));
