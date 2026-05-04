<script lang="ts">
  import { search } from "$lib/ipc/commands";
  import type { NodeSummary } from "$lib/ipc/commands";
  import { formatBytes, formatAge, CATEGORY_LABELS } from "$lib/utils/format";
  import { rootId } from "$lib/store/scanStore";
  import { navigateTo, setSelectedId } from "$lib/store/uiStore";

  let nodes    = $state<NodeSummary[]>([]);
  let loading  = $state(false);
  let minMb    = $state(100);

  $effect(() => { load(); });

  async function load() {
    loading = true;
    try {
      const result = await search({
        name_pattern: null,
        categories: [],
        extensions: [],
        size_min: minMb * 1024 * 1024,
        size_max: null,
        mtime_older_than_days: null,
        mtime_newer_than_days: null,
        include_dirs: true,
        limit: 500,
        offset: 0,
        root_node_id: rootId(),
      });
      nodes = result.nodes;
    } catch {}
    finally { loading = false; }
  }
</script>

<div class="large-view">
  <div class="header border-bottom">
    <span class="dim">LARGE FILES  min:</span>
    <input type="number" bind:value={minMb} onchange={load} style="width:6ch" />
    <span class="dim">MB</span>
    <span class="spacer"></span>
    {#if nodes.length > 0}
      <span class="dim">{nodes.length} results</span>
    {/if}
  </div>

  {#if loading}
    <div class="empty dim">searching…</div>
  {:else}
    <div class="list">
      <div class="row hdr dim border-bottom">
        <span class="col-name">name</span>
        <span class="col-size num">size</span>
        <span class="col-age">age</span>
        <span class="col-type">type</span>
      </div>
      {#each nodes as n (n.id)}
        <div
          class="row item"
          onclick={() => setSelectedId(n.id)}
          ondblclick={() => { if (n.is_dir) navigateTo(n); }}
          onkeydown={e => e.key === "Enter" && setSelectedId(n.id)}
          role="row" tabindex="0"
        >
          <span class="col-name truncate">{n.name}</span>
          <span class="col-size num red">{formatBytes(n.size)}</span>
          <span class="col-age dim">{formatAge(n.mtime)}</span>
          <span class="col-type dim">{CATEGORY_LABELS[n.category] ?? "file"}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .large-view { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
  .header { display: flex; align-items: center; gap: 6px; padding: 3px 8px; height: 26px; font-size: 11px; flex-shrink: 0; }
  .spacer { flex: 1; }
  .list { flex: 1; overflow-y: auto; }
  .row { display: grid; grid-template-columns: 1fr 9ch 8ch 7ch; align-items: center; height: 22px; padding: 0 8px; gap: 4px; }
  .hdr { font-size: 11px; }
  .item:hover { background: var(--bg3); }
  .col-size { text-align: right; }
  .empty { padding: 12px; text-align: center; font-size: 11px; }
</style>
