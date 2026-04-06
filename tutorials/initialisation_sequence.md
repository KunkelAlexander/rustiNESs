# 🧭 Typical NES Initialization Sequence

When a ROM starts executing, the CPU begins at the reset vector, but the **PPU is not immediately usable**. So the game must carefully bring the graphics system online in stages.

---

# 🧩 Phase 0 — Power-on state (what the emulator should assume)

At reset:

* PPU memory is **undefined**
* Palette RAM is **undefined**
* Scroll registers are **undefined**
* Rendering is typically **off**

👉 The game must initialize everything explicitly.

---

# ⏳ Phase 1 — Wait for PPU to stabilize (VBlank sync)

### Why this exists

The NES PPU takes a short time after reset before it behaves predictably.
Games usually wait for **VBlank** before touching VRAM.

### Typical code pattern

```asm
wait_vblank:
    BIT $2002      ; read PPUSTATUS
    BPL wait_vblank
```

### What this means

* Bit 7 of `$2002` = VBlank flag
* Loop until it becomes `1`

👉 If your emulator never sets this bit correctly, the game **hangs here forever**

---

# ⚙️ Phase 2 — Configure PPU control registers

### Registers involved

* `$2000` → **PPUCTRL**
* `$2001` → **PPUMASK**

### Example

```text
write $2000 = %10010000
write $2001 = %00011110
```

### What gets configured

* which pattern table to use
* whether NMI is enabled
* whether rendering is enabled
* color emphasis, grayscale, etc.

👉 Important: many games **enable NMI here**, expecting your emulator to generate it later

---

# 🧭 Phase 3 — Reset scroll / address latch

### Why this matters

The NES uses a shared internal latch for:

* `$2005` (scroll)
* `$2006` (VRAM address)

If not reset properly, later VRAM writes go to the wrong place.

### Typical pattern

```text
write $2005 = 0
write $2005 = 0
```

👉 This synchronizes internal scroll registers

---

# 🧱 Phase 4 — Initialize nametables (background layout)

### What nametables are

* Stored at: `0x2000–0x2FFF`
* Each byte = **tile index**

### What the game does

```text
write $2006 = 20
write $2006 = 00   ; address = 0x2000

loop:
    write $2007 = <tile index>
```

### What this fills

* background tiles
* attribute tables (palette selection per 2x2 tile block)

👉 Your earlier log:

```text
PPU write 23FF = 00
```

means you were here — filling nametable memory

---

# 🎨 Phase 5 — Load palette (critical!)

### Where palette lives

```text
0x3F00–0x3F1F → palette RAM
```

### Typical code

```text
write $2006 = 3F
write $2006 = 00   ; address = 0x3F00

write $2007 = 0F   ; universal background color
write $2007 = 30
write $2007 = 21
write $2007 = 11
...
```

### Structure

* 4 background palettes × 4 entries
* 4 sprite palettes × 4 entries

👉 This is what your emulator is currently **not reaching**

---

# 🧬 Phase 6 — Load pattern data (CHR RAM only)

### Only happens if:

```text
CHR banks: 0
```

### Typical code

```text
write $2006 = 00
write $2006 = 00   ; address = 0x0000

loop:
    write $2007 = <tile byte>
```

### What this does

* uploads tile graphics into PPU memory

👉 In your case (CHR ROM), this phase is skipped

---

# 🚀 Phase 7 — Enable rendering

### Final step

```text
write $2001 = <enable background + sprites>
```

### Effects

* background becomes visible
* sprites become visible
* PPU starts real rendering

---

# 🔁 Phase 8 — Enter main loop (often NMI-driven)

After initialization, most games:

* enable NMI in `$2000`
* rely on **NMI every frame** (during VBlank)

### Typical structure

```text
main loop:
    wait for NMI
    update game state
    write to PPU (during VBlank)
    repeat
```

---

# 🧠 Putting it all together

Here is the full flow:

```text
RESET
  ↓
wait for VBlank
  ↓
configure PPU ($2000/$2001)
  ↓
reset scroll ($2005)
  ↓
fill nametable ($2006/$2007)
  ↓
load palette ($2006/$2007)   ← YOU ARE NOT HERE YET
  ↓
(optional) load CHR RAM
  ↓
enable rendering
  ↓
main loop (NMI-driven)
```

---

# 🔍 Why your emulator stops early

From your logs:

* you reach **nametable writes** (`23FF`)
* you set `$2005`, `$2000`
* then fall into a loop

That strongly matches:

```text
→ waiting for VBlank / NMI / correct $2002 behavior
```

So the ROM likely never progresses to:

```text
→ Phase 5 (palette load)
```

---

# 🚨 Critical emulator requirements for this sequence

To get past initialization, you need:

### ✅ `$2002` (PPUSTATUS)

* bit 7 = VBlank
* cleared on read
* resets write latch

### ✅ VBlank timing

* set at scanline ~241
* cleared at pre-render line

### ✅ NMI generation

* if `$2000 & 0x80 != 0`, trigger NMI at VBlank

### ✅ `$2005` latch behavior

* must toggle internal write state

### ✅ `$2006` address logic

* must share latch with `$2005`

---

# 🧩 Final mental model

Think of startup as:

> “Wait until the PPU is ready → configure it → fill its memory → then turn it on”

And your emulator is currently stuck in:

> “waiting until the PPU is ready”
