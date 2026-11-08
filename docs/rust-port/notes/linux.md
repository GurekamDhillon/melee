# Linux build (2026-11-07, branch port/linux)

Status (2026-11-07, after merging rust-port 4bc899145): the native engine builds and runs headless on Linux (WSL2,
Debian 13). Hero's 428-replay corpus gives the same slpcheck CSV on Linux and Windows (0 rows differ: 365 match, 63
diverged on both). WSL<->Windows engine netplay through the mock matchmaker: 1168 checksums compared, 0 desyncs.
See also tools/rust-port/slippi/REAL_NAT_TEST.md for real-network NAT tests.

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

## Determinism: the Corneria Arwing laser (fixed, 6f9ffb56b)
Game_20260613T224702 matched on Windows but diverged on Linux at frame 797: a live Arwing laser (item 234, life
timer 399) was destroyed at frame 796 with destroy_type 0, meaning its `animated` callback had returned true.
The C declares itArwinglaser_UnkMotion2/3_Anim as `void` and casts them into the bool `animated` slot. On the
GameCube, the trailing it_80273130 call (has the life timer run out?) leaves its result in r3. The Rust port called
the void function through the bool pointer (`pred_v`, a transmute), which is undefined behaviour: the predicate
read whatever was in the return register. Windows' code happened to leave the right value there and Linux's did
not; a calltrace build changed the code again, which is why it matched. The callbacks now return
`it_80273130(gobj) != 0` and `pred_v` has been removed. The other void-to-function-pointer transmutes in the tree
cast void to void and are safe.

## Netplay, WSL <-> Windows (done)
On Windows, run `python -I tools/rust-port/slippi/mock_mm.py --bind <WSL switch address, e.g. 172.21.48.1>
--port 43110`. Start the Windows engine first (`slippi-netplay --mm ws://172.21.48.1:43110/v1/queue --mm-lan
--port 43211 --code TSTA#001 --search TSTB#001 --frames 3600`), then the WSL engine with the same --mm and
--mm-lan, its own --port, and the codes swapped. The relay runs on Windows. UDP works in both directions over the
vSwitch with Windows' default firewall. Result: 1168 checksums compared with the peer, 0 soft and 0 hard desyncs,
51 rollbacks (max depth 7), ping 33-55 ms, and both checksum CSVs are equal (1170 rows). The match ended before
3600 frames.
Why (round 10): it is a normal game end, not a fault. Without `--padgen` both engines play `pads::pad_fixed`, whose
port-1 cycle (284 frames) sends Falcon off the stage once per cycle; four stocks are gone at online frame 1179 (Windows
loopback reproduces it: "the match ended at online frame 1179 (stocks left: p1 0, p2 4)", now printed by
slippi-netplay). `--padgen 7` lasts longer (1995) for the same reason. For a 3600-frame run use inputs that keep the
fighters alive, or more stocks.

## Known gaps
- No GC adapter or Linux pad support yet.
- The windowed run (WSLg is present) was not done.
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
