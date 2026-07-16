# Memory Layout

## Overview
### Memory
The console has a total of 64,000 u16 slots of memory. What's special about this console is that it uses u16s instead of simple u8s. Unlike traditional byte-addressed systems, this console utilizes a 16-bit word-addressed architecture. Every address points to a full u16 slot rather than a u8 pair.
The memory of the console is divided into 4 sections. Their size and usage are the following:

- `0x0000 - 0x012B` **(0 - 299)**: System info (e.g., frame counters, input states, VBLANK flags)
- `0x012C - 0x03E7` **(300 - 999)**: System config (e.g., global 32-color palette lookup table, hardware layer scroll registers, sprite attribute tables)
- `0x03E8 - 0x19A7` **(1000 - 6,567)**: Graphics buffer, which contains 3 contiguous tilemap layers (1,000 words each for Background, Middleground, and Foreground UI) followed by 2 dynamic RAM spritesheets (2,560 words each, 256 tiles per sheet, 20 words per tile) drawn simultaneously by the pipeline **`TODO: update this doc`** 
- `0x19A8 - 0xFFFF` **(6,568 - 65,535)**: Will not be used by the System / Memory available for programs

## Memory Map
### System Info Registers (Read-Only)
- `0x0000` **(0)**: Major console version
- `0x0001` **(1)**: Minor console version
- `0x0002` **(2)**: Patch console version
- `0x0003` **(3)**: Major ROM version
- `0x0004` **(4)**: Minor ROM version
- `0x0005` **(5)**: Patch ROM version
- `0x0006` **(6)**: CPU cycle counter

### System Config Registers (Read/Write)
- `0x012C` **(300 - 314)**: Here you can override the default palette which has 15 colors + blank (because of blank it is 1 indexed). One word per color, packed like this: [ Red (5b) : 00 01 02 03 04 | Green (6b): 05 06 07 08 09 10 | Blue (5b): 11 12 13 14 15 ]
