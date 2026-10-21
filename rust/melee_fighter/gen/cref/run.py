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
import os, re, subprocess, sys
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
    "ftlib": [("src/melee/ft/ftcommon.c", "void ftCommon_8007ED2C(", "void ftCommon_8007ED50("),
              ("src/melee/ft/ftcommon.c", "float ftCommon_GetModelScale(", "void ftCommon_8007F6A4("),
              ("src/melee/ft/ftcommon.c", "bool ftCommon_80080144(", "void ftCommon_8008021C("),
              ("src/melee/ft/ftlib.c", "s32 ftLib_800860C4(", "void ftLib_800871A8("),
              ("src/melee/ft/ftlib.c", "bool ftLib_80087284(", "void ftLib_80087508("),
              ("src/melee/ft/ftlib.c", "void ftLib_800876B4(", "float ftLib_8008777C("),
              ("src/melee/ft/ftlib.c", "bool ftLib_800877D4(", None)],
    "plbonus": [("src/melee/pl/player.c", "typedef struct _ftMapping {", "/// Matches when everything"),
                ("src/melee/pl/player.c", "StaticPlayer* Player_GetPtrForSlot(", "void Player_80031AD0("),
                ("src/melee/pl/player.c", "void Player_80031DA8(", "void Player_InitOrResetPlayer("),
                ("src/melee/pl/player.c", "void Player_MexSetMapping(", "#endif"),
                ("src/melee/pl/pl_040D.c", "void pl_80040DDC(", None),
                ("src/melee/pl/plbonuslib.c", "static inline bool unk_cond(", "void pl_8003D644("),
                # group 3 round 10: pl_8003DF44..pl_8003ED0C (src/plbonus2.rs)
                ("src/melee/pl/plbonuslib.c", "void pl_8003DF44(", "void fn_8003EE2C("),
                ("src/melee/pl/plbonuslib.c", "int pl_8003FBFC(", None)],
    # group 3 round 11: plbonus.c primitives (src/plbonus3.rs)
    "plbonusc": [("src/melee/pl/player.c", "typedef struct _ftMapping {", "/// Matches when everything"),
                 ("src/melee/pl/player.c", "StaticPlayer* Player_GetPtrForSlot(", "void Player_80031AD0("),
                 ("src/melee/pl/player.c", "void Player_80031DA8(", "void Player_InitOrResetPlayer("),
                 ("src/melee/pl/player.c", "void Player_MexSetMapping(", "#endif"),
                 ("src/melee/pl/plbonus.c", "static void setFlag(", "void pl_8003891C("),
                 ("src/melee/pl/plbonus.c", "void pl_80038F10(", "void pl_80039450(")],
    "ft0877": [("src/melee/ft/ftcommon.c", "bool ftCommon_80080144(", "void ftCommon_80080174("),
               ("src/melee/ft/ft_0877.c", "#define TEST(expr)", None)],
    "ft0881": [("src/melee/ft/ftcommon.c", "bool ftCommon_80080144(", "void ftCommon_80080174("),
               ("src/melee/ft/ft_0877.c", "#define TEST(expr)", None),
               ("src/melee/ft/ft_0881.c", "#if defined(TARGET_PC)", "void ft_800890BC(")],
    "ft0892": [("src/melee/pl/player.c", "static inline void Player_CheckSlot(", "/// Matches when everything"),
               ("src/melee/pl/player.c", "plActionStats* Player_GetActionStats(", "struct pl_x5EC_t* Player_GetUnk6A8Ptr("),
               ("src/melee/pl/plattack.c", "u16 plAttack_80037B08(", None),
               ("src/melee/ft/ft_0892.c", "void lbBgFlash_80020E38(", None),
               ("src/melee/pl/pltrick.c", "int pl_803BCE70[16]", "static void fn_80037F00("),
               ("src/melee/pl/pltrick.c", "void pl_800384DC(", None)],
    "ftcommon3": [("src/melee/ft/ftcommon.c", "void ftCommon_ClampGroundVel(", "void ftCommon_ApplyGroundedKnockbackFriction("),
                  ("src/melee/ft/ftcommon.c", "void ftCommon_UnlockECB(", "^/* Script collision"),
                  ("src/melee/ft/ftcommon.c", "^/* Script collision", "float ftCo_GetLStickAngle("),
                  ("src/melee/ft/ftcommon.c", "void ftCommon_8007DB24(", "void ftCommon_InitGrab("),
                  ("src/melee/ft/ftcommon.c", "void ftCommon_8007DD7C(", "HSD_GObj* ftCommon_8007E2A4("),
                  ("src/melee/ft/ftcommon.c", "void ftCommon_8007E358(", "void ftCommon_8007E5AC("),
                  ("src/melee/ft/ftcommon.c", "void ftCommon_SetAccessory(", "void ftCommon_8007EA90("),
                  ("src/melee/ft/ftcommon.c", "void ftCommon_8007EA90(", "void ftCommon_8007ECD4("),
                  ("src/melee/ft/ftcommon.c", "void ftCommon_8007EEC8(", "void ftCommon_8007EFC0("),
                  ("src/melee/ft/ftcommon.c", "extern void (*ftData_OnItemInvisible[])", "float ftCommon_GetModelScale("),
                  ("src/melee/ft/ftcommon.c", "void ftCommon_8007F6A4(", "void ftCommon_8007F76C("),
                  ("src/melee/ft/ftcommon.c", "void ftCommon_8007F824(", "void ftCommon_8007FF74("),
                  ("src/melee/ft/ft_0881.c", "void ft_80088770(", "void ft_80088828(")],
    # port round 9: "fn:NAME" cuts one definition (signature line through its closing "}" line)
    "ftleaf": [("src/melee/lb/lbtrigf.c", "#define SIGN_BIT", "float acosf("),
               ("src/melee/lb/lbtrigf.c", "#define SILVER_RATIO_1_CONJUGATE", None),
               *[("src/melee/ft/ftaction.c", "fn:ftAction_" + a, None) for a in (
                   "800711DC 800715EC 8007168C 800716F8 80071774 800717C8 80071810 80071CA4 "
                   "80071D30 80071F0C 800722C8 800726C0 80072E24 80072FE0 8007309C 8007320C "
                   "8007296C 800729C4 80072A4C 80072AAC 80072B04 80072B84 80072BE4 80072C5C "
                   "80073108 80071908 8007192C 80071950 80071974").split()],
               ("src/melee/ft/fighter.c", "u32 Fighter_NewSpawn_80068E40(", "void Fighter_80068E64("),
               ("src/melee/ft/fighter.c", "static struct {", "/* UCF 0.84's cardinal snap"),
               *[("src/melee/ft/fighter.c", "fn:" + n, None) for n in
                 ("Fighter_GetPosX", "Fighter_GetPosY", "Fighter_UnkSetFlag_8006CFBC", "setBit")],
               *[("src/melee/ft/ftanim.c", "fn:" + n, None) for n in
                 ("scale_inline", "get_part_joint", "ftAnim_8007077C", "ftAnim_80070FB4")],
               *[("src/melee/ft/ftcoll.c", "fn:" + n, None) for n in
                 ("ftColl_800765E0", "getEnvDmg", "inlineB1", "getUnkVal")],
               *[("src/melee/ft/ftcommon.c", "fn:" + n, None) for n in
                 ("ftCo_GetLStickAngle", "ftCo_GetCStickAngle", "ftCommon_8007D9D4")]],
    # group 3 round 12: ft_084E.c + two ft_081B.c tail functions + ft_08A1.c stick test (src/ft084e.rs)
    "ft084e": [("src/melee/ft/ftcommon.c", "void ftCommon_CalcGroundAccel_Deaccel(", "void ftCommon_UnlockECB("),
               ("src/melee/ft/ft_084E.c", "void ft_80084E1C(", None),
               ("src/melee/ft/ft_081B.c", "fn:ft_80084CB0", None),
               ("src/melee/ft/ft_081B.c", "fn:ft_80084DB0", None),
               ("src/melee/ft/ft_08A1.c", "fn:ft_8008A1FC", None),
               ("src/melee/ft/ft_08A1.c", "fn:ft_8008A244_inline", None),
               ("src/melee/ft/ft_08A1.c", "fn:ft_8008A244", None)],
    "ftcommon": [("src/melee/ft/ftcommon.c", "void ftCommon_CalcGroundAccel_Deaccel(", "void ftCommon_UnlockECB("),
                 ("src/melee/ft/ftcommon.c", "void ftCommon_UpdateFacing(", "void ftCommon_8007DB24(")],
}


def extract(name):
    parts = []
    for src, start, stop in EXTRACTS.get(name, []):
        lines = (ROOT / src).read_text(encoding="utf-8").splitlines()
        if start.startswith("fn:"):
            pat = re.compile(r"\b" + re.escape(start[3:]) + r"\s*\(")
            i = next(k for k, l in enumerate(lines) if pat.search(l) and not l.lstrip().startswith(("/", "*"))
                     and not l.rstrip().endswith(";") and k + 1 < len(lines) and lines[k + 1] == "{")
            j = next(k for k in range(i, len(lines)) if lines[k] == "}") + 1
            parts.append(f"/* extracted from {src}:{i + 1}-{j} by gen/cref/run.py */\n" + "\n".join(lines[i:j]) + "\n")
            continue
        # start "^X": begin one line earlier (at the `#if` that opens X's block)
        pre = 1 if start.startswith("^") else 0
        start = start[pre:]
        i = next(k for k, l in enumerate(lines) if l.startswith(start)) - pre
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
