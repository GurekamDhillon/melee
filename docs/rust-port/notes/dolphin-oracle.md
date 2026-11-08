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
human's CPU scratch, the `x2188` stick counter), `--persistent` all of them, `--zero-eq` treats f32 +0/-0 as equal.

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
