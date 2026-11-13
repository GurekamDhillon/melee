# Pose dump (`melee pose-dump`), schema `pose-dump/2`

A headless dump of a fighter's animation clips as per-frame joint poses, for animation tooling and as a future
training-data format. Code: `rust/melee/src/pose_dump.rs`. Cross-check: `tools/rust-port/posedump/check_pose.py`.

```
melee pose-dump --fighter falcon --all --out "E:/Projects/Melee Workspace/scratch/animloop/dump" [--raw]
melee pose-dump --fighter marth --row 52 --out DIR
melee pose-dump --fighter marth --symbol PlyMars5K_Share_ACTION_Wait1_figatree --out DIR
```
Fighter names are the scenario names (`melee_engine::scenario::ckind_by_name`). The disc comes from `--iso` or
`GW_ISO_VANILLA`. No window is opened. The output is disc-derived: write it outside the repo (the scratch folder has a
`.gitignore` of `*`). `--raw` also copies the fighter's `Pl??AJ.dat` and `Pl??Nr.dat` into `raw/` for the cross-check.

## Evaluator
The reference is the engine itself. The command boots a VS match on Final Destination (P1 the fighter, P2 a Fox or
Falco, idle pads), waits until P1 stands in Wait, then for each clip:
1. `ScriptGame_LabSetPreviewRow(0, row, +1)` and `facing_dir = +1`;
2. `Fighter_ChangeMotionState(Wait, flags 0, start 0, rate 1, blend -1)`: the Lab preview hook swaps the Wait row's
   animation for `row`'s FigaTree; Wait's flag word, script and bone settings stay. So no subaction script runs (no
   hitboxes, effects, hand swaps of the clip itself), and root-motion extraction happens only if Wait's own flags set it;
2b. blend is passed as -1 (none): 0 would take the row's default blend and mix in the previous pose for the first frames;
3. frame k (k = 0 .. ceil(frames)-1): frame 0 is read right after the change, frame k>0 after k `Engine::tick`s;
   before reading, `ScriptGame_JointsSetup` builds the world matrices, and the world matrix written is exactly
   `ScriptGame_JointMtx(0, joint, 0..11)`. The dump stops early if the fighter leaves the preview (it is recorded in
   `frames_dumped`).

What the engine adds that is NOT part of the clip (a consumer removes it):
- joint 0 (TopN) local SRT is the fighter root: translation = the fighter position, rotation Y = +pi/2 (facing +1),
  scale = the model scale. Clip tracks on joint 0 (if any) are overwritten by the game. Use joint-0-relative world
  matrices (`inverse(W[0]) * W[i]`) for a clip-only pose.
- root-motion extraction (`x594_b0`): when the per-frame `extract` flag is 1, the engine zeroed TransN's translation
  after sampling it; the sampled value times the model scale is in the frame's `transN_pos`, its per-frame delta in
  `transN_offset` (`Fighter.x68C_transNPos` / `x6A4_transNOffset`). Raw TransN translation = `transN_pos / scale`.
  In the preview path the flag is Wait's (`index.json` `wait_row_flags`); the clip's own flag is `anim_vel` per clip,
  i.e. what gameplay would do with it.
- the Wait script, hand states and IK are those of Wait; blending is off.

## Layout
```
<out>/<fighter>/index.json      fighter metadata, joints, rest pose, one entry per motion row
<out>/<fighter>/census.json     track type / op / format / byte-order census over the dumped clips
<out>/<fighter>/clips/NNN_<symbol>.pose   one binary file per distinct clip (NNN = first row that plays it)
<out>/<fighter>/raw/            (--raw only) the AJ and costume DATs
```

### index.json (hand-written JSON, numbers in f32 shortest-round-trip form, non-finite as null)
- `schema`: `"pose-dump/2"`. Any change to a field's meaning or the binary layout bumps it.
- `fighter`, `char_kind` (CharacterKind), `fighter_kind` (FighterKind), `evaluator`: `"engine-lab-preview"`,
  `facing`: 1, `wait_row_flags`: Wait's flag word, `rows`: number of motion rows, `joint_count`.
- `joints[]`: `index`, `name`, `parent` (index, -1 for the root), `flags` (the live JObj flags), `anim_applied`
  (false: the game never applies clip tracks to this joint), `part` (raw
  `FighterPartsTable.joint_to_part` value), `rest`: the costume `HSD_Joint` `flags`, `scale`, `rotation` (Euler
  radians), `translation`, `inverse_bind` (the 3x4 `mtx`, row-major, or null). Joint order is the depth-first order of
  `Fighter.parts` (child before sibling).
- Joint naming rule: `name` is the common part table's name for part p (`TopN`, `TransN`, `XRotN`, `YRotN`, `HipN`,
  `WaistN`, ..., `WaistB` (16), ..., `LHaveN` (31), ..., `RHaveN` (49), ..., `ThrowN`, `ExtraN`; part ids 0..53,
  `PART_NAMES` in pose_dump.rs) when p = `joint_to_part[i]` is in 0..53 (and below the table's `parts_num`) AND
  `part_to_joint[p] == i`; otherwise `JOBJ_<i>` with i the decimal joint index. Exported files use the same names.
  The decomp's `Fighter_Part` enum (`FtPart_*` in melee_types) has the right numeric ids but shifted NAMES from 16 on (it
  lacks WaistB, LHaveN, RHaveN and has LHandNb, RHandNb, TransN2): do not name joints from it.
