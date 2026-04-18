import init, { NES } from "./pkg/nes_emulator.js";

let emu       = null;
let running   = false;
let rafHandle = null;
let mode      = "nes";

// --- DOM helpers ---
const $ = (id) => document.getElementById(id);

function log(msg) {
  const el = $("log");
  const t = new Date().toLocaleTimeString();
  el.innerText += `[${t}] ${msg}\n`;
  el.scrollTop = el.scrollHeight;
}

// --- NES Canvas ---
let ctx = null;
let imageData = null;

// Render pattern tables 
let pattern0Ctx = null;
let pattern1Ctx = null;
let pattern0ImageData = null;
let pattern1ImageData = null;

const NES_PALETTE = [
  [84, 84, 84],
  [0, 30, 116],
  [8, 16, 144],
  [48, 0, 136],
  [68, 0, 100],
  [92, 0, 48],
  [84, 4, 0],
  [60, 24, 0],
  [32, 42, 0],
  [8, 58, 0],
  [0, 64, 0],
  [0, 60, 0],
  [0, 50, 60],
  [0, 0, 0],
  [0, 0, 0],
  [0, 0, 0],

  [152, 150, 152],
  [8, 76, 196],
  [48, 50, 236],
  [92, 30, 228],
  [136, 20, 176],
  [160, 20, 100],
  [152, 34, 32],
  [120, 60, 0],
  [84, 90, 0],
  [40, 114, 0],
  [8, 124, 0],
  [0, 118, 40],
  [0, 102, 120],
  [0, 0, 0],
  [0, 0, 0],
  [0, 0, 0],

  [236, 238, 236],
  [76, 154, 236],
  [120, 124, 236],
  [176, 98, 236],
  [228, 84, 236],
  [236, 88, 180],
  [236, 106, 100],
  [212, 136, 32],
  [160, 170, 0],
  [116, 196, 0],
  [76, 208, 32],
  [56, 204, 108],
  [56, 180, 204],
  [60, 60, 60],
  [0, 0, 0],
  [0, 0, 0],

  [236, 238, 236],
  [168, 204, 236],
  [188, 188, 236],
  [212, 178, 236],
  [236, 174, 236],
  [236, 174, 212],
  [236, 180, 176],
  [228, 196, 144],
  [204, 210, 120],
  [180, 222, 120],
  [168, 226, 144],
  [152, 226, 180],
  [160, 214, 228],
  [160, 162, 160],
  [0, 0, 0],
  [0, 0, 0],
];

const controller1 = {
  x: 0,     // NES A
  z: 0,     // NES B
  a: 0,     // Select
  s: 0,     // Start
  up: 0,
  down: 0,
  left: 0,
  right: 0,
};



function initCanvas() {
  const canvas = $("screen");
  if (canvas) {
    ctx = canvas.getContext("2d");
    imageData = ctx.createImageData(256, 240);
  }

  const p0 = $("pattern0");
  if (p0) {
    pattern0Ctx = p0.getContext("2d");
    pattern0ImageData = pattern0Ctx.createImageData(128, 128);
  }

  const p1 = $("pattern1");
  if (p1) {
    pattern1Ctx = p1.getContext("2d");
    pattern1ImageData = pattern1Ctx.createImageData(128, 128);
  }
}

// --- WASM availability ---
function setStatus(ok, text) {
  $("statusDot").style.background = ok ? "#36d399" : "#ff5c7c";
  $("statusText").innerText = text;
}

// --- Debug window ---
function formatStatusFlags(status) {
  const flags = [
    ["N", 7],
    ["V", 6],
    ["U", 5],
    ["B", 4],
    ["D", 3],
    ["I", 2],
    ["Z", 1],
    ["C", 0],
  ];

  return flags
    .map(([name, bit]) => {
      const value = (status >> bit) & 1;
      const cls = value ? "flag-on" : "flag-off";
      return `<span class="flag ${cls}">${name}</span>`;
    })
    .join(" ");
}

function renderRegisters() {
  if (!emu) return;

  const r = emu.get_registers();
  const [a, x, y, sp, pc, status] = r;

  $("a").innerText = a.toString(16).padStart(2, "0").toUpperCase();
  $("x").innerText = x.toString(16).padStart(2, "0").toUpperCase();
  $("y").innerText = y.toString(16).padStart(2, "0").toUpperCase();
  $("sp").innerText = sp.toString(16).padStart(2, "0").toUpperCase();
  $("pc").innerText = pc.toString(16).padStart(4, "0").toUpperCase();
  $("status").innerHTML = formatStatusFlags(status);

  
  const s = emu.get_cpu_state();
  const [fetched, addr_abs, addr_rel, opcode, cycles] = s;

  $("fetched").innerText = fetched.toString(16).padStart(2, "0").toUpperCase();
  $("addr_abs").innerText = addr_abs.toString(16).padStart(4, "0").toUpperCase();
  $("addr_rel").innerText = addr_rel.toString(16).padStart(4, "0").toUpperCase();
  $("opcode").innerText = opcode.toString(16).padStart(2, "0").toUpperCase();
  $("cycles").innerText = cycles.toString(16).padStart(2, "0").toUpperCase();
}
function renderRam() {
  if (!emu) return;

  const grid = $("ramGrid");
  grid.innerHTML = "";

  const ram = emu.get_ram(0, 0x00800); // Get 2 KB of RAM

  const [a, x, y, sp, pc, status] = emu.get_registers();
  const [fetched, addr_abs, addr_rel, opcode, cycles] = emu.get_cpu_state();



  function renderBlock(startAddr) {
    for (let row = 0; row < 128; row++) {
      const base = startAddr + row * 16;

      // Row label
      const label = document.createElement("div");
      label.className = "cell row-label";
      label.innerText = base.toString(16).padStart(4, "0").toUpperCase();
      grid.appendChild(label);

      for (let col = 0; col < 16; col++) {
        const addr = base + col;

        const cell = document.createElement("div");
        cell.className = "cell";
        cell.innerText = ram[addr]
          .toString(16)
          .padStart(2, "0")
          .toUpperCase();

        // 🔵 Highlight PC
        if (addr === pc) {
          cell.classList.add("pc-highlight");
        }

        // 🟡 Highlight effective address
        if (addr === addr_abs) {
          cell.classList.add("addr-highlight");
        }

        grid.appendChild(cell);
      }
    }
  }

  renderBlock(0x0000);
}



function updateUI() {
  renderRegisters();
  
  if (mode === "cpu") {
    renderRam();
  }
}

function renderFrame() {
  if (!emu || !ctx) return;

  const frame = emu.frame();

  for (let i = 0; i < frame.length; i++) {
    const index = frame[i] & 0x3f;
    const [r, g, b] = NES_PALETTE[index];

    imageData.data[i * 4 + 0] = r;
    imageData.data[i * 4 + 1] = g;
    imageData.data[i * 4 + 2] = b;
    imageData.data[i * 4 + 3] = 255;
  }

  ctx.putImageData(imageData, 0, 0);
}

function syncController() {
  if (!emu) return;

  emu.set_controller(
    0,
    controller1.x,
    controller1.z,
    controller1.a,
    controller1.s,
    controller1.up,
    controller1.down,
    controller1.left,
    controller1.right
  );
}

function setButtonState(name, pressed) {
  if (!(name in controller1)) return;
  controller1[name] = pressed ? 1 : 0;
  syncController();
}

function keyToButton(code) {
  switch (code) {
    case "KeyA": return "x";            // NES A
    case "KeyF": return "z";            // NES B
    case "KeyS": return "s";            // Start
    case "KeyD": return "a";            // Select
    case "ArrowUp": return "up";
    case "ArrowDown": return "down";
    case "ArrowLeft": return "left";
    case "ArrowRight": return "right";
    default: return null;
  }
}

function releaseAllButtons() {
  for (const key in controller1) {
    controller1[key] = 0;
  }
  syncController();

  document.querySelectorAll("[data-btn]").forEach((el) => {
    el.classList.remove("pressed");
  });
}

window.addEventListener("blur", releaseAllButtons);
document.addEventListener("visibilitychange", () => {
  if (document.hidden) releaseAllButtons();
});

document.addEventListener("fullscreenchange", () => {
  updateFullscreenUI();
  if (!isFullscreen()) releaseAllButtons();
});

document.addEventListener("webkitfullscreenchange", () => {
  updateFullscreenUI();
  if (!isFullscreen()) releaseAllButtons();
});


function renderPatternTableToCanvas(buffer, imageData, ctx) {
  if (!buffer || !imageData || !ctx) return;

  for (let i = 0; i < buffer.length; i++) {
    const index = buffer[i] & 0x3f;
    const [r, g, b] = NES_PALETTE[index];

    imageData.data[i * 4 + 0] = r;
    imageData.data[i * 4 + 1] = g;
    imageData.data[i * 4 + 2] = b;
    imageData.data[i * 4 + 3] = 255;
  }

  ctx.putImageData(imageData, 0, 0);
}

function renderPatternTables() {
  if (!emu) return;
  if (!pattern0Ctx || !pattern1Ctx) return;

  const palette = Number($("paletteSelect")?.value ?? 0);

  const table0 = emu.get_pattern_table(0, palette);
  const table1 = emu.get_pattern_table(1, palette);

  renderPatternTableToCanvas(table0, pattern0ImageData, pattern0Ctx);
  renderPatternTableToCanvas(table1, pattern1ImageData, pattern1Ctx);
}

let patternFrameCounter = 0;

function frame() {
  if (!running) return;

  try {
    if (mode === "cpu") {
      emu.cpu_clock();
      updateUI();
    } else {
      emu.run_frame();
      renderFrame();
      
      patternFrameCounter++;
      if (patternFrameCounter >= 60) {
        renderPatternTables();
        patternFrameCounter = 0;
      }

      updateUI();
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
  if (!emu) return;
  if (running) return;
  running = true;
  log("Run started");
  rafHandle = requestAnimationFrame(frame);
}

function pauseRun() {
  running = false;
  if (rafHandle) cancelAnimationFrame(rafHandle);
  rafHandle = null;
  log("Paused");
}

function switchMode(btn) {
  mode = btn.dataset.mode;

  // set body mode attribute (drives CSS visibility)
  document.body.dataset.mode = mode;

  // update active button styling
  document.querySelectorAll(".mode-switch button")
    .forEach(b => b.classList.remove("active"));

  btn.classList.add("active");

  if (mode == "nes") {
    startRun();
    log(`Switched to NES Console mode`);
  } else {
    pauseRun();
    log(`Switched to 6502 Lab mode`);
    loadProgram();
  }
}

function isFullscreen() {
  return document.fullscreenElement || document.webkitFullscreenElement;
}

function focusGameSurface() {
  $("screen")?.focus();
}

function isMobileLike() {
  return window.matchMedia("(hover: none) and (pointer: coarse)").matches;
}

function updateFullscreenUI() {
  const shell = $("playShell");
  if (!shell) return;

  const active = !!(document.fullscreenElement === shell || document.webkitFullscreenElement === shell);
  shell.classList.toggle("is-fullscreen", active);
  shell.classList.toggle("is-mobile-fs", active && isMobileLike());
  shell.classList.toggle("is-desktop-fs", active && !isMobileLike());
}

async function enterFullscreen() {
  const shell = $("playShell");
  if (!shell) return;

  try {
    if (shell.requestFullscreen) {
      await shell.requestFullscreen({ navigationUI: "hide" });
    } else if (shell.webkitRequestFullscreen) {
      await shell.webkitRequestFullscreen();
    } else {
      log("Fullscreen is not supported on this device/browser.");
      return;
    }

    updateFullscreenUI();
    focusGameSurface();
  } catch (e) {
    log(`Fullscreen failed: ${e.message || e}`);
  }
}

async function exitFullscreen() {
  try {
    if (document.exitFullscreen) {
      await document.exitFullscreen();
    } else if (document.webkitExitFullscreen) {
      await document.webkitExitFullscreen();
    }
  } catch (e) {
    log(`Exit fullscreen failed: ${e.message || e}`);
  }
}

async function toggleFullscreen() {
  if (!isFullscreen()) {
    await enterFullscreen();
  } else {
    await exitFullscreen();
  }
}

// --- Button wiring ---
function bindUI() {
  $("btnReset").addEventListener("click", () => {
    if (!emu) return;
    pauseRun();
    emu.reset();
    updateUI();
    log("Reset");
  });

  $("btnClock").addEventListener("click", () => {
    if (!emu) return;
    if (mode === "cpu") {
      emu.cpu_clock();
    } else {
      emu.clock();
    }
    renderFrame();
    updateUI();
    log("Clock()");
  });

  $("btnStep").addEventListener("click", () => {
    if (!emu) return;

    emu.step_instruction();
    renderFrame();
    updateUI();
    log(`Step instruction()`);
  });


  $("btnAssemble").addEventListener("click", () => {
    if (!emu) return;
    pauseRun();
    loadProgram();

  });

  $("btnRun").addEventListener("click", startRun);

  $("btnPause").addEventListener("click", pauseRun);

  $("btnLoadROM").addEventListener("click", () => {
    $("romLoader").click();
  });

  $("btnFullscreen")?.addEventListener("click", toggleFullscreen);
  $("btnExitFullscreen")?.addEventListener("click", exitFullscreen);

  $("romLoader").addEventListener("change", async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    await loadRomFile(file);
  });

  $("paletteSelect")?.addEventListener("change", () => {
    renderPatternTables();
  });

  window.addEventListener("keydown", (e) => {
    const btn = keyToButton(e.code);
    if (!btn) return;

    e.preventDefault();
    setButtonState(btn, true);
  });

  window.addEventListener("keyup", (e) => {
    const btn = keyToButton(e.code);
    if (!btn) return;

    e.preventDefault();
    setButtonState(btn, false);
  });

  document.querySelectorAll("[data-btn]").forEach((btnEl) => {
    const btnName = btnEl.dataset.btn;

    const press = (e) => {
      e.preventDefault();
      btnEl.setPointerCapture?.(e.pointerId);
      btnEl.classList.add("pressed");
      setButtonState(btnName, true);
      focusGameSurface();
    };

    const release = (e) => {
      e.preventDefault();
      btnEl.classList.remove("pressed");
      setButtonState(btnName, false);
    };

    btnEl.addEventListener("pointerdown", press);
    btnEl.addEventListener("pointerup", release);
    btnEl.addEventListener("pointercancel", release);
    btnEl.addEventListener("lostpointercapture", release);
    btnEl.addEventListener("pointerleave", (e) => {
      if (e.pointerType === "mouse") release(e);
    });
  });


  window.addEventListener("pointerup", releaseAllButtons);



  // --- Mode switch ---
  document.querySelectorAll(".mode-switch button").forEach(btn => {
    btn.addEventListener("click", () => {
      switchMode(btn);
  });
});
}

function loadProgram() {
  try {
    const src = $("asmInput").value;
    const program = parseHexProgram(src);

    emu.load_program(program, 0x0000);

    updateUI();
    
    renderPatternTables();
    log(`Loaded program of length ${program.length} bytes at $0000`);
  } catch (e) {
    log(`Parse error: ${e.message}`);
  }
}

async function loadRomFile(file) {
  if (!emu) return;

  const arrayBuffer = await file.arrayBuffer();
  const romBytes = new Uint8Array(arrayBuffer);

  emu.insert_cartridge(romBytes);
  log(`Loaded ROM: ${file.name}`);
  emu.reset();
  log(`Reset state`);

  updateUI();
  log(`Updated UI`);
  
  renderPatternTables();
  log(`Rendered pattern tables`);
}

function parseHexProgram(text) {
  // Remove commentsF
  const cleaned = text
    .replace(/;.*/g, "")
    .replace(/[^0-9a-fA-F]/g, " ")
    .trim();

  if (!cleaned) return new Uint8Array([]);

  const bytes = cleaned.split(/\s+/).map(b => {
    const value = parseInt(b, 16);
    if (Number.isNaN(value) || value < 0 || value > 255) {
      throw new Error(`Invalid byte: ${b}`);
    }
    return value;
  });

  return new Uint8Array(bytes);
}

// --- Boot ---
async function boot() {
  try {
    setStatus(false, "Loading WASM…");
    await init();

    emu = new NES();
    setStatus(true, "WASM loaded");

    bindUI();
    initCanvas();
    updateUI();

    log("Emulator created");
    log("Ready.");

    startRun();


  } catch (e) {
    console.error(e);
    setStatus(false, "Failed to load WASM");
    log(`Failed to init wasm: ${e}`);
  }

}

boot();
