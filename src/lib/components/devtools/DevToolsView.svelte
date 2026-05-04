<script lang="ts">
  import { getDevArtifacts } from "$lib/ipc/commands";
  import type { DevArtifact } from "$lib/ipc/commands";
  import { formatBytes, formatAge, ARTIFACT_LABELS } from "$lib/utils/format";
  import { stageNode } from "$lib/store/selectionStore";

  let artifacts = $state<DevArtifact[]>([]);
  let loading   = $state(false);
  let loaded    = $state(false);

  $effect(() => {
    if (!loaded) {
      loaded = true;
      load();
    }
  });

  async function load() {
    loading = true;
    try { artifacts = await getDevArtifacts(); }
    catch {}
    finally { loading = false; }
  }

  const totalSize = $derived(artifacts.reduce((s, a) => s + a.size, 0));
  const staleCount = $derived(artifacts.filter(a => a.stale).length);

  function stageArtifact(a: DevArtifact) {
    stageNode({
      id: a.id,
      name: a.artifact_name,
      path: a.project_root + "/" + a.artifact_name,
      size: a.size,
      is_dir: true,
      category: 3,
      mtime: a.mtime,
      child_count: 0,
    });
  }
</script>

<div class="devtools">
  <div class="devtools-header border-bottom">
    <span class="dim">DEV ARTIFACTS</span>
    <span class="spacer"></span>
    {#if artifacts.length > 0}
      <span class="dim">{artifacts.length} items · {formatBytes(totalSize)}</span>
      {#if staleCount > 0}
        <span class="red"> · {staleCount} stale</span>
      {/if}
    {/if}
    <button class="ascii-btn dim" onclick={load}>refresh</button>
  </div>

  {#if loading}
    <div class="empty dim">scanning artifacts…</div>
  {:else if artifacts.length === 0}
    <div class="empty dim">— no artifacts found —</div>
  {:else}
    <!-- Header -->
    <div class="row header dim border-bottom">
      <span class="col-kind">kind</span>
      <span class="col-name">project</span>
      <span class="col-size num">size</span>
      <span class="col-age">age</span>
      <span class="col-actions"></span>
    </div>

    <div class="artifact-list">
      {#each artifacts as a (a.id)}
        <div class="row artifact-row">
          <span class="col-kind dim">{ARTIFACT_LABELS[a.kind] ?? "?"}</span>
          <span class="col-name truncate" title={a.project_root}>{a.project_root.split("/").pop()}</span>
          <span class="col-size num {a.size > 500_000_000 ? 'red' : ''}">{formatBytes(a.size)}</span>
          <span class="col-age dim">
            {a.stale ? "[!] " : ""}{formatAge(a.mtime)}
          </span>
          <span class="col-actions">
            <button class="ascii-btn row-btn safe" onclick={() => stageArtifact(a)} title="Add to pending trash">+</button>
          </span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .devtools {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .devtools-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    height: 26px;
    flex-shrink: 0;
    font-size: 11px;
  }

  .spacer { flex: 1; }

  .row {
    display: grid;
    grid-template-columns: 12ch 1fr 9ch 10ch 5ch;
    align-items: center;
    height: 22px;
    padding: 0 8px;
    gap: 4px;
  }

  .header { font-size: 11px; }

  .artifact-list { flex: 1; overflow-y: auto; }
  .artifact-row:hover { background: var(--bg3); }

  .col-size { text-align: right; }
  .col-actions { display: flex; gap: 2px; visibility: hidden; }
  .artifact-row:hover .col-actions { visibility: visible; }

  .row-btn { font-size: 11px; padding: 0 1px; }
  .row-btn::before, .row-btn::after { content: none; }

  .empty { padding: 12px 8px; text-align: center; font-size: 11px; }
</style>
