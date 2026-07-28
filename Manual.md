# Table of Contents
- [Architecture](#architecture)
- [Assembly Language](#assembly-language)
  - [Arguments](#arguments)
  - [OpCodes](#opcodes)

---

# Architecture
Pine16 is a 16-bit console split into multiple memory sectors with distinct behaviors:

* **SRAM**: General-purpose RAM used for reading inputs, rendering graphics, and general data storage.
* **Registers**: Four general-purpose 16-bit registers (`A`, `B`, `C`, `D`). Math operations can be performed directly in memory, so registers are primarily used for fast temporary data storage and indirect memory addressing.
* **ROM (Code Sector)**: Executable memory containing instructions. Read-only by the CPU parser; cannot be directly read as data by programs.
* **ROM (Data Sector)** *(Not Yet Implemented)*: Static data storage that can be copied into SRAM.

---

# Assembly Language

## Arguments

* **Empty (` `)**: Only used by the assembler, the console just reads 0. *Read-only.*
* **Immediate (`#`)**: Stored directly in bytecode (e.g., `#42`). *Read-only.*
* **Address (`$#`)**: Points to a fixed SRAM address using an immediate (e.g., `$#42`). *Readable and writable.*
* **Register (`*`)**: References a register (`*A`, `*B`, `*C`, `*D`). *Readable and writable.*
* **Register-Pointed Address (`$*`)**: Points to an SRAM address using the value stored in a register (e.g., `$*A`). *Readable and writable.*
* **Label**: Resolved to an immediate at compile time. Defined with `name:` and referenced with `name`. *Read-only.*

### Example
```
MOV $#20 #10 ; 10 is stored at address 20
MOV *A #30 ; 30 is stored in register A
MOV $*A #40 ; 40 is stored at address 30
```

## OpCodes

### NOOP
Performs no operation.
* **Arguments:** Accepts any argument type or none.

### ADD / SUB / MUL / DIV
Performs the arithmetic operation on `ARG1` using `ARG2` and stores the result in `ARG1` (`ARG1 = ARG1 <op> ARG2`).
* **1.ARG:** Must be writable
* **2.ARG:** Must be readable

### STALL
Pauses CPU execution for the specified number of clock cycles.
* **1.ARG:** Must be readable
* **2.ARG:** Must be empty

### MOV
Copies the value from `ARG2` into `ARG1`. The value in `ARG2` remains unchanged.
* **1.ARG:** Must be writable
* **2.ARG:** Must be readable

### JMP
Unconditionally jumps to the specified address in `ARG1`.
* **1.ARG:** Must be readable
* **2.ARG:** Must be empty

### JEQ
Jumps to the address in `ARG1` if `ARG2` is equal to 0.
* **1.ARG:** Must be readable (Target Address)
* **2.ARG:** Must be readable (Value to test)

### JNE
Jumps to the address in `ARG1` if `ARG2` is not equal to 0.
* **1.ARG:** Must be readable (Target Address)
* **2.ARG:** Must be readable (Value to test)

### AND / OR / XOR
Performs a bitwise operation between `ARG1` and `ARG2`, storing the result in `ARG1`.
* **1.ARG:** Must be writable
* **2.ARG:** Must be readable

---

# Memory Map

Pine16 features a 16-bit word-addressable memory space ranging from `$0000` to `$FFFF` (65,536 total words).

| Address Range | Size (Words) | Sector Name | Description |
| :--- | :--- | :--- | :--- |
| `0x0000` – `0x012B` | 300 | **Console State** | Read-Only section containing the version inputs etc.
| `0x012C` – `0x03E7` | 16 | **Console Config** | section where Console defaults can be overriden / read
| `0x03E8` – `0x0DE7` | 2,560 | **Tilesheet** | Storage for 256 tile definitions ($8 \times 8$ pixels, 4bpp) |
| `0x0DE8` – `0x11CF` | 1,000 | **Background Map** | $40 \times 25$ grid entries for background rendering |
| `0x11D0` – `0x15B7` | 1,000 | **Sprite Layer** | $40 \times 25$ grid entries for sprite graphics |
| `0x15B8` – `0x199F` | 1,000 | **Foreground Map** | $40 \times 25$ grid entries for UI and foreground elements |
| `0x1A00` – `0xFFFF` | 58,880 | **Free SRAM** | Unmapped general-purpose RAM for game state logic |

# Graphics & Display Specification

The Pine16 visual subsystem uses a layered, tile-based hardware engine rendering to a fixed internal frame canvas.

### Screen Specifications
* **Canvas Resolution:** $320 \times 200$ pixels
* **Tile Dimensions:** $8 \times 8$ pixels
* **Grid Capacity:** $40 \text{ columns} \times 25 \text{ rows}$ (1,000 tile entries per layer)
* **Color Depth:** 4 bits per pixel (16 palette entries)
* **Color Format:** 16-bit RGB565 (`RRRR RGGG GGGB BBBB`)