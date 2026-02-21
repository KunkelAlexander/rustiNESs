import init, { Emulator } from "./pkg/nes_emulator.js";

let emu = null;
let running = false;
let rafHandle = null;

// --- DOM helpers ---
const $ = (id) => document.getElementById(id);

function log(msg) {
  const el = $("log");
  const t = new Date().toLocaleTimeString();
  el.innerText += `[${t}] ${msg}\n`;
  el.scrollTop = el.scrollHeight;
}

function setStatus(ok, text) {
  $("statusDot").style.background = ok ? "#36d399" : "#ff5c7c";
  $("statusText").innerText = text;
}

// --- UI updates ---
function renderRegisters() {
  if (!emu) return;

  const r = emu.get_registers();
  const [a, x, y, sp, pc, status] = r;

  $("a").innerText = a.toString(16).padStart(2, "0").toUpperCase();
  $("x").innerText = x.toString(16).padStart(2, "0").toUpperCase();
  $("y").innerText = y.toString(16).padStart(2, "0").toUpperCase();
  $("sp").innerText = sp.toString(16).padStart(2, "0").toUpperCase();
  $("pc").innerText = pc.toString(16).padStart(4, "0").toUpperCase();
  $("status").innerText = status.toString(2).padStart(8, "0");

  
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

  const ram = emu.get_ram(0, 0x10000);

  const [a, x, y, sp, pc, status] = emu.get_registers();
  const [fetched, addr_abs, addr_rel, opcode, cycles] = emu.get_cpu_state();



  function renderBlock(startAddr) {
    for (let row = 0; row < 16; row++) {
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
  renderBlock(0x8000);
}



function updateUI() {
  renderRegisters();
  renderRam();
}

// --- Run loop ---
function frame() {
  if (!running) return;

  const cycles = Number($("cyclesPerFrame").value || 1);

  try {
    if (emu.run_cycles) {
      emu.run_cycles(cycles);
    } else {
      // fallback if you don't have run_cycles yet
      for (let i = 0; i < cycles; i++) {
        emu.clock();
      }
    }
  } catch (e) {
    running = false;
    setStatus(false, "Runtime error");
    log(`ERROR: ${e}`);
    return;
  }

  updateUI();
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
    emu.clock();
    updateUI();
    log("Clock()");
  });

  $("btnStep").addEventListener("click", () => {
    if (!emu) return;
    emu.step_instruction();

    updateUI();
  });


  $("btnAssemble").addEventListener("click", () => {
    if (!emu) return;
    pauseRun();

    try {
      const src = $("asmInput").value;
      const program = parseHexProgram(src);

      emu.load_program(program, 0x8000);

      updateUI();
      log(`Loaded ${program.length} bytes at $8000`);
    } catch (e) {
      log(`Parse error: ${e.message}`);
    }
  });
}

function parseHexProgram(text) {
  // Remove comments
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

    emu = new Emulator();
    setStatus(true, "WASM loaded");

    bindUI();
    updateUI();

    log("Emulator created");
    log("Ready.");
  } catch (e) {
    console.error(e);
    setStatus(false, "Failed to load WASM");
    log(`Failed to init wasm: ${e}`);
  }
}

boot();
