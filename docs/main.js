import init, { NES } from "./pkg/nes_emulator.js";

// ═══════════════════════════════════════════════════════
//  State
// ═══════════════════════════════════════════════════════

let emu        = null;
let running    = false;
let rafHandle  = null;
let audioCtx   = null;
let nesNode    = null;
let audioBufferLevel = 0;
const AUDIO_BUF_SAMPLES = 1024;             // upper bound for one frame's audio
const AUDIO_POOL_INITIAL = 8;
const TARGET_FILL_MS = 120;
let TARGET_FILL_SAMPLES = 0;
let pumpHandle = null;
const audioBufferPool = [];                 // ArrayBuffer[]

let wasmMemory = null; 
let frameView  = null;

let debugOverlay  = false;

// Lightweight FPS counter for fullscreen overlay
let fps_lastTime  = 0;
let fps_frames    = 0;
let fps_display   = 0;

// Performance breakdown overlay (EMA-smoothed, fullscreen only)
const PERF_ALPHA  = 0.15;
let perf_wasm     = 0;   // ms per run_frame() call
let perf_render   = 0;   // ms for renderFullscreenFrame()
let perf_gap      = 0;   // ms between consecutive rendered frames (scheduler jitter)
let perf_prevTs   = 0;   // RAF timestamp of last rendered frame

// "nes" | "cpu" | "fullscreen"
let mode       = "nes";
// Mode we came from before entering fullscreen
let preFullscreenMode = "nes";

// ═══════════════════════════════════════════════════════
//  DOM helpers
// ═══════════════════════════════════════════════════════

const $  = (id) => document.getElementById(id);

function log(msg) {
  const el = $("log");
  if (!el) return;
  const t = new Date().toLocaleTimeString();
  el.innerText += `[${t}] ${msg}\n`;
  el.scrollTop = el.scrollHeight;
}

// ═══════════════════════════════════════════════════════
//  NES palette
// ═══════════════════════════════════════════════════════

const NES_PALETTE = [
  [84,84,84],[0,30,116],[8,16,144],[48,0,136],[68,0,100],[92,0,48],[84,4,0],[60,24,0],
  [32,42,0],[8,58,0],[0,64,0],[0,60,0],[0,50,60],[0,0,0],[0,0,0],[0,0,0],
  [152,150,152],[8,76,196],[48,50,236],[92,30,228],[136,20,176],[160,20,100],[152,34,32],[120,60,0],
  [84,90,0],[40,114,0],[8,124,0],[0,118,40],[0,102,120],[0,0,0],[0,0,0],[0,0,0],
  [236,238,236],[76,154,236],[120,124,236],[176,98,236],[228,84,236],[236,88,180],[236,106,100],[212,136,32],
  [160,170,0],[116,196,0],[76,208,32],[56,204,108],[56,180,204],[60,60,60],[0,0,0],[0,0,0],
  [236,238,236],[168,204,236],[188,188,236],[212,178,236],[236,174,236],[236,174,212],[236,180,176],[228,196,144],
  [204,210,120],[180,222,120],[168,226,144],[152,226,180],[160,214,228],[160,162,160],[0,0,0],[0,0,0],
];

const NES_PALETTE_U32 = new Uint32Array(64);
for (let i = 0; i < 64; i++) {
  const [r, g, b] = NES_PALETTE[i];
  NES_PALETTE_U32[i] = (255 << 24) | (b << 16) | (g << 8) | r; // little-endian RGBA
}



// ═══════════════════════════════════════════════════════
//  Canvas / image data
// ═══════════════════════════════════════════════════════

// Debug canvas (inside card)
let ctx         = null;
let imageData   = null;
let imageData32 = null;

// Fullscreen canvas
let fsCtx       = null;
let fsImageData = null;
let fsImageData32 = null;

// Pattern table canvases
let pattern0Ctx       = null;
let pattern1Ctx       = null;
let pattern0ImageData = null;
let pattern1ImageData = null;

function initCanvas() {
  const screen = $("screen");
  if (screen) {
    ctx       = screen.getContext("2d");
    imageData = ctx.createImageData(256, 240);
    imageData32 = new Uint32Array(imageData.data.buffer);
  }

  const fsScreen = $("fsScreen");
  if (fsScreen) {
    fsCtx       = fsScreen.getContext("2d");
    fsImageData = fsCtx.createImageData(256, 240);
    fsImageData32 = new Uint32Array(fsImageData.data.buffer);
  }

  const p0 = $("pattern0");
  if (p0) { pattern0Ctx = p0.getContext("2d"); pattern0ImageData = pattern0Ctx.createImageData(128,128); }

  const p1 = $("pattern1");
  if (p1) { pattern1Ctx = p1.getContext("2d"); pattern1ImageData = pattern1Ctx.createImageData(128,128); }
}

// ═══════════════════════════════════════════════════════
//  Status bar
// ═══════════════════════════════════════════════════════

function setStatus(ok, text) {
  $("statusDot").style.background = ok ? "#36d399" : "#ff5c7c";
  $("statusText").innerText = text;
}

