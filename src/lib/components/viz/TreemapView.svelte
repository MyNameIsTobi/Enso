<script lang="ts">
  import * as d3 from "d3";
  import type { NodeSummary } from "$lib/ipc/commands";
  import { setStatusMsg } from "$lib/store/uiStore";
  import { formatBytes } from "$lib/utils/format";

  interface Props {
    children: NodeSummary[];
    ondrilldown: (n: NodeSummary) => void;
  }
  let { children, ondrilldown }: Props = $props();

  interface Tip { x: number; y: number; name: string; path: string; size: number; is_dir: boolean; }
  let tip = $state<Tip | null>(null);
  let lastX = 0, lastY = 0;

  function updateTipAtCursor() {
    const el = document.elementFromPoint(lastX, lastY);
    if (!el || !svg.contains(el)) { tip = null; return; }
    const d = (d3.select(el as Element).datum() as any);
    if (d?.data?.name) {
      tip = { x: lastX, y: lastY, name: d.data.name, path: d.data.path, size: d.data.size, is_dir: d.data.is_dir };
      setStatusMsg(d.data.path);
    } else {
      tip = null;
    }
  }

  let container: HTMLDivElement;
  let svg: SVGSVGElement;
  let w = $state(0);
  let h = $state(0);

  const CAT_COLORS = ["#4a9eff","#e06c75","#98c379","#e5c07b","#c678dd","#5c6370"];

  function color(cat: number): string {
    return CAT_COLORS[cat] ?? CAT_COLORS[5];
  }

  $effect(() => {
    if (!svg || children.length === 0 || w === 0 || h === 0) return;
    drawTreemap();
    updateTipAtCursor();
  });

  function drawTreemap() {
    const total = children.reduce((s, n) => s + n.size, 0);
    if (total === 0) return;

    const root = d3.hierarchy({ id: -1, size: 0, children: children.map(n => ({ ...n, children: [] })) })
      .sum((d: any) => d.size)
      .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));

    d3.treemap<any>()
      .size([w, h])
      .paddingInner(1)
      .paddingOuter(2)
      (root);

    const sel = d3.select(svg);
    sel.selectAll("*").remove();

    const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const dur = prefersReduced ? 0 : 150;

    const leaves = root.leaves();

    const g = sel.selectAll<SVGGElement, d3.HierarchyRectangularNode<any>>("g")
      .data(leaves)
      .join("g")
        .attr("transform", d => `translate(${d.x0},${d.y0})`)
        .style("cursor", d => d.data.is_dir ? "pointer" : "default")
        .on("click", (_e, d) => { if (d.data.is_dir) ondrilldown(d.data as NodeSummary); })
        .on("mouseenter", (e, d) => {
          setStatusMsg(d.data.path);
          tip = { x: e.clientX, y: e.clientY, name: d.data.name, path: d.data.path, size: d.data.size, is_dir: d.data.is_dir };
        })
        .on("mousemove", (e) => {
          lastX = e.clientX; lastY = e.clientY;
          if (tip) tip = { ...tip, x: e.clientX, y: e.clientY };
        })
        .on("mouseleave", () => { setStatusMsg(""); tip = null; });

    g.append("rect")
      .attr("width",  d => Math.max(0, d.x1 - d.x0))
      .attr("height", d => Math.max(0, d.y1 - d.y0))
      .attr("fill",   d => color(d.data.category))
      .attr("opacity", 0.75);

    // Labels only if cell is big enough
    g.filter(d => (d.x1 - d.x0) >= 40 && (d.y1 - d.y0) >= 20)
      .append("text")
        .attr("x", 3)
        .attr("y", 13)
        .attr("font-family", "ui-monospace, monospace")
        .attr("font-size", "11px")
        .attr("fill", "#fff")
        .attr("pointer-events", "none")
        .text(d => d.data.name);
  }
</script>

<div class="treemap-wrap" bind:this={container} bind:clientWidth={w} bind:clientHeight={h}>
  <svg bind:this={svg} width={w} height={h}></svg>
</div>

{#if tip}
  <div class="cursor-tip" style="left:{tip.x + 14}px; top:{tip.y - 8}px">
    <span class="tip-name">{tip.name}</span>
    <span class="tip-size">{formatBytes(tip.size)}</span>
    <span class="tip-path">{tip.path}</span>
  </div>
{/if}

<style>
  .treemap-wrap {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
  svg { display: block; }

  .cursor-tip {
    position: fixed;
    z-index: 999;
    pointer-events: none;
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--bg2);
    border: 1px solid var(--border);
    padding: 4px 7px;
    font-family: ui-monospace, monospace;
    font-size: 11px;
    max-width: 48ch;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tip-name { color: var(--fg); font-weight: bold; }
  .tip-size { color: var(--blue); }
  .tip-path { color: var(--dim); font-size: 10px; overflow: hidden; text-overflow: ellipsis; }
</style>
