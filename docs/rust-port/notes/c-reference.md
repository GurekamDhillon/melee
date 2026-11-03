# Which C build is the reference

Use `_build/agents/sfxparity/melee-pc.exe` (built from rust-port lineage) as the C reference for parity work.
`_build/melee-pc.exe` (the main lane) crashes at boot with an unhandled C++ exception (0xE06D7363).

## Why the main lane crashes (diagnosed 2026-11-03, gx5)

- The crash stack (`crashlogs/*-full.log`, resolved against `melee-pc.map`) ends in
  `std::chrono::current_zone()` called from `system_time_to_local_time` in Aurora's
  `extern/aurora/lib/dolphin/os/OSTime.cpp`, then MSVCP140 throws.
- MSVC's `current_zone()` maps the Windows time zone through ICU and throws for a zone ICU does not know.
  This machine's zone ("British Columbia Standard Time") is one of those, so every boot dies. Any environment
  dies the same way because the time zone is a machine setting.
- e5a59f87e added a try/catch fallback to the C runtime (`localtime_s`/`mktime`). The main lane builds from the
  default checkout `melee/`, whose branch (`integration/2026-10-01/roguelite-100-game`, f5662df14) predates that
  commit. Its Aurora libs (`_build/ax86m`, Oct 11) never contained the fix. So the cause is a stale source, not a
  missing DLL or a config.
- The fix is to rebuild the main lane from a source that contains e5a59f87e. The main lane's Aurora CMake cache is
  tied to the `melee/` source path, so pointing it at another tree needs a fresh configure.
- Status: a rebuild attempt removed the Aurora `.obj` files in `_build/ax86m` (the libs remain) and then failed for
  lack of disk space. The original CMake cache was restored and the failed one kept as `*.gx5-failed`. The next
  main-lane build recompiles Aurora.

## Z textures

Aurora's `GXSetZTexture` is an empty `// TODO`. The C build ignores Z textures, while melee_gx implements them
(`shader.rs`, "Z texture"). Draws that use them, such as `HSD_EraseRect`'s depth erase (displayfunc.c: Z8 REPLACE,
used by `HSD_CObjEraseScreen` with depth, e.g. the magnifier bubble) and sobjlib's Z24X8 composite, are not a pixel
reference in C.

## GX round 5 findings (pad_falcon_fox_fd, match frame 221)

- **Shield.** I dumped the draw state with a temporary Aurora patch in a private lane (`_build/agents/gx5c`,
  worktree `gx5c`, never committed) and with `MELEE_GX_DRAWLOG`/`MELEE_GX_DRAWLOG_TEV` on our side. The two match:
  - TEV stages, konst colours (K1 alpha runs 0.92 down to 0.07 frame by frame, the same on both), colour registers,
    channel configs, texgens, post-texture matrices, the C8 64x64 texture and its TLUT, blend and depth.
  - All of our generated WGSL shaders for the frame are byte-identical to a shader Aurora generated.
  - At the same render scale (`MELEE_RENDER_SCALE=3`) the shield crop differs by 0.06/255 mean. At c4be01815 the
    shield matches C. The darker look with a separate outer ring did not reproduce in view-golden.
- **Magnifier bubble** (cpu_peach_samus_ys_items, frame 1220): the bubble, its fill colour and the fighter inside
  match C (0-255 mean diff 1.9; ours a little softer).
- **Z textures.** The only reachable Z-texture user is `HSD_EraseRect`'s depth pass (Z8 REPLACE, used by the magnifier
  erase). Its z compare runs before texturing, so the hardware never sees the texel and melee_gx applies no Z texture
  there. The SObj Z24X8 composite (`HSD_SObjLib_803A4A68`, flag 4 from the SObj data) is used by gm_1832's 1P-ending
  snapshots, which no test flow reaches yet. No reachable scene shows a Z texture, and Aurora ignores them anyway.
- Debug aids kept: `MELEE_GX_DRAWLOG_TEV` also prints texgens, the first two textures, the TLUT slots, the vertex
  attributes and the uniform buffer; `MELEE_GX_TEXCOPYLOG=1` logs every GXCopyTex.