// ═══════════════════════════════════════════════════════
//  CPU flag rendering
// ═══════════════════════════════════════════════════════

function formatStatusFlags(status) {
  return [["N",7],["V",6],["U",5],["B",4],["D",3],["I",2],["Z",1],["C",0]]
    .map(([name, bit]) => {
      const v = (status >> bit) & 1;
      return `<span class="flag ${v ? "flag-on" : "flag-off"}">${name}</span>`;
    }).join(" ");
}

// ═══════════════════════════════════════════════════════
//  UI update (only runs in debug modes)
// ═══════════════════════════════════════════════════════

function renderRegisters() {
  if (!emu) return;
  const [a, x, y, sp, pc, status] = emu.get_registers();
  $("a").innerText  = a .toString(16).padStart(2,"0").toUpperCase();
  $("x").innerText  = x .toString(16).padStart(2,"0").toUpperCase();
  $("y").innerText  = y .toString(16).padStart(2,"0").toUpperCase();
  $("sp").innerText = sp.toString(16).padStart(2,"0").toUpperCase();
  $("pc").innerText = pc.toString(16).padStart(4,"0").toUpperCase();
  $("status").innerHTML = formatStatusFlags(status);

  const [fetched, addr_abs, addr_rel, opcode, cycles] = emu.get_cpu_state();
  $("fetched")  .innerText = fetched  .toString(16).padStart(2,"0").toUpperCase();
  $("addr_abs") .innerText = addr_abs .toString(16).padStart(4,"0").toUpperCase();
  $("addr_rel") .innerText = addr_rel .toString(16).padStart(4,"0").toUpperCase();
  $("opcode")   .innerText = opcode   .toString(16).padStart(2,"0").toUpperCase();
  $("cycles")   .innerText = cycles   .toString(16).padStart(2,"0").toUpperCase();
}

function renderRam() {
  if (!emu) return;
  const grid = $("ramGrid");
  grid.innerHTML = "";
  const ram = emu.get_ram(0, 0x0800);
  const [,,,, pc] = emu.get_registers();
  const [, addr_abs] = emu.get_cpu_state();

  for (let row = 0; row < 128; row++) {
    const base = row * 16;
    const label = document.createElement("div");
    label.className = "cell row-label";
    label.innerText = base.toString(16).padStart(4,"0").toUpperCase();
    grid.appendChild(label);

    for (let col = 0; col < 16; col++) {
      const addr = base + col;
      const cell = document.createElement("div");
      cell.className = "cell";
      cell.innerText = ram[addr].toString(16).padStart(2,"0").toUpperCase();
      if (addr === pc)       cell.classList.add("pc-highlight");
      if (addr === addr_abs) cell.classList.add("addr-highlight");
      grid.appendChild(cell);
    }
  }
}

function updateDebugUI() {
  renderRegisters();
  if (mode === "cpu") renderRam();
}

// ═══════════════════════════════════════════════════════
//  Frame rendering helpers
// ═══════════════════════════════════════════════════════

function writeFrameToImageData(frame, data32) {
  const pal = NES_PALETTE_U32;
  for (let i = 0; i < frame.length; i++) {
    data32[i] = pal[frame[i] & 0x3f];
  }
}

// This is a pointer to my emulator's memory
function getFrameView() {
  // Recreate only if memory grew (buffer was replaced)
  if (frameView === null || frameView.buffer !== wasmMemory.buffer) {
    frameView = new Uint8Array(
      wasmMemory.buffer,
      emu.frame_ptr(),
      emu.frame_len()
    );
  }
  return frameView;
}

function renderDebugFrame() {
  if (!emu || !ctx) return;
  writeFrameToImageData(getFrameView(), imageData32);
  ctx.putImageData(imageData, 0, 0);
}

function renderFullscreenFrame() {
  if (!emu || !fsCtx) return;
  writeFrameToImageData(getFrameView(), fsImageData32);
  fsCtx.putImageData(fsImageData, 0, 0);
}

function renderPatternTableToCanvas(buffer, imgData, canvasCtx) {
  if (!buffer || !imgData || !canvasCtx) return;
  for (let i = 0; i < buffer.length; i++) {
    const [r,g,b] = NES_PALETTE[buffer[i] & 0x3f];
    imgData.data[i*4+0] = r; imgData.data[i*4+1] = g;
    imgData.data[i*4+2] = b; imgData.data[i*4+3] = 255;
  }
  canvasCtx.putImageData(imgData, 0, 0);
}

let patternFrameCounter = 0;

function renderPatternTables() {
  if (!emu || !pattern0Ctx || !pattern1Ctx) return;
  const palette = Number($("paletteSelect")?.value ?? 0);
  renderPatternTableToCanvas(emu.get_pattern_table(0, palette), pattern0ImageData, pattern0Ctx);
  renderPatternTableToCanvas(emu.get_pattern_table(1, palette), pattern1ImageData, pattern1Ctx);
}

