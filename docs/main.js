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

function initCanvas() {
  const canvas = $("screen");
  if (!canvas) return;

  ctx = canvas.getContext("2d");
  imageData = ctx.createImageData(256, 240);
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
  renderRam();
}

function renderFrame() {
  if (!emu || !ctx) return;

  const frame = emu.frame(); // Vec<u8>

  for (let i = 0; i < frame.length; i++) {
    const c = frame[i] ;

    // temporary grayscale palette
    const color = c;

    imageData.data[i * 4 + 0] = color;
    imageData.data[i * 4 + 1] = color;
    imageData.data[i * 4 + 2] = color;
    imageData.data[i * 4 + 3] = 255;
  }

  ctx.putImageData(imageData, 0, 0);
}

function frame() {
  if (!running) return;

  try {
    if (mode === "cpu") {
      emu.cpu_clock();
      updateUI();
    } else {
      emu.run_frame();
      renderFrame();
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

  $("romLoader").addEventListener("change", async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    await loadRomFile(file);
  });



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
    log(`Loaded program of length ${program.length} bytes at $0000`);
  } catch (e) {
    log(`Parse error: ${e.message}`);
  }
}

async function loadRomFile(file) {
  if (!emu) return;

  pauseRun();

  const arrayBuffer = await file.arrayBuffer();
  const romBytes = new Uint8Array(arrayBuffer);

  emu.insert_cartridge(romBytes);
  emu.reset();

  updateUI();
  log(`Loaded ROM: ${file.name}`);
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
