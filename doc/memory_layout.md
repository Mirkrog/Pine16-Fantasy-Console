# Memory Layout

## Overview
### Memory
The console has a total of 64,000 u16 slots of memory. What's special about this console is that it uses u16s instead of simple u8s. Because it is emulated, it has the ability to run on real u16s not u8 pairs.
The memory of the console is divided into 4 sections. Their size and usage are the following:

- `0x0000 - 0x012B` **(300 u16)**: System info (e.g., frame counters, input states, VBLANK flags)
- `0x012C - 0x03E7` **(700 u16)**: System config (e.g., global 32-color palette lookup table, hardware layer scroll registers, sprite attribute tables)
- `0x03E8 - 0x19A7` **(8,120 u16)**: Graphics buffer, which contains 3 contiguous tilemap layers (1,000 words each for Background, Middleground, and Foreground UI) followed by 2 dynamic RAM spritesheets (2,560 words each, 128 tiles per sheet, 20 words per tile) drawn simultaneously by the pipeline
- `0x19A8 - 0xF9FF` **(54,880 u16)**: Will not be used by the System / Memory available for programs

## Memory Map
### System Info Registers (Read-Only)
- `0x0000` **(1 u16)**: Major console version
- `0x0001` **(1 u16)**: Minor console version
- `0x0002` **(1 u16)**: Patch console version
- `0x0003` **(1 u16)**: Major ROM version
- `0x0004` **(1 u16)**: Minor ROM version
- `0x0005` **(1 u16)**: Patch ROM version
- `0x0006` **(1 u16)**: CPU cycle counter (wrapping)

### System Config Registers (Read/Write)
