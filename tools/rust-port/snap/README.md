# State snapshots: resume the Rust golden sim from the C build's state

Start `melee golden` from the C build's exact gameplay state at any logged match frame, instead of
from frame 0, and compare the neutral hash from that frame on. Snapshots are disc-derived
(all of MEM1): keep them under the build root, never commit them.

## Dump (C side)

`MELEE_SNAP_DUMP=<frame>[,<frame>...]:<file>` (`%d` in the file name = the frame) makes the C build
write the state after that match frame's logic, at the point `MELEE_NEUTRAL_HASHLOG` hashes (so
snapshot N pairs with golden row N). Code: `gw_snap_state_dump` / `gw_snap_dump_hook` in
`pc/platform/gw_snap.c`, hooked from `gw_script.c`; it reuses the rollback snapshot's range table
(every non-zero MEM1 page plus the game's section-3 globals, found by map name). Format: comment above
`gw_snap_state_dump`. Everything after the header is the game's big-endian guest memory.

```
export GW_MELEE=<this worktree> GW_BUILD_ROOT=<private build root>
bash tools/port/build.sh --shim gw_snap.c --shim gw_script.c       # once
bash tools/rust-port/snap/dump.sh pad_falcon_fox_fd 0,300           # -> $GW_BUILD_ROOT/snaps/pad_falcon_fox_fd.f{0,300}.snap
```

`dump.sh` runs the same scene/pad script/seed as `golden/record.sh`; its hash CSV is identical to the
golden (the dump does not disturb the run).

## Resume (Rust side)

```
melee golden --iso DISC --out CAND.csv --from-snapshot FILE.snap [--start N] [--dump-state N:FILE]
python tools/rust-port/compare_golden.py docs/rust-port/golden/pad_falcon_fox_fd.csv CAND.csv --allow-short --from N
```

`VsMatch::from_snapshot` (rust/melee/src/vs_match/snapshot*.rs) builds the match with `VsMatch::new`
(disc files, stage, banks, ECB tables: everything a frame does not change), then overlays the dynamic
state read through the GameCube offset tables of `melee_types` (`melee_core::gcdecode::GcDecoder::new_raw`:
pointers stay guest addresses, bitfield bytes stay in GC order, fields are read at their GC offsets):

| sim state | C source |
|---|---|
| frame, RNG seed, pads | `_seed`, `HSD_PadGameStatus[4]` |
| each `Fighter` | `p_link 8` GObj user data, `Fighter` record decoded flat |
| motion vars, command script (DAT offsets from `ft_data`), shield/input flags | `fp->mv` by motion state, `x3E4_fighterCmdScript`, bitfields |
| JObj trees (`parts[i].joint` / `x4_jobj2`), animations (AObj/FObj state), part-anim slots | `fp->parts`, `x8B0` |
| stocks, respawn countdowns | `player_slots[i]`, `controller.state.fighters[i].x8` |
| stage particle tracks, grLast sway | map gobjs (`p_link 5`) JObj trees paired with the joint descriptors, `Ground.u.last` |
| HSD particle system | `hsd_804D78FC` generators, `hsd_804D0908` lists, AppSRTs, counters; command lists by `ptclref_804D0E5C` address |
| effects (`efLib`) | `p_link 11` GObjs; identity recovered by matching the live JObj tree and particle-track bytes to the loaded effect banks |
| items (Fox blaster gun and laser) | `p_link 9` |

`--dump-state N:FILE` writes `VsMatch::state_dump()` (canonical text of that state) after frame N, from a
from-0 run or right after a load, to diff the two (this is how each loader was checked).

Refused, with an error naming it: live hitboxes, a non-empty efAsync queue, particle callbacks/userdata,
generator user funcs, item kinds other than Fox's, effects that match no loaded descriptor.

## Proof (`prove.sh`)

`bash tools/rust-port/snap/prove.sh` (needs the dumps and a built `melee`): resumed runs must match the
golden at least as far as a from-0 run. On `pad_falcon_fox_fd` (dumps at 0 and 300, 2026-10-24):

```
from-0 run: first divergence at frame 555
resume from frame 0: first divergence at frame 555  [OK]
resume from frame 300: first divergence at frame 564  [OK]
```

(Frames N..divergence-1 are identical on every column, seed and hash included; both runs stop matching
for reasons unrelated to loading: the second KO / respawn is not ported yet.)

## Known model differences found while building it

- `VsMatch::new` leaves `smash_attrs.x2135` 0; the C has -1 from `Fighter_UnkInit`. With -1 the ported
  respawn probe (`ft_80082A68`, rebirth.rs) lands the first Rebirth 1 ulp off the golden. The loader keeps
  the model's 0 for a fighter with no falls (`MELEE_SNAP_EXACT_X2135=1` loads the C value).
- The stage particle track's AObj frame rate is a stray JObj pointer in the C build (`aobj->framerate`
  of map gobj 4's joint), so the track never advances; the sim models rate 1.0. Harmless before frame ~1000,
  and the loader copies the C bits.
- `respawn_busy` is never counted down in the sim (`fn_8016758C` not ported); the loader takes the C value.

To see WHICH fields differ from the C state at a frame, use `melee diff` (docs/rust-port/notes/diff.md); it reads these same dumps.
