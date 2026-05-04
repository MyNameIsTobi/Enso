<script lang="ts">
  import { search } from "$lib/ipc/commands";
  import {
    namePattern, setNamePattern, sizeMinMb, setSizeMinMb,
    ageDays, setAgeDays, setSearchResults, setSearching,
    clearFilter, hasFilter
  } from "$lib/store/filterStore";
  import { rootId } from "$lib/store/scanStore";
  import { currentId } from "$lib/store/uiStore";

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let localName = $state(namePattern());
  let localSize = $state(sizeMinMb() ?? "");
  let localAge  = $state(ageDays()  ?? "");
  let searchFromCurrent = $state(false);

  function onNameInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      setNamePattern(localName);
      triggerSearch();
    }, 80);
  }

  async function triggerSearch() {
    const rid = rootId();
    if (!localName && !localSize && !localAge) {
      clearFilter();
      return;
    }
    setSearching(true);
    try {
      const result = await search({
        name_pattern: localName || null,
        categories: [],
        extensions: [],
        size_min: localSize ? Number(localSize) * 1024 * 1024 : null,
        size_max: null,
        mtime_older_than_days: localAge ? Number(localAge) : null,
        mtime_newer_than_days: null,
        include_dirs: true,
        limit: 500,
        offset: 0,
        root_node_id: searchFromCurrent ? (currentId() ?? rid) : rid,
      });
      setSearchResults(result.nodes, result.total);
    } catch {
      setSearching(false);
    }
  }

  function applyFilters() {
    setSizeMinMb(localSize ? Number(localSize) : null);
    setAgeDays(localAge ? Number(localAge) : null);
    triggerSearch();
  }

  function reset() {
    localName = "";
    localSize = "";
    localAge  = "";
    clearFilter();
  }
</script>

<div class="searchbar border-bottom border-top">
  <span class="dim label">search:</span>
  <input
    id="search-input"
    type="text"
    placeholder="name…"
    bind:value={localName}
    oninput={onNameInput}
  />
  <span class="dim label"> size:</span>
  <input
    type="number"
    placeholder=">MB"
    bind:value={localSize}
    style="width:8ch"
    onkeydown={e => e.key === "Enter" && applyFilters()}
  />
  <span class="dim label"> age:</span>
  <input
    type="number"
    placeholder=">d"
    bind:value={localAge}
    style="width:6ch"
    onkeydown={e => e.key === "Enter" && applyFilters()}
  />
  <label class="scope-toggle dim" title="Search only in current folder">
    <input type="checkbox" bind:checked={searchFromCurrent} onchange={applyFilters} />
    here
  </label>
  <button class="ascii-btn" onclick={applyFilters}>apply</button>
  {#if hasFilter()}
    <button class="ascii-btn dim" onclick={reset}>clear</button>
  {/if}
</div>

<style>
  .searchbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    height: 26px;
    flex-shrink: 0;
  }
  .label { white-space: nowrap; }
  input { flex-shrink: 1; }
  .scope-toggle {
    display: flex;
    align-items: center;
    gap: 3px;
    white-space: nowrap;
    cursor: pointer;
    font-size: 11px;
  }
  .scope-toggle input[type="checkbox"] {
    width: auto;
    margin: 0;
    cursor: pointer;
  }
</style>
