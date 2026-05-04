import type { NodeSummary } from "$lib/ipc/commands";

let _namePattern = $state<string>("");
let _sizeMinMb   = $state<number | null>(null);
let _ageDays     = $state<number | null>(null);

// Pending search results (from Rust search command)
let _searchResults = $state<NodeSummary[]>([]);
let _searchTotal   = $state<number>(0);
let _isSearching   = $state<boolean>(false);

export const namePattern   = () => _namePattern;
export const sizeMinMb     = () => _sizeMinMb;
export const ageDays       = () => _ageDays;
export const searchResults = () => _searchResults;
export const searchTotal   = () => _searchTotal;
export const isSearching   = () => _isSearching;
export const hasFilter     = () => !!_namePattern || _sizeMinMb !== null || _ageDays !== null;

export function setNamePattern(v: string) { _namePattern = v; }
export function setSizeMinMb(v: number | null) { _sizeMinMb = v; }
export function setAgeDays(v: number | null) { _ageDays = v; }

export function setSearchResults(nodes: NodeSummary[], total: number) {
  _searchResults = nodes;
  _searchTotal   = total;
  _isSearching   = false;
}

export function setSearching(v: boolean) { _isSearching = v; }

export function clearFilter() {
  _namePattern   = "";
  _sizeMinMb     = null;
  _ageDays       = null;
  _searchResults = [];
  _searchTotal   = 0;
}

// Local (client-side) filter for small node lists
export function localFilter(nodes: NodeSummary[]): NodeSummary[] {
  let result = nodes;
  const pat = _namePattern.toLowerCase();
  if (pat) {
    result = result.filter(n => n.name.toLowerCase().includes(pat));
  }
  if (_sizeMinMb !== null) {
    const minBytes = _sizeMinMb * 1024 * 1024;
    result = result.filter(n => n.size >= minBytes);
  }
  if (_ageDays !== null) {
    const cutoff = Date.now() - _ageDays * 86400_000;
    result = result.filter(n => n.mtime < cutoff);
  }
  return result;
}
