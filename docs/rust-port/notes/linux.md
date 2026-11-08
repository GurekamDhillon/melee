# Linux build (2026-11-07, branch port/linux)

Status: the native engine builds and runs headless on Linux (WSL2, Debian 13, rustc 1.99). The cross-OS netplay
test has not been run. One cross-OS sim difference is still open (see "Determinism").
See also tools/rust-port/slippi/REAL_NAT_TEST.md (being written on port/slippi7) for real-network NAT tests.

## Packages
`bash tools/rust-port/linux/deps.sh` (sudo) installs: build-essential pkg-config libasound2-dev libudev-dev libvulkan1
mesa-vulkan-drivers libx11-dev libxkbcommon-x11-0 libwayland-dev, plus Rust from rustup.
On a stock desktop only the ALSA (cpal) and libudev (gilrs) headers are missing. Without sudo, use
`NOSUDO=1 bash tools/rust-port/linux/deps.sh`: it unpacks the two -dev packages into ~/opt/linuxdeps and prints the
`PKG_CONFIG_PATH` to export.

## Build
```
cd rust
cargo build --release -p melee                  # build/cargo/release/melee
cargo build --release -p melee --features real-net
```
In WSL, building from /mnt/e works but is slow. Put CARGO_TARGET_DIR on the Linux filesystem and delete it afterwards.

## What was fixed for Linux (Windows code paths unchanged)
- `melee_engine::cow`: the copy-on-first-write arena capture had only a Windows implementation, so the crate did not
  compile. Added a unix twin using mprotect and a SIGSEGV handler, which hands faults outside the range back to the
  previous handler.
- `melee_engine::lowmem`: on unix the block used MAP_32BIT, which places it below 2 GiB, but guest MEM1 needs the
  0x80000000 bit. Every replay failed lbMemory's arena assert. It now uses the same 16 MiB hint probe as Windows from
  0x81000000, with MAP_FIXED_NOREPLACE.
- `melee_engine::arena`: the game arena allocator was Windows-only. On Linux every game heap object went to the
  system heap, where a rollback cannot restore it. It now works on unix too: an mmap reserve, mprotect to commit,
  madvise to trim. `static_sections` now returns the main program's writable PT_LOAD segments minus RELRO, so
  snapshots cover the statics on Linux.
- gate.sh, perf/snapguard.sh and slp/run_all.sh: they fall back to `melee` (no .exe suffix) and to `python3`.
- Netplay across machines: `melee slippi-netplay --mm ws://<private IP>:P/v1/queue --mm-lan` (Target::Lan accepts
  ws:// to private literal IPv4 addresses only, with sockets bound to every interface), plus `mock_mm.py --bind ADDR`
  (players are reached at the address their WebSocket came from). Not tested yet.

## Results (WSL, Linux build compared with a Windows build of the same commit)
- Goldens: all 4 scenarios match at their recorded counts (1801/1801/3601/3601). Synctest depth 4: 0 mismatches.
  Audio mix hash matches the C capture.
- slpcheck, local replay set: 123/125 match. The CSV is identical to Windows in every column (125 rows, 0 differences).
- slpcheck, Hero's corpus (428 files): 10 rows differ from Windows.
  - 7 crash on both OSes with the same Akaneia panic (`PlyTailsColor` has no type in dat_symbols.tsv). Only the abort
    exit code differs (127 on Windows, 134 on Linux).
  - 2 Akaneia files segfault on Windows but run to a divergence on Linux. This looks like a memory-safety bug rather
    than a determinism problem.
  - 1 vanilla file, Game_20260613T224702 (Marth vs Samus on Corneria), matches on Windows but diverges on Linux at
    frame 797 (percent). This accounts for 323 vs 322.
- Windows gate on this branch: all checks passed except snapguard, which ran while the machine was at 88% CPU.
  Re-run it on a quiet machine.

## Determinism: the open case
In Game_20260613T224702, an Arwing laser (item kind 234) is destroyed one frame early on Linux. The destroy comes
from an item GObj proc called directly from HSD_GObj_RunProcs.
- At frame 795 the item records are byte-identical on both OSes apart from pointers.
- A `--features calltrace` build on Linux matches Windows exactly: the call traces and CSVs are equal.
- So the difference depends on how the code is optimised, not on float maths in the inputs. Suspects: UB
  (pointer-to-u32 truncation, out-of-bounds read) or a blast-zone or item-proc comparison that LLVM compiles
  differently.
- Next step: print destroy_type and the blast-zone values at the destroy (Item_802696CC / Item_80269528 / Item_80269978)
  in a normal release build. A temporary print for this was written but not run.
- Until this is fixed, cross-OS netplay can desync on Corneria.

## Known gaps
- No GC adapter or Linux pad support yet.
- The windowed run (WSLg is present) and the cross-OS netplay run were not done.
- snapguard baselines are specific to the Windows machine.
- When a WSL session reads a Windows checkout, the files have CRLF line endings. Strip `\r` from the
  pre_draws.tsv fields, or the pad goldens fail at frame 0. A native Linux clone is not affected.

## Steps for the laptop agent
1. Clone, check out port/linux, and run `bash tools/rust-port/linux/deps.sh`. Set GW_ISO_VANILLA, and GW_ISO_AKANEIA
   for Akaneia replays. Never print or commit them.
2. `cd rust && cargo build --release -p melee --features real-net`.
3. Run `bash tools/rust-port/gate.sh` (snapguard will fail until there is a Linux baseline) and
   `bash tools/rust-port/slp/run_all.sh <replays> <out>`. Compare all.csv with a Windows run of the same commit.
4. Netplay against a Windows peer: on the host, run
   `python -I tools/rust-port/slippi/mock_mm.py --bind <host LAN IP> --port 43110`. On each engine, run
   `melee slippi-netplay --mm ws://<host LAN IP>:43110/v1/queue --mm-lan --port P --code A#1 --search B#1`. Compare
   the checksum CSVs. Set MELEE_VOLUME=3 and MELEE_WINDOW_TITLE.
