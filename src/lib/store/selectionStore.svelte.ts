import type { NodeSummary } from "$lib/ipc/commands";

let _staged = $state<NodeSummary[]>([]);

export const staged        = () => _staged;
export const stagedCount   = () => _staged.length;
export const stagedBytes   = () => _staged.reduce((a, n) => a + n.size, 0);
export const isStaged      = (id: number) => _staged.some(n => n.id === id);

export function stageNode(node: NodeSummary) {
  if (!isStaged(node.id)) {
    _staged = [..._staged, node];
  }
}

export function unstageNode(id: number) {
  _staged = _staged.filter(n => n.id !== id);
}

export function clearStaging() {
  _staged = [];
}

export function removeDeletedFromStaging(deletedIds: number[]) {
  const set = new Set(deletedIds);
  _staged = _staged.filter(n => !set.has(n.id));
}
