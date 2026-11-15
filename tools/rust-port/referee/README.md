# Gekko referee: probes run on Dolphin

Small bare-metal PowerPC programs that execute single Gekko instructions on chosen inputs inside Dolphin and print the
result bits. They settle "what does the console do" for instruction-level semantics, independent of the native C build
(x86) and of Rust/LLVM. No disc, no game data: the probe is our own code.

    bash tools/rust-port/referee/run.sh nanprobe [OUT]    # NaN propagation, default NaN, fnmsubs, frsp
    bash tools/rust-port/referee/run.sh intprobe [OUT]    # fctiwz, slw/srw/sraw, divw/divwu edge cases

Needs clang and ld.lld (PowerPC target), `dolphin-emu`, and `xvfb-run` (package `xorg-server-xvfb`). Dolphin only boots
headless under Xvfb; `QT_QPA_PLATFORM=offscreen` starts but never runs the game. `run.sh` uses batch mode (`-b -v Null`), a private
user folder with `Logger.ini` (file logging, OSREPORT / OSREPORT_HLE / BOOT channels) and `Dolphin.ini` (interpreter core
`CPUCore = 0`, no audio, no panic dialogs). It has a hard timeout because the probe never exits, and it kills its own Dolphin afterwards.
Dolphin ignores the first SIGTERM.

How a probe reports: it defines `OSReport`, and Dolphin's HLE intercepts it by symbol name and logs the formatted string. Keep
the stub's body opaque (`asm volatile` on the format pointer), or clang deletes every call. Keep format strings short with
integer arguments only; the nan probe fills a table first and prints `REF <row> <bits>` lines.

## Results (Dolphin 2609, interpreter, 2026-11-14) and the port

intprobe: fctiwz NaN -> 80000000, +-inf / out of range saturate, truncation toward zero; slw/srw/sraw use 6 bits of the
amount (32..63 shift everything out, 64 wraps); divw x/0 -> 0 or -1 by the dividend's sign, INT_MIN/-1 -> -1; divwu x/0 -> 0.
All as melee_core::ppc (notes/int-audit.md).

nanprobe rows (QP 7FC00000, QN FFC00000, QP1 7FC00001, SP 7F800001 signalling):

| rows | op | Gekko | port |
|---|---|---|---|
| 0-3 | fneg QP/QN/SP, fabs QN | ffc00000 7fc00000 ff800001 7fc00000 (sign only, SNaN not quieted) | same |
| 4-5, 8-12 | one NaN operand (fadds, fsubs, fmuls) | that NaN, sign and payload kept, SNaN quieted | same |
| 15-17, 19 | fmadds with one NaN, or C and B | the NaN (B before C) | same |
| 6, 7, 13, 14, 18, 20 | two NaN operands | the FIRST: frA, then frB, then frC | known gap: x86 / LLVM pick by instruction order |
| 21-23 | fnmsubs with a NaN | the NaN, NOT negated | melee_core::fp::fnmsubs / fnmadds (port/gekko-nan) |
| 24-27 | inf-inf, 0*inf, 0/0, fmadds 0*inf+1 | default NaN 7fc00000 (positive) | fp's fused helpers; `fp::invalid_nan` for a plain op |
| 28-30 | frsp of a double NaN | sign kept, payload truncated (7fc00009) | same (`as f32`) |

`melee_core::fp::tests::gekko_nan_rules` pins rows 21-27. NaN does not occur in normal play; these matter for the C oracles
(which feed NaN on purpose) and for any NaN that would reach the NHV1 hash.