// ═══════════════════════════════════════════════════════
//  Audio
// ═══════════════════════════════════════════════════════
async function initAudio() {
  const blob = new Blob([`
  class NESProcessor extends AudioWorkletProcessor {
    constructor() {
      super();
      this.RING = 16384;
      this.MASK = this.RING - 1;
      this.ring = new Float32Array(this.RING);
      this.r = 0;
      this.w = 0;
      this.lastReportedR = 0;
      this.tick = 0;

      this.port.onmessage = ({ data }) => {
        // Reset signal from main thread
        if (data && data.type === "reset") {
          this.r = 0; this.w = 0; this.lastReportedR = 0;
          return;
        }

        // Otherwise it's a Float32Array of samples (buffer was transferred to us)
        const buf = data;
        const len = buf.length;
        const free = this.RING - (this.w - this.r);
        if (len > free) {
          // Should never happen with proper pacing; drop oldest, never tear
          this.r = this.w - (this.RING - len);
        }

        const writePos = this.w & this.MASK;
        const first = Math.min(len, this.RING - writePos);
        this.ring.set(buf.subarray(0, first), writePos);
        if (first < len) this.ring.set(buf.subarray(first), 0);
        this.w += len;

        // Bounce the ArrayBuffer back to main for reuse — zero-copy
        this.port.postMessage(buf.buffer, [buf.buffer]);
      };
    }

    process(_, outputs) {
      const out = outputs[0][0];
      const need = out.length;
      const available = this.w - this.r;

      // Report consumption delta every ~23 ms (8 quanta at 128/44100)
      if ((++this.tick & 7) === 0) {
        const delta = this.r - this.lastReportedR;
        if (delta > 0) {
          this.lastReportedR = this.r;
          this.port.postMessage({ type: "consumed", n: delta });
        }
      }

      if (available < need) { out.fill(0); return true; }

      const readPos = this.r & this.MASK;
      const first = Math.min(need, this.RING - readPos);
      out.set(this.ring.subarray(readPos, readPos + first));
      if (first < need) out.set(this.ring.subarray(0, need - first), first);
      this.r += need;
      return true;
    }
  }
  registerProcessor("nes-processor", NESProcessor);
`], { type: "application/javascript" });

  audioCtx = new AudioContext({ sampleRate: 44100 });
  console.log("actual sampleRate:", audioCtx.sampleRate);  // add this

  await audioCtx.audioWorklet.addModule(URL.createObjectURL(blob));
  nesNode = new AudioWorkletNode(audioCtx, "nes-processor");
  nesNode.connect(audioCtx.destination);

  // Track buffer level reported back from worklet
  nesNode.port.onmessage = ({ data }) => {
    if (data instanceof ArrayBuffer) {
      // Worklet returned a buffer — back into the pool
      audioBufferPool.push(data);
    } else if (data && data.type === "consumed") {
      // Worklet consumed `n` samples since last report
      audioBufferLevel = Math.max(0, audioBufferLevel - data.n);
    }
  };

  initAudioBufferPool();
}




function initAudioBufferPool() {
  for (let i = 0; i < AUDIO_POOL_INITIAL; i++) {
    audioBufferPool.push(new ArrayBuffer(AUDIO_BUF_SAMPLES * 4));
  }
}


function startPump() {
  if (pumpHandle) return;
  TARGET_FILL_SAMPLES = (audioCtx.sampleRate * TARGET_FILL_MS / 1000) | 0;
  pumpHandle = setInterval(pump, 4);
}

function stopPump() {
  if (pumpHandle) { clearInterval(pumpHandle); pumpHandle = null; }
}

function pump() {
  if (!running) return;
  if (mode !== "nes" && mode !== "fullscreen") return;
  if (!nesNode || audioCtx.state !== "running") return;

  // Run frames until we're at or above target. Safety cap stops runaway
  // catch-up after a tab-throttle gap.
  let safety = 8;
  while (audioBufferLevel < TARGET_FILL_SAMPLES && safety-- > 0 && running) {
    const t_wasm = performance.now();
    emu.run_frame();
    perf_wasm = perf_wasm * (1 - PERF_ALPHA) + (performance.now() - t_wasm) * PERF_ALPHA;
    const len = emu.audio_len();
    if (len === 0) break;

    // Re-derive WASM view each call — buffer can detach on heap growth
    const src = new Float32Array(wasmMemory.buffer, emu.audio_ptr(), len);

    let buf = audioBufferPool.pop();
    if (!buf || buf.byteLength < len * 4) {
      buf = new ArrayBuffer(Math.max(AUDIO_BUF_SAMPLES, len) * 4);
    }
    const dst = new Float32Array(buf, 0, len);
    dst.set(src);

    nesNode.port.postMessage(dst, [buf]);   // transfer, no copy, no GC
    audioBufferLevel += len;
  }
}

// ═══════════════════════════════════════════════════════
//  Run loops
//  Three independent loops: cpu-debug, nes-debug, nes-fullscreen
// ═══════════════════════════════════════════════════════

// Audio-clock-driven sync state
const NES_FPS         = 60.0988;   // NTSC
let audioClockStart   = null;      // audioCtx.currentTime snapshot at loop start
let emuFramesProduced = 0;         // frames run since audioClockStart was set

