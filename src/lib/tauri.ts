import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

function isInsideTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Wrapper around `invoke()` that throws a helpful error when run in a plain
 * browser (`bun run dev` without `tauri dev`). All commands require the
 * desktop shell, so there is no dev-mode fallback.
 */
export async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isInsideTauri() && import.meta.env.DEV) {
    throw new Error(
      `Tauri IPC not available in browser. Command "${cmd}" requires the Tauri desktop shell. Run with: bun run tauri dev`,
    );
  }
  return invoke<T>(cmd, args);
}

export async function tauriListen<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  if (!isInsideTauri()) {
    return () => {};
  }
  return listen<T>(event, (e) => handler(e.payload));
}
