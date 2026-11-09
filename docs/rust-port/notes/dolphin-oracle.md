# Dolphin oracle: the console's own memory as a third NHV1 oracle (2026-11-05, branch port/dolphin-oracle)

No replay data, disc data or disc path is in this note or in git. Replays: the workspace's untracked `scratch/replays-ext/`
(notes/replays-ext.md, notes/slpcheck.md). Everything below is a local tool: nothing is published.

nhv1.md section 9 asked for a third oracle: read the NHV1 fields from GameCube memory after each frame's
`HSD_GObj_RunProcs` and run the same `nhv1_ref.py` words over them. This is it. It plays a `.slp` in Slippi's Dolphin
(playback mode, vanilla NTSC 1.02 disc), dumps the console's `struct Fighter`/`struct Item` records every frame, and
compares them with the Rust engine running the same replay (`melee slpcheck`), first as NHV1 rows, then field by field.

## Usage

```
bash tools/rust-port/dolphin/oracle.sh REPLAY.slp OUT_DIR [--wide] [--window N] [--at SLIPPI_FRAME] [--zero-eq]
```
Last line: `ORACLE <file> match (...)` or `ORACLE <file> diverged at frame F (slippi F-123): <columns>; window ending at
slippi S, new at that frame: <inst.field ...>; earliest new difference in the window: slippi E`.
OUT_DIR keeps `dolphin.csv`, `rust.csv`, `compare.txt` (seed ignored), `compare_seed.txt` (with seed/hash, `--print-count`),
`rust_wide.txt`, `dolphin_wide.txt`, `wide_diff.txt`, `rust.log`, `slpcheck.csv`, `dolphin.stderr.txt`. `KEEP=1` keeps the raw dump.

| step | what |
|---|---|
| 1 | `run_dolphin.ps1`: `dolphin-emu-nogui -p headless -v Null -u <private user dir> -i comm.json -e <disc>`; hidden, Idle priority (`DOLPHIN_PRIORITY`), stdout/stderr redirected, killed after `-TimeoutSec`. Writes `dolphin.nhraw` |
| 2 | `raw2csv.py`: GC fields at the nhv1.md 5.3/5.4 offsets -> `nhv1_ref.fighter_inputs` / `cpu_inputs` / `item_word` / `game_word` -> `dolphin.csv` (`frame` = slippi frame + 123) |
| 3 | `melee slpcheck --full --nhv1-out rust.csv`: the engine on the same replay with slpcheck's replay setup (Game Start seed, per-stage pre-draws, widescreen code, pre-frame reseed), the row of `Engine::neutral_row` per frame |
| 4 | `compare_golden.py rust.csv dolphin.csv --ignore-seed --allow-short` (nhv1.md 8: the seed column differs by render draws) |
| 5 | on a mismatch (or at `--at S`): `melee dolphin-wide` and `slpcheck --wide-out --wide-from/--wide-to` dump slippi frames S-N..S by name; `wide_diff.py` lists the fields that newly differ per frame |

No keyboard, mouse or focus: Dolphin is driven by the command line and the Slippi comm file only; it opens no window
(headless platform, Null video backend) and has no audio backend (`Backend = No Audio Output`, volume 0, the MELEE_VOLUME rule's
equivalent). Each Dolphin run takes about 40 s for a 10,000-frame replay (unthrottled, single-core); slpcheck about 30 s.

## The Dolphin build (`_build/dolphin-oracle/`)

| | |
|---|---|
| source | `project-slippi/dolphin` (the mainline-based Slippi Dolphin, the one with the playback code), shallow clone of `0653df819` (2026-11-02) in `_build/dolphin-oracle/src`, submodules shallow |
| patch | `tools/rust-port/dolphin/slippi-dolphin-nhv1.patch` (Dolphin is GPL-2.0-or-later; the diff stays local) |
| build | `tools/rust-port/dolphin/build_dolphin.bat` (VS 2022 Build Tools, CMake + Ninja, Release, `dolphin-nogui` only, Rust 1.88 for the Slippi extensions); copies Sys, Slippi's playback Gecko codes as the game settings, the Rust DLL and the user config (`Dolphin.ini`, `Logger.ini`). Output `_build/dolphin-oracle/src/Binary/x64/dolphin-emu-nogui.exe` |
| disk | about 6 GB (source + submodules + build tree) |

What the patch changes, and why:

| change | why |
|---|---|
| `Core/Slippi/NHV1Oracle.{h,cpp}` + an HLE hook `NHV1FrameEnd` (HookType::Start, HookFlag::Fixed) at **0x801A4FA4** | the instruction after `bl HSD_GObj_RunProcs` (0x801A4FA0) in `gm_801A4D34`, found by scanning main.dol; nhv1.md section 2's log point exactly. Not touched by any playback Gecko code. Only installed when `MELEE_DOLPHIN_NHV1` is set |
| `NHV1Oracle::NoteFrame` in `CEXISlippi::prepareFrameData` | the slippi frame the game is about to simulate; a record is written once per fetched frame, so fast-forward iterations are logged too |
| `prepareCharacterFrameData`: `std::bit_cast<u32>` instead of `static_cast<u32>` for the float inputs | an upstream bug of the WIP mainline playback: the sticks, trigger, x/y, facing and percent went to the game as float->int conversions, and the first frame hit `assert(pos3d->x > -50000 ...)` in `lbVector_WorldToScreen` |
| `IS_PLAYBACK=1` for `SlippiPlayback.cpp`, `EXI_DeviceSlippi.cpp`, UICommon and DolphinNoGUI only | the `-i` option and the playback code paths are behind it and upstream never defines it; defining it globally does not compile (HotkeyManager, OnScreenDisplay) |
| MainNoGUI: set `slippi_input` again after the second `UICommon::Init()` | that call re-creates SConfig and dropped `-i` |
| DolphinNoGUI links `ZLIB::ZLIB` instead of `z` | no `z.lib` on Windows |
| `Common::Timer::GetLocalTimeSinceJan1970` falls back to UTC | `std::chrono::current_zone()` throws on this machine (no tzdb), aborting at `ExpansionInterfaceManager::Init` |
| diagnostics | `%TEMP%\nhv1-oracle-crash.txt` stack on abort/terminate/invalid parameter; `MELEE_DOLPHIN_NHV1_WATCHDOG=1` prints the PPC pc every 2 s; `MELEE_DOLPHIN_NHV1_NOHOOK=1` records nothing but NoteFrame |