// Debug counters for fullscreen loop
let dbg_rafCount      = 0;
let dbg_fpsTimestamp  = 0;
let dbg_fps           = 0;
let dbg_framesThisTick = 0;
let dbg_driftMax      = 0;

const TARGET_FRAME_MS = 1000 / 60.0988;  // ~16.639 ms
let lastRenderTime    = 0;

function frame(timestamp) {
  if (!running) return;

  // Throttle render to NES framerate — skips on high-refresh-rate displays
  // (90 Hz phones, 120 Hz tablets) so we don't burn GPU for no benefit.
  const elapsed = timestamp - lastRenderTime;
  if (elapsed < TARGET_FRAME_MS - 1) {   // -1 ms tolerance for scheduler jitter
    rafHandle = requestAnimationFrame(frame);
    return;
  }
  lastRenderTime = timestamp - (elapsed % TARGET_FRAME_MS);
  
  const t0 = performance.now();
  let tWasm = 0, tRender = 0, tUI = 0;
  
  try {
    if (mode === "cpu") {
      emu.cpu_clock();
      updateDebugUI();
      
    } else if (mode === "nes") {
      const b = performance.now();
      renderDebugFrame();
      tRender = performance.now() - b;

      patternFrameCounter++;
      if (patternFrameCounter >= 60) { renderPatternTables(); patternFrameCounter = 0; }

      const c = performance.now();
      updateDebugUI();
      tUI = performance.now() - c;

    } else if (mode === "fullscreen") {
      dbg_rafCount++;
      if (dbg_rafCount === 1) dbg_fpsTimestamp = performance.now();

      const b = performance.now();
      renderFullscreenFrame();
      tRender = performance.now() - b;

      perf_render = perf_render * (1 - PERF_ALPHA) + tRender * PERF_ALPHA;
      if (perf_prevTs > 0)
        perf_gap = perf_gap * (1 - PERF_ALPHA) + (timestamp - perf_prevTs) * PERF_ALPHA;
      perf_prevTs = timestamp;

      // FPS counter — update display value once per second
      fps_frames++;
      const fps_now = performance.now();
      if (fps_now - fps_lastTime >= 1000) {
        fps_display  = fps_frames;
        fps_frames   = 0;
        fps_lastTime = fps_now;
      }

      if (debugOverlay) {
        // Draw FPS overlay directly on the fullscreen canvas (after putImageData)
        fsCtx.font         = "bold 10px monospace";
        fsCtx.textBaseline = "top";
        fsCtx.fillStyle    = "rgba(0,0,0,0.55)";
        fsCtx.fillRect(2, 2, 38, 14);
        fsCtx.fillStyle    = "#ffffff";
        fsCtx.fillText(`${fps_display} FPS`, 5, 4);

        // Performance breakdown — bottom-right corner, all white
        const boxW = 99, boxH = 46;
        const boxX = 256 - boxW - 2, boxY = 240 - boxH - 2;
        fsCtx.fillStyle = "rgba(0,0,0,0.55)";
        fsCtx.fillRect(boxX, boxY, boxW, boxH);
        fsCtx.fillStyle = "#ffffff";
        const tx = boxX + 4;
        fsCtx.fillText(`wasm:   ${perf_wasm.toFixed(1)} ms`,   tx, boxY + 4);
        fsCtx.fillText(`render: ${perf_render.toFixed(1)} ms`, tx, boxY + 17);
        fsCtx.fillText(`gap:    ${perf_gap.toFixed(1)} ms`,    tx, boxY + 30);
      }
    }
  } catch (e) {
    running = false;
    setStatus(false, "Runtime error");
    log(`ERROR: ${e}`);
    return;
  }

  
  rafHandle = requestAnimationFrame(frame);
}

function startRun() {
  if (!emu || running) return;
  running = true;
  audioBufferLevel = 0;
  lastRenderTime = 0;
  startPump();
  rafHandle = requestAnimationFrame(frame);
  if (mode !== "fullscreen") log("Run started");
}

function pauseRun() {
  running = false;
  stopPump();
  if (rafHandle) { cancelAnimationFrame(rafHandle); rafHandle = null; }
  if (mode !== "fullscreen") log("Paused");
}

// ═══════════════════════════════════════════════════════
//  Controller
// ═══════════════════════════════════════════════════════

const controller1 = { x:0, z:0, a:0, s:0, up:0, down:0, left:0, right:0 };

function syncController() {
  if (!emu) return;
  emu.set_controller(0,
    controller1.x, controller1.z, controller1.a, controller1.s,
    controller1.up, controller1.down, controller1.left, controller1.right
  );
}

function setButtonState(name, pressed) {
  if (!(name in controller1)) return;
  controller1[name] = pressed ? 1 : 0;
  syncController();
}

function keyToButton(code) {
  switch (code) {
    case "KeyA":      return "x";
    case "KeyF":      return "z";
    case "KeyS":      return "s";
    case "KeyD":      return "a";
    case "ArrowUp":   return "up";
    case "ArrowDown": return "down";
    case "ArrowLeft": return "left";
    case "ArrowRight":return "right";
    default: return null;
  }
}

