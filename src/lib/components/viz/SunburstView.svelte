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

  let svg: SVGSVGElement;
  let w = $state(0);
  let h = $state(0);

  interface Tip { x: number; y: number; name: string; path: string; size: number; is_dir: boolean; }
  let tip = $state<Tip | null>(null);
  let lastX = 0, lastY = 0;

  $effect(() => {
    if (!svg || children.length === 0 || w === 0 || h === 0) return;
    drawSunburst();
    updateTipAtCursor();
  });

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

  function drawSunburst() {
    const radius = Math.min(w, h) / 2;
    const innerHole = radius * 0.28;
    const sel = d3.select(svg);
    sel.selectAll("*").remove();

    const root = d3.hierarchy({ id: -1, size: 0, children: children.map(n => ({ ...n, children: [] })) })
      .sum((d: any) => d.size)
      .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));

    d3.partition<any>().size([2 * Math.PI, radius - innerHole])(root);

    root.each((d: any) => {
      d.y0 += innerHole;
      d.y1 += innerHole;
    });

    const arc = d3.arc<d3.HierarchyRectangularNode<any>>()
      .startAngle(d => (d as any).x0)
      .endAngle(d => (d as any).x1)
      .innerRadius(d => Math.max(0, (d as any).y0))
      .outerRadius(d => Math.max(0, (d as any).y1 - 1))
      .padAngle(0.008)
      .padRadius(radius * 0.5);

    const topLevelNodes = root.children ?? [];
    const numTop = Math.max(topLevelNodes.length, 1);

    function getColor(d: any): string {
      if (d.depth === 0) return "transparent";
      let top = d;
      while (top.depth > 1 && top.parent) top = top.parent;
      const idx = topLevelNodes.indexOf(top);
      const hue = (idx / numTop) * 360;
      const lightness = 42 + (d.depth - 1) * 9;
      const saturation = 78 - (d.depth - 1) * 8;
      return `hsl(${hue.toFixed(1)},${Math.max(saturation, 30).toFixed(0)}%,${Math.min(lightness, 80).toFixed(0)}%)`;
    }

    const g = sel.append("g")
      .attr("transform", `translate(${w / 2},${h / 2})`);

    g.selectAll("path")
      .data(root.descendants().filter(d => d.depth > 0))
      .join("path")
        .attr("d", arc as any)
        .attr("fill", d => getColor(d))
        .attr("stroke", "var(--bg)")
        .attr("stroke-width", 1)
        .style("cursor", d => (d as any).data.is_dir ? "pointer" : "default")
        .on("click", (_e, d) => { if ((d as any).data.is_dir) ondrilldown((d as any).data as NodeSummary); })
        .on("mouseenter", (e, d) => {
          const data = (d as any).data as NodeSummary;
          setStatusMsg(data.path);
          tip = { x: e.clientX, y: e.clientY, name: data.name, path: data.path, size: data.size, is_dir: data.is_dir };
        })
        .on("mousemove", (e) => {
          lastX = e.clientX; lastY = e.clientY;
          if (tip) tip = { ...tip, x: e.clientX, y: e.clientY };
        })
        .on("mouseleave", () => { setStatusMsg(""); tip = null; });

    const totalBytes = root.value ?? 0;
    const label = g.append("g").attr("class", "center-label");
    label.append("text")
      .attr("text-anchor", "middle")
      .attr("dy", "-0.2em")
      .attr("font-size", Math.max(10, innerHole * 0.38))
      .attr("fill", "var(--fg)")
      .text(formatBytes(totalBytes));
    label.append("text")
      .attr("text-anchor", "middle")
      .attr("dy", "1.1em")
      .attr("font-size", Math.max(8, innerHole * 0.28))
      .attr("fill", "var(--dim)")
      .text("total");
  }
</script>

<div class="sun-wrap" bind:clientWidth={w} bind:clientHeight={h}>
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
  .sun-wrap { width: 100%; height: 100%; overflow: hidden; }
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
