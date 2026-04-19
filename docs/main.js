import init, { NES } from "./pkg/nes_emulator.js";

// ═══════════════════════════════════════════════════════
//  State
// ═══════════════════════════════════════════════════════

let emu        = null;
let running    = false;
let rafHandle  = null;

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

// ═══════════════════════════════════════════════════════
//  Canvas / image data
// ═══════════════════════════════════════════════════════

// Debug canvas (inside card)
let ctx         = null;
let imageData   = null;

// Fullscreen canvas
let fsCtx       = null;
let fsImageData = null;

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
  }

  const fsScreen = $("fsScreen");
  if (fsScreen) {
    fsCtx       = fsScreen.getContext("2d");
    fsImageData = fsCtx.createImageData(256, 240);
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

function writeFrameToImageData(frame, imgData) {
  for (let i = 0; i < frame.length; i++) {
    const [r,g,b] = NES_PALETTE[frame[i] & 0x3f];
    imgData.data[i*4+0] = r;
    imgData.data[i*4+1] = g;
    imgData.data[i*4+2] = b;
    imgData.data[i*4+3] = 255;
  }
}

function renderDebugFrame() {
  if (!emu || !ctx) return;
  const frame = emu.frame();
  writeFrameToImageData(frame, imageData);
  ctx.putImageData(imageData, 0, 0);
}

function renderFullscreenFrame() {
  if (!emu || !fsCtx) return;
  const frame = emu.frame();
  writeFrameToImageData(frame, fsImageData);
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
//  Run loops
//  Three independent loops: cpu-debug, nes-debug, nes-fullscreen
// ═══════════════════════════════════════════════════════

function frame() {
  if (!running) return;

  try {
    if (mode === "cpu") {
      // CPU debug: step one clock, refresh all debug panels
      emu.cpu_clock();
      updateDebugUI();

    } else if (mode === "nes") {
      // NES debug: run full frame, update debug + small canvas + pattern tables
      emu.run_frame();
      renderDebugFrame();
      patternFrameCounter++;
      if (patternFrameCounter >= 60) { renderPatternTables(); patternFrameCounter = 0; }
      updateDebugUI();

    } else if (mode === "fullscreen") {
      // Fullscreen: run full frame, render to fullscreen canvas ONLY — no debug work
      emu.run_frame();
      renderFullscreenFrame();
      // No updateDebugUI, no renderPatternTables, no log
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
  rafHandle = requestAnimationFrame(frame);
  if (mode !== "fullscreen") log("Run started");
}

function pauseRun() {
  running = false;
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
  if (document.hidden) releaseAllButtons();
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
  // Debug mode buttons
  document.querySelectorAll(".mode-switch button[data-mode]").forEach(btn => {
    btn.addEventListener("click", () => switchDebugMode(btn));
  });

  // Fullscreen button (separate from mode switch)
  $("btnEnterFullscreen").addEventListener("click", enterFullscreen);
  $("fsExitBtn")         .addEventListener("click", exitFullscreen);

  // Debug controls
  $("btnReset").addEventListener("click", () => {
    if (!emu) return;
    pauseRun(); emu.reset(); updateDebugUI(); log("Reset");
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
    await init();

    emu = new NES();
    setStatus(true, "WASM loaded");

    bindUI();
    initCanvas();
    updateDebugUI();

    log("Emulator ready.");
    startRun();
  } catch (e) {
    console.error(e);
    setStatus(false, "Failed to load WASM");
    log(`Failed to init WASM: ${e}`);
  }
}

boot();