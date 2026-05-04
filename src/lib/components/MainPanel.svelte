<script lang="ts">
  import {
    sideView, vizMode, setVizMode,
    breadcrumb, currentId,
    navigateTo, navigateToRoot, navigateBack, navigateForward, navigateUp,
    canGoBack, canGoForward,
    resetNavigation,
  } from "$lib/store/uiStore";
  import { scanState, scanProgress, scanComplete, rootId, fetchChildren } from "$lib/store/scanStore";
  import { getNode } from "$lib/ipc/commands";
  import type { NodeSummary } from "$lib/ipc/commands";

  import TreemapView  from "./viz/TreemapView.svelte";
  import SunburstView from "./viz/SunburstView.svelte";
  import FileListView from "./files/FileListView.svelte";
  import SearchBar    from "./search/SearchBar.svelte";
  import DevToolsView from "./devtools/DevToolsView.svelte";
  import LargeFilesView from "../../views/LargeFilesView.svelte";
  import FileTypesView  from "../../views/FileTypesView.svelte";
  import TrashView      from "../../views/TrashView.svelte";
  import DuplicatesView from "../../views/DuplicatesView.svelte";

  let children = $state<NodeSummary[]>([]);
  let loading  = $state(false);

  // Load children whenever currentId changes
  $effect(() => {
    const id = currentId();
    if (id === null) {
      const rid = rootId();
      if (rid !== null) _loadChildren(rid);
      return;
    }
    _loadChildren(id);
  });

  // When scan starts, reset navigation
  $effect(() => {
    if (scanState() === "scanning") {
      resetNavigation();
    }
  });

  // When scan completes, always navigate to root
  $effect(() => {
    if (scanState() === "complete") {
      const rid = rootId();
      if (rid !== null) {
        getNode(rid).then(n => {
          if (n) navigateToRoot(n);
        });
        _loadChildren(rid);
      }
    }
  });

  async function _loadChildren(id: number) {
    loading = true;
    try { children = await fetchChildren(id); }
    catch {}
    finally { loading = false; }
  }

  function handleDrillDown(node: NodeSummary) {
    navigateTo(node);
  }

  const currentPath = $derived(
    breadcrumb().length > 0 ? breadcrumb()[breadcrumb().length - 1].path : "—"
  );

  const errorCount = $derived(scanComplete()?.errors ?? 0);
</script>

<main class="main-panel border-right">
  <!-- Path + navigation + view toggle -->
  <div class="panel-header border-bottom">
    <button class="ascii-btn nav" onclick={navigateBack}    disabled={!canGoBack()}    title="Back">←</button>
    <button class="ascii-btn nav" onclick={navigateForward} disabled={!canGoForward()} title="Forward">→</button>
    <button class="ascii-btn nav" onclick={navigateUp}      disabled={!canGoBack()}    title="Up">↑</button>
    <span class="sep dim">│</span>
    <span class="path dim truncate">{currentPath}</span>
    {#if errorCount > 0}
      <span class="error-badge" title="{errorCount} permission errors">⚠ {errorCount}</span>
    {/if}
    <span class="spacer"></span>
    <button class="ascii-btn" class:active={vizMode() === "tree"} onclick={() => setVizMode("tree")}>tree</button>
    <button class="ascii-btn" class:active={vizMode() === "sun"}  onclick={() => setVizMode("sun") }>sun</button>
  </div>

  {#if scanState() === "idle"}
    <div class="empty-state dim">
      Enter a path above and press [Scan] to start.
    </div>
  {:else if scanState() === "scanning"}
    <div class="empty-state">
      <span class="dim">scanning</span>
      {#if scanProgress()}
        <span class="dim"> · {scanProgress()?.scanned.toLocaleString()} files</span>
        <div class="scan-path dim">{scanProgress()?.path ?? ""}</div>
        {#if (scanProgress()?.errors ?? 0) > 0}
          <span class="dim error-inline">⚠ {scanProgress()?.errors} permission errors</span>
        {/if}
      {/if}
    </div>
  {:else if sideView() === "dev"}
    <DevToolsView />
  {:else if sideView() === "large"}
    <LargeFilesView />
  {:else if sideView() === "types"}
    <FileTypesView />
  {:else if sideView() === "dupes"}
    <DuplicatesView />
  {:else if sideView() === "trash"}
    <TrashView />
  {:else}
    <!-- Visualization area -->
    <div class="viz-area">
      {#if vizMode() === "tree"}
        <TreemapView {children} ondrilldown={handleDrillDown} />
      {:else if vizMode() === "sun"}
        <SunburstView {children} ondrilldown={handleDrillDown} />
      {/if}
    </div>

    <!-- Search bar -->
    <SearchBar />

    <!-- File list -->
    <div class="file-list-area">
      <FileListView {children} {loading} />
    </div>
  {/if}
</main>

<style>
  .main-panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    height: 26px;
    flex-shrink: 0;
  }

  .nav { padding: 0 3px; min-width: 1.6ch; }
  .nav:disabled { opacity: 0.3; cursor: default; }
  .sep { user-select: none; }
  .path { max-width: 50ch; }
  .spacer { flex: 1; }

  .error-badge {
    font-size: 10px;
    color: var(--error, #e06c75);
    white-space: nowrap;
  }

  .viz-area {
    flex: 0 0 220px;
    border-bottom: 1px solid var(--border);
    overflow: hidden;
  }

  .file-list-area {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    font-size: 12px;
  }

  .scan-path {
    font-size: 11px;
    max-width: 60ch;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .error-inline {
    font-size: 11px;
    color: var(--error, #e06c75);
  }
</style>
