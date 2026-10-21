#!/usr/bin/env python3
"""Build and run the C reference programs; write their expected values for the Rust tests.

  python rust/melee_fighter/gen/cref/run.py          (from the worktree root; Windows, LLVM clang)

Each <name>_ref.c is compiled natively (x86_64-pc-windows-msvc, the same flags and masks as
tools/rust-port/gen_cc.py) against the real headers, with the SHIPPING build's float
contraction: clang default -ffp-contract=on, -O2 -Xclang -disable-llvm-passes,
-ftrivial-auto-var-init=zero (as _build/masstest/pipe_win.sh), emitted as LLVM IR, then
gwtool's lowering redone by rust/melee_runtime/oracle/lower.py (f32 fmuladd -> fmadds, f64 ->
fma, float->int saturating), then linked. The drivers contain no a*b+c of their own; source that must
be the game's own is cut out of the game's .c file (EXTRACTS) rather than retyped. Output goes
to rust/melee_fighter/tests/data/<name>.txt, which the tests read. The data is computed from C
source only: no disc data.
"""
import os, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
DATA = ROOT / "rust/melee_fighter/tests/data"
BUILD = ROOT / "rust/target/cref"
CLANG = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
GALE = os.environ.get("GALE_INCLUDE", str(ROOT / "../../melee/build/GALE01/include"))
FLAGS = ["--target=x86_64-pc-windows-msvc", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
         "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-fno-strict-aliasing", "-fwrapv",
         "-fcommon", "-fgnu89-inline", "-ffp-contract=on", "-fno-fast-math", "-O2", "-Xclang",
         "-disable-llvm-passes", "-ftrivial-auto-var-init=zero", "-S", "-emit-llvm",
         "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t", "-mlong-double-64", "-Wno-everything",
         "-I", str(ROOT / "src"), "-I", str(ROOT / "src/melee"), "-I", str(ROOT / "pc"),
         "-I", str(ROOT / "pc/gameworld"), "-I", str(BUILD),
         "-isystem", str(ROOT / "tools/rust-port/include"), "-isystem", str(ROOT / "src/MSL"),
         "-isystem", str(ROOT / "libs/dolphin/include"), "-isystem", str(ROOT / "libs/dolphin/src"),
         "-isystem", GALE, "-include", str(ROOT / "src/MSL/math_ppc.h")]

# name -> list of (game source, first line starting with, stop before line starting with | None = EOF;
#   a leading ^ stops one line before that line)
EXTRACTS = {
    "nh": [("src/melee/ft/fighter.c", "static u32 nh_Mix(", "/* Kept in game memory")],
    "stale": [("src/melee/pl/player.c", "static inline void Player_CheckSlot(", "/// Matches when everything"),
              ("src/melee/pl/player.c", "plActionStats* Player_GetActionStats(", "struct pl_x5EC_t* Player_GetUnk6A8Ptr("),
              ("src/melee/ft/ft_0881.c", "void ft_800890BC(", None)],
    "ftcommon2": [
("src/melee/ft/ftcommon.c", 'float ftCommon_SandbagGetKnockbackDeaccelX(', 'void ftCommon_ApplyShieldKnockbackFriction('),
                  ("src/melee/ft/ftcommon.c", 'void ftCommon_UnlockECB(', '^/* Script collision'),
                  ("src/melee/ft/ftcommon.c", 'void ftCommon_8007D780(', 'void ftCommon_8007D7FC('),
                  ("src/melee/ft/ftcommon.c", 'void ftCommon_InitGrab(', 'void ftCommon_8007DD7C('),
                  ("src/melee/ft/ftcommon.c", 'HSD_GObj* ftCommon_8007E2A4(', 'void ftCommon_8007E358('),
                  ("src/melee/ft/ftcommon.c", 'void ftCommon_8007EBAC(', 'void ftCommon_8007EC30('),
                  ("src/melee/ft/ftcommon.c", 'void ftCommon_8007ECD4(', 'void ftCommon_8007EEC8('),
                  ("src/melee/ft/ftcommon.c", 'void ftCommon_8007EFC0(', 'void ftCommon_8007EFC8('),
                  ("src/melee/ft/ftcommon.c", 'float ftCommon_GetModelScale(', 'void ftCommon_8007F6A4('),
                  ("src/melee/ft/ftcommon.c", 'void ftCommon_8007F76C(', 'void ftCommon_8007F824('),
                  ("src/melee/ft/ftcommon.c", 'void ftCommon_8007F8B4(', 'extern void (*ftData_UnkMotionStates2'),
                  ("src/melee/ft/ftcommon.c", 'static inline float my_fminf(', 'void ftCommon_8007FE84('),
                  ("src/melee/ft/ftcommon.c", 'void ftCommon_8007FF74(', '#if defined(TARGET_PC)')],
    "player": [("src/melee/pl/player.c", "typedef struct _ftMapping {", "/// Matches when everything"),
               ("src/melee/pl/player.c", "StaticPlayer* Player_GetPtrForSlot(", "void Player_80031AD0("),
               ("src/melee/pl/player.c", "void Player_80031DA8(", "void Player_InitOrResetPlayer("),
               ("src/melee/pl/player.c", "void Player_MexSetMapping(", "#endif")],
    "ftcommon": [("src/melee/ft/ftcommon.c", "void ftCommon_CalcGroundAccel_Deaccel(", "void ftCommon_UnlockECB("),
                 ("src/melee/ft/ftcommon.c", "void ftCommon_UpdateFacing(", "void ftCommon_8007DB24(")],
}


def extract(name):
    parts = []
    for src, start, stop in EXTRACTS.get(name, []):
        lines = (ROOT / src).read_text(encoding="utf-8").splitlines()
        i = next(k for k, l in enumerate(lines) if l.startswith(start))
        # stop "^X": stop one line earlier (before the `#if` that opens X's block)
        back = 1 if stop and stop.startswith("^") else 0
        stop = stop[back:] if stop else stop
        j = len(lines) if stop is None else next(k for k in range(i, len(lines)) if lines[k].startswith(stop)) - back
        parts.append(f"/* extracted from {src}:{i + 1}-{j} by gen/cref/run.py */\n" + "\n".join(lines[i:j]) + "\n")
    (BUILD / f"{name}_extract.inc").write_text("".join(parts), encoding="utf-8")


LOWER = ROOT / "rust/melee_runtime/oracle/lower.py"


def run(cmd, n):
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        sys.stderr.write(r.stdout + r.stderr)
        raise SystemExit(f"cref: {n} failed to build")


def main():
    BUILD.mkdir(parents=True, exist_ok=True)
    DATA.mkdir(parents=True, exist_ok=True)
    names = sys.argv[1:] or sorted(p.name[:-6] for p in HERE.glob("*_ref.c"))
    for n in names:
        extract(n)
        exe = BUILD / f"{n}_ref.exe"
        # gekko_fp.c: __frsqrte and friends that math_ppc.h (force-included) calls
        lls = []
        for src in (HERE / f"{n}_ref.c", ROOT / "pc/gameworld/gekko_fp.c"):
            ll = BUILD / f"{n}__{src.stem}.ll"
            lo = BUILD / f"{n}__{src.stem}.lo.ll"
            run([CLANG, *FLAGS, str(src), "-o", str(ll)], n)
            run([sys.executable, str(LOWER), str(ll), str(lo)], n)
            lls.append(str(lo))
        run([CLANG, "-O0", "-w", *lls, "-o", str(exe)], n)
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        (DATA / f"{n}.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"cref: {n}: {len(out.splitlines())} lines -> {DATA / (n + '.txt')}")


if __name__ == "__main__":
    main()
