# GD's Melee — native PC port

This fork of [doldecomp/melee](https://github.com/doldecomp/melee) adds a native PC port of
*Super Smash Bros. Melee* (NTSC 1.02, `GALE01`) on the **`pc-port`** branch. It is built from the
matching decompilation — every game translation unit is real, editable C — rather than by emulation
or by statically recompiling the retail binary.

On top of that it runs **[m-ex](https://github.com/akaneia/m-ex) modded content**: extra fighters,
stages, items and music, loaded from a modded disc the same way the mod loads them on hardware.

> **Status: work in progress — a research/engineering project, not a release.** It boots, renders,
> plays VS matches, and has audio, memory-card saves, GameCube-adapter input and hard 60 Hz frame
> pacing. Sonic is playable end to end from m-ex data. Netplay, replays, packaging and distribution
> are not started.

## What the port adds

Everything lives under [`pc/`](pc/):

```
pc/platform/   native shims: GX→Aurora, OS, PAD, CARD, AX (DSP-ADPCM mixer), DVD, AR, VI, libc
               plus the m-ex layer: gw_ppc.c (interpreter), gw_mex_ftfunction*.c (fighters),
               gw_mex_grfunction.c (stages), gw_mex_bridge.c (guest→native call bridge)
pc/gameworld/  game-world code compiled through the PowerPC→x86 retarget (gekko_fp.c, mtx_pc.c)
pc/tools/      gwtool (the PPC→x86 retargeter) and asset extraction
pc/build/      Windows build/link/run scripts
pc/tests/      in-engine tests (46, run headless against a real disc image)
pc/docs/       port dev quick-reference
```

The game's own translation units are compiled for PowerPC with clang and retargeted to x86 by
`gwtool`, which keeps guest memory big-endian and provides the GameCube SDK surface through native
shims over [Aurora](https://github.com/encounter/aurora) / [Dawn](https://dawn.googlesource.com/dawn)
(D3D12). See [`pc/README.md`](pc/README.md) for the architecture.

## m-ex content support

m-ex extends Melee by shipping **PowerPC code inside its data files** — each custom fighter and
stage carries a relocatable code blob that the retail engine calls into. Natively compiled x86 has
no way to execute that, so the port runs those blobs through a small **PowerPC interpreter**
(`pc/platform/gw_ppc.c`) and bridges their calls back to the port's own native functions. Content
is read from `MxDt.dat` (m-ex's `mexData`) rather than hardcoded, so adding a fighter is a data
question, not a code one.

| Area | State |
|---|---|
| **Fighters** | Table-driven from `mexData`. **Sonic is playable and hands-on verified**, including his custom spring article (up-B). Akaneia's other six — Wolf, Diddy, Charizard, Lucas, Dedede, Tails — register, load and resolve every call target, but are **not yet play-tested**. |
| **Stages** | `grFunction` plus the expanded stage tables. The first custom stage loads, installs and runs its code; **it does not display correctly yet.** |
| **Items** | Custom articles work — Sonic's spring is a custom item kind spawned from his blob. |
| **Audio/menus** | Per-fighter BGM, weighted menu playlists, announcer, victory themes, results screen, CSS cursor scaling. |
| **Kirby hats** | Not implemented — every m-ex fighter currently gives Kirby the clone base's hat. |
| **ACE** | Dumped and reconciled (31 fighters, 155 stages) but **not enabled**; several tables are still sized for Akaneia. |

m-ex content requires a disc that carries it. A stock `GALE01` image boots fine and simply has none.

### Development conveniences

- `MELEE_SCENE="mode=training;p1=fox"` boots straight to any screen with any configuration —
  characters, stages, CPU levels, teams. A number naming a character or stage **must** say which
  index space it is in (`ck:`/`fk:`/`mex:`, `ext:`/`int:`), because the port juggles four of them.
- A host-side loading overlay reports real progress during the multi-second boot read.
- 46 in-engine tests run headless (no window, no GPU) against a real disc image, so most of the
  m-ex data layer is verified without launching the game.

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
image. Nothing in this repository contains disc data: no `.iso`, `.dol`, `.dat`, `.usd`, `.ssm`,
`.sem`, `.hps`, `.thp` or `.gci` files are tracked, and `.gitignore` refuses them.

**m-ex is treated as a specification only.** m-ex publishes no licence, so none of its sources,
headers or data files are vendored or redistributed here; the support described above is an
independent reimplementation in original C of its published behaviour and file formats. Modded
content itself — Akaneia, ACE or any other build — is not distributed and must be supplied by you.

Not affiliated with, endorsed by or sponsored by Nintendo; *Super Smash Bros.* and *Melee* are
trademarks of Nintendo.

Dependencies — including the vendored Aurora and its port patches — are inventoried in
[`pc/DEPENDENCIES.md`](pc/DEPENDENCIES.md).

## Credits

[doldecomp/melee](https://github.com/doldecomp/melee) · [encounter/aurora](https://github.com/encounter/aurora) ·
[google/dawn](https://dawn.googlesource.com/dawn) · [TwilitRealm/dusklight](https://github.com/TwilitRealm/dusklight) ·
[akaneia/m-ex](https://github.com/akaneia/m-ex) · Dolphin Emulator.
