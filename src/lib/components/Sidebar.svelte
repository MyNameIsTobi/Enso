<script lang="ts">
  import { sideView, setSideView, type SideView } from "$lib/store/uiStore";
  import { stagedCount } from "$lib/store/selectionStore";

  const views: { key: SideView; label: string }[] = [
    { key: "disk",  label: "disk"  },
    { key: "large", label: "large" },
    { key: "types", label: "types" },
    { key: "dev",   label: "dev"   },
    { key: "dupes", label: "dupes" },
    { key: "trash", label: "trash" },
  ];
</script>

<nav class="sidebar border-right">
  <div class="label dim">VIEWS</div>
  {#each views as v}
    <button
      class="ascii-btn nav-btn"
      class:active={sideView() === v.key}
      onclick={() => setSideView(v.key)}
    >
      {v.label}{#if v.key === "trash" && stagedCount() > 0}<span class="badge">{stagedCount()}</span>{/if}
    </button>
  {/each}
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 4px;
    width: 9ch;
    overflow: hidden;
  }
  .label {
    font-size: 11px;
    margin-bottom: 4px;
    padding-left: 1px;
  }
  .nav-btn {
    display: block;
    text-align: left;
    width: 100%;
    padding: 1px 2px;
  }
  .badge {
    color: var(--danger, #e55);
    margin-left: 2px;
    font-size: 11px;
  }
</style>
