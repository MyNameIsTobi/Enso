import type { NodeSummary } from "$lib/ipc/commands";

export type VizMode  = "tree" | "sun";
export type SideView = "disk" | "large" | "types" | "dev" | "dupes" | "trash";

// ── Reactive state ───────────────────────────────────────────────────────────

let _vizMode    = $state<VizMode>("tree");
let _sideView   = $state<SideView>("disk");
let _breadcrumb = $state<NodeSummary[]>([]);  // path from root to current node
let _future     = $state<NodeSummary[]>([]);  // forward navigation stack
let _currentId  = $state<number | null>(null);
let _selectedId = $state<number | null>(null);
let _statusMsg  = $state<string>("");

// ── Getters ──────────────────────────────────────────────────────────────────

export const vizMode     = () => _vizMode;
export const sideView    = () => _sideView;
export const breadcrumb  = () => _breadcrumb;
export const currentId   = () => _currentId;
export const selectedId  = () => _selectedId;
export const statusMsg   = () => _statusMsg;
export const canGoBack   = () => _breadcrumb.length > 1;
export const canGoForward = () => _future.length > 0;

// ── Actions ──────────────────────────────────────────────────────────────────

export function setVizMode(m: VizMode)    { _vizMode  = m; }
export function setSideView(v: SideView)  { _sideView = v; }
export function setSelectedId(id: number | null) { _selectedId = id; }
export function setStatusMsg(msg: string) { _statusMsg = msg; }

export function navigateTo(node: NodeSummary) {
  const existing = _breadcrumb.findIndex(n => n.id === node.id);
  if (existing === -1) {
    _breadcrumb = [..._breadcrumb, node];
  } else {
    _breadcrumb = _breadcrumb.slice(0, existing + 1);
  }
  _future    = [];  // clear forward history on new navigation
  _currentId = node.id;
}

export function navigateToRoot(root: NodeSummary) {
  _breadcrumb = [root];
  _future     = [];
  _currentId  = root.id;
}

export function navigateBack() {
  if (_breadcrumb.length <= 1) return;
  const current = _breadcrumb[_breadcrumb.length - 1];
  _future     = [current, ..._future];
  _breadcrumb = _breadcrumb.slice(0, _breadcrumb.length - 1);
  _currentId  = _breadcrumb[_breadcrumb.length - 1].id;
}

export function navigateForward() {
  if (_future.length === 0) return;
  const next  = _future[0];
  _future     = _future.slice(1);
  _breadcrumb = [..._breadcrumb, next];
  _currentId  = next.id;
}

export function navigateUp() {
  navigateBack();
}

export function resetNavigation() {
  _breadcrumb = [];
  _future     = [];
  _currentId  = null;
}
