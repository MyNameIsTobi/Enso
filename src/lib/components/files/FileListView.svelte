<script lang="ts">
  import type { NodeSummary } from "$lib/ipc/commands";
  import { formatBytes, formatAge, CATEGORY_LABELS } from "$lib/utils/format";
  import { setSelectedId, selectedId, navigateTo } from "$lib/store/uiStore";
  import { stageNode, isStaged } from "$lib/store/selectionStore";
  import { localFilter, hasFilter, searchResults } from "$lib/store/filterStore";

  interface Props {
    children: NodeSummary[];
    loading: boolean;
  }
  let { children, loading }: Props = $props();

  const ROW_HEIGHT = 22;
  const OVERSCAN   = 5;

  let scrollTop    = $state(0);
  let containerH   = $state(0);
  let container = $state<HTMLDivElement | undefined>(undefined);

  // Use search results if active global filter, else local-filter children
  const displayNodes = $derived(
    hasFilter()
      ? searchResults()
      : localFilter(children)
  );

  const totalH    = $derived(displayNodes.length * ROW_HEIGHT);
  const startIdx  = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN));
  const endIdx    = $derived(Math.min(displayNodes.length, startIdx + Math.ceil(containerH / ROW_HEIGHT) + OVERSCAN * 2));
  const visible   = $derived(displayNodes.slice(startIdx, endIdx));
  const offsetY   = $derived(startIdx * ROW_HEIGHT);

  function handleScroll() {
    scrollTop = container?.scrollTop ?? 0;
  }

  function selectNode(n: NodeSummary) {
    setSelectedId(n.id);
  }

  function drillDown(n: NodeSummary) {
    if (n.is_dir) navigateTo(n);
  }

  function stage(n: NodeSummary, e: MouseEvent) {
    e.stopPropagation();
    stageNode(n);
  }

</script>

<div class="file-list-wrap">
  <!-- Header row -->
  <div class="row header dim border-bottom">
    <span class="col-name">name</span>
    <span class="col-size num">size</span>
    <span class="col-age">age</span>
    <span class="col-type">type</span>
    <span class="col-actions"></span>
  </div>

  {#if loading}
    <div class="empty dim">loading…</div>
  {:else if displayNodes.length === 0}
    <div class="empty dim">— empty —</div>
  {:else}
    <div
      class="scroll-body"
      bind:this={container}
      bind:clientHeight={containerH}
      onscroll={handleScroll}
    >
      <div style="height:{totalH}px; position:relative;">
        <div style="position:absolute; top:{offsetY}px; left:0; right:0;">
          {#each visible as node (node.id)}
            {@const staged = isStaged(node.id)}
            {@const selected = selectedId() === node.id}
            <div
              class="row file-row"
              class:selected
              class:staged
              onclick={() => selectNode(node)}
              ondblclick={() => drillDown(node)}
              onkeydown={e => e.key === "Enter" && drillDown(node)}
              role="row"
              tabindex="0"
            >
              <span class="col-name truncate">
                {#if node.is_dir}── {node.name}/{:else}   {node.name}{/if}
              </span>
              <span class="col-size num {node.size > 500_000_000 ? 'red' : ''}">{formatBytes(node.size)}</span>
              <span class="col-age dim">{formatAge(node.mtime)}</span>
              <span class="col-type dim">{CATEGORY_LABELS[node.category] ?? "?"}</span>
              <span class="col-actions">
                <button class="ascii-btn row-btn" onclick={e => stage(node, e)} title="Add to pending trash">+</button>
              </span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .file-list-wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .scroll-body {
    flex: 1;
    overflow-y: auto;
    contain: strict;
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 9ch 8ch 7ch 5ch;
    align-items: center;
    height: 22px;
    padding: 0 8px;
    gap: 4px;
    white-space: nowrap;
  }

  .header { font-size: 11px; }

  .file-row {
    cursor: default;
  }

  .file-row:hover      { background: var(--bg3); }
  .file-row.selected   { background: var(--sel); }
  .file-row.staged     { color: var(--green); }

  .col-name    { overflow: hidden; text-overflow: ellipsis; }
  .col-size    { text-align: right; }
  .col-actions { display: flex; gap: 2px; visibility: hidden; }
  .file-row:hover .col-actions { visibility: visible; }

  .row-btn {
    font-size: 11px;
    padding: 0 1px;
    line-height: 1;
  }
  .row-btn::before, .row-btn::after { content: none; }

  .empty {
    padding: 12px 8px;
    text-align: center;
  }
</style>
