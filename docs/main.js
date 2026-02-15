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
  $("addr_abs").innerText = addr_abs.toString(16).padStart(2, "0").toUpperCase();
  $("addr_rel").innerText = addr_rel.toString(16).padStart(2, "0").toUpperCase();
  $("opcode").innerText = opcode.toString(16).padStart(2, "0").toUpperCase();
  $("cycles").innerText = cycles.toString(16).padStart(4, "0").toUpperCase();
}

function renderRam() {
  const grid = $("ramGrid");
  const ram = emu.get_ram(0, 1024*64);

  grid.innerHTML = "";

  for (let i = 0; i < 256; i++) {
    const cell = document.createElement("div");
    cell.className = "cell";
    cell.innerText = ram[i].toString(16).padStart(2, "0").toUpperCase();
    grid.appendChild(cell);
  }
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
    pauseRun();
    emu.clock();
    updateUI();
    log("Clock()");
  });

  $("btnStep").addEventListener("click", () => {
    if (!emu) return;
    pauseRun();

    if (emu.step_instruction) {
      emu.step_instruction();
      log("Step instruction");
    } else {
      // fallback: do a few clocks
      for (let i = 0; i < 10; i++) emu.clock();
      log("Step fallback (10 cycles)");
    }

    updateUI();
  });

  $("btnRun").addEventListener("click", () => {
    startRun();
  });

  $("btnPause").addEventListener("click", () => {
    pauseRun();
  });

  $("btnAssemble").addEventListener("click", () => {
    if (!emu) return;
    pauseRun();

    const src = $("asmInput").value;
    log("Assemble clicked (placeholder)");
    log("Assembler not wired yet.");

    // Later:
    // const bytes = emu.assemble(src) OR assemble in JS
    // emu.load_program(bytes, 0x8000)
    // emu.reset()
  });
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
