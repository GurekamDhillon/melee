# `pc/` — native PC port layer

This directory is the platform layer that turns the matching decompilation of *Super Smash Bros.
Melee* (NTSC 1.02, `GALE01`) into a native x86-64/32-bit Windows build. It is not part of upstream
`doldecomp/melee`; it lives on the `pc-port` branch of this fork.

## How it fits together

The game's own translation units are compiled for PowerPC with clang and then retargeted to x86 by
`pc/tools/gwtool`, which byte-swaps every memory access (game memory stays big-endian) and prefixes
every symbol with `gw_`. Everything the game needs from outside its own objects — the GameCube SDK
and the C library — is provided by the native shims in `pc/platform`.

```
pc/platform/     native shims: GX→Aurora, OS, PAD, CARD, AX (DSP-ADPCM mixer), DVD, AR, VI, libc
pc/gameworld/    game-world code compiled through the PPC retarget (gekko_fp.c, mtx_pc.c)
pc/tools/        gwtool (the PPC→x86 retargeter) and asset extraction
```

Rendering goes through [Aurora](https://github.com/encounter/aurora) (a GameCube/Wii SDK
reimplementation) on top of Dawn, to D3D12. Audio is implemented in `shim_ax.c` (DSP-ADPCM decode +
a 64-voice mixer) feeding SDL3. See `pc/platform/gw.h` for the shim/retarget contract.

## Building

The build scripts and the full per-translation-unit pipeline are documented in the project's
`_research/port-dev-quickref.md` and `docs/DEVLOG.md`. The short version: build Aurora + Dawn +
SDL3, compile each game translation unit through `gwtool`, link with the platform shims, and run
`melee-pc.exe --iso <your GALE01 v1.02 image>`.

## Licence

`pc/` is licensed **GPL-2.0-or-later** — see [`LICENSE`](LICENSE).

Upstream `doldecomp/melee` publishes no licence of its own. Nothing here re-licenses the
decompilation; this licence covers the port layer (`pc/`) and is compatible with the GPL-2.0
material it builds on. `pc/gameworld/gekko_fp.c` derives in part from Dolphin Emulator's
`Common/FloatUtils.cpp` (GPL-2.0-or-later) and carries an attribution header.

No copyrighted game material is distributed here. You must supply your own legally dumped disc
image. Not affiliated with Nintendo.
