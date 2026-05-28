import { useEffect, useRef, useState } from "react";
import { uninstallerApi } from "@/lib/api/uninstaller";

/**
 * Resolve an app bundle's recursive size on demand. Returns the cached value
 * for paths we've already measured this session, otherwise fires a fresh
 * `get_app_size` and reports `loading` until it resolves.
 *
 * Walking large bundles (Xcode, Office) is expensive, so the scan does not
 * include this — we only pay the cost for the app the user actually selects.
 */
const sizeCache = new Map<string, number | null>();

export function useAppSize(path: string | null) {
  // `tick` is bumped whenever a fetch completes, so a re-render picks up the
  // new cached value. `size` and `loading` are otherwise derived from `path`
  // + the cache during render, avoiding setState-in-effect entirely.
  const [, bumpTick] = useState(0);
  const requestId = useRef(0);

  useEffect(() => {
    if (!path || sizeCache.has(path)) return;

    const id = ++requestId.current;

    uninstallerApi
      .getAppSize(path)
      .then((value) => {
        sizeCache.set(path, value);
        if (id !== requestId.current) return; // newer selection won; drop stale result
        bumpTick((n) => n + 1);
      })
      .catch(() => {
        if (id !== requestId.current) return;
        sizeCache.set(path, null);
        bumpTick((n) => n + 1);
      });
  }, [path]);

  if (!path) return { size: null, loading: false };
  if (sizeCache.has(path))
    return { size: sizeCache.get(path) ?? null, loading: false };
  return { size: null, loading: true };
}
