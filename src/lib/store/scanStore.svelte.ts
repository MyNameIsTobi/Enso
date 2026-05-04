import { onScanProgress, onScanComplete, onScanUpdate, onScanCancelled } from "$lib/ipc/events";
import { startScan, cancelScan, getChildren, getStorageInfo } from "$lib/ipc/commands";
import type { NodeSummary, ScanProgress, ScanComplete, StorageInfo } from "$lib/ipc/commands";
import type { UnlistenFn } from "@tauri-apps/api/event";

export type ScanState = "idle" | "scanning" | "complete" | "cancelled" | "error";

// ── Reactive state (Svelte 5 runes) ─────────────────────────────────────────

let _state       = $state<ScanState>("idle");
let _progress    = $state<ScanProgress | null>(null);
let _complete    = $state<ScanComplete | null>(null);
let _rootId      = $state<number | null>(null);
let _rootPath    = $state<string>("");
let _storageInfo = $state<StorageInfo | null>(null);
let _error       = $state<string | null>(null);

// Node cache: id → NodeSummary (populated on demand)
const _nodeCache = new Map<number, NodeSummary>();

// Active event unlisteners
let _unlisteners: UnlistenFn[] = [];

// ── Public reactive getters ──────────────────────────────────────────────────

export const scanState       = () => _state;
export const scanProgress    = () => _progress;
export const scanComplete    = () => _complete;
export const rootId          = () => _rootId;
export const rootPath        = () => _rootPath;
export const storageInfo     = () => _storageInfo;
export const scanError       = () => _error;

export function getNodeFromCache(id: number): NodeSummary | undefined {
  return _nodeCache.get(id);
}

export function cacheNode(n: NodeSummary) {
  _nodeCache.set(n.id, n);
}

// ── Actions ──────────────────────────────────────────────────────────────────

export async function beginScan(path: string) {
  // Tear down previous listeners
  await _teardown();

  _state    = "scanning";
  _rootPath = path;
  _progress = null;
  _complete = null;
  _rootId   = null;
  _error    = null;
  _nodeCache.clear();

  // Register listeners before calling start_scan to avoid race
  _unlisteners = await Promise.all([
    onScanProgress(p => { _progress = p; }),
    onScanComplete(c => {
      _complete = c;
      _rootId   = c.root_id;
      _state    = "complete";
      _fetchStorageInfo(path);
    }),
    onScanUpdate(u => {
      // Remove stale nodes from cache
      for (const id of u.removed_ids) _nodeCache.delete(id);
      // Update ancestors
      for (const n of u.updated_ancestors) _nodeCache.set(n.id, n);
    }),
    onScanCancelled(() => { _state = "cancelled"; }),
  ]);

  try {
    await startScan(path);
  } catch (e) {
    _state = "error";
    _error = String(e);
    await _teardown();
  }
}

export async function abortScan() {
  try { await cancelScan(); } catch {}
}

async function _fetchStorageInfo(path: string) {
  try {
    _storageInfo = await getStorageInfo(path);
  } catch {}
}

async function _teardown() {
  for (const fn of _unlisteners) { try { fn(); } catch {} }
  _unlisteners = [];
}

// Fetch children and cache them; returns sorted list
export async function fetchChildren(nodeId: number): Promise<NodeSummary[]> {
  const nodes = await getChildren(nodeId);
  for (const n of nodes) _nodeCache.set(n.id, n);
  return nodes;
}
