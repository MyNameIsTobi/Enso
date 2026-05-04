<script lang="ts">
  import { asciiBar, formatBytes } from "$lib/utils/format";

  interface Props {
    ratio: number;   // 0–1
    bytes?: number;
    total?: number;
    width?: number;
    label?: string;
    tooltip?: string;
  }
  let { ratio, bytes, total, width = 16, label, tooltip }: Props = $props();

  const bar   = $derived(asciiBar(ratio, width));
  const pct   = $derived(Math.round(ratio * 100));
</script>

<span class="ascii-progress" title={tooltip}>
  <span class="bar">{bar}</span>
  <span class="dim"> {pct}%</span>
  {#if bytes !== undefined && total !== undefined}
    <span class="dim"> · {formatBytes(bytes)} / {formatBytes(total)}</span>
  {:else if bytes !== undefined}
    <span class="dim"> · {formatBytes(bytes)}</span>
  {/if}
  {#if label}
    <span class="dim"> · {label}</span>
  {/if}
</span>

<style>
  .ascii-progress { font-variant-numeric: tabular-nums; }
  .bar { color: var(--blue); }
</style>
