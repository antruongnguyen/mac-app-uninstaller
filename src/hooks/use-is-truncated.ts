import { useCallback, useEffect, useRef, useState } from "react";

/**
 * Tracks whether an element's content overflows its visible width — i.e.
 * whether `truncate` is currently clipping text. Re-measures on window resize.
 *
 * Returns a ref to attach to the truncating element and a boolean flag.
 */
export function useIsTruncated<T extends HTMLElement>() {
  const ref = useRef<T | null>(null);
  const [truncated, setTruncated] = useState(false);

  const measure = useCallback(() => {
    const el = ref.current;
    if (!el) return;
    setTruncated(el.scrollWidth > el.clientWidth);
  }, []);

  useEffect(() => {
    measure();
    const el = ref.current;
    if (!el || typeof ResizeObserver === "undefined") {
      window.addEventListener("resize", measure);
      return () => window.removeEventListener("resize", measure);
    }
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, [measure]);

  return [ref, truncated] as const;
}
