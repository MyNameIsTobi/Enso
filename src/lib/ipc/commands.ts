import { invoke } from "@tauri-apps/api/core";

// ── Wire types (must match Rust structs) ────────────────────────────────────

export interface NodeSummary {
  id: number;
  name: string;
  path: string;
  size: number;
  is_dir: boolean;
  category: number; // FileCategory as u8
  mtime: number;    // unix ms
  child_count: number;
}

export interface StorageInfo {
  total: number;
  used: number;
  available: number;
  mount: string;
}

export interface ScanProgress {
  scanned: number;
  bytes: number;
  path: string;
  errors: number;
}

export interface ScanComplete {
  root_id: number;
  files: number;
  dirs: number;
  bytes: number;
  errors: number;
  duration_ms: number;
}

export interface DevArtifact {
  id: number;
  project_root: string;
  artifact_name: string;
  size: number;
  mtime: number;
  stale: boolean;
  kind: number; // ArtifactKind as u8
}

export interface SearchQuery {
  name_pattern: string | null;
  categories: number[];
  extensions: string[];
  size_min: number | null;
  size_max: number | null;
  mtime_older_than_days: number | null;
  mtime_newer_than_days: number | null;
  include_dirs: boolean;
  limit: number;
  offset: number;
  root_node_id: number | null;
}

export interface SearchResult {
  nodes: NodeSummary[];
  total: number;
}

export interface TrashResult {
  trashed: string[];
  failed: { path: string; error: string }[];
}

export interface DeleteResult {
  deleted: string[];
  failed: { path: string; error: string }[];
}

export interface ScanUpdateEvent {
  removed_ids: number[];
  updated_ancestors: NodeSummary[];
}

// ── Command wrappers ─────────────────────────────────────────────────────────

export const startScan = (rootPath: string) =>
  invoke<void>("start_scan", { rootPath });

export const cancelScan = () =>
  invoke<void>("cancel_scan");

export const getChildren = (nodeId: number, offset = 0) =>
  invoke<NodeSummary[]>("get_children", { nodeId, offset });

export const getNode = (nodeId: number) =>
  invoke<NodeSummary | null>("get_node", { nodeId });

export const getStorageInfo = (path: string) =>
  invoke<StorageInfo>("get_storage_info", { path });

export const search = (query: SearchQuery) =>
  invoke<SearchResult>("search", { query });

export const getDevArtifacts = () =>
  invoke<DevArtifact[]>("get_dev_artifacts");

export interface SteamGame {
  name: string;
  path: string;
  size: number;
}

export const getSteamGames = () =>
  invoke<SteamGame[]>("get_steam_games");

export const moveToTrash = (paths: string[]) =>
  invoke<TrashResult>("move_to_trash", { paths });

export const deletePermanently = (paths: string[], confirmed: boolean) =>
  invoke<DeleteResult>("delete_permanently", { paths, confirmed });

export const openInFileManager = (path: string) =>
  invoke<void>("open_in_file_manager", { path });

// ── Trash management ────────────────────────────────────────────────────────

export interface TrashEntry {
  name: string;
  original_path: string;
  deletion_date: string;
  size: number;
  is_dir: boolean;
}

export const listTrash = () =>
  invoke<TrashEntry[]>("list_trash");

export const emptyTrash = (names: string[]) =>
  invoke<DeleteResult>("empty_trash", { names });

export const restoreFromTrash = (names: string[]) =>
  invoke<string[]>("restore_from_trash", { names });

// ── Duplicates ─────────────────────────────────────────────────────────────

export interface DuplicateGroup {
  name: string;
  size: number;
  nodes: NodeSummary[];
}

export const findDuplicates = () =>
  invoke<DuplicateGroup[]>("find_duplicates");