- `clips[]`, one per motion row (the `ftData` animation table, `Fighter.x24`):
  `row`, `symbol` (FigaTree public name or null), `aj_offset`, `aj_size` (the row's place in Pl??AJ.dat), `flags`
  (`x10_animCurrFlags` as the big-endian word's value), `loop` (bit 30), `anim_vel` (bit 31, root motion),
  `has_script`; then exactly one of:
  - `file`, `tree_type`, `tree_flags`, `frames` (`FigaTree.frames`), `frames_dumped`, `anim_joints_beyond_skeleton`
    (0 normally), `tracks[joint][]` = `{type, value_format, slope_format, start (s16), bytes}`;
  - `alias_of`: the row whose clip (same AJ offset and size) was dumped;
  - `skipped`: the reason (no animation data, did not bind, panic message).
- `summary`: rows selected, dumped, aliases, skipped.

### .pose (little endian)
```
header  "MPD1"  u32 version=1  u32 joints J  u32 frames F  u32 words_per_joint=31
frame   u32 k  f32 cur_anim_frame  u32 extract  f32 transN_pos[3]  f32 transN_offset[3]
        J x { u32 track_mask
              f32 raw_scale[3]  raw_rotation[3]  raw_translation[3]
              f32 eng_scale[3]  eng_rotation[3]  eng_translation[3]
              f32 world[12] }
```
- `raw_*`: the clip-only pose. Each channel is the value the joint's own FObj track produced this frame (read from the
  game's live FObj on a private copy, `melee_runtime::pose_capture::track_values`), before anything else wrote the
  joint; a channel no track drives holds the costume rest value. `track_mask` bit c (c = 0..8 in the order scale XYZ,
  rotation XYZ, translation XYZ) is set where a track supplied the value. Rotation is Euler XYZ in radians.
- `eng_*`: the JObj's local SRT as the game left it after the step (TopN = fighter root; TransN zeroed under extraction;
  dynamics bones simulated; per-joint scale applied).
- `world`: the engine's world matrix (3x4 row-major), exactly `ScriptGame_JointMtx`. It is built from `eng_*`.
- `cur_anim_frame`: the engine's frame counter after the step. Frame k samples clip time k.
- The HSD local matrix is T * Rz * Ry * Rx * S, with the classical-scale / parent-scale compensation of
  `HSD_JObjSetupMatrix`.

To reproduce the engine view from the raw view, apply in order: the TopN root (`eng` of joint 0), extraction (when
`extract` = 1, TransN translation = 0), and the engine-rewritten joints (any joint whose `eng` differs from `raw`;
`anim_applied: false` joints never take their tracks). The raw view is `raw_*` composed with joint 0 at its rest
value.

### Engine effects seen on Falcon and Marth (2026-11-12)
- Joint 0 (TopN): position, facing turn (+pi/2 about Y) and model scale (Falcon 0.97).
- Joints with `anim_applied: false` (FighterBone flags b0 / b5, so `ftAnim_8006E7B8` skips `HSD_JObjAnim` on them):
  Marth has 12 (cape bones 45-58, driven by dynamics). The clips do carry tracks for them; the game ignores them.
- A per-joint scale on one item-hold joint, from engine code rather than the clip: Falcon RHaveN (57) about 1.031,
  Marth LHaveN (40) about 0.87.
- Extraction never happens in the preview path for these two (Wait's flag word is 2, so bit 31 is clear).

### Track types and FObj streams (for readers of `tracks`)
Types: 1-3 rotation X/Y/Z, 4 path, 5-7 translation X/Y/Z, 8-10 scale X/Y/Z, others visibility/branch. A stream is
`header (op | (count-1)<<4 | 0x80 more)`, optional varint count extension, then per key value, slope (SPL/SLP), and
wait varint. Value formats: `frac >> 5` = 0 f32, 1 s16, 2 u16, 3 s8, 4 u8, scaled by `1 / 2^(frac & 31)`.
Multi-byte numbers are LITTLE endian in the disc data (while archive headers are big endian); see census.json.

### census.json
`track_types`, `ops` (1 CON, 2 LIN, 3 SPL0, 4 SPL, 5 SLP, 6 KEY), `value_formats` / `slope_formats` (`kind/shiftN`) as
counts over the dumped clips' tracks; `rest_quaternion_joints`; `byte_order`: each track with start 0 has its first value
decoded both ways and compared with the raw frame-0 value (`match_little_endian` / `match_big_endian`), with one hex example.

## Verification
`python -I tools/rust-port/posedump/check_pose.py DUMP/<fighter> <geno_workbench dir> [--clips N] [--frames ...]` (a dump
made with `--raw`). It compares the raw local SRT with the external decoder's channel values, and the world matrices
relative to TopN with `figatree_import.world_matrices` for joints the engine leaves alone. 2026-11-12, 40 clips x 9
frames per fighter: Falcon raw max 8.1e-6, world max 2.9e-4; Marth raw max 1.8e-6, world max 3.9e-4. These are f32
against f64.

## Stability
History: `pose-dump/1` (2026-11-12) named joints from the decomp `Fighter_Part` enum, wrong from part 16 on.
`pose-dump/2` (2026-11-13) changes only joint names (the 54-part common table, `ExtraN` for part 53); the .pose binary is
unchanged and keeps version 1.

`pose-dump/2` is frozen: new information goes into new JSON keys (readers ignore unknown keys). A change to the .pose
layout or to the meaning of a field bumps the schema to `pose-dump/3` (and the .pose version when the binary changes).
