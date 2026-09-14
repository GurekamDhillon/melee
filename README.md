# GD's Melee — native PC port

This fork of [doldecomp/melee](https://github.com/doldecomp/melee) adds a native PC port of
*Super Smash Bros. Melee* (NTSC 1.02, `GALE01`) on the **`pc-port`** branch. It is built from the
matching decompilation — every game translation unit is real, editable C — rather than by emulation
or by statically recompiling the retail binary.

> **Status: work in progress — a research/engineering project, not a release.** It boots, renders,
> plays VS matches, and has audio, memory-card saves, GameCube-adapter input and hard 60 Hz frame
> pacing. Netplay, replays, packaging and distribution are not started.

## What the port adds

Everything lives under [`pc/`](pc/):

```
pc/platform/   native shims: GX→Aurora, OS, PAD, CARD, AX (DSP-ADPCM mixer), DVD, AR, VI, libc
pc/gameworld/  game-world code compiled through the PowerPC→x86 retarget (gekko_fp.c, mtx_pc.c)
pc/tools/      gwtool (the PPC→x86 retargeter) and asset extraction
pc/build/      Windows build/link/run scripts
pc/docs/       port dev quick-reference
```

The game's own translation units are compiled for PowerPC with clang and retargeted to x86 by
`gwtool`, which keeps guest memory big-endian and provides the GameCube SDK surface through native
shims over [Aurora](https://github.com/encounter/aurora) / [Dawn](https://dawn.googlesource.com/dawn)
(D3D12). See [`pc/README.md`](pc/README.md) for the architecture.

## Building

See [`pc/build/README.md`](pc/build/README.md) and [`pc/docs/PORT_DEV_QUICKREF.md`](pc/docs/PORT_DEV_QUICKREF.md).
Broadly: build Aurora + Dawn + SDL3, compile each translation unit through `gwtool`, link with the
platform shims, then run `melee-pc.exe --iso <your GALE01 v1.02 image>`.

## Licence and legal

`pc/` is licensed **GPL-2.0-or-later** — see [`pc/LICENSE`](pc/LICENSE). Upstream `doldecomp/melee`
publishes no licence of its own; nothing here re-licences the decompilation. `pc/gameworld/gekko_fp.c`
derives in part from Dolphin Emulator's `Common/FloatUtils.cpp` (GPL-2.0-or-later) and carries an
attribution header.

**No copyrighted game material is distributed here.** You must supply your own legally dumped disc
image. Not affiliated with, endorsed by or sponsored by Nintendo; *Super Smash Bros.* and *Melee*
are trademarks of Nintendo.

## Credits

[doldecomp/melee](https://github.com/doldecomp/melee) · [encounter/aurora](https://github.com/encounter/aurora) ·
[google/dawn](https://dawn.googlesource.com/dawn) · [TwilitRealm/dusklight](https://github.com/TwilitRealm/dusklight) ·
Dolphin Emulator.