function releaseAllButtons() {
  for (const k in controller1) controller1[k] = 0;
  syncController();
  document.querySelectorAll("[data-btn]").forEach(el => el.classList.remove("pressed"));
}

// ═══════════════════════════════════════════════════════
//  Mode switching
// ═══════════════════════════════════════════════════════

function applyMode(newMode) {
  mode = newMode;
  document.body.dataset.mode = newMode;

  // Update nav button states
  document.querySelectorAll(".mode-switch button[data-mode]").forEach(b => {
    const isActive = b.dataset.mode === newMode;
    b.classList.toggle("active", isActive);
    b.setAttribute("aria-pressed", isActive ? "true" : "false");
  });
}

function switchDebugMode(btn) {
  const newMode = btn.dataset.mode; // "nes" or "cpu"
  pauseRun();
  applyMode(newMode);

  if (newMode === "nes") {
    startRun();
    log("Switched to NES Debug mode");
  } else {
    log("Switched to 6502 Lab mode");
    loadProgram();
  }
}

// ── Fullscreen entry / exit ──────────────────────────

async function enterFullscreen() {
  if (!emu) { log("No emulator loaded"); return; }

  preFullscreenMode = mode;
  pauseRun();
  applyMode("fullscreen");
  releaseAllButtons();

  // Request real OS/browser fullscreen on the overlay element.
  // Must be called synchronously inside the user-gesture handler.
  const el = $("fullscreenOverlay");
  try {
    if (el.requestFullscreen) {
      await el.requestFullscreen({ navigationUI: "hide" });
    } else if (el.webkitRequestFullscreen) {
      await el.webkitRequestFullscreen();
    }
  } catch (e) {
    // Browser denied (e.g. iframe sandbox) — overlay already visible, carry on.
    log(`Native fullscreen unavailable: ${e.message ?? e}`);
  }
  if (audioCtx?.state === "suspended") await audioCtx.resume()
  // Let the first RAF tick anchor the clock; avoids catch-up after a pause
  audioClockStart = null;
  dbg_rafCount    = 0;
  $("fsScreen")?.focus();
  startRun();
  log("Entered fullscreen mode");
}

function _leaveFullscreenMode() {
  if (mode !== "fullscreen") return;
  pauseRun();
  releaseAllButtons();
  applyMode(preFullscreenMode);
  if (preFullscreenMode === "nes") {
    startRun();
  } else {
    loadProgram();
  }
  $("screen")?.focus();
  log("Exited fullscreen mode");
}

async function exitFullscreen() {
  // Ask browser to leave native fullscreen; the fullscreenchange handler
  // will call _leaveFullscreenMode() once the transition completes.
  // If native fullscreen isn't active (e.g. it was denied), clean up directly.
  const isNativeFs = document.fullscreenElement || document.webkitFullscreenElement;
  if (isNativeFs) {
    try {
      if (document.exitFullscreen)       await document.exitFullscreen();
      else if (document.webkitExitFullscreen) document.webkitExitFullscreen();
    } catch (e) { /* ignore */ }
    // _leaveFullscreenMode will be triggered by fullscreenchange
  } else {
    _leaveFullscreenMode();
  }
}

// Sync our app state whenever the browser fullscreen state changes
// (covers: Escape key, browser back button, swipe-up on Android, etc.)
function onFullscreenChange() {
  const isNativeFs = document.fullscreenElement || document.webkitFullscreenElement;
  if (!isNativeFs && mode === "fullscreen") {
    _leaveFullscreenMode();
  }
}
document.addEventListener("fullscreenchange",       onFullscreenChange);
document.addEventListener("webkitfullscreenchange", onFullscreenChange);

// ── Keyboard exit (Escape) ───────────────────────────
// Note: browsers fire Escape → fullscreenchange automatically for native
// fullscreen, so _leaveFullscreenMode is called via that event.
// This handler covers the non-native fallback case.
window.addEventListener("keydown", (e) => {
  if (e.code === "Escape" && mode === "fullscreen") {
    exitFullscreen();
    return;
  }

  if (e.code === "KeyQ" && mode === "fullscreen") { e.preventDefault(); saveState(); return; }
  if (e.code === "KeyE" && mode === "fullscreen") { e.preventDefault(); loadState(); return; }

  const btn = keyToButton(e.code);
  if (!btn) return;
  e.preventDefault();
  setButtonState(btn, true);

  // Visual feedback on fullscreen buttons
  document.querySelectorAll(`[data-btn="${btn}"]`)
    .forEach(el => el.classList.add("pressed"));
});

window.addEventListener("keyup", (e) => {
  const btn = keyToButton(e.code);
  if (!btn) return;
  e.preventDefault();
  setButtonState(btn, false);

  document.querySelectorAll(`[data-btn="${btn}"]`)
    .forEach(el => el.classList.remove("pressed"));
});

// ── Visibility / blur cleanup ────────────────────────

window.addEventListener("blur", releaseAllButtons);
document.addEventListener("visibilitychange", () => {
  if (document.hidden) {
    releaseAllButtons();
    if (audioCtx?.state === "running") audioCtx.suspend().catch(() => {});
    stopPump();
  } else if (running) {
    if (audioCtx?.state === "suspended") audioCtx.resume().catch(() => {});
    if (nesNode) nesNode.port.postMessage({ type: "reset" });
    audioBufferLevel = 0;
    startPump();
  }
});

