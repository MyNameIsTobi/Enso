import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ScanProgress, ScanComplete, ScanUpdateEvent } from "./commands";

export const onScanProgress = (cb: (p: ScanProgress) => void): Promise<UnlistenFn> =>
  listen<ScanProgress>("scan://progress", e => cb(e.payload));

export const onScanComplete = (cb: (c: ScanComplete) => void): Promise<UnlistenFn> =>
  listen<ScanComplete>("scan://complete", e => cb(e.payload));

export const onScanUpdate = (cb: (u: ScanUpdateEvent) => void): Promise<UnlistenFn> =>
  listen<ScanUpdateEvent>("scan://update", e => cb(e.payload));

export const onScanCancelled = (cb: () => void): Promise<UnlistenFn> =>
  listen<void>("scan://cancelled", () => cb());
