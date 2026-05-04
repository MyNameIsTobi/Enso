<script lang="ts">
  import TopBar      from "$lib/components/TopBar.svelte";
  import Sidebar     from "$lib/components/Sidebar.svelte";
  import MainPanel   from "$lib/components/MainPanel.svelte";
  import Inspector   from "$lib/components/inspector/Inspector.svelte";

  import { navigateUp } from "$lib/store/uiStore";
  import { setSelectedId } from "$lib/store/uiStore";

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      navigateUp();
      setSelectedId(null);
    }
    if (e.key === "/") {
      e.preventDefault();
      document.getElementById("search-input")?.focus();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-shell">
  <TopBar />

  <div class="main-grid">
    <Sidebar />
    <MainPanel />
    <div class="right-col border-left">
      <Inspector />
    </div>
  </div>

  <div class="status-bar border-top dim">
    <span id="status-msg"></span>
  </div>
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .main-grid {
    flex: 1;
    display: grid;
    grid-template-columns: 9ch 1fr 28ch;
    min-height: 0;
    overflow: hidden;
  }

  .right-col {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .status-bar {
    height: 20px;
    padding: 0 8px;
    display: flex;
    align-items: center;
    font-size: 11px;
    flex-shrink: 0;
  }
</style>
