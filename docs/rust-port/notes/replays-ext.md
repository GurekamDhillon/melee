# External tournament replays (Slippi .slp) vs the engine's replay check

2026-11-01. No replay data is in this note or in git. The files live in `scratch/replays-ext/` (workspace root,
untracked) with `manifest.txt` (one line per file: local name, source URL, event page). Treat them as untrusted data.

## Harvest
- Source: Ausmash result pages carry per-set Slippi links (`ausmashstorage.blob.core.windows.net/ausmash-content/<guid>.slp`,
  plus a per-set .zip and a "download all replays" link). No event called "New Blood" was found in the Ausmash Melee
  results listing, so recent public events were used instead.
- Events: 20689 (Wellington Ranbats 14/06/2026, all 35 .slp) and 20476 (Second Wind 2026, first 90 of 270 .slp).
  Event pages with no replay links (Bliss Weekly 90, Project Pkaru 9) were skipped.
- Result: 125 files, 446 MB (cap was 5 GB). All are Slippi 3.17.0 files, 2 or more minutes of frames each.

## Survey (what the engine can replay)
`melee golden --slp` accepts only Falcon / Fox / Marth / Sheik on FD (ext 32) or Battlefield (ext 31), two humans.
Of 125 files: 20 playable, 105 refused up front by `slp_open` with "not playable in the Rust sim yet"
(other stages, e.g. ext 8/3/28/2, or other characters, e.g. ck:20 Falco-class not yet wired).
Stage counts over all 125 files: ext31 33, ext8 25, ext32 21, ext3 18, ext28 18, ext2 10.

## Rust vs C playback
Sample of 5 playable files (3 Marth/Sheik/Fox mixes on FD and BF), both sides headless and unfocused:
- Rust: `melee golden --iso $GW_ISO_VANILLA --slp F --out rs.csv` (default 1802 rows for a non-cpu run).
- C: `MELEE_SLP=F MELEE_SLP_RESYNC=1 MELEE_NEUTRAL_HASHLOG=c.csv tools/port/run.sh --max-seconds N <sandbox>` with the
  consol-r8 C build, MELEE_TURBO=1, off-screen window.
- Compare: `compare_golden.py c.csv rs.csv --allow-short --ignore-seed`.
- Result: 0 of 5 match. Every sample differs from frame 0 (fighter words s0 and the second player's words), so this is
  a setup difference, not an in-match drift. One of 6 attempted files (a Marth mirror on BF) panicked in the Rust
  engine: `Fighter_SmashChargeShakeTable not loaded` (melee_fighter/src/hooks.rs:536) at row ~600.
- Control: `tools/rust-port/golden/check_slp.sh` (C-recorded Falcon v Fox replay and its .gdlp) is identical over
  1801 / 1802 frames in the same session, so the harness itself is sound.

Likely causes (not yet verified, nothing changed in code):
1. The Rust `golden --slp` builds its scene as `...match=time;minutes=0;items=off` with default costumes and rules; the
   C player applies the match from the file (costume `cN`, ports, stocks, timer). Frame-0 words include stocks,
   percent and position, so any rules/spawn mismatch shows at row 0.
2. Port handling: external files use ports like p1/p4 or p2/p3; the Rust path maps the two fighters to slots 0 and 1
   in order, while C keeps the file's player indices, so the hash slot columns (s0..s11) land in different places.
   Only p1+p2 files avoid this, and the sampled p1/p2 file still diverged at frame 0, so (1) matters too.
3. The seed column is skipped (replays reseed every frame), as in check_slp.sh.

## Caveats of Slippi's build versus vanilla
These replays come from Slippi Dolphin netplay/tournament builds, not a vanilla 1.02 disc:
- UCF (UnclePunch Community Fix) / 20XX-style Gecko patches: UCF dashback and shield-drop fixes change fighter input
  handling, so a vanilla engine given the same inputs can legitimately diverge after a dashback or shield drop. Slippi
  files record the toggle only indirectly (via the game-start block's dashback/shield-drop fields), so engine-vs-file
  divergence cannot be told apart from a port bug without checking those fields.
- Slippi's Gecko codes (frozen stages, PAL toggles, stage-select and Pokemon Stadium transforms, gameplay-affecting
  tournament codes) and the stage/fighter coverage of real tournaments (many stages and 26 characters) exceed what
  the engine plays, so full-set parity is limited by coverage first.
- Replays record post-frame inputs and a per-frame RNG seed; with resync the seed is forced each frame, so RNG-driven
  divergences are masked, not detected.
- Slippi inputs are the post-controller-adjustment values (UCF/box-style analog handling applied before recording);
  the C and Rust players both feed them raw, which is the right thing.

## Next
- Make `slp_open` / the scene string carry costume, ports, stocks and timer from the file, keep player indices,
  then re-run the 20 playable files; start with one p1/p2 file and bisect at frame 0 via a field diff.
- Check the game-start block for UCF flags before treating a later divergence as a port bug.
- Reproduce: the harvest is a plain curl loop over the `.slp` links on the Ausmash event page HTML; files under
  `scratch/replays-ext/` are named `<event>_<guid>.slp`.
