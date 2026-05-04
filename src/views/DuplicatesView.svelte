<script lang="ts">
  import { findDuplicates } from "$lib/ipc/commands";
  import type { DuplicateGroup, NodeSummary } from "$lib/ipc/commands";
  import { formatBytes } from "$lib/utils/format";
  import { setSelectedId } from "$lib/store/uiStore";
  import { stageNode, isStaged } from "$lib/store/selectionStore";
  import { isDismissedName, dismissName, restoreName, dismissedCount } from "$lib/store/duplicatesDismissStore.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let groups  = $state<DuplicateGroup[]>([]);
  let loading = $state(false);
  type FilterMode = "normal" | "all" | "only-hidden";
  let filterMode = $state<FilterMode>("normal");

  function cycleFilter() {
    if (filterMode === "normal") filterMode = "all";
    else if (filterMode === "all") filterMode = "only-hidden";
    else filterMode = "normal";
  }

  const visibleGroups = $derived(
    filterMode === "normal"      ? groups.filter(g => !isDismissedName(g.name)) :
    filterMode === "only-hidden" ? groups.filter(g => isDismissedName(g.name))  :
    groups
  );
  const totalGroups = $derived(visibleGroups.length);
  const reclaimable = $derived(
    visibleGroups.reduce((sum, g) => sum + g.size * (g.nodes.length - 1), 0)
  );

  async function load() {
    loading = true;
    try {
      groups = await findDuplicates();
    } catch {}
    finally { loading = false; }
  }

  onMount(() => {
    load();
    const unlisten = listen("scan://update", () => load());
    return () => { unlisten.then(fn => fn()); };
  });

  function parentDir(path: string): string {
    const i = path.lastIndexOf("/");
    return i > 0 ? path.slice(0, i) : path;
  }

  function handleStage(e: MouseEvent, node: NodeSummary) {
    e.stopPropagation();
    stageNode(node);
  }
</script>

<div class="dupes-view">
  <div class="header border-bottom">
    <span class="dim">DUPLICATES</span>
    <span class="spacer"></span>
    {#if totalGroups > 0}
      <span class="dim">{totalGroups} groups</span>
      <span class="dim">&middot;</span>
      <span class="red">{formatBytes(reclaimable)} reclaimable</span>
    {/if}
    {#if dismissedCount() > 0}
      <button class="ascii-btn" onclick={cycleFilter}>
        {#if filterMode === "normal"}
          {dismissedCount()} hidden
        {:else if filterMode === "all"}
          show all
        {:else}
          only hidden
        {/if}
      </button>
    {/if}
  </div>

  {#if loading}
    <div class="empty dim">scanning for duplicates…</div>
  {:else if visibleGroups.length === 0}
    <div class="empty dim">no duplicates found</div>
  {:else}
    <div class="list">
      {#each visibleGroups as group (group.name + group.size)}
        <div class="group-header dim border-bottom" class:dismissed={filterMode === "all" && isDismissedName(group.name)}>
          <span class="truncate">{group.name}</span>
          <span class="spacer"></span>
          <span class="num">{formatBytes(group.size)}</span>
          <span>&times;{group.nodes.length}</span>
          {#if isDismissedName(group.name)}
            <button class="ascii-btn green" onclick={() => restoreName(group.name)} title="Unhide this name">unhide</button>
          {:else}
            <button class="ascii-btn" onclick={() => dismissName(group.name)} title="Hide all groups with this name">hide</button>
          {/if}
        </div>
        {#each group.nodes as node (node.id)}
          <div
            class="row item"
            onclick={() => setSelectedId(node.id)}
            onkeydown={e => e.key === "Enter" && setSelectedId(node.id)}
            role="row" tabindex="0"
          >
            <span class="col-name truncate indent">{node.name}</span>
            <span class="col-path dim truncate">{parentDir(node.path)}</span>
            <span class="col-action">
              {#if isStaged(node.id)}
                <span class="dim">staged</span>
              {:else}
                <button class="ascii-btn stage-btn" onclick={(e) => handleStage(e, node)} title="Stage for deletion">[+]</button>
              {/if}
            </span>
          </div>
        {/each}
      {/each}
    </div>
  {/if}
</div>

<style>
  .dupes-view { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
  .header { display: flex; align-items: center; gap: 6px; padding: 3px 8px; height: 26px; font-size: 11px; flex-shrink: 0; }
  .spacer { flex: 1; }
  .list { flex: 1; overflow-y: auto; }
  .group-header {
    display: flex; align-items: center; gap: 6px;
    padding: 4px 8px; height: 24px; font-size: 11px;
    font-weight: bold; background: var(--bg2, #1e1e2e);
  }
  .row { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 5ch; align-items: center; height: 22px; padding: 0 8px; gap: 6px; font-size: 11px; white-space: nowrap; }
  .row > * { min-width: 0; overflow: hidden; }
  .item:hover { background: var(--bg3); }
  .indent { padding-left: 1ch; }
  .col-path { font-size: 11px; }
  .col-action { text-align: right; }
  .stage-btn { font-size: 11px; padding: 0 2px; }
  .empty { padding: 12px; text-align: center; font-size: 11px; }
  .dismissed { opacity: 0.4; }
  .green { color: var(--green, #a6e3a1); }
</style>
