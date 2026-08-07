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
| `1000` - `3543` | Tilesheet Memory (2544 words) | Read-Write |
| `3544` - `4543` | Background Map Layer (1000 words; 40x25 grid) | Read-Write |
| `4544` - `5542` | Sprite Layer (999 words; 333 sprite objects) | Read-Write |
| `5543` - `6542` | Foreground Map Layer (1000 words; 40x25 grid) | Read-Write |
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

### Tilesheet Memory (`1000` - `3543`)
Tiles are 8x8 pixels. Each tile requires 16 words of memory. It is recommended to define Sprites in hex, because every pixel is one digit.
* Each 16-bit word stores half a row of pixels (4 pixels per word, 4 bits per pixel).
* The 4-bit value acts as an index pointing to the system color palette.
* Tile ID `0` is ignored by the renderer and effectively transparent.
* The renderer offsets Tile IDs by `-1` during calculation, mapping Tile ID `1` to address `1000`.

### Background and Foreground Layers (`3544` and `5543`)
Both maps consist of a 40x25 tile grid (1000 words). The layout is row-major (left-to-right, top-to-bottom). 
Each word defines a tile's properties:
* **Lower Byte (Bits 0-7):** Tile ID (0-254).
* **Upper Byte (Bits 8-15):** Flags.
  * Bit 8 (Flag `1`): Flip Y axis.
  * Bit 9 (Flag `2`): Flip X axis.

### Sprite Layer (`4544` - `5542`)
The sprite layer manages up to 333 independent sprites. Each sprite is defined by a 3-word chunk:
1. `[id_flags]`: Same format as Map layers (Lower Byte = ID, Upper Byte = Flags).
2. `[x_position]`: Absolute X coordinate on the canvas.
3. `[y_position]`: Absolute Y coordinate on the canvas. 

Sprites positioned beyond the canvas boundaries (X > 320, Y > 200) are culled.

---

## 5. System Palette

The system supports a 16-color palette. Index 0 is always reserved for transparency when drawing tiles. Memory address `300` defines the screen clear color. Addresses `301` through `315` define the standard palette indices 1-15. 

Colors are loaded into SRAM at boot using the RGB565 format. You can overwrite these addresses at runtime to change the palette dynamically.

### Default Palette Layout

| Index | SRAM Address | Color Name | RGB565 Value |
| :--- | :--- | :--- | :--- |
| 1 | `301` | Deep Night | `0x18E5` |
| 2 | `302` | Cloud White | `0xF7BF` |
| 3 | `303` | Charcoal | `0x424A` |
| 4 | `304` | Silver | `0xA535` |
| 5 | `305` | Cyber Pink | `0xFA50` |
| 6 | `306` | Ocean Blue | `0x347F` |
| 7 | `307` | Sky Cyan | `0x3EBE` |
| 8 | `308` | Slime Green | `0x8787` |
| 9 | `309` | Magic Violet | `0xBAFD` |
| 10 | `310` | Electric Indigo | `0x6ADE` |
| 11 | `311` | Toasted Peach | `0xECAF` |
| 12 | `312` | Neon Coral | `0xFB4B` |
| 13 | `313` | Sunny Amber | `0xFDC6` |
| 14 | `314` | Electric Lemon | `0xF747` |
| 15 | `315` | Minty Green | `0x2EF1` |