// ═══════════════════════════════════════════════════════
//  Touch controller wiring (works in both NES debug and fullscreen)
// ═══════════════════════════════════════════════════════

function wireTouchButton(btnEl) {
  const btnName = btnEl.dataset.btn;

  const press = (e) => {
    e.preventDefault();
    btnEl.setPointerCapture?.(e.pointerId);
    btnEl.classList.add("pressed");
    setButtonState(btnName, true);
    $("fsScreen")?.focus();
  };

  const release = (e) => {
    e.preventDefault();
    btnEl.classList.remove("pressed");
    setButtonState(btnName, false);
  };

  btnEl.addEventListener("pointerdown",       press);
  btnEl.addEventListener("pointerup",         release);
  btnEl.addEventListener("pointercancel",     release);
  btnEl.addEventListener("lostpointercapture",release);
  btnEl.addEventListener("pointerleave", (e) => {
    if (e.pointerType === "mouse") release(e);
  });
}

// ═══════════════════════════════════════════════════════
//  UI binding
// ═══════════════════════════════════════════════════════

function bindUI() {
  // Resume AudioContext on any interaction (browsers can suspend it)
  document.addEventListener("pointerdown", () => {
    if (audioCtx?.state === "suspended") audioCtx.resume();
  });

  // Debug mode buttons
  document.querySelectorAll(".mode-switch button[data-mode]").forEach(btn => {
    btn.addEventListener("click", () => switchDebugMode(btn));
  });

  // Fullscreen button (separate from mode switch)
  $("btnEnterFullscreen").addEventListener("click", enterFullscreen);
  $("fsExitBtn")         .addEventListener("click", exitFullscreen);
  $("fsSaveBtn")         .addEventListener("click", saveState);
  $("fsLoadBtn")         .addEventListener("click", loadState);
  $("fsDownloadBtn")     .addEventListener("click", downloadState);
  $("fsUploadBtn")       .addEventListener("click", uploadState);

  // Debug controls
  $("btnReset").addEventListener("click", () => {
    if (!emu) return;
    pauseRun();
    emu.reset();
    if (nesNode) nesNode.port.postMessage({ type: "reset" });
    audioBufferLevel = 0;
    updateDebugUI();
    log("Reset");
  });

  $("btnClock").addEventListener("click", () => {
    if (!emu) return;
    if (mode === "cpu") emu.cpu_clock(); else emu.clock();
    renderDebugFrame(); updateDebugUI(); log("Clock()");
  });

  $("btnStep").addEventListener("click", () => {
    if (!emu) return;
    emu.step_instruction(); renderDebugFrame(); updateDebugUI();
    log("Step instruction()");
  });

  $("btnAssemble").addEventListener("click", () => {
    if (!emu) return;
    pauseRun(); loadProgram();
  });

  $("btnRun").  addEventListener("click", startRun);
  $("btnPause").addEventListener("click", pauseRun);

  $("btnLoadROM").addEventListener("click", () => $("romLoader").click());

  $("romLoader").addEventListener("change", async (e) => {
    const file = e.target.files[0];
    if (file) await loadRomFile(file);
  });

  $("paletteSelect")?.addEventListener("change", renderPatternTables);

  // Touch buttons — wire all [data-btn] elements
  document.querySelectorAll("[data-btn]").forEach(wireTouchButton);

  // Note: no global pointerup → releaseAllButtons here.
  // Each touch button handles its own release via pointerup/pointercancel/
  // lostpointercapture on the element itself, so held keyboard keys are
  // never accidentally cleared when a touch button is lifted.

  // Virtual joystick
  initJoystick();
}

// ═══════════════════════════════════════════════════════
//  Virtual joystick
// ═══════════════════════════════════════════════════════

function initJoystick() {
  const canvas = document.getElementById("joystick");
  if (!canvas) return;

  // Resize canvas backing store to match CSS size (handles hi-DPI too)
  function resizeJoystick() {
    const rect = canvas.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;
    const dpr  = window.devicePixelRatio || 1;
    canvas.width  = rect.width  * dpr;
    canvas.height = rect.height * dpr;
    drawJoystick();
  }

  const ctx2d = canvas.getContext("2d");

  // Joystick state
  let active    = false;  // finger is down
  let thumbX    = 0;      // offset from centre, normalised -1..1
  let thumbY    = 0;
  let lastDir   = null;   // last 8-way direction string we applied

  // 8-direction snap: returns one of "N","NE","E","SE","S","SW","W","NW" or null
  function snapDir(nx, ny) {
    const DEADZONE = 0.25;
    if (Math.sqrt(nx*nx + ny*ny) < DEADZONE) return null;

    // atan2: 0 = east, positive = clockwise (screen coords: y down)
    let angle = Math.atan2(ny, nx) * 180 / Math.PI; // -180..180
    if (angle < 0) angle += 360;                     // 0..360, 0=E, 90=S

    // Rotate so 0 = North, divide into 8 × 45° sectors
    const adjusted = (angle + 90) % 360;             // 0=N, 90=E, 180=S, 270=W
    const sector   = Math.round(adjusted / 45) % 8;  // 0..7
    const DIRS     = ["N","NE","E","SE","S","SW","W","NW"];
    return DIRS[sector];
  }

  // Map direction string → { up, down, left, right }
  function dirToButtons(dir) {
    return {
      up:    (dir === "N" || dir === "NE" || dir === "NW") ? 1 : 0,
      down:  (dir === "S" || dir === "SE" || dir === "SW") ? 1 : 0,
      left:  (dir === "W" || dir === "NW" || dir === "SW") ? 1 : 0,
      right: (dir === "E" || dir === "NE" || dir === "SE") ? 1 : 0,
    };
  }

  function applyDir(dir) {
    if (dir === lastDir) return;
    lastDir = dir;
    const btns = dirToButtons(dir ?? "");
    controller1.up    = btns.up;
    controller1.down  = btns.down;
    controller1.left  = btns.left;
    controller1.right = btns.right;
    syncController();
  }

  function releaseJoystick() {
    active = false;
    thumbX = 0;
    thumbY = 0;
    applyDir(null);
    drawJoystick();
  }

  // ── Drawing ─────────────────────────────────────────

  function drawJoystick() {
    const w   = canvas.width;
    const h   = canvas.height;
    if (w <= 0 || h <= 0) return;
    const cx  = w / 2;
    const cy  = h / 2;
    const R   = Math.min(cx, cy) - 4;   // base radius
    const TR  = R * 0.36;               // thumb radius
    const MAX = R - TR;                 // max thumb travel

    ctx2d.clearRect(0, 0, w, h);

    // Base ring
    ctx2d.beginPath();
    ctx2d.arc(cx, cy, R, 0, Math.PI * 2);
    ctx2d.strokeStyle = "rgba(255,255,255,0.18)";
    ctx2d.lineWidth   = 2;
    ctx2d.stroke();
    ctx2d.fillStyle   = "rgba(255,255,255,0.06)";
    ctx2d.fill();

    // Cardinal tick marks
    const ticks = active ? [] : [0, 90, 180, 270]; // hide when dragging
    for (const deg of [0, 90, 180, 270]) {
      const rad = (deg - 90) * Math.PI / 180;
      const ix  = cx + Math.cos(rad) * (R * 0.62);
      const iy  = cy + Math.sin(rad) * (R * 0.62);
      const ox  = cx + Math.cos(rad) * (R * 0.82);
      const oy  = cy + Math.sin(rad) * (R * 0.82);
      ctx2d.beginPath();
      ctx2d.moveTo(ix, iy);
      ctx2d.lineTo(ox, oy);
      ctx2d.strokeStyle = "rgba(255,255,255,0.22)";
      ctx2d.lineWidth   = 1.5;
      ctx2d.stroke();
    }

    // Direction highlight arc when active
    if (active && lastDir) {
      const SECTOR_ANGLE = 45;
      const NORTH_OFFSET = -90;
      const dirIndex = ["N","NE","E","SE","S","SW","W","NW"].indexOf(lastDir);
      const midAngle = dirIndex * SECTOR_ANGLE + NORTH_OFFSET;
      const startRad = (midAngle - SECTOR_ANGLE / 2) * Math.PI / 180;
      const endRad   = (midAngle + SECTOR_ANGLE / 2) * Math.PI / 180;

      ctx2d.beginPath();
      ctx2d.moveTo(cx, cy);
      ctx2d.arc(cx, cy, R - 1, startRad, endRad);
      ctx2d.closePath();
      ctx2d.fillStyle = "rgba(0, 113, 227, 0.28)";
      ctx2d.fill();
    }

    // Thumb
    const tx = cx + thumbX * MAX;
    const ty = cy + thumbY * MAX;

    // Thumb shadow ring
    ctx2d.beginPath();
    ctx2d.arc(tx, ty, TR + 3, 0, Math.PI * 2);
    ctx2d.fillStyle = "rgba(0,0,0,0.35)";
    ctx2d.fill();

    // Thumb body
    ctx2d.beginPath();
    ctx2d.arc(tx, ty, TR, 0, Math.PI * 2);
    ctx2d.fillStyle = active
      ? "rgba(0, 113, 227, 0.85)"
      : "rgba(255,255,255,0.22)";
    ctx2d.fill();
    ctx2d.strokeStyle = active
      ? "rgba(0, 113, 227, 1)"
      : "rgba(255,255,255,0.4)";
    ctx2d.lineWidth = 1.5;
    ctx2d.stroke();
  }

  // ── Pointer events ───────────────────────────────────

  function onPointerDown(e) {
    e.preventDefault();
    canvas.setPointerCapture(e.pointerId);
    active = true;
    updateThumb(e);
  }

  function onPointerMove(e) {
    if (!active) return;
    e.preventDefault();
    updateThumb(e);
  }

  function onPointerUp(e) {
    e.preventDefault();
    releaseJoystick();
  }

  function updateThumb(e) {
    const rect = canvas.getBoundingClientRect();
    const dpr  = window.devicePixelRatio || 1;
    const cx   = rect.width  / 2;
    const cy   = rect.height / 2;
    const R    = Math.min(cx, cy) - 4;
    const TR   = R * 0.36;
    const MAX  = R - TR;

    // Raw offset in CSS pixels
    let dx = e.clientX - rect.left - cx;
    let dy = e.clientY - rect.top  - cy;

    // Clamp to unit circle
    const dist = Math.sqrt(dx*dx + dy*dy);
    const norm = Math.min(dist, MAX);
    if (dist > 0) { dx = dx / dist * norm; dy = dy / dist * norm; }

    thumbX = dx / MAX;
    thumbY = dy / MAX;

    applyDir(snapDir(thumbX, thumbY));
    drawJoystick();
  }

  canvas.addEventListener("pointerdown",   onPointerDown,  { passive: false });
  canvas.addEventListener("pointermove",   onPointerMove,  { passive: false });
  canvas.addEventListener("pointerup",     onPointerUp,    { passive: false });
  canvas.addEventListener("pointercancel", onPointerUp,    { passive: false });

  // Re-draw when overlay becomes visible (canvas may have been zero-sized before)
  new ResizeObserver(() => resizeJoystick()).observe(canvas);
  resizeJoystick();
}

