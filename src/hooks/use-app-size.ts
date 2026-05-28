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
  const [size, setSize] = useState<number | null>(() =>
    path ? sizeCache.get(path) ?? null : null,
  );
  const [loading, setLoading] = useState(false);
  const requestId = useRef(0);

  useEffect(() => {
    if (!path) {
      setSize(null);
      setLoading(false);
      return;
    }

    if (sizeCache.has(path)) {
      setSize(sizeCache.get(path) ?? null);
      setLoading(false);
      return;
    }

    const id = ++requestId.current;
    setLoading(true);
    setSize(null);

    uninstallerApi
      .getAppSize(path)
      .then((value) => {
        sizeCache.set(path, value);
        if (id !== requestId.current) return; // newer selection won; drop stale result
        setSize(value);
        setLoading(false);
      })
      .catch(() => {
        if (id !== requestId.current) return;
        setSize(null);
        setLoading(false);
      });
  }, [path]);

  return { size, loading };
}
