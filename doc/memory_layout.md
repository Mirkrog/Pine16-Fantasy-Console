# Memory Layout

## Overview
The console has a total of 64,000 u16 slots of memory. What's special about this console is that it uses u16s instead of simple u8s. Because it is emulated,
it has the ability to run on real u16s not u8 pairs.

The memory of the console is divided into 4 sections. Their size and usage are the following:

- `0x0000 - 0x012B` **(300 u16)**: System info (TODO: add examples)
- `0x012C - 0x03E7` **(700 u16)**: System config (e.g: changing the draw mode)
- `0x03E8 - 0x176F` **(8,000 u16)**: Graphics buffer, depending on the draw mode the console will read these u16s to draw an image to the screen
- `0x1770 - 0xF9FF` **(55,000 u16)**: Will not be used by the System / Memory available for programs