"""Compile the real state.c/tev.c with shipping contraction and record logical state.

The only C source normalization is the RGB word mask: GC RGBA bytes use the
high 24 bits, whereas native little-endian RGBA bytes use the low 24 bits.
No disc data or other worktree is read. Reuses the runtime's gwtool IR lowering.
"""
from pathlib import Path
import os, re, subprocess, sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
OUT = ROOT / "rust/target/oracle-state-tev"
OUT.mkdir(parents=True, exist_ok=True)
CL = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
import gen_types as gt

# Replace the generator's external include directory; it is unused for these TUs.
flags = list(gt.NAT)
flags = [str(ROOT / "build/GALE01/include") if a == gt.GALE else a for a in flags]
flags += ["-I.", "-O2", "-ffp-contract=on", "-fno-fast-math", "-ftrivial-auto-var-init=zero", "-Xclang", "-disable-llvm-passes", "-S", "-emit-llvm"]
src = (ROOT / "src/sysdolphin/baselib/tev.c").read_text()
for old, new in [("((*d0 ^ *d1) & 0xFFFFFF00)", "((*d0 ^ *d1) & 0x00FFFFFF)"),
                 ("(*d & 0xff) | (*s & 0xffffff00)", "(*d & 0xff000000) | (*s & 0x00ffffff)")]:
    assert src.count(old) == 1
    src = src.replace(old, new)
(OUT / "tev_native.c").write_text(src)

mapping = [
    "GXSetCullMode", "GXSetBlendMode", "GXSetZMode", "GXSetAlphaCompare",
    "GXSetColorUpdate", "GXSetAlphaUpdate", "GXSetDstAlpha", "GXSetZCompLoc",
    "GXSetDither", "GXSetNumChans", "GXSetChanAmbColor", "GXSetChanMatColor",
    "GXSetChanCtrl", "GXSetNumTexGens", "GXSetNumTevStages", "GXSetTevOrder",
    "GXSetTevOp", "GXSetTevSwapMode", "GXSetTevColorIn", "GXSetTevAlphaIn",
    "GXSetTevColorOp", "GXSetTevAlphaOp", "GXSetTevKColorSel", "GXSetTevKAlphaSel",
    "HSD_ClearVtxDesc", "GXSetLineWidth", "GXSetPointSize", "GXSetTevColorS10",
]
probe = gt.ci.Index.create().parse("src/sysdolphin/baselib/tev.c", args=gt.NAT)
decls = {c.spelling: c for c in probe.cursor.get_children() if c.kind == gt.K.FUNCTION_DECL}
# state.c introduces the other GX declarations.
probe2 = gt.ci.Index.create().parse("src/sysdolphin/baselib/state.c", args=gt.NAT)
decls.update({c.spelling: c for c in probe2.cursor.get_children() if c.kind == gt.K.FUNCTION_DECL})
stubs = []
for i, name in enumerate(mapping, 1):
    d = decls[name]
    args = list(d.get_arguments())
    params = ", ".join(f"{a.type.spelling} a{j}" for j,a in enumerate(args)) or "void"
    words = [f"T({i});"]
    for j,a in enumerate(args):
        if a.type.spelling in ("GXColor", "GXColorS10"):
            words.extend(f"T(a{j}.{field});" for field in "rgba")
        else:
            words.append(f"T(a{j});")
    stubs.append(f"{d.result_type.spelling} {name}({params}) {{ {' '.join(words)} }}")
(OUT / "gx_stubs.h").write_text("\n".join(stubs))
ll = OUT / "driver.ll"
lo = OUT / "driver.lo.ll"
env = dict(os.environ, GW_NARROW_FPTOUI_AS_SI="1")
def run(cmd):
    subprocess.run([str(x) for x in cmd], cwd=ROOT, env=env, check=True)
run([CL, *flags, "-I" + str(OUT), "-Isrc/sysdolphin/baselib", HERE / "state_tev_driver.c", "-o", ll])
run([sys.executable, HERE / "lower.py", ll, lo])
run([CL, "-O2", "-ffp-contract=off", "-fno-builtin", "-fuse-ld=lld", lo, "-o", OUT / "oracle.exe"])
data = subprocess.check_output([str(OUT / "oracle.exe")], cwd=ROOT).decode().replace("\r\n", "\n")
(ROOT / "rust/melee_runtime/tests/data/oracle_state_tev.txt").write_text(data, newline="\n")
print(f"state/tev: {len(data.splitlines())} C state cases; {len(re.findall('call .*@llvm.fmuladd', ll.read_text()))} fused IR sites")