Env: `MELEE_DOLPHIN_NHV1=<file>` (on), `MELEE_DOLPHIN_NHV1_WIDE=1` (+ camera and stage), `MELEE_DOLPHIN_NHV1_EXIT=1` (exit the process
after the replay's last frame; set by run_dolphin.ps1).

Playback settings: `shouldResync = false` in the comm file. Slippi's `RestoreGameFrame.asm` otherwise writes the recorded x, y,
facing and percent and the RNG seed into every fighter each frame (`PDB_SHOULD_RESYNC`), which would make the console state
partly a copy of the file. With it off, only the inputs are restored, as in slpcheck. (`DOLPHIN_RESYNC=1` restores Slippi's default;
the first measured file gave the same first divergence either way.)

Traps met: a GUI-subsystem exe started without redirected stdio dies silently (`0xC0000409`) at the first stderr write
in some paths, so run_dolphin.ps1 always redirects; the Bash tool's `cmd //c` does not run a `.bat` (use PowerShell `cmd /c`).

## Record format (`.nhraw`)

Header `NHRAW001`, u32 flags (bit 0 wide). Per logged frame (little-endian header words, GC memory blobs raw big-endian):
`u32 'RFRM'`, `s32 slippi frame`, `u32 gm_80479D58.unk_0`, `u32 *HSD_RandSeedPtr`, `u32 n_fighters`, `u32 n_items`, then per
fighter `u8 slot (player*2+j), u8 pkind, s8 stocks, u8 transformed[j], u32 fp address, Fighter[0x23EC]`, per item (item p-link 9,
list order) `u32 ip address, Item[0xFCC]`, and with wide `game_camera[0x39C]` (0x80452C68) and `stage_info[0x748]` (0x8049E6C8).
Fighters are `Player_GetEntityAtIndex(i, j)` = `player_slots[i].player_entity[transformed[j]]` (0x80453080, 0xE90 per slot);
a frame with no fighter is not logged (nhv1.md 3). About 27 KB per frame for two fighters (300 MB per 10,000 frames; deleted after the run).

The CPU block (nhv1.md 5.3 words 22-47) is hashed when `ftCo_IsCpuControlled` would answer true: pkind CPU, or a sub fighter
of a character without a transformation (Nana), and `cpu.kind != 5`. Zelda/Sheik's inactive form is a sub fighter but not a CPU.

## The by-name dump (Rust side and Dolphin side, one format)

`<slippi frame> <instance> <field> <value>`; instances `s<slot>`, `it<k>` (item list order), `cam`, `stage`, `rng`.
Field names, GC offsets and kinds are `rust/melee/src/diff_layout.rs` (tools/rust-port/diff/gen_layout.py from melee_types'
generated decoders, i.e. from the decomp headers), now generated for `StageInfo Camera Fighter Item`. The Dolphin side decodes
the GC bytes with the generated `GcDecode` (pointers null) and walks the same tables (`diff::named`), so both files print a field
the same way; pointers and unions are left out. `melee dolphin-wide --raw R --out T [--from F --to F]`;
`melee slpcheck ... --wide-out T [--wide] [--wide-from F --wide-to F]` (a whole match is ~2 GB, hence the window).
`wide_diff.py` prints, per frame, only the fields that newly differ (many never agree: heap addresses in int fields, a
human's CPU scratch, and the render-written fields below), `--persistent` all of them, `--zero-eq` treats f32 +0/-0 as equal.

**Render-written fields are one frame apart in the two wide dumps.** Dolphin's record is taken before the frame's
render; the engine's wide dump is taken after its logic-side stand-ins for the draw callback (`ftLib_UpdateLogicOffCamera`,
the Screen-KO `place_in_view`) have run. So `x2188` (the screen position `ftLib_UpdateScreenVisibility` stores; not a
stick counter) and `cur_pos` during DeadUpFallHitCamera read in Dolphin's frame F what the engine's frame F-1 shows.
The NHV1 row (`neutral_row`, taken at Script_FramePost) is not affected. A "new" `x2188` difference means only that
the two happened to be equal the frame before.

## Results (rust-port 9f07d196a + this branch, fast profile)

slpcheck over the 125 files at this tip: 102 frame-exact, 23 diverging (the 39 of slpcheck.md round 5 are fewer now).

**No replay matches Dolphin all the way.** The 102 slpcheck-exact files compare 11 Post-Frame fields; NHV1 hashes 21 words per fighter,
and two classes show up that slpcheck cannot see:

| class | where | what |
|---|---|---|
| sign of zero in `x8c_kb_vel.y` | every slpcheck-exact file tried (6 of 6: first NHV1 difference at CSV frames 353-1870) | the console holds **-0.0**, the engine +0.0, on a frame where knockback ends (all other fields equal). NHV1 hashes bit patterns (section 4), so the word differs; the run rejoins on later frames (01c6d260: 151 of 11293 rows differ, all in s0/s2) |
| Nana's CPU block at frame 0 | Ice Climbers files (18e83ec4 slot s1) | `cpu.x34` 178 (Rust) vs 161 (console) from the first frame; also `x20A4.bf_0` |
| spawn landing on Yoshi's Story | 50c50fc5, 20689_62bd9250 (slot s6, CSV frame 20, slippi -103) | `coll_data.prev_pos.y` 23.02 vs 28.0, `cur_pos.y` 1 ULP, `bf_221c` one bit: the spawn-platform landing frame differs |

The replay used as "the one we play exactly": `20476_01c6d260` (Battlefield, Falco vs Sheik; slpcheck `match`,
11,169 frames). Dolphin and Rust agree on CSV frames 0-1387 in every NHV1 column, `seed` and `hash` included. First
difference: **CSV frame 1388 (slippi 1265), s0, `x8c_kb_vel.y` +0 vs -0**; wide_diff names it as the only new field on that frame.
Our own vanilla `.slp` exports (`pad_falcon_fox_fd.slp`, from the C build) do not play in Slippi Dolphin: they have no
Post-Frame/Game End events, so playback waits on frame -123 forever (`frame -123 not ready`).

Five of the 23 diverging files (oracle run twice: the first NHV1 difference, and `--at <slpcheck frame> --zero-eq`):

| file | slpcheck first divergence | first NHV1 difference (oracle) | fields at slpcheck's frame (new in the window) |
|---|---|---|---|
| 0a7cb485 (Dream Land) | 9243 p4 action 38 vs 358 | 2557 s0 `x8c_kb_vel.y` -0 | 9242: s6 `x2188.y`; 9243: s6 `motion_id` 38 vs 358, `anim_id`, `cur_anim_frame`, `x1064_thrownHitbox.x4C` (-89.2 vs -91.8), rng seed: the console's fighter is grabbed/thrown (358), ours is not |
| 18e83ec4 (Final Destination, Ice Climbers) | 3275 p4 hitlag 5 vs 8 | 0 s1 (Nana CPU block, `cpu.x34`) | 3273: s1 ECB x 1 ULP (`coll_data.ecb.left/right.x`), `cpu.x55C` 1 ULP; 3275: s1 (Nana) `anim_id`, `cur_anim_frame`, `cur_pos.z` -3.2 vs 0, `bf_2218`/`bf_221c` |
| 325cee5d (Yoshi's) | 642 p1 action 41 vs 40 | 765 s0 | 640: s6 `x2188.y`; 642: s0 `motion_id` 41 vs 40, `anim_id`, `x1064_thrownHitbox.x4C.y` 3.97505 vs 3.97559, `x1670[0].x18.x`, `bf_221c` one bit |
| 4c3516cc (FD) | 11858 p1 percent 82.4 vs 92.4 | 307 s6 `x8c_kb_vel.y` -0 | 11858: item it0 hitbox `hurt_coll_pos` 1 ULP, `coll_distance`, `xCBC_hitlagFrames` 7 vs 11; s0 `dmg.x1830_percent` 82.38 vs 92.38: an item hit with a 1-ULP different collision point |
| 50c50fc5 (Yoshi's) | 10003 p4 shield 49.4 vs 42.6 | 20 s6 spawn landing (`coll_data.prev_pos.y`) | 10001: s6 hurt capsules 1 ULP, it5 hitbox 1 ULP; 10002: 700 fields; 10003: s6 `shield_health`, `gr_vel`, `frame_speed_mul`, `x206C_attack_instance` 1855 vs 1856: one more attack instance in the console |

Seed: with resync off the console's seed equals ours from frame 0 for the first 20 to 1388 rows (`compare_seed.txt`; 0 rows on the Ice Climbers file),
then differs (render-side draws, nhv1.md 8), so `--ignore-seed` decides.

## Known limits

- One fighter record per `Player_GetEntityAtIndex`; items only from the item p-link (nhv1.md 6).
- `it<k>` pairs items by list order; a differently ordered list shows as many item differences.
- Some int fields decode differently by construction (`Item.xDA4_word` 1 vs 0x1000000: a field the native struct holds in another
  width); they are steady and drop out of `wide_diff`'s new-difference view.
- Not an emulator-accuracy check of Dolphin itself: Dolphin is taken as the console.

## Probe hook and stop frame (port/oracle-fix)

`MELEE_DOLPHIN_NHV1_PROBE_AT=<hex pc>[,<pc>...]` adds an HLE hook at each PC. Each hit appends one line to
`MELEE_DOLPHIN_NHV1_PROBE` (a text file) with the playback frame, pc, lr, r3-r6, r29-r31, f1/f2 bits and, with
`MELEE_DOLPHIN_NHV1_PROBE_MEM=rN+0xOFF:0xLEN` (LEN up to 0x4000), the hex of that memory. `r2+OFF` reaches a fixed
address (r2 = 0x804DF9E0). The logged `pc` is the start of the JIT block, not the hooked address; `lr` is the reliable
field. `MELEE_DOLPHIN_NHV1_STOP_AT=F` exits Dolphin after it logs frame F, so a probe run needs no kill.

## Bone dump (port/oracle-fix round 3)

`MELEE_DOLPHIN_NHV1_BONES=<txt>` (frames `MELEE_DOLPHIN_NHV1_BONES_FROM`..`_TO`) writes, for every logged fighter, each part's
jobj world matrix and its inputs: `<frame> s<k> bone[<i>].mtx[<n>] <bits>` (`fp->parts[i].joint->mtx`, jobj+0x44, 12 words) and
`bone[<i>].in[<n>]` (n 0 = jobj flags, 1-4 rotate, 5-7 scale, 8-10 translate). The Rust side writes the same lines with
`MELEE_WIDE_BONES=1 melee slpcheck ... --wide --wide-out T --wide-from F --wide-to F`. A joint whose matrix the Rust engine
has not computed (dirty flag 0x40 set, identity matrix) is a lazy matrix nobody read that frame, not a difference. Found with it:
the PSMTXQuat class (notes/fp-audit.md "Paired-single library routines").

`MELEE_DOLPHIN_NHV1_PROBE_MEM=r1+0x0:0x400` at a callee's entry dumps the caller's stack frame (r1 is the caller's SP, its
first word the back chain): locals passed by pointer can be read at `r3 - r1`. `MELEE_DOLPHIN_NHV1_PROBE_CHAIN=rN:0xNEXT:0xLEN:COUNT`
walks a linked list from register N (next pointer at +NEXT) and appends `node<i>@<addr>=<LEN bytes hex>` for up to COUNT
nodes (Samus's grapple ItemLink chain: `r4:0x4:0x30:64` at it_802BA3BC).

## Slippi playback's first-frame write (an oracle artifact, undone)

On the first playback frame (-123), Slippi's playback code injected at `Fighter_procInput` 0x8006B0DC writes
`coll_data.cur_pos`/`last_pos` from `fp->cur_pos` and `coll_data.x38` from the line counter. It does this even with
`shouldResync=false`, and only on that frame. The recording console never ran this code. Found with the probe:
fighter memory dumped before and after 0x8006B0DC differs at exactly +0x6F8, +0x710 and +0x728.

Effect: for a fighter whose spawn ground probe misses (`ft_80082A68`), the first air step after the entry animation
starts from a different `last_pos`. The landing frame's `cur_pos.y` then differs by 1 ULP. This was the frame 5/10/15/20
class (entry delays 5*(slot+1)) in 23 replays. The recorded Post-Frame agrees with Rust, not with playback: 03b84423,
-118 y = 41a85c00 in the .slp and in Rust, 41a85bff from Dolphin. Hero's replay-validation note independently lists
the same original-vs-playback 1-ULP pre-frame Y pairs.

The oracle now saves fp+0x6F4..0x72C at 0x8006B0DC and restores them at 0x8006B120 on frame -123 (`NHV1UndoSave`
and `NHV1UndoRestore`). `MELEE_DOLPHIN_NHV1_KEEP_PLAYBACK_INIT=1` keeps Slippi's write. With the undo, 03b84423's first
difference moves from frame 5 to frame 8015.

The same injection also calls `Camera_8002F3AC` (snap the camera to its target) once per fighter on frame -123 (probe:
`lr` 0x8065D3C8, called from the Fighter_procInput hook). The camera then started 2 updates ahead (`x2B8` 4 vs 2) and
from another position, and its interest/eye stayed a few ULP off for the whole match (every replay, matching ones too).
The oracle now also saves `game_camera` (0x80452C68, 0x39C bytes) at the first fighter's save and restores it at each
restore (round 10; `MELEE_DOLPHIN_NHV1_KEEP_PLAYBACK_CAMERA=1` keeps the snap). With it, 01c6d260's camera equals
Rust's from frame -123. It was not the cause of the Ice Climbers drift (that was the two fixes in "Nana" below).

## Nana (round 10)

- `ftCo_800ADE48`, the 0x12 command test: the branch taken when `fp->x221A_b3` is set (the level roll) assigns
  nothing to `switch_cmd`, which MWCC keeps in r31. The prologue saves r31 without clearing it, so the test reads the
  caller's r31: a pointer (fp or the CPU block) at every call site except ftCo_800B21C8's last call (0x800B2494,
  after `li r31, 0`). Probe at 0x800AE270: 46 hits with r31 = the CPU block in 18e83ec4, the first at frame 41.
- UCF dashback (0.8 `UCF DB.asm` for Popo, 0.84 `UCF Dashback.asm` for any fighter with a sub fighter) also
  rewrites the sub fighter's last queued input entry (`cpu.x444`, fp+0x1ECC): facing = the new facing, stick x =
  0x7F right / 0x80 left. Found from `cpu.xFC[23]` holding -128 on the console after ftCo_800B0918 had stored -41.
- Result: 18e83ec4 matches Dolphin on all 8718 frames and slpcheck to the end; Hero Game_20260321T214020 matches.
- Oracle builds while another agent's Dolphin runs: the shared exe is locked, so round 10 linked a private copy under
  `_build/dolphin-oracle-of10` (`DOLPHIN_ORACLE_DIR`). The shared `_build/dolphin-oracle` source has the camera
  change; its exe is rebuilt by the next `build_dolphin.bat`.

## Widescreen codes and the strict sweep (port/strict1)

**The recording console's widescreen code.** Slippi's playback code set does not re-apply the replay's own Gecko
codes. 35 of the 125 local replays (all `20689_*`) were recorded with a widescreen code whose camera bounds
(0x80030C7C/0x80030C88) and bubble constants (0x804DDB28..0x804DDB84) change the off-screen test
(`ifMagnify_802FC998`), so playback's 1% offscreen tick (`dmg.x1910` vs `x7AC`) fired a frame off. The .slp decides:
20689_ccd2e52b slot 4 has 115.83 at slippi 11504 in its Post-Frame and in Rust, 116.83 in playback. `run_dolphin.ps1`
now runs `replay_codes.py`, which copies the replay's codes at the addresses of Slippi's "Widescreen 16:9" verbatim
into the run's `<user>/GameSettings/GALE01.ini` (`DOLPHIN_NO_REPLAY_CODES=1` plays without them). Not a mask: the
oracle console runs what the recording console ran. Fixed 16 strict divergers.

**Strict sweep.** Dolphin's NHV1 rows depend only on the replay and the oracle exe, so the sweep caches one
`dolphin.csv` per replay and re-runs only the engine (`slpcheck --full --nhv1-out` + `compare_golden.py --ignore-seed`).
Dolphin at most 2 at a time, one run per replay (no nondeterminism check: re-run a single file with `oracle.sh` before
blaming the engine). Results, all 125 local replays:

| | strict (all NHV1 words equal) |
|---|---|
| rust-port c8ae14749, shared oracle exe (no camera undo), no replay codes | 93 of 121 (4 dumps truncated) |
| + widescreen codes, + it_8027781C fix | 113 / 125 |
| rebased on ef0c6dad2 (round 10 Nana fixes), `_build/dolphin-oracle-of10` exe (camera undo) | **117 / 125** |

Engine fix found by the sweep: `it_8027781C` squares the velocity unfused (fmuls, fmuls, fadds at 0x80277894 and
0x80277B24; only the dot products with the surface normals are fmadds). Dr. Mario's pill bounce was 1 ULP off
(3135285b, 3417139d).

Left after port/strict1 (8), all resolved or narrowed by port/strict2 below.

## Round 2 of the strict sweep (port/strict2): 117 -> 124 of 125

Oracle exe: `_build/dolphin-oracle-strict2` (private copy of the shared source + the change below; `build.bat` there
is `build_dolphin.bat` with `D` changed). Cache: one `dolphin.csv` per replay as in round 1.

| | strict |
|---|---|
| port/strict1 end (of10 exe) | 117 / 125 |
| + whole first-frame undo (strict2 exe) | 119 / 125 (383fcf15 lost to nondeterminism, see "Polling drift") |
| + the recording console's Polling Drift Fix (replay_codes.py), + Slippi's Preload Stadium Transformations in the engine | **124 / 125** |

**The whole first-frame write is undone.** `RestoreGameFrame.asm`'s spawn correction (frame -123) also turns the
root joint (`ftPartSetRotX`), writes the static player block (`Player_80032828`), the CameraBox
(`ftCamera_UpdateCameraBox` computes the camera bone's matrices, then x2C/x30 from x40/x44) and `fp+0x2344`. The
undo now snapshots MEM1 at 0x8006B0DC and, at 0x8006B120, puts back every changed byte inside the fighter's own
objects: the fighter (0x23EC bytes, except its input words fp+0x620..0x660, which are the replay's), its CmSubject
(fp->x890, 0x6C), its jobj tree (from gobj->hsd_obj) and `player_slots`. Restoring all of MEM1 hung playback at
frame -118 (it also undid Slippi's own state). `MELEE_DOLPHIN_NHV1_UNDO_LOG=1` lists every changed byte (`kept` /
`restored`), `MELEE_DOLPHIN_NHV1_UNDO_PARTIAL=1` restores only round 10's fields. Fixed 0854ab08, 4e71d712, f41d427a.

**Polling drift.** Online replays carry Slippi's "Polling Drift Fix" (C2 0x801A4DA0, 04 0x801A4DB4, 04 0x80019860;
part of "Required: Slippi Online"), which playback's code set lacks. Without it the engine loop is paced by an OS
timer at 60 Hz (`lb_80019880(OSSecondsToTicks(1/60))` in gm_801A4BD4) against the 59.94 Hz retrace, so every ~1001
frames the loop runs two logic frames before one render. A probe on the loop start (0x801A4DE4) and render start
(0x801A5024) found such frames at 1825, 2826, ... 7831 in 03b84423 and 1316, 2312, 3317 ... in 21cd149c. The render
pass is what sets the off-camera flag (`x221F_b0`, ftLib_UpdateScreenVisibility), so the skipped render moved the
1% offscreen tick, `x7AC` frames later: 03b84423 at 7892 (skip at 7831), 21cd149c at 3378 (skip at 3317). The phase
of the cycle depends on how long playback waited before the match, which is what made Dolphin nondeterministic:
two runs of 383fcf15 with the same exe differed from slippi frame 4830, one equal to Rust. `replay_codes.py` now
copies the replay's drift codes too (verbatim, like the widescreen code); with them the probe finds no skipped render
in game and both files match. The recording console ran them; its Post-Frame percent agrees with the engine.

**Preload Stadium Transformations (engine).** The Slippi Recording core set carries `Common/Preload Stadium
Transformations` (C2 0x801D14C8, 0x801D45EC, 0x801D4F14; 04 0x801D460C, 0x801D4610, 0x801D4724; tagged
affects-gameplay; playback runs the replay's copy). Pokemon Stadium decides its next transformation (the HSD_Randi(4)
loop) at the first frame of the normal countdown instead of when it ends, loads it then, and at the end takes the
preloaded id without a draw. 4cb4510f: an extra HSD_Randi at slippi -39 (Dolphin probe lr 0x80BE0C08, inside the
code; the recording's Pre-Frame seed at -38 is one draw past the engine's), so `cpu.x30` (120 * 0.25 * Randf, Nana's
CPU) differed at -38. grpstadium.rs now follows the six inserts when the replay's Gecko list has 0x801D45EC
(`melee_core::codes::ps_preload_transformations`). The 3.x Frozen PS (04 0x801D45FC, `bge` -> `b`) still runs the
case's entry, so a frozen stadium decides once; the later IngameCheckIfFrozen (C2 0x801D457C) returns first
(`ps_freeze_at_entry`). Fixed 4cb4510f. adc68cd8 (camera, slippi 5323) also matches after these two changes; which of
them fixed it was not isolated.

Left (1): **602a5cd8** at csv 5452 (slippi 5329): s6 `x2188.x` (screen position, 300 in Rust, 295 in playback) on
one frame, then s0. Resolved in round 3 below (a playback artifact: the second polling-drift variant).

## Round 3 (port/f602): 602a5cd8, the second polling-drift variant: 124 -> 125 of 125

**What differed.** The cached `dolphin.csv` (re-run: identical, so deterministic here) differs from the engine in `s0`
only, csv 5452-5476 (slippi 5329-5353). The wide window shows at 5329 the camera (`transform.interest/position`, a few
hundredths), s0's `prev_pos`, `pos_delta` (0 in Dolphin), hurt capsules and `x1064_thrownHitbox`; s0 is Marth in
`DeadUpFallHitCamera` (motion 7, the Screen KO, `cur_pos.z` 231). The s6 `x2188.x` line that round 2 reported was the
one-frame offset described under "The by-name dump": 5327 and 5328 happened to be equal (295, 295).

**Why.** A probe on the engine loop start (0x801A4DE4) and the render start (0x801A5024) finds playback frames with two
logic iterations and no render between them at 1323, 2319, 3324, 4326, **5328**, 6328, 7330: the ~1002-frame polling
drift of "Round 2", still present. In a Screen KO the draw callback writes the fighter's position
(`ftDrawCommon_80080E18` inline2: camera-space `mv.co.walk.fast_anim_frame` through the inverse viewing matrix into
`cur_pos`), so the skipped render left s0's `cur_pos` at its 5328 value for logic frame 5329, the camera followed the
stale position, and camera and position stayed a few hundredths off until the respawn 25 frames later. The .slp
decides: its Post-Frame for p1 at 5329 is x -6.023935 (c0c0c414), y 34.842739 (420b5ef7), the engine's values;
playback has 5328's -7.455233 (c0ee9145), 29.374111 (41eafe2e) again. The recording console rendered that frame.

**Why the drift codes of round 2 did not apply.** This replay's Gecko list has no code at 0x801A4DA0/0x801A4DB4/
0x80019860. It carries Slippi's newer **"Polling Drift + Half Visual Buffer Fixes"**
(`Sys/Slippi/InjectionLists/list_console_lag_pdhalfvb.json`: C2 0x801A4BEC, 0x801A4D98, 0x801A5050 in
gm_801A4BD4/gm_801A4D34, 04 0x80158268, C2 0x8034EB60, 04 0x80397878, and 480pDeflickerOff at 0x801A4570), which
`replay_codes.py` did not know. Of the 125 local replays, 90 carry both variants (round 2's copy covered them) and the
35 `20689_*` widescreen files carry only this one, so they played back with no pacing fix; 34 of them matched anyway
because a skipped render only matters when it hits a Screen KO or an offscreen tick. `replay_codes.py` now copies this
variant too, verbatim (`DRIFT_HALFVB`). With it the probe finds one render per logic frame on all 7778 in-game frames,
and Dolphin's NHV1 rows equal the engine's on all 7779 (`compare_golden.py --ignore-seed`: identical). The engine was
not changed: it matched the recording's Post-Frame on every frame throughout (slpcheck `match`, 15558 fighter-frames).

**Results (strict2 exe, rust-port b05adf319, engine unchanged).**

| run | result |
|---|---|
| 602a5cd8, no pacing code (cached dump, and a re-run) | the two Dolphin runs identical; 25 rows differ from the engine (csv 5452-5476, s0) |
| 602a5cd8 + the replay's half-VB codes, twice | the two runs identical; **7779 / 7779 rows equal to the engine** |
| 03804bd3, ccd2e52b, adc68cd8 + their half-VB codes | each identical to its round-2 (no pacing fix) dump: a skipped render did not reach any NHV1 word in them |

Strict sweep: **125 / 125**. The 31 other half-VB dumps in the round-2 cache were made without a pacing fix and matched
the engine; a re-dump would be the way to make them the recording console's run too, but their rows can only move on a
frame where a skipped render reached gameplay, and none did.

## Round 3 of the strict sweep (port/strict3): Hero's vanilla replays, 376 -> 377 of 377

Hero's `_build/replays/hero-2026-11-06`, the 377 vanilla files (the 51 Akaneia files with external char ids > 25
skipped), all Slippi 3.19.x online games. Same exe (`_build/dolphin-oracle-strict2`), one cached `dolphin.csv` per
replay, engine re-run only.

| | strict |
|---|---|
| rust-port 470d07b9b | 376 / 377 |
| + Kirby's hat.kind read (below) | **377 / 377** |

**3.19's online core is not the drift fix.** 3.19 replays carry no standalone Polling Drift Fix (C2 0x801A4DA0). The
online core has a C2 at 0x801A4DB4 and an 04 at 0x80019860 instead. It is part of the online engine loop and spins on
flags in the core's own block (r13-0x49E4 +0xDF/+0x90D) that the rest of the core sets. `replay_codes.py` had copied
it on the address alone, and playback then hung before the match (every file timed out with no records). The script
now copies the drift group only when the standalone C2 0x801A4DA0 is present, so all 377 Hero files play with
Slippi's own playback set. No skipped render was seen (`x2188`, written by the render pass, keeps a constant 1-frame
offset against the engine's logic-side value: Rust frame N equals playback frame N+1, a sampling offset, not a
divergence). `replay_codes.py` also reads a raw array whose length is 0 (the recorder never closed it) to the end of
the file.

**Replays without Game End.** Game_20260718T225826, Game_20261023T125724 and Game_20261023T130503 have no Game End
event (the recording stopped). Playback waits for the frame after the last one ("frame N not ready") until
run_dolphin's timeout. `raw2csv.py` now drops the incomplete last record, so the dump covers every frame but the
last. All three match strictly over those frames (2007 / 3658 / 3737).

**Engine fix: Kirby's copied ability at fp+0x2238.** `ftCo_800B77E8` (CPU attack exclusions, Kirby case) and
`ftKb_SpecialNMs_8010BC40/8010BC90` (Kirby's Marth/Roy neutral-B loop) read `fp->u.gw.x2238_panicCharge`, which on GC
is the word at fp+0x2238, Kirby's `hat.kind`. On 64-bit, `ftKb_Hat.x0` is an 8-byte pointer, so `hat.kind` moves to
u+0x10 while `gw.x2238` stays at u+0xC, and the port read Kirby's padding. Game_20260926T161418 (CPU Kirby): from
slippi 826 `cpu.xCC_array[0]` was 0x12 in Rust and 0x11 in playback (hat.kind 3/0xD/0x10, a chargeable neutral-B),
then s2's `cpu.command_duration` at 925 (csv 1048). Rust and C (TARGET_PC, `FTKB_HAT_KIND_2238`) now read
`u.kb.hat.kind`. slpcheck matched this file before the fix, because the recorded inputs drive the CPU; only the
strict sweep sees it.

**602a5cd8** (local, csv 5452, s6 `x2188.x` 300 vs 295) stays open: a 5-pixel gap is not the 1-frame sampling
offset above. Not re-run this round.

## Nondeterminism

Dolphin's playback is occasionally nondeterministic. Of four runs of 03b84423 with the same exe and the same replay,
one gave s6 `dmg.x1830_percent` 15.98 at slippi frame 7892. The recording's Post-Frame, Rust and the other three runs
all have 16.98. That run was made while other Dolphin and game processes were loading the machine; its offscreen-damage
counter (`dmg.x1910`) was one frame behind. A single cached `dolphin.csv` can therefore diverge from Rust for no fault
of the engine. `oracle.sh` now runs Dolphin twice, concurrently (`DOLPHIN_RUNS=1` for one run). When the runs differ it
logs `DOLPHIN NONDETERMINISTIC` and appends "the Dolphin runs disagree from frame F" to the ORACLE line. Before
classifying a divergence, check that it is not past that frame, and check the field against the .slp's Post-Frame
where one exists.

## UCF code semantics (round 4)

Two replay divergers were UCF 0.84 ports that read the Gecko code wrong, not engine bugs.

- **Replace, not add.** A C2 code can replace the instruction it hooks. UCF Tumble.asm at
  0x800908F4 swaps `cmpw r3, r0` (timer against x214) for `cmpwi r3, 1`, so its stick-out window
  is "timer 0, or timer 1 after a fast stick move". The port had OR-ed it onto vanilla's window
  (3417139d, frame 4689: DamageFlyTop to Fall). Check: `slpcheck --gecko` lists a replay's
  injection sites, and the hooked instruction is in `ppcdis.py ADDR LEN` on main.dol. Read the
  code body for the `nop` where the original instruction would go.
- **Slippi's UCF bundle carries more than the dashback and shield-drop fixes.** It also carries
  the DBOOC SquatRv fix (m-ex `ucf_dbooc_squatrv_fix`). slpcheck now sets
  `MELEE_MEX=ucf_dbooc_squatrv_fix` for a replay that has a UCF dashback port (325cee5d).
- **Finding the function.** slpcheck now opens calltrace frames per replay frame, so a
  `--features calltrace` build with `MELEE_CT_LIVE=1 MELEE_CT_LIVE_FROM=F` prints the entries
  from frame F onward.

## Slippi 1.7.1 replays (port/oracle171)

A Slippi 1.x replay has no Gecko list (Message Splitter 0x10 / command 0x3D came with 2.x/3.x). Modern playback
(`CEXISlippi::prepareGeckoList`) then sends its built-in `legacy_code_list` (SlippiPlayback.cpp): UCF **0.74**
toggle-checked dashback and shield drop, `ToggledStaticOverwrites`, the PS preload handlers and the PAL handlers. That
is modern Slippi's guess, not the 2019 console's set. Oracle exe `_build/dolphin-oracle-slp171` (strict2 + the change
below): `MELEE_DOLPHIN_LEGACY_CODES=<file>` makes `getLegacyCodelist` return that file instead (abort if unreadable;
stderr says `nhv1: legacy code list from ...`). `run_dolphin.ps1` builds the file per replay with
`legacy171_codes.py` (only for a version < 2 replay with no Gecko list; `DOLPHIN_LEGACY171=0` plays with the built-in
list, `DOLPHIN_LEGACY171_ARGS="--no-spawns"` / `"--qol"`).

**Where the codes come from.** project-slippi/slippi-ssbm-asm at **496f0ae** (2019-02-25, "increment version for PS
toggle", the 1.7.1 bump; `_build/slippi-src/ssbm-asm`), the console binaries the Slippi Nintendont build loaded,
`Output/Console/g_*.bin`, read with `git show` at run time (nothing of them is committed here). Gecko codes copied
verbatim, in file order:

| bin | used | why |
|---|---|---|
| g_core.bin | always, minus the 10 below | boot/rules/unlock writes, Disable Screenshots (nops at 0x801A4F84/0x801A5070, in the game loop), the three PS preload C2s + `04 804DEC18 01000000` (PSPreloadToggle on), ToggledStaticOverwrites C2 0x8016E750 |
| g_ucf.bin | yes | UCF 0.73 (C2 0x800C9A44 dashback, 0x800998A4 shield drop, 0x802662D0) + `04 804DEC08 01010101` (ControllerFixOptions all on). All 3,479 Gang-Steals files record UCF toggle words 1,1,1,1 (8 words), PAL 0, version 1.7.1.0: the "UCF" setting, not the per-player g_toggles.bin (the script refuses any other toggle pattern) |
| g_spawns.bin | yes (`--no-spawns` off) | 20XX Neutral Spawns, C2 0x80263058 and 0x801C0A48; the engine assumes it for every 1.x replay (gmvs.rs `legacy_neutral_spawn`) |
| g_pal.bin | only with the PAL byte | |
| g_qol.bin | only with `--qol` | Salty Runback, Disable FoD During Doubles, cosmetics; the replay does not say |

Left out of g_core: the Slippi Recording transport (the five `bl` hooks at 0x8016E74C, 0x8006B0DC, 0x8006C5D4,
0x801A5B14, 0x802FED3C and the 06 block at 0x8032ED94; they only send EXI data, and 0x8006B0DC / 0x8016E74C are the
sites of playback's own hooks), and four sites the playback set (Sys/GameSettings/GALE01r2.ini) already patches with
Slippi's own copy: Init Player Data C2 0x80068EEC and Init Stage Data C2 0x801C154C (same zeroing), Disable Special /
Trophy Messages 0x8015D94C / 0x8015D984 (identical words). Checked: g_ucf's two UCF bodies are the toggled
g_toggles bodies minus the toggle test (the shield drop's toggled copy reloads r5 = `lwz r5,-0x514C(r13)`), so with
every port on the two sets behave the same.

Known differences that remain between this oracle and the 2019 console (not codes, so not reproducible from the file):
- Polling drift: 1.7.1 had no Polling Drift Fix, so the console ran a second logic frame without a render about every
  1001 frames, at a phase set by how long the console had been running ("Polling drift" above). Playback in Dolphin
  drifts too, at its own phase. Both move the 1% offscreen tick (`x221F_b0` is set by the render pass).
- Modern playback's Init Stage Data also resets Dream Land's wind and Japes' logs (later determinism codes); a 2019
  console carried them over from the previous game.

**20XX Neutral Spawns needs the console's SSS stage id.** The code's stage half (C2 0x801C0A48, in
Ground_801C0800) patches the spawn joints of Battlefield, Dream Land, Stadium and Yoshi's by the stage id in
`gmVsMelee_SssData` (u16 0x80480686), which the stage select screen writes. Playback never runs the SSS (it restores
`gmVsMelee_StartData` only), so the first runs spawned everyone at (0, -2) and hung at an early game end (pc in the
playback wait loop, frame 1697 of 13/Game_20190309T101721). The CSS half's result (spawn bytes in the player blocks)
is in the recorded StartData already. `legacy171_codes.py` adds one 2-byte `06 80480686` write of the replay's own
stage id: console state the code reads, not a code.

**Validation (rust-port b05adf319 + this branch, fast profile).** 20 Gang-Steals files that match at .slp level
(Battlefield, FD, Dream Land, Yoshi's, Fountain, Stadium; Fox, Falco, Marth, Sheik, Peach, Puff, Falcon, ICs, Mario,
Doc): **20 / 20 NHV1-identical on every frame** (seed ignored), 2,243 to 14,536 frames each. One Dolphin run per
file; `DOLPHIN_PRIORITY=BelowNormal` (Idle starved while builds ran).

**Bucket files** (first differing word, slippi frame):

| file | stage, fighters | slpcheck (vs .slp) | oracle (vs console in Dolphin) | class |
|---|---|---|---|---|
| 12/Game_20190309T162629 | FD, Marth v Marth | 1922 P4 percent 143.3 vs 142.3 (offscreen tick) | **match**, 15,481 frames | recording console only: Dolphin = engine. Polling-drift class (no drift fix in 1.7.1) |
| 13/Game_20190309T123307 | Yoshi's, Falco v Fox | 3960 P1 x | **match** | recording console only |
| 23/Game_20190309T125751 | BF, Fox v Falco | 5562 P1 x | **match** | recording console only |
| 21/Game_20190309T173910 | stage 5, Puff v Marth | -123 P4 x 1e-3 | **match** | recording console only |
| 14/Game_20190309T111022 | Yoshi's, ICs v Fox | match | **match**, 11,430 | - |
| 12/Game_20190309T094028 | Stadium, Marth v Falcon | match | **match**, 7,374 | - |
| 14/Game_20190309T134017 | BF teams, Peach/Sheik/Peach/Marth | 207 P2 percent 0 vs 14 | 206 s2 (P2 Sheik): hit by s0 (Peach, motion 64) lands on the console, not in Rust (motion 42 vs 80); s0 `x2144` 0 vs 13019 at 204 | engine: doubles hit |
| 20/Game_20190309T131733 | BF teams, Fox/Fox/Falco/Sheik | 1030 P2 percent | 1029 s2 (P2 Fox) damaged on the console (motion 14 vs 87); s4 (P3 Falco, motion 60) hitbox `x914[0..1].hurt_coll_pos` differs at 1028 | engine: doubles hit |
| 16/Game_20190309T123832 | DL teams, Marth/ICs/Marth/Fox | 368 P1 x | 367 s0 (P1 Marth) damaged on the console (motion 60 vs 83), 13.95 % vs 10 % | engine: doubles hit |
| stream/Game_20190309T162146 | BF teams, Falco/Marth/Peach/Fox | 517 P2 x | 516 s4 (P3 Peach, motion 341) damaged on the console (vs 90); s2 (P2 Marth, motion 56) hitbox collides elsewhere at 515, `x195c_hitlag` 0 vs 7 | engine: doubles hit |
| 19/Game_20190309T152343 | stage 15, 4 players | -123 P1 x 38.42 vs -77.07 | -123 s0 cur_pos: Dolphin -77.07 = the .slp | engine: 4-player spawn off the neutral-spawn table |
| 11/Game_20190308T115404 | Stadium, ICs v Peach | match | 2773 s3 (Nana) `cpu.x30` 15 vs 24 only | Nana CPU scratch (Randf draw count), Stadium |
| 13/Game_20190309T122423 | Stadium, ICs v Marth | match | 5327 s1 (Nana) `cpu.x30` 14 vs 7 only | same |

"Recording console only": Dolphin playing the replay with the 1.7.1 codes agrees with the engine on every NHV1 word, and
both differ from the recorded Post-Frame. The 2019 console's state differs in something neither replays (render
skips from polling drift, state carried over from the previous game). Not engine bugs by this oracle.
All four doubles divergers are teams games (`is_teams` 1) where a hit lands on the console and not in Rust (the
attacker's hitbox/hurtbox collision point and `x2144` differ one or two frames earlier). Plain engine bug candidate
(no 2019 code touches hit detection); a modern doubles replay should show it too.

**Engine sweep at b05adf319** (`slpcheck`, stop at the first divergence): 3,162 / 3,479 match. Split by player count:
singles **116 / 3,276** diverge, 4-player (all teams) **201 / 203** diverge. The doubles class above is the bulk of what
is left. Diverging files per character (files containing the character), singles | 4-player: Fox 60/1245 | 146/146,
Falco 32/1067 | 101/102, Marth 25/1135 | 99/100, Sheik 20/552 | 87/87, Peach 14/305 | 64/64, Falcon 12/645 | 47/47,
ICs 12/308 | 39/41, Puff 18/358 | 25/25, Luigi 7/122 | -, Ganon 6/106 | 14/14, Samus 5/50 | 17/17, Doc 3/30 | 12/12,
Pikachu 3/30 | 8/8, G&W 1/40 | 8/9, YLink 0/27 | 20/20, Ness 0/5 | 14/14, Mario 0/40 | 4/4, Zelda 0/28 | 2/2,
Bowser 0/6 | 1/1; DK, Link, Mewtwo, Roy, Yoshi 0. Singles first fields: x 56, percent 41, facing ~15, action ~10.

## Akaneia (m-ex) replays (port/akoracle)

Hero's 2026-11-06 corpus has 50 replays recorded on Akaneia with one of its m-ex fighters (external id > 25: 26-32,
Charizard, Wolf, Sonic, Tails, Diddy, Dedede...). The oracle plays them on the Akaneia disc, the same one slpcheck uses.

**Which disc.** slpcheck (melee/src/slpcheck.rs) switches to `GW_ISO_AKANEIA` (workspace `.env`) when no `--iso` is given,
the Gecko list has Akaneia's codes.gct marker (C2 0x803D7060, `melee_data::slp::akaneia_codeset`) and a human/CPU player
has an external id > 25; an Akaneia replay with only vanilla fighters stays on the vanilla disc. `replay_codes.py --disc
REPLAY` applies the same rule and prints `akaneia` or `vanilla`; `run_dolphin.ps1` and `oracle.sh` use it for both sides
(`DOLPHIN_DISC=vanilla|akaneia` overrides). oracle.sh passes the chosen disc to slpcheck explicitly.

**Is it the recording build?** Checked from the disc and the replays (nothing written): the Akaneia disc is GALE01 with
the retail section layout; its main.dol differs from vanilla in 2994 text words, none at the oracle's hook addresses
(0x801A4FA4, 0x8006B0DC, 0x8006B120) or the replay-code sites. The disc ships its own codes.gct (1163 codes) that m-ex
loads at boot, so playback runs Akaneia's codes from the disc without help; the replay's Gecko list (287 codes in the
April files, 291 from May on) holds 86 of them byte-identical, the 0x803D7060 marker among them. Codes at 25 addresses
exist in both with different bodies (Slippi's own versions of the same hooks, e.g. 0x8016E510, 0x80394A74); the
engine's 16 exact files on this disc are the evidence it is the recording build.

**Build.** `_build/dolphin-oracle-akaneia`: a copy of the strict2 source (same patch as
`slippi-dolphin-nhv1.patch`, plus `MELEE_DOLPHIN_NHV1_WATCHDOG=boot`: the PC/scene watchdog from process start), built fresh by its `build.bat` (`-j 4`), `DOLPHIN_ORACLE_DIR` selects it.
The NHV1 word set is unchanged: the m-ex fighters use the retail `struct Fighter` layout at the same offsets.

**Akaneia or ACE?** ACE (Akaneia Community Edition, `GW_ISO_ACE`) has the same main.dol (same hash) and the same
fighter order in MxDt.dat for the first 34 fighter files, so ids 26-32 are the same fighters. The two discs still run
different code: codes.gct shares 1144 codes, 6 are Akaneia-only and 12 ACE-only (ACE adds C2 hooks at 0x800857E0 and
0x800A1CF8 in fighter code, plus menu/CSS hooks). PlCo.dat, MxDt.dat, MxPt/MxSr and several Ef*Data.dat files also
differ. None of the 18 codes that tell the discs apart is in any of the 50 replays' Gecko lists: the replays carry
Slippi's list, not the disc's codes.gct. slpcheck gives line-identical results on both discs for all 50 (15 match, 35
diverge at the same frames). So the replays do not say which disc recorded them. Playing them all on ACE is **not**
shown valid (the code differs). Akaneia stays the default (it is slpcheck's), and `DOLPHIN_DISC=ace` plays one file on
ACE as a tie-break: whichever disc's console matches the recorded Post-Frame is the recording disc.

**Hero's "drift" codes are not the drift fix.** Hero's 2026 replays carry a 26-line C2 at 0x801A4DB4 and a 04 at
0x80019860, but no C2 at 0x801A4DA0. That is not Slippi's Polling Drift Fix. Copied into playback (as replay_codes.py
did by address), it kept the game from ever requesting a replay frame (boot watchdog: main loop, no NoteFrame), on the
vanilla disc too. replay_codes.py now copies the drift-site codes only when the C2 at 0x801A4DA0 is in the list. So
these replays play without any drift code. The 60 Hz pacing skip of "Polling drift" can then move a 1% offscreen tick.
None of the runs below shows it.

### Results (port/akoracle on rust-port ade7c4862, `_build/dolphin-oracle-akaneia`, one Dolphin run per file)

The 15 slpcheck-exact Akaneia files: **11 strict-equal** (all NHV1 words, seed ignored). The other 4 (224457, 000702,
232021, 225437) all have external id 28 (Charizard; Akaneia's ids 26-32 are Wolf, Diddy, Charizard, Lucas, Sonic,
Dedede, Tails, from MxDt's file order Wf Dd Lz Lc Sn De Ts). Each diverges only in `itemsum`, the first time that
fighter's m-ex item kind 0x106 exists: `hold_kind` is 5 in Rust and 8 on the console, then its position and item
collision follow. slpcheck cannot see items, so this is a real engine difference in the m-ex item's setup, not an
oracle artifact.

Akaneia vs ACE: two Wolf files (225652, 232036) played on both discs give byte-identical `dolphin.csv` (seed
included), and both match Rust. For these games the two discs behave the same. The question of which disc recorded
them is still open, but nothing measured depends on it.

First NHV1 difference on diverging files (all are the same class: a large ECB difference with a 1-ULP x):

| file | first diff (slippi) | fields |
|---|---|---|
| 20260815T232950 (Dedede, s0) | -8 | `coll_data.cur_pos.x` c227d3d8 vs c227d3d7; `ecb`/`desired_ecb` top.y 21.03 vs 14.09; `prev_ecb` from another pose |
| 20260411T225454 (Wolf game, s2) | 7719 | `cur_pos.x`/`last_pos.x`/`prev_pos.x` 1 ULP; `prev_ecb` bottom.y 0.49 vs 1.60, top.y 12.01 vs 8.41 |
| 20260612T230702 (Tails, s0) | 476 | `cur_pos.x` 1 ULP; `prev_pos` -4.031,4.777 vs -4.158,6.147; `prev_ecb`, `xE4_ecb` |

On each file the frame before shows only `x2188` (screen position, render side). On the divergent frame `prev_pos`
and `prev_ecb` differ by far more than the final x does. The console's `prev_pos` is the step's start point, the
engine's a mid-move point. This fits the lead that the engine splits the move into two mpColl_80043754 steps where the
console takes one (an ECB, i.e. bone-matrix, computed from another pose).

### Round 7 (port/fak7): what the step-split class was

The three "step split" files had three different causes, none of them in mpColl; the oracle's bone dump
(`MELEE_DOLPHIN_NHV1_BONES` / `MELEE_WIDE_BONES`) and a `MELEE_DOLPHIN_NHV1_PROBE_AT=8000B1CC` probe (every `lb_8000B1CC`
read: r3 = the JObj, `PROBE_MEM=r3+0x0:0x80` its flags, SRT and matrix) told them apart:

- **Dedede 232950 (slippi -8).** Local SRT and frame-end matrices equal on both sides; only the sixth ECB joint differs:
  the console reads `parts[5]` (translate 0, 2.96, -0.02), the engine `parts[46]` (the hammer tip, 0, 17.35, 0), which is
  what PlDe.dat's ftData x44 table says (32 39 16 63 57 **46**). Dedede's guest `OnActionStateChange` stores
  `fp->coll_data.ecb_source.joint[5] = motion_id == 69 ? parts[46] : parts[5]` on every state change (the two JObj
  handles its onLoad keeps at fp+0x223C/0x2240). The engine never dispatched m-ex slot 24 (onActionStateChange) nor 25
  (onReapplyAttr): `SLOT_EVENTS` had no row and the seam ran the clone base's `ftData_UnkMotionStates4` entry only. Both
  now go through `Mex_GObjDispatch` (platform events 11/12, the C port's numbering). The same file then diverged at 809:
  Dedede's MoveLogic rows 0-3 (the multi-jump states 341-344) keep retail `ftCo_JumpAerialF1_{Anim,IASA,Phys,Coll}`, and
  `gen_mex_bridge.py`'s `gobj_fn` only accepted a literal `*mut HSD_GObj` parameter, not the TUs' `G` alias, so the rows
  got `None` callbacks (no gravity or drift on the jump frame: self_vel 2.2/0.5 vs 2.11/0.53). The generator takes `G`
  now (7249 gobj callbacks, was 5204), and the install log names every row callback the bridge cannot wire and every
  override the engine does not dispatch (`not dispatched: ...`). Next divergence of the file: 1207.
- **Tails 230702 (slippi 476).** A thrown-victim pose: while Falcon throws Tails (Falcon motion 221, Tails 241) the
  console animates Tails' bones (bone[5] rotate -0.61, 0.61, 0.27 ...) and the engine leaves them at bind pose (0, 0, ...).
  The ECB is locked during the throw, so nothing shows until the release frame reads the pose for the first ECB of the
  step (xE4_ecb -2.30/8.22/12.70 vs -2.0/7.35/10.63). Open: which animation m-ex plays on a thrown custom fighter.
- **Wolf game 225454 (slippi 7719, s2).** The diverging fighter is the retail Falcon (p2), not Wolf. Open.

Items (`hold_kind` 5 vs 8 on the four Charizard files): the disc's codes.gct has a C2 at `Item_802674AC`'s entry
(`lwz r0,8(r3); cmpwi r0,0xED; blt; li r0,8; stw r0,0xC(r3); blr`): every item kind >= 237 is hold kind 8, the retail
ladder skipped. `Item_802674AC` has the rung now; on 224457 the item's `hold_kind` is 8 on both sides at its first frame
(slippi 4239) and the file's first strict difference is now the item's own collision step on that frame
(`x378_itemColl.cur_pos` -17.93/73.28 vs -20.62/70.24, the console's `last_pos` equal to the engine's `cur_pos`: the
console's item has run one collision step more on its spawn frame; `xB54.unk_x` -1.34 vs -2.68). Open: the proc order
of a gobj the guest spawns mid-frame.

Numbers (port/fak7 on rust-port b05adf319 + port/akoracle): Akaneia files 16 -> 17 of 51 at .slp level (231850 Dedede
matches), no crash; the other five Dedede files move far later (223810 65 -> 138, 230601 169 -> 1098, 233221 437 -> 2122,
233518 81 -> 1882, 232950 -8 -> 1207). Hero 391 -> 392 of 428, local 125/125, own 917/917 (no row changed).
