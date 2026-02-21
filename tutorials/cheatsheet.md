# Rust vs C/C++/Python — Key Learnings Cheat Sheet

## 1) Rust doesn’t have classes

Rust replaces “class” with:

* `struct` → data (fields)
* `impl` → methods
* traits → interfaces / polymorphism

---

## 2) Public vs private

Rust is **private by default**.

```rust
pub struct Foo {
    pub x: u32,  // public
    y: u32,      // private
}
```

No `public:` / `private:` blocks like C++.

---

## 3) Constructors aren’t special

Rust has no constructor syntax.

You write a normal function:

```rust
impl Foo {
    pub fn new() -> Self { ... }
}
```

`Self` means “this type”.

---

## 4) Fixed-size arrays are part of the type

```rust
ram: [u8; 65536]
```

This is like C++ `uint8_t ram[65536]`.

To zero it:

```rust
ram: [0u8; 65536]
```

---

## 5) Returning values: semicolon matters

Rust returns the last expression **only if there is no semicolon**:

```rust
fn f() -> u8 { 5 }     // ok
fn g() -> u8 { 5; }    // returns (), error
```

Also: `if` must have `else` if you want it to be an expression.

---

## 6) Ownership replaces “who frees this?”

Rust forces you to model:

* who owns data
* who can mutate it
* who can borrow it temporarily

No GC like Python, no implicit copying like C++.

---

## 7) `&self` vs `&mut self`

Rust makes mutability explicit:

* `&self` → read-only access
* `&mut self` → exclusive mutable access

This is a core difference from C++.

---

## 8) You can’t have two mutable references at once

Rust forbids:

* aliasing mutable access

This is why emulator designs that work in C++ (with pointers) often break in Rust.

---

## 9) CPU owning Bus is *not* a good design in Rust

If you try:

```rust
struct Bus { cpu: Cpu }
Cpu needs &mut Bus
```

You hit borrow-checker errors.

### The clean Rust solution:

Make them siblings:

```rust
struct Nes {
    cpu: Cpu,
    bus: Bus,
}
```

Then:

```rust
nes.cpu.clock(&mut nes.bus);
```

---

## 10) Traits = interfaces

A trait is like a C++ abstract base class:

```rust
trait BusLike {
    fn read(&mut self, addr: u16) -> u8;
}
```

---

## 11) Template vs virtual dispatch (very important)

### Template style (static dispatch)

```rust
fn clock<B: BusLike>(&mut self, bus: &mut B)
```

Like C++ templates:

```cpp
template<typename B>
void clock(B& bus)
```

Fast, inlined, no vtable.

---

### Virtual/interface style (dynamic dispatch)

```rust
fn clock(&mut self, bus: &mut dyn BusLike)
```

Like C++ virtual functions:

```cpp
void clock(BusLike& bus)
```

Uses a vtable, slightly more indirection, but easier for lookup tables.

---

## 12) `dyn` means “runtime interface object”

```rust
&mut dyn BusLike
```

= “a mutable reference to something that implements BusLike, via vtable”.

---

## 13) Opcode lookup tables

6502 opcodes are 1 byte → 256 values.

So you build:

```rust
[Instruction; 256]
```

Where each entry maps:

opcode → (name, addressing mode fn, operation fn, cycles)

Hex codes like `0xA9` are literally the opcode bytes.

---

## 14) Rust macros (like `macro_rules! op!`)

Macros are compile-time code generation tools.

Used to avoid writing 256 repetitive entries.

---

## 15) `Copy` vs `Clone`

If you want:

```rust
let t = [xxx; 256];
```

then `Instruction` must be `Copy`.

Fix:

```rust
#[derive(Copy, Clone)]
struct Instruction { ... }
```

---

## 16) C++ header/implementation separation doesn’t exist

No `.h`.

Rust modules are split using:

* `mod foo;` (declares a module/file)
* `use crate::foo::Bar;` (imports a symbol)

---

## 17) `mod` vs `use`

* `mod bus;` → “compile bus.rs”
* `use crate::bus::Bus;` → “bring Bus into scope”

---

## 18) Emulators in Rust often need architectural refactors

In C++ you can store pointers everywhere.

In Rust, you usually redesign to avoid:

* circular ownership
* self-referential structs
* hidden mutable aliasing

---

# The big mental shift

### C++ mindset:

“Objects point to each other and mutate freely.”

### Rust mindset:

“Ownership is explicit; mutation is exclusive; borrowing is temporary.”
