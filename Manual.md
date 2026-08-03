# PINE16 Architecture Manual

This document outlines the architecture, instruction set, and rendering pipeline of the PINE16 system, derived directly from the console's source code.

## 1. System Architecture and Memory Map

The PINE16 processor operates on 16-bit integers and features a single contiguous 65,535-word SRAM array.

### Registers
The system contains 5 internally managed registers. They can be targeted as arguments using the `*` prefix (e.g., `*A`).
* **A, B, C, D:** General-purpose registers (Indices 0-3).
* **SP:** Stack Pointer (Index 4).

### Memory Map (SRAM)
Addresses `0x0000` through `0x012B` (0-299) are designated as Read-Only. Writing to these addresses with guardrails enabled will cause a system panic.

| Address Range | Description | Access |
| :--- | :--- | :--- |
| `0` - `2` | Console Version (Major, Minor, Patch) | Read-Only |
| `3` - `5` | ROM Version (Major, Minor, Patch) | Read-Only |
| `6` | CPU Cycle Counter | Read-Only |
| `7` | Program Counter | Read-Only |
| `8` | Vblank Flag (`1` during Vblank, `0` otherwise) | Read-Only |
| `9` | Current Pressed Key (ASCII value) | Read-Only |
| `300` | Clear Color (RGB565 format) | Read-Write |
| `301` - `315` | Color Palette Index 1-15 (RGB565 format) | Read-Write |
| `1000` - `3559` | Tilesheet Memory (2560 words; 256 tiles) | Read-Write |
| `3560` - `4559` | Background Map Layer (1000 words; 40x25 grid) | Read-Write |
| `4560` - `5559` | Sprite Layer (999 words; 333 sprite objects) | Read-Write |
| `5560` - `6559` | Foreground Map Layer (1000 words; 40x25 grid) | Read-Write |
| `u16::MAX` down | Stack Memory (Grows downward) | Read-Write |

---

## 2. Argument Types and Syntax

The assembler uses prefixes to determine the argument type. Arguments are separated by spaces and or commas (`, `,` ,`,` `,`,` are all valid, even this one: `, , , , ,`)

| Type | Syntax | Example | Description |
| :--- | :--- | :--- | :--- |
| **Immediate** | `#` | `#10` | A direct, literal 16-bit integer value. |
| **Address** | `$#` | `$#300` | A direct value pointing to a specific SRAM address. |
| **Register** | `*` | `*A` | The value contained within the specified register. |
| **Reg-Pointed** | `$*` | `$*B` | Uses the specified register's value as an SRAM address. |
| **Label** | None | `loop` | A target for jumps. Resolves to an Immediate value. |

---

## 3. Instruction Set (OpCodes)

Attempting to write to a read-only argument type (like Immediate) or an empty argument will result in an assembler error.

### Mathematical & Logical Operations (Requires: `Writable`, `Readable`)
* `add dst src`: Adds `src` to `dst`. Wraps on overflow.
* `sub dst src`: Subtracts `src` from `dst`. Wraps on underflow.
* `mul dst src`: Multiplies `dst` by `src`. Wraps on overflow.
* `div dst src`: Divides `dst` by `src`. Wraps on division by zero.
* `and dst src`: Bitwise AND on `dst` and `src`.
* `or dst src`: Bitwise OR on `dst` and `src`.
* `xor dst src`: Bitwise XOR on `dst` and `src`.
* `shl dst src`: Bitwise Shift Left `dst` by `src` bits.
* `shr dst src`: Bitwise Shift Right `dst` by `src` bits.
* `mov dst src`: Copies the value of `src` into `dst`.

### Control Flow (Requires: `Readable`)
* `jmp target`: Sets the Program Counter to `target`.
* `jeq target cond`: Jumps to `target` if `cond` is `0`.
* `jne target cond`: Jumps to `target` if `cond` is not `0`.
* `jsr target`: Pushes the current Program Counter to the stack, then jumps to `target`.
* `rtr`: Pops a value from the stack and sets the Program Counter to it. Takes no arguments.

### Stack & System
* `push src`: Pushes the value of `src` to the stack.
* `pop dst`: Pops the top of the stack and stores it in `dst` (Requires `Writable`).
* `stall cycles`: Halts CPU execution for the specified number of `cycles`.
* `await address`: Pauses CPU execution until the memory at `address` equals `0`.
* `noop`: Consumes one CPU cycle and performs no operation.

---

## 4. Rendering Pipeline

The PINE16 renderer outputs a fixed `320x200` pixel canvas operating at 30 frames per second. 

### Tilesheet Memory (`1000` - `3559`)
Tiles are 8x8 pixels. Each tile requires 16 words of memory (256 tiles total). It is recommended to define Sprites in hex, because every pixel is one digit
* Each 16-bit word stores half a row of pixels (4 pixels per word, 4 bits per pixel).
* The 4-bit value acts as an index pointing to the system color palette.
* Tile ID `0` is ignored by the renderer and effectively transparent.

### Modifying the Palette
The system supports a 16-color palette (Index 0 is transparency). Memory address `300` defines the screen clear color. Addresses `301` through `315` define the standard palette indices 1-15. Colors must be written in RGB565 format.

### Background and Foreground Layers (`3560` and `5560`)
Both maps consist of a 40x25 tile grid (1000 words). The layout is row-major (left-to-right, top-to-bottom). 
Each word defines a tile's properties:
* **Lower Byte (Bits 0-7):** Tile ID (0-255).
* **Upper Byte (Bits 8-15):** Flags.
  * Bit 8 (Flag `1`): Flip Y axis.
  * Bit 9 (Flag `2`): Flip X axis.

### Sprite Layer (`4560` - `5559`)
The sprite layer manages up to 333 independent sprites. Each sprite is defined by a 3-word chunk:
1. `[id_flags]`: Same format as Map layers (Lower Byte = ID, Upper Byte = Flags).
2. `[x_position]`: Absolute X coordinate on the canvas.
3. `[y_position]`: Absolute Y coordinate on the canvas. <br>
Sprites positioned beyond the canvas boundaries (X > 320, Y > 200) are culled.