// ═══════════════════════════════════════════════════════
//  Save / Load state
// ═══════════════════════════════════════════════════════

const SAVE_KEY = "nes_save_state";

function saveState() {
  if (!emu) return;
  const bytes = emu.save_state();
  let binary = "";
  for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
  try {
    localStorage.setItem(SAVE_KEY, btoa(binary));
    log("State saved");
  } catch (e) {
    log(`Save failed: ${e.message}`);
  }
}

function loadState() {
  if (!emu) return;
  const b64 = localStorage.getItem(SAVE_KEY);
  if (!b64) { log("No save state found"); return; }
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  try {
    emu.load_state(bytes);
    frameView = null; // screen Vec reallocated on load — old pointer is stale
    if (nesNode) nesNode.port.postMessage({ type: "reset" });
    audioBufferLevel = 0;
    log("State loaded");
  } catch (e) {
    log(`Load failed: ${e}`);
  }
}

function downloadState() {
  if (!emu) return;
  const bytes = emu.save_state_json();
  const blob = new Blob([bytes], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "nes_state.json";
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(() => URL.revokeObjectURL(url), 100);
  log("State downloaded");
}

function uploadState() {
  if (!emu) return;
  const input = document.createElement("input");
  input.type = "file";
  input.accept = ".json,application/json";
  input.onchange = async () => {
    const file = input.files[0];
    if (!file) return;
    const bytes = new Uint8Array(await file.arrayBuffer());
    try {
      emu.load_state_json(bytes);
      frameView = null;
      if (nesNode) nesNode.port.postMessage({ type: "reset" });
      audioBufferLevel = 0;
      log("State uploaded");
    } catch (e) {
      log(`Upload failed: ${e}`);
    }
  };
  input.click();
}

// ═══════════════════════════════════════════════════════
//  Program / ROM loading
// ═══════════════════════════════════════════════════════

function loadProgram() {
  try {
    const src = $("asmInput").value;
    const program = parseHexProgram(src);
    emu.load_program(program, 0x0000);
    updateDebugUI();
    renderPatternTables();
    log(`Loaded ${program.length} bytes at $0000`);
  } catch (e) {
    log(`Parse error: ${e.message}`);
  }
}

async function loadRomFile(file) {
  if (!emu) return;
  const bytes = new Uint8Array(await file.arrayBuffer());
  emu.insert_cartridge(bytes);
  emu.reset();
  if (nesNode) nesNode.port.postMessage({ type: "reset" });
  audioBufferLevel = 0;
  if (!running) startRun();
  updateDebugUI();
  renderPatternTables();
  log(`Loaded ROM: ${file.name} — reset done`);
}

function parseHexProgram(text) {
  const cleaned = text.replace(/;.*/g,"").replace(/[^0-9a-fA-F]/g," ").trim();
  if (!cleaned) return new Uint8Array([]);
  return new Uint8Array(
    cleaned.split(/\s+/).map(b => {
      const v = parseInt(b, 16);
      if (isNaN(v) || v < 0 || v > 255) throw new Error(`Invalid byte: ${b}`);
      return v;
    })
  );
}

// ═══════════════════════════════════════════════════════
//  Boot
// ═══════════════════════════════════════════════════════

async function boot() {
  try {
    setStatus(false, "Loading WASM…");
    const wasm = await init();
    wasmMemory = wasm.memory; // Get pointer to WASM memory so that I don't have the reallocate the frame buffer

    emu = new NES();
    setStatus(true, "WASM loaded");

    bindUI();
    initCanvas();
    await initAudio();
    updateDebugUI();

    log("Emulator ready — load a ROM to start");
  } catch (e) {
    console.error(e);
    setStatus(false, "Failed to load WASM");
    log(`Failed to init WASM: ${e}`);
  }
}

boot();