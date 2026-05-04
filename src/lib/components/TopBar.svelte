<script lang="ts">
  import AsciiProgress from "./AsciiProgress.svelte";
  import { scanState, scanProgress, scanComplete, storageInfo, beginScan, abortScan, rootPath } from "$lib/store/scanStore";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let inputPath = $state("/");
  let scanning  = $derived(scanState() === "scanning");

  const appWindow = getCurrentWindow();

  async function pickDir() {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      inputPath = selected;
    }
  }

  async function doScan() {
    const path = inputPath.trim() || "/";
    await beginScan(path);
  }

  const progressRatio = $derived.by(() => {
    const si = storageInfo();
    const c  = scanComplete();
    if (si && si.total > 0 && c) return c.bytes / si.total;
    if (si && si.total > 0)      return si.used  / si.total;
    return c ? 1 : 0;
  });

  // bytes shown in the progress bar: scan total (logical size)
  const progressBytes = $derived(scanComplete()?.bytes ?? storageInfo()?.used);
  // total shown next to the bar: actual disk capacity from statvfs
  const progressTotal = $derived(storageInfo()?.total);

  const progressLabel = $derived.by(() => {
    const p = scanProgress();
    const c = scanComplete();
    if (c) return `${c.files.toLocaleString()} files`;
    if (p) return `${p.scanned.toLocaleString()} scanned · ${p.path.slice(-40)}`;
    return "";
  });
</script>

<header class="topbar border-bottom" data-tauri-drag-region>
  <span class="title">ENSO</span>
  <span class="sep dim">─</span>

  {#if !scanning}
    <button class="ascii-btn" onclick={doScan}>Scan</button>
  {:else}
    <button class="ascii-btn danger" onclick={abortScan}>Cancel</button>
  {/if}

  <span class="sep dim"> path: </span>
  <input
    type="text"
    class="path-input"
    bind:value={inputPath}
    placeholder="/"
    onkeydown={e => e.key === "Enter" && doScan()}
  />
  <button class="ascii-btn dim" onclick={pickDir}>…</button>

  <span class="spacer"></span>

  {#if scanState() === "scanning"}
    <span class="dim">scanning… {scanProgress()?.path.split("/").pop() ?? ""}</span>
  {:else if scanState() === "complete" || scanState() === "idle"}
    <AsciiProgress
      ratio={progressRatio}
      bytes={progressBytes}
      total={progressTotal}
      width={12}
      tooltip="Scanned file size / disk capacity&#10;&#10;The left number is the sum of all logical file sizes found by the scan.&#10;The right number is the total disk capacity.&#10;&#10;Your filesystem (Btrfs zstd) compresses data on disk, so the&#10;actual space used on the drive is lower than the logical file sizes."
    />
  {/if}

  {#if progressLabel}
    <span class="dim status"> · {progressLabel}</span>
  {/if}

  <div class="window-controls">
    <button class="win-btn" onclick={() => appWindow.minimize()}>─</button>
    <button class="win-btn" onclick={() => appWindow.toggleMaximize()}>□</button>
    <button class="win-btn close" onclick={() => appWindow.close()}>×</button>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    height: 26px;
    flex-shrink: 0;
    overflow: hidden;
  }
  .title {
    font-weight: bold;
    color: var(--fg);
  }
  .sep { user-select: none; }
  .path-input {
    width: 28ch;
    flex-shrink: 1;
  }
  .spacer { flex: 1; }
  .status { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 40ch; }
  .window-controls {
    display: flex;
    gap: 2px;
    margin-left: 8px;
  }
  .win-btn {
    width: 20px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--border);
    color: var(--dim);
    cursor: pointer;
    font-size: 11px;
    padding: 0;
    line-height: 1;
  }
  .win-btn:hover {
    color: var(--fg);
    background: var(--bg-hover, rgba(255,255,255,0.05));
  }
  .win-btn.close:hover {
    color: #e06c75;
  }
</style>
