# Apply gwtool's IR rewrites (pc/tools/gwtool/gwtool.cpp) to textual LLVM IR, so a natively
# compiled C oracle computes what the real (clang + gwtool) build computes:
#   lowerFMulAdd:  f32 llvm.fmuladd -> fptrunc(fpext a * fpext b + fpext c); f64 -> llvm.fma.f64
#   makeArithmeticSafe (float->int only): fptosi/fptoui to i32 -> llvm.fpto[su]i.sat.i32
# Integer div/rem/shift safety is not applied (no such operand reaches the oracle's inputs).
import re
import sys

src = open(sys.argv[1]).read()
pat = re.compile(r'^(\s*)(%[\w.]+) = (?:tail )?call float @llvm\.fmuladd\.f32\(float ([^,]+), '
                 r'float ([^,]+), float ([^)]+)\)[^\n]*$', re.M)


def rep(m):
    i, r, a, b, c = m.groups()
    t = r.lstrip('%')
    return (f"{i}%gw{t}xa = fpext float {a} to double\n"
            f"{i}%gw{t}xb = fpext float {b} to double\n"
            f"{i}%gw{t}xc = fpext float {c} to double\n"
            f"{i}%gw{t}xp = fmul double %gw{t}xa, %gw{t}xb\n"
            f"{i}%gw{t}xs = fadd double %gw{t}xp, %gw{t}xc\n"
            f"{i}{r} = fptrunc double %gw{t}xs to float")


src, n = pat.subn(rep, src)
src = src.replace('@llvm.fmuladd.f64', '@llvm.fma.f64')
assert 'fmuladd.f32(float %' not in src, 'unlowered fmuladd'

fp = re.compile(r'(%[\w.]+) = fpto([su])i (float|double) ([^ ]+) to i32')
used = set()


def frep(m):
    r, su, t, v = m.groups()
    name = f'@llvm.fpto{su}i.sat.i32.{"f32" if t == "float" else "f64"}'
    used.add((name, t))
    return f'{r} = call i32 {name}({t} {v})'


src, k = fp.subn(frep, src)
assert not re.search(r'= fpto[su]i (float|double) [^ ]+ to i(8|16)\b', src), 'narrow fptosi: add'
src += ''.join(f'\ndeclare i32 {name}({t})' for name, t in sorted(used)) + '\n'
open(sys.argv[2], 'w').write(src)
print(sys.argv[1], n, 'fmuladd.f32 lowered,', k, 'fptoi saturated')
