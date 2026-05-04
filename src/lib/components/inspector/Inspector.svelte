<script lang="ts">
  import { selectedId } from "$lib/store/uiStore";
  import { getNodeFromCache } from "$lib/store/scanStore";
  import { getNode, openInFileManager } from "$lib/ipc/commands";
  import { stageNode } from "$lib/store/selectionStore";
  import { formatBytes, formatAge, formatCount, CATEGORY_LABELS } from "$lib/utils/format";
  import type { NodeSummary } from "$lib/ipc/commands";

  let node = $state<NodeSummary | null>(null);

  $effect(() => {
    const id = selectedId();
    if (id === null) { node = null; return; }
    const cached = getNodeFromCache(id);
    if (cached) { node = cached; return; }
    getNode(id).then(n => { node = n; });
  });

  function openFm() {
    if (node) openInFileManager(node.path);
  }

  function stage() {
    if (node) stageNode(node);
  }
</script>

<div class="inspector border-bottom">
  <div class="section-title dim">INSPECTOR</div>

  {#if node}
    <div class="info-row"><span class="dim key">name</span><span class="val truncate">{node.name}</span></div>
    <div class="info-row"><span class="dim key">size</span><span class="val {node.size > 500_000_000 ? 'red' : ''}">{formatBytes(node.size)}</span></div>
    <div class="info-row"><span class="dim key">mtime</span><span class="val dim">{formatAge(node.mtime)}</span></div>
    <div class="info-row"><span class="dim key">type</span><span class="val dim">{node.is_dir ? "dir" : CATEGORY_LABELS[node.category] ?? "file"}</span></div>
    {#if node.is_dir && node.child_count > 0}
      <div class="info-row"><span class="dim key">items</span><span class="val dim">{formatCount(node.child_count)}</span></div>
    {/if}

    <div class="actions">
      <button class="ascii-btn" onclick={openFm} title={node.is_dir ? "Show folder in file manager" : "Open file with default app"}>open</button>
      <button class="ascii-btn safe" onclick={stage} title="Add to pending trash">stage</button>
    </div>
  {:else}
    <div class="empty dim">— select a file —</div>
  {/if}
</div>

<style>
  .inspector {
    padding: 6px 8px;
    flex-shrink: 0;
  }

  .section-title {
    font-size: 11px;
    margin-bottom: 6px;
  }

  .info-row {
    display: grid;
    grid-template-columns: 6ch 1fr;
    gap: 4px;
    height: 20px;
    align-items: center;
  }

  .key { font-size: 11px; }
  .val { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .actions {
    display: flex;
    gap: 4px;
    margin-top: 6px;
    flex-wrap: wrap;
  }

  .empty {
    font-size: 11px;
    padding: 4px 0;
  }
</style>
