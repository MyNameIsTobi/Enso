<script lang="ts">
  import { search, getSteamGames } from "$lib/ipc/commands";
  import type { SteamGame } from "$lib/ipc/commands";
  import { formatBytes } from "$lib/utils/format";
  import { rootId } from "$lib/store/scanStore";

  interface CategoryStat {
    id: number;
    label: string;
    color: string;
    bytes: number;
    count: number;
  }

  const CATEGORIES = [
    { id: 0, label: "Images",       color: "#4a9eff" },
    { id: 1, label: "Videos",       color: "#e06c75" },
    { id: 2, label: "Documents",    color: "#98c379" },
    { id: 3, label: "Development",  color: "#e5c07b" },
    { id: 4, label: "Archives",     color: "#c678dd" },
    { id: 5, label: "Other",        color: "#5c6370" },
  ];

  let stats   = $state<CategoryStat[]>([]);
  let games   = $state<SteamGame[]>([]);
  let loading = $state(false);

  // Tooltip state for game list
  let tip = $state<{ x: number; y: number } | null>(null);

  $effect(() => { load(); });

  async function load() {
    loading = true;
    try {
      const [catResults, steamGames] = await Promise.all([
        Promise.all(CATEGORIES.map(cat =>
          search({
            name_pattern: null,
            categories: [cat.id],
            extensions: [],
            size_min: null,
            size_max: null,
            mtime_older_than_days: null,
            mtime_newer_than_days: null,
            include_dirs: false,
            limit: 500,
            offset: 0,
            root_node_id: rootId(),
          }).then(r => ({
            ...cat,
            bytes: r.nodes.reduce((s, n) => s + n.size, 0),
            count: r.total,
          }))
        )),
        getSteamGames(),
      ]);
      stats = catResults.sort((a, b) => b.bytes - a.bytes);
      games = steamGames;
    } catch {}
    finally { loading = false; }
  }

  const gamesTotal = $derived(games.reduce((s, g) => s + g.size, 0));

  // Merge games into the sorted rows list
  type Row = { kind: "cat"; stat: CategoryStat } | { kind: "games" };
  const rows = $derived.by((): Row[] => {
    const list: Row[] = stats.map(s => ({ kind: "cat", stat: s }));
    if (games.length > 0) list.push({ kind: "games" });
    return list.sort((a, b) => {
      const sa = a.kind === "games" ? gamesTotal : a.stat.bytes;
      const sb = b.kind === "games" ? gamesTotal : b.stat.bytes;
      return sb - sa;
    });
  });

  const maxBytes = $derived(Math.max(
    stats.reduce((m, s) => Math.max(m, s.bytes), 1),
    gamesTotal,
  ));

  function bar(bytes: number, max: number, width = 30): string {
    const filled = Math.round((bytes / max) * width);
    return "▓".repeat(filled) + "░".repeat(width - filled);
  }
</script>

<div class="types-view">
  <div class="header border-bottom dim">FILE TYPES</div>

  {#if loading}
    <div class="empty dim">loading…</div>
  {:else}
    <div class="stats-list">
      {#each rows as row}
        {#if row.kind === "games"}
          <div
            class="stat-row games-row"
            onmouseenter={e => tip = { x: e.clientX, y: e.clientY }}
            onmousemove={e => tip && (tip = { x: e.clientX, y: e.clientY })}
            onmouseleave={() => tip = null}
          >
            <span class="label" style="color:#a8cc8c">Games</span>
            <span class="bar dim">{bar(gamesTotal, maxBytes)}</span>
            <span class="bytes num dim">{formatBytes(gamesTotal)}</span>
            <span class="count dim"> {games.length}</span>
          </div>
        {:else}
          <div class="stat-row">
            <span class="label" style="color:{row.stat.color}">{row.stat.label}</span>
            <span class="bar dim">{bar(row.stat.bytes, maxBytes)}</span>
            <span class="bytes num dim">{formatBytes(row.stat.bytes)}</span>
            <span class="count dim"> {row.stat.count.toLocaleString()}</span>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<!-- Game list tooltip -->
{#if tip && games.length > 0}
  <div class="game-tip" style="left:{tip.x + 14}px; top:{tip.y - 8}px">
    <div class="tip-header dim">installed games</div>
    {#each games as g (g.name)}
      <div class="tip-row">
        <span class="tip-name">{g.name}</span>
        <span class="tip-size">{formatBytes(g.size)}</span>
      </div>
    {/each}
  </div>
{/if}

<style>
  .types-view { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
  .header { padding: 6px 8px; font-size: 11px; flex-shrink: 0; }
  .stats-list { padding: 8px; display: flex; flex-direction: column; gap: 4px; overflow-y: auto; }
  .stat-row { display: grid; grid-template-columns: 13ch 32ch 9ch 8ch; align-items: center; height: 22px; gap: 4px; }
  .games-row { cursor: default; border-radius: 2px; }
  .games-row:hover { background: var(--bg3); }
  .label { font-size: 12px; }
  .bar { font-size: 11px; letter-spacing: -1px; }
  .bytes { text-align: right; font-size: 11px; }
  .count { font-size: 11px; }
  .empty { padding: 12px; text-align: center; font-size: 11px; }

  .game-tip {
    position: fixed;
    z-index: 999;
    pointer-events: none;
    background: var(--bg2);
    border: 1px solid var(--border);
    padding: 4px 0;
    font-family: ui-monospace, monospace;
    font-size: 11px;
    min-width: 28ch;
    max-width: 52ch;
    max-height: 60vh;
    overflow-y: auto;
  }
  .tip-header {
    padding: 2px 8px 4px;
    font-size: 10px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 2px;
  }
  .tip-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 1px 8px;
  }
  .tip-row:hover { background: var(--bg3); }
  .tip-name { color: var(--fg); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tip-size { color: var(--blue); white-space: nowrap; flex-shrink: 0; }
</style>
