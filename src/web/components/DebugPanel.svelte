<script>
  import { onMount } from "svelte";
  import { showStats, lastSeed, showSidebar } from "../stores.js";

  export let pkg = null;

  let seedInput = "";
  let widthInput = "";
  let heightInput = "";
  let tracesText = "";

  // runtime trace recording
  let traces = [];
  let traceInterval = null;
  let nextTraceId = 1;
  const TRACE_INTERVAL_MS = 50;
  const TRACE_MAX_LINES = 500;

  function _updateTracesText() {
    tracesText = traces.map((t) => JSON.stringify(t)).join("\n");
  }

  function sampleTrace() {
    if (!pkg) return;
    try {
      const px = pkg.get_pos_x();
      const py = pkg.get_pos_y();
      const dx = pkg.get_dir_x();
      const dy = pkg.get_dir_y();
      const plx = pkg.get_plane_x();
      const ply = pkg.get_plane_y();
      const angle_deg = (Math.atan2(dy, dx) * 180) / Math.PI;
      const entry = {
        id: nextTraceId++,
        timestamp_ms: Date.now(),
        position: { x: px, y: py },
        direction: { x: dx, y: dy },
        plane: { x: plx, y: ply },
        angle_deg,
      };
      traces.push(entry);
      if (traces.length > TRACE_MAX_LINES) traces.shift();
      _updateTracesText();
    } catch (e) {
      console.error("trace sample failed", e);
    }
  }

  function startTrace() {
    if (traceInterval) return;
    traceInterval = setInterval(sampleTrace, TRACE_INTERVAL_MS);
  }

  function stopTrace() {
    if (!traceInterval) return;
    clearInterval(traceInterval);
    traceInterval = null;
  }

  function clearTrace() {
    traces = [];
    nextTraceId = 1;
    tracesText = "";
  }

  async function copyTrace() {
    try {
      await navigator.clipboard.writeText(tracesText || "");
    } catch (e) {
      console.error("copy failed", e);
    }
  }

  function logButtonEvent(name, action) {
    try {
      const now = Date.now();
      let px = null,
        py = null,
        dx = null,
        dy = null,
        plx = null,
        ply = null,
        angle_deg = null;
      if (pkg) {
        try {
          px = pkg.get_pos_x();
          py = pkg.get_pos_y();
          dx = pkg.get_dir_x();
          dy = pkg.get_dir_y();
          plx = pkg.get_plane_x();
          ply = pkg.get_plane_y();
          angle_deg = (Math.atan2(dy, dx) * 180) / Math.PI;
        } catch (e) {
          // ignore sampling failure
        }
      }
      const entry = {
        id: nextTraceId++,
        timestamp_ms: now,
        position: { x: px, y: py },
        direction: { x: dx, y: dy },
        plane: { x: plx, y: ply },
        angle_deg,
        button: { name, action },
      };
      traces.push(entry);
      if (traces.length > TRACE_MAX_LINES) traces.shift();
      _updateTracesText();
    } catch (e) {
      console.error("logButtonEvent failed", e);
    }
  }

  function regenFromInput() {
    if (!pkg) return;
    const seed = Number(seedInput) || 0;
    try {
      pkg.regenerate_map(seed);
      lastSeed.set(seed);
    } catch (e) {
      console.error("regenerate_map failed", e);
    }
  }

  function regenWithSize() {
    const w = Number(widthInput) || 0;
    const h = Number(heightInput) || 0;
    const seed = Number(seedInput) || 0;
    if (!pkg) return;
    try {
      if (typeof pkg.regenerate_map_with_size === "function") {
        pkg.regenerate_map_with_size(w, h, seed);
      } else {
        pkg.regenerate_map(seed);
        setTimeout(() => {
          try {
            const mw = pkg.get_map_w();
            const mh = pkg.get_map_h();
            console.log("map size after regen:", mw, mh);
          } catch (e) {}
        }, 50);
      }
      lastSeed.set(seed);
    } catch (e) {
      console.error("regen failed", e);
    }
  }

  onMount(() => {
    const unsubLast = lastSeed.subscribe((v) => {
      seedInput = v ? String(v) : "";
    });
    return () => {
      stopTrace();
      unsubLast();
    };
  });
</script>

<div class="panel">
  <div style="display:flex; justify-content:flex-end; margin-bottom:6px;">
    <button
      title="Hide panel"
      on:click={() => showSidebar.update((v) => !v)}
      on:mousedown={() => logButtonEvent("Toggle", "down")}
      on:mouseup={() => logButtonEvent("Toggle", "up")}
      on:touchstart={() => logButtonEvent("Toggle", "down")}
      on:touchend={() => logButtonEvent("Toggle", "up")}
      style="padding:4px 6px; font-size:12px;">Toggle</button
    >
    >
  </div>
  <div class="row">
    <label for="debug-seed" style="width:64px">Seed</label>
    <input
      id="debug-seed"
      type="text"
      bind:value={seedInput}
      placeholder="0 = random"
    />
    <button
      on:click={regenFromInput}
      on:mousedown={() => logButtonEvent("Regenerate", "down")}
      on:mouseup={() => logButtonEvent("Regenerate", "up")}
      on:touchstart={() => logButtonEvent("Regenerate", "down")}
      on:touchend={() => logButtonEvent("Regenerate", "up")}
      title="Regenerate using seed">Regenerate</button
    >
    >
  </div>

  <div class="row">
    <label for="debug-width" style="width:64px">Size</label>
    <input
      id="debug-width"
      type="number"
      min="0"
      bind:value={widthInput}
      placeholder="width"
    />
    <input
      id="debug-height"
      type="number"
      min="0"
      bind:value={heightInput}
      placeholder="height"
    />
    <button
      on:click={regenWithSize}
      on:mousedown={() => logButtonEvent("Apply", "down")}
      on:mouseup={() => logButtonEvent("Apply", "up")}
      on:touchstart={() => logButtonEvent("Apply", "down")}
      on:touchend={() => logButtonEvent("Apply", "up")}
      title="Regenerate with optional size">Apply</button
    >
    >
  </div>

  <div class="row">
    <label for="debug-stats" style="width:64px">Stats</label>
    <input id="debug-stats" type="checkbox" bind:checked={$showStats} />
    <div style="flex:1; text-align:right; font-size:12px; opacity:0.8">
      FPS / ms overlay
    </div>
  </div>

  <div style="margin-top:6px">
    <label for="debug-traces" style="display:block; margin-bottom:4px"
      >Trace</label
    >
    <div style="display:flex; gap:6px; margin-bottom:6px;">
      <button
        on:click={startTrace}
        on:mousedown={() => logButtonEvent("Start trace", "down")}
        on:mouseup={() => logButtonEvent("Start trace", "up")}
        on:touchstart={() => logButtonEvent("Start trace", "down")}
        on:touchend={() => logButtonEvent("Start trace", "up")}
        disabled={traceInterval}
        title="Start recording traces">Start trace</button
      >
      >
      <button
        on:click={stopTrace}
        on:mousedown={() => logButtonEvent("Stop trace", "down")}
        on:mouseup={() => logButtonEvent("Stop trace", "up")}
        on:touchstart={() => logButtonEvent("Stop trace", "down")}
        on:touchend={() => logButtonEvent("Stop trace", "up")}
        disabled={!traceInterval}
        title="Stop recording">Stop trace</button
      >
      >
      <button
        on:click={copyTrace}
        on:mousedown={() => logButtonEvent("Copy trace", "down")}
        on:mouseup={() => logButtonEvent("Copy trace", "up")}
        on:touchstart={() => logButtonEvent("Copy trace", "down")}
        on:touchend={() => logButtonEvent("Copy trace", "up")}
        title="Copy full trace to clipboard">Copy trace</button
      >
      >
      <button
        on:click={clearTrace}
        on:mousedown={() => logButtonEvent("Clear trace", "down")}
        on:mouseup={() => logButtonEvent("Clear trace", "up")}
        on:touchstart={() => logButtonEvent("Clear trace", "down")}
        on:touchend={() => logButtonEvent("Clear trace", "up")}
        title="Clear trace textarea">Clear trace</button
      >
      >
      <div style="flex:1; text-align:right; font-size:12px; opacity:0.8">
        Interval: {TRACE_INTERVAL_MS}ms · Max lines: {TRACE_MAX_LINES}
      </div>
    </div>
    <textarea id="debug-traces" readonly bind:value={tracesText}></textarea>
  </div>
</div>

<style>
  .panel {
    background: rgba(0, 0, 0, 0.85);
    color: #fff;
    padding: 8px;
    font-family: monospace;
    width: 100%;
    box-sizing: border-box;
  }
  .row {
    display: flex;
    gap: 8px;
    margin-bottom: 6px;
    align-items: center;
  }
  input[type="text"],
  input[type="number"] {
    flex: 1;
    padding: 4px;
  }
  textarea {
    width: 100%;
    height: 200px;
    max-height: calc(100vh - 220px);
    background: #111;
    color: #0f0;
    padding: 8px;
    font-family: monospace;
    font-size: 12px;
    overflow: auto;
    box-sizing: border-box;
  }
</style>
