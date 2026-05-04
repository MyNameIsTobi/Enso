<script lang="ts">
  import { staged, stagedCount, stagedBytes, unstageNode, clearStaging } from "$lib/store/selectionStore";
  import { moveToTrash, listTrash, emptyTrash, restoreFromTrash, type TrashEntry } from "$lib/ipc/commands";
  import { formatBytes, formatAge } from "$lib/utils/format";
  import { onMount } from "svelte";

  let trashEntries = $state<TrashEntry[]>([]);
  let loading = $state(false);
  let confirmingEmpty = $state(false);
  let confirmTimer = $state<ReturnType<typeof setTimeout> | null>(null);

  onMount(() => {
    refreshTrash();
  });

  async function refreshTrash() {
    loading = true;
    try {
      trashEntries = await listTrash();
    } catch {
      trashEntries = [];
    }
    loading = false;
  }

  async function confirmTrash() {
    const paths = staged().map(n => n.path);
    if (!paths.length) return;
    await moveToTrash(paths);
    clearStaging();
    await refreshTrash();
  }

  async function handleRestore(name: string) {
    await restoreFromTrash([name]);
    await refreshTrash();
  }

  function handleEmptyClick() {
    if (confirmingEmpty) {
      // Second click — actually empty
      if (confirmTimer) clearTimeout(confirmTimer);
      confirmTimer = null;
      confirmingEmpty = false;
      doEmpty();
    } else {
      // First click — start countdown
      confirmingEmpty = true;
      confirmTimer = setTimeout(() => {
        confirmingEmpty = false;
        confirmTimer = null;
      }, 3000);
    }
  }

  async function doEmpty() {
    await emptyTrash([]);
    await refreshTrash();
  }

  function trashTotalSize(): number {
    return trashEntries.reduce((a, e) => a + e.size, 0);
  }

  function parseAge(dateStr: string): string {
    if (!dateStr) return "";
    const ms = Date.parse(dateStr);
    if (isNaN(ms)) return "";
    return formatAge(ms);
  }
</script>

<div class="trash-view">
  <!-- Pending section -->
  <div class="section-header border-bottom">
    <span class="dim">PENDING</span>
    <span class="spacer"></span>
    {#if stagedCount() > 0}
      <span class="dim count">{stagedCount()} items &middot; {formatBytes(stagedBytes())}</span>
    {/if}
  </div>

  <div class="item-list pending-list">
    {#each staged() as node (node.id)}
      <div class="item-row">
        <span class="green truncate">── {node.name}</span>
        <span class="dim size">{formatBytes(node.size)}</span>
        <button class="ascii-btn row-btn dim" onclick={() => unstageNode(node.id)}>x</button>
      </div>
    {/each}

    {#if stagedCount() === 0}
      <div class="empty dim">— no pending items —</div>
    {/if}
  </div>

  <div class="section-footer border-bottom">
    <button class="ascii-btn danger" onclick={confirmTrash} disabled={stagedCount() === 0}>confirm trash</button>
    <button class="ascii-btn dim" onclick={clearStaging} disabled={stagedCount() === 0}>clear</button>
  </div>

  <!-- System Trash section -->
  <div class="section-header border-bottom">
    <span class="dim">SYSTEM TRASH</span>
    <span class="spacer"></span>
    <span class="dim count">
      {#if loading}
        loading…
      {:else}
        {trashEntries.length} items &middot; {formatBytes(trashTotalSize())}
      {/if}
    </span>
  </div>

  <div class="item-list trash-list">
    {#each trashEntries as entry (entry.name)}
      <div class="item-row">
        <span class="truncate">{entry.is_dir ? "── " : "── "}{entry.name}{entry.is_dir ? "/" : ""}</span>
        <span class="dim size">{formatBytes(entry.size)}</span>
        <span class="dim age">{parseAge(entry.deletion_date)}</span>
        <button class="ascii-btn row-btn dim" onclick={() => handleRestore(entry.name)} title="Restore to {entry.original_path}">↺</button>
      </div>
    {/each}

    {#if !loading && trashEntries.length === 0}
      <div class="empty dim">— trash is empty —</div>
    {/if}
  </div>

  <div class="section-footer">
    <button
      class="ascii-btn danger"
      onclick={handleEmptyClick}
      disabled={trashEntries.length === 0}
    >
      {#if confirmingEmpty}
        confirm? (3s)
      {:else}
        empty trash
      {/if}
    </button>
  </div>
</div>

<style>
  .trash-view { display: flex; flex-direction: column; height: 100%; overflow: hidden; }

  .section-header {
    display: flex;
    align-items: center;
    padding: 3px 8px;
    height: 26px;
    font-size: 11px;
    flex-shrink: 0;
    gap: 4px;
  }

  .spacer { flex: 1; }
  .count { font-size: 11px; }

  .item-list {
    overflow-y: auto;
    padding: 2px 0;
  }

  .pending-list { flex: 0 1 auto; max-height: 40%; }
  .trash-list { flex: 1; min-height: 0; }

  .item-row {
    display: grid;
    grid-template-columns: 1fr 8ch 6ch 3ch;
    align-items: center;
    height: 22px;
    padding: 0 8px;
    gap: 4px;
    font-size: 12px;
  }

  .item-row:hover { background: var(--bg3); }

  .size { text-align: right; font-size: 11px; }
  .age { text-align: right; font-size: 11px; }

  .row-btn {
    font-size: 11px;
    padding: 0;
  }
  .row-btn::before, .row-btn::after { content: none; }

  .section-footer {
    display: flex;
    gap: 6px;
    padding: 3px 8px;
    height: 26px;
    align-items: center;
    flex-shrink: 0;
  }

  .empty {
    padding: 8px;
    font-size: 11px;
    text-align: center;
  }
</style>
