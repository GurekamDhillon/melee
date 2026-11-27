"""Shared helpers for the static audits of the retail main.dol (NTSC 1.02).

Reads the DOL, symbols.txt and splits.txt; decodes Gekko instructions into register
def/use sets; parses decomp C prototypes (src/**) for parameter and return classes.
Nothing here writes disc-derived data anywhere: callers print counts and names only.
"""
import os
import re
import struct
from bisect import bisect_right

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", ".."))
MAIN_CHECKOUT = "E:/Projects/Melee Workspace/melee"


def default_path(rel):
    """`rel` inside this checkout if present, else inside the main checkout (orig/ is not in worktrees)."""
    p = os.path.join(REPO, rel)
    if os.path.exists(p):
        return p
    return os.path.join(MAIN_CHECKOUT, rel)


DOL_PATH = default_path("orig/GALE01/sys/main.dol")
SYM_PATH = default_path("config/GALE01/symbols.txt")
SPLIT_PATH = default_path("config/GALE01/splits.txt")
SRC_DIR = os.path.join(REPO, "src")
SRC_ROOTS = [SRC_DIR, os.path.join(REPO, "libs", "dolphin", "src"), os.path.join(REPO, "libs", "dolphin", "include"),
             os.path.join(REPO, "libs", "doldecomp", "include")]
RUST_DIR = os.path.join(REPO, "rust")

# ---------------------------------------------------------------- DOL


class Dol:
    def __init__(self, path=DOL_PATH):
        self.data = open(path, "rb").read()
        d = self.data
        offs = struct.unpack(">18I", d[0:0x48])
        addrs = struct.unpack(">18I", d[0x48:0x90])
        sizes = struct.unpack(">18I", d[0x90:0xD8])
        self.sections = [(addrs[i], offs[i], sizes[i]) for i in range(18) if sizes[i]]

    def word(self, addr):
        for a, o, s in self.sections:
            if a <= addr < a + s:
                return struct.unpack(">I", self.data[o + addr - a: o + addr - a + 4])[0]
        return None


# ---------------------------------------------------------------- symbols / splits


class Func:
    __slots__ = ("name", "addr", "size", "tu", "area")

    def __init__(self, name, addr, size):
        self.name, self.addr, self.size = name, addr, size
        self.tu = None
        self.area = None

    @property
    def end(self):
        return self.addr + self.size


def area_of_tu(n):
    if n is None:
        return "unsplit"
    if n.startswith("melee/ft/") or n.startswith("melee/pl/"):
        return "fighter"
    if n.startswith("melee/it/"):
        return "items"
    if n.startswith(("melee/gr/", "melee/mp/", "melee/cm/", "melee/ef/")):
        return "stage/coll/cam/fx"
    if n.startswith("melee/lb/"):
        return "lb"
    if n.startswith("sysdolphin/"):
        return "HSD"
    return "other"


ONLINE_AREAS = ("fighter", "items", "stage/coll/cam/fx", "lb", "HSD")
AREA_ORDER = ONLINE_AREAS + ("other", "unsplit")


def load_functions(sym_path=SYM_PATH, split_path=SPLIT_PATH):
    funcs = []
    for line in open(sym_path, encoding="utf-8"):
        m = re.match(r"(\S+) = (\.init|\.text):0x([0-9A-Fa-f]+); // type:function size:0x([0-9A-Fa-f]+)", line)
        if m:
            funcs.append(Func(m.group(1), int(m.group(3), 16), int(m.group(4), 16)))
    funcs.sort(key=lambda f: f.addr)
    tus = []
    cur = None
    for line in open(split_path, encoding="utf-8"):
        if line and not line[0].isspace() and line.rstrip().endswith(":"):
            cur = line.strip()[:-1]
        m = re.match(r"\s+(\.text|\.init)\s+start:0x([0-9A-Fa-f]+) end:0x([0-9A-Fa-f]+)", line)
        if m and cur:
            tus.append((int(m.group(2), 16), int(m.group(3), 16), cur))
    tus.sort()
    starts = [t[0] for t in tus]
    for f in funcs:
        i = bisect_right(starts, f.addr) - 1
        if i >= 0 and tus[i][0] <= f.addr < tus[i][1]:
            f.tu = tus[i][2]
        f.area = area_of_tu(f.tu)
    return funcs


# ---------------------------------------------------------------- register numbering
# 0..31 GPR, 32..63 FPR, 64..95 CR bits, 96 CTR, 97 XER[CA]
F = 32
CR = 64
CTR = 96
CA = 97
NREG = 98


def regname(r):
    if r < 32:
        return "r%d" % r
    if r < 64:
        return "f%d" % (r - 32)
    if r < 96:
        b = r - 64
        return "cr%d" % (b // 4)
    return "ctr" if r == CTR else "ca"


def crf(n):
    return [CR + 4 * n + i for i in range(4)]


CALLER_SAVED = [0] + list(range(3, 13)) + [F + i for i in range(14)] + crf(0) + crf(1) + crf(5) + crf(6) + crf(7) + [CTR, CA]
CALLEE_SAVED = list(range(14, 32)) + [F + i for i in range(14, 32)] + crf(2) + crf(3) + crf(4)

# ---------------------------------------------------------------- decoder


class Ins:
    """Decoded instruction: uses/defs are register indices (see above).

    kind: alu, load, store, cmp, br (b/bc within or out), call, ret, bctr, nop, other
    mem: (base_reg or None for rA=0, index_reg or None, disp, nbytes, update)  for loads/stores
    """
    __slots__ = ("addr", "w", "mn", "kind", "uses", "defs", "mem", "target", "cond", "link", "ctr_dec", "stval")

    def __init__(self, addr, w):
        self.addr, self.w = addr, w
        self.mn = "?"
        self.kind = "other"
        self.uses = []
        self.defs = []
        self.mem = None
        self.target = None
        self.cond = None   # CR bit read by a conditional branch
        self.link = False
        self.ctr_dec = False
        self.stval = []    # value registers of a store


def _s16(v):
    return v - 0x10000 if v & 0x8000 else v


FP_A = {18: "div", 20: "sub", 21: "add", 22: "sqrt", 23: "sel", 24: "res", 25: "mul", 26: "rsqrte",
        28: "msub", 29: "madd", 30: "nmsub", 31: "nmadd"}
FUSED = ("fmadds", "fmsubs", "fnmadds", "fnmsubs", "fmadd", "fmsub", "fnmadd", "fnmsub")
PS_FUSED = ("ps_madd", "ps_msub", "ps_nmadd", "ps_nmsub", "ps_madds0", "ps_madds1", "ps_sum0", "ps_sum1")

LOADS_D = {32: ("lwz", 4, 0), 33: ("lwzu", 4, 1), 34: ("lbz", 1, 0), 35: ("lbzu", 1, 1), 40: ("lhz", 2, 0),
           41: ("lhzu", 2, 1), 42: ("lha", 2, 0), 43: ("lhau", 2, 1)}
STORES_D = {36: ("stw", 4, 0), 37: ("stwu", 4, 1), 38: ("stb", 1, 0), 39: ("stbu", 1, 1), 44: ("sth", 2, 0),
            45: ("sthu", 2, 1)}
FLOADS_D = {48: ("lfs", 4, 0), 49: ("lfsu", 4, 1), 50: ("lfd", 8, 0), 51: ("lfdu", 8, 1)}
FSTORES_D = {52: ("stfs", 4, 0), 53: ("stfsu", 4, 1), 54: ("stfd", 8, 0), 55: ("stfdu", 8, 1)}
LOADS_X = {23: ("lwzx", 4, 0), 55: ("lwzux", 4, 1), 87: ("lbzx", 1, 0), 119: ("lbzux", 1, 1), 279: ("lhzx", 2, 0),
           311: ("lhzux", 2, 1), 343: ("lhax", 2, 0), 375: ("lhaux", 2, 1), 534: ("lwbrx", 4, 0), 790: ("lhbrx", 2, 0),
           20: ("lwarx", 4, 0)}
STORES_X = {151: ("stwx", 4, 0), 183: ("stwux", 4, 1), 215: ("stbx", 1, 0), 247: ("stbux", 1, 1), 407: ("sthx", 2, 0),
            439: ("sthux", 2, 1), 662: ("stwbrx", 4, 0), 918: ("sthbrx", 2, 0), 150: ("stwcx.", 4, 0)}
FLOADS_X = {535: ("lfsx", 4, 0), 567: ("lfsux", 4, 1), 599: ("lfdx", 8, 0), 631: ("lfdux", 8, 1)}
FSTORES_X = {663: ("stfsx", 4, 0), 695: ("stfsux", 4, 1), 727: ("stfdx", 8, 0), 759: ("stfdux", 8, 1),
             983: ("stfiwx", 4, 0)}
XO_ARITH = {266: "add", 40: "subf", 235: "mullw", 459: "divwu", 491: "divw", 11: "mulhwu", 75: "mulhw",
            8: "subfc", 10: "addc", 136: "subfe", 138: "adde", 104: "neg", 200: "subfze", 202: "addze",
            232: "subfme", 234: "addme"}
X_LOGIC = {24: "slw", 536: "srw", 792: "sraw", 28: "and", 60: "andc", 124: "nor", 284: "eqv", 316: "xor",
           412: "orc", 444: "or", 476: "nand", 824: "srawi", 26: "cntlzw", 922: "extsh", 954: "extsb"}


def decode(addr, w):
    i = Ins(addr, w)
    op = w >> 26
    D = (w >> 21) & 31
    A = (w >> 16) & 31
    B = (w >> 11) & 31
    C = (w >> 6) & 31
    rc = w & 1
    xo10 = (w >> 1) & 0x3FF
    xo5 = (w >> 1) & 0x1F
    simm = _s16(w & 0xFFFF)
    U, Df = i.uses, i.defs
    i.kind = "alu"

    def ra0():
        return None if A == 0 else A

    if w == 0x60000000:
        i.mn, i.kind = "nop", "nop"
        return i
    if op in (14, 15):
        i.mn = "addi" if op == 14 else "addis"
        if A:
            U.append(A)
        Df.append(D)
        i.stval = [simm]
    elif op == 7:
        i.mn = "mulli"; U.append(A); Df.append(D)
    elif op == 8:
        i.mn = "subfic"; U.append(A); Df += [D, CA]
    elif op in (12, 13):
        i.mn = "addic" + ("." if op == 13 else ""); U.append(A); Df += [D, CA]
        if op == 13:
            Df += crf(0)
    elif op in (10, 11):
        i.mn = "cmpli" if op == 10 else "cmpi"; i.kind = "cmp"; U.append(A); Df += crf(D >> 2)
        i.stval = [w & 0xFFFF if op == 10 else simm]
    elif op == 3:
        i.mn = "twi"; i.kind = "other"; U.append(A)
    elif op in (20, 21, 23):
        i.mn = {20: "rlwimi", 21: "rlwinm", 23: "rlwnm"}[op]
        U.append(D)
        if op == 20:
            U.append(A)
        if op == 23:
            U.append(B)
        Df.append(A)
        if rc:
            Df += crf(0)
    elif op in (24, 25, 26, 27, 28, 29):
        i.mn = {24: "ori", 25: "oris", 26: "xori", 27: "xoris", 28: "andi.", 29: "andis."}[op]
        U.append(D); Df.append(A)
        if op in (28, 29):
            Df += crf(0)
    elif op in LOADS_D or op in FLOADS_D:
        mn, n, upd = LOADS_D.get(op) or FLOADS_D[op]
        i.mn, i.kind = mn, "load"
        i.mem = (ra0(), None, simm, n, upd)
        if A:
            U.append(A)
        Df.append(D if op in LOADS_D else F + D)
        if upd:
            Df.append(A)
    elif op in STORES_D or op in FSTORES_D:
        mn, n, upd = STORES_D.get(op) or FSTORES_D[op]
        i.mn, i.kind = mn, "store"
        i.mem = (ra0(), None, simm, n, upd)
        v = D if op in STORES_D else F + D
        U.append(v); i.stval = [v]
        if A:
            U.append(A)
        if upd:
            Df.append(A)
    elif op == 46:
        i.mn, i.kind = "lmw", "load"; i.mem = (ra0(), None, simm, 4 * (32 - D), 0)
        if A:
            U.append(A)
        Df += list(range(D, 32))
    elif op == 47:
        i.mn, i.kind = "stmw", "store"; i.mem = (ra0(), None, simm, 4 * (32 - D), 0)
        i.stval = list(range(D, 32)); U += i.stval
        if A:
            U.append(A)
    elif op in (56, 57):
        i.mn, i.kind = ("psq_l", "psq_lu")[op - 56], "load"
        d12 = w & 0xFFF
        d12 = d12 - 0x1000 if d12 & 0x800 else d12
        i.mem = (ra0(), None, d12, 8 if not (w >> 15) & 1 else 4, op == 57)
        if A:
            U.append(A)
        Df.append(F + D)
        if op == 57:
            Df.append(A)
    elif op in (60, 61):
        i.mn, i.kind = ("psq_st", "psq_stu")[op - 60], "store"
        d12 = w & 0xFFF
        d12 = d12 - 0x1000 if d12 & 0x800 else d12
        i.mem = (ra0(), None, d12, 8 if not (w >> 15) & 1 else 4, op == 61)
        U.append(F + D); i.stval = [F + D]
        if A:
            U.append(A)
        if op == 61:
            Df.append(A)
    elif op == 18:
        li = w & 0x03FFFFFC
        if li & 0x02000000:
            li -= 0x04000000
        i.target = (li if (w & 2) else addr + li) & 0xFFFFFFFF
        i.link = bool(w & 1)
        i.mn = "bl" if i.link else "b"
        i.kind = "call" if i.link else "br"
    elif op == 16 or (op == 19 and xo10 in (16, 528)):
        BO, BI = D, A
        if not (BO & 0x10):
            i.cond = CR + BI
            U.append(CR + BI)
        if not (BO & 4):
            i.ctr_dec = True
            U.append(CTR); Df.append(CTR)
        i.link = bool(w & 1)
        if op == 16:
            bd = w & 0xFFFC
            if bd & 0x8000:
                bd -= 0x10000
            i.target = (bd if (w & 2) else addr + bd) & 0xFFFFFFFF
            i.mn = "bc" + ("l" if i.link else "")
            i.kind = "call" if i.link else "br"
        elif xo10 == 16:
            i.mn = "bclr" + ("l" if i.link else "")
            i.kind = "callind" if i.link else "ret"
        else:
            i.mn = "bcctr" + ("l" if i.link else "")
            U.append(CTR)
            i.kind = "callind" if i.link else "bctr"
    elif op == 19:
        i.kind = "alu"
        if xo10 == 0:
            i.mn = "mcrf"; U += crf(A >> 2); Df += crf(D >> 2)
        elif xo10 in (33, 129, 193, 225, 257, 289, 417, 449):
            i.mn = {33: "crnor", 129: "crandc", 193: "crxor", 225: "crnand", 257: "crand", 289: "creqv",
                    417: "crorc", 449: "cror"}[xo10]
            if not (xo10 in (193, 289) and A == B):
                U += [CR + A, CR + B]
            Df.append(CR + D)
        else:
            i.mn = "x19_%d" % xo10; i.kind = "other"
    elif op == 17:
        i.mn, i.kind = "sc", "other"
    elif op in (59, 63):
        fd = F + D
        if xo5 in FP_A:
            i.mn = ("f" if True else "") + FP_A[xo5] + ("s" if op == 59 else "")
            if FP_A[xo5] in ("sqrt", "res", "rsqrte"):
                U.append(F + B)
            elif FP_A[xo5] == "mul":
                U += [F + A, F + C]
            elif FP_A[xo5] in ("div", "sub", "add"):
                U += [F + A, F + B]
            else:
                U += [F + A, F + C, F + B]
            Df.append(fd)
            if rc:
                Df += crf(1)
        elif op == 63:
            if xo10 in (0, 32):
                i.mn, i.kind = ("fcmpu", "fcmpo")[xo10 == 32], "cmp"
                U += [F + A, F + B]; Df += crf(D >> 2)
            elif xo10 in (12, 14, 15, 40, 72, 136, 264):
                i.mn = {12: "frsp", 14: "fctiw", 15: "fctiwz", 40: "fneg", 72: "fmr", 136: "fnabs", 264: "fabs"}[xo10]
                U.append(F + B); Df.append(fd)
                if rc:
                    Df += crf(1)
            elif xo10 == 583:
                i.mn = "mffs"; Df.append(fd)
            elif xo10 == 711:
                i.mn = "mtfsf"; U.append(F + B); i.kind = "other"
            elif xo10 == 64:
                i.mn = "mcrfs"; Df += crf(D >> 2)
            else:
                i.mn = {38: "mtfsb1", 70: "mtfsb0", 134: "mtfsfi"}.get(xo10, "fp_other"); i.kind = "other"
        else:
            i.mn, i.kind = "fp_other", "other"
    elif op == 4:
        x6 = (w >> 1) & 0x3F
        if xo5 in (10, 11, 12, 13, 14, 15, 18, 20, 21, 23, 24, 25, 26, 28, 29, 30, 31):
            i.mn = {10: "ps_sum0", 11: "ps_sum1", 12: "ps_muls0", 13: "ps_muls1", 14: "ps_madds0", 15: "ps_madds1",
                    18: "ps_div", 20: "ps_sub", 21: "ps_add", 23: "ps_sel", 24: "ps_res", 25: "ps_mul",
                    26: "ps_rsqrte", 28: "ps_msub", 29: "ps_madd", 30: "ps_nmsub", 31: "ps_nmadd"}[xo5]
            if xo5 in (24, 26):
                U.append(F + B)
            elif xo5 in (12, 13, 25):
                U += [F + A, F + C]
            elif xo5 in (18, 20, 21):
                U += [F + A, F + B]
            else:
                U += [F + A, F + C, F + B]
            Df.append(F + D)
            if rc:
                Df += crf(1)
        elif x6 in (6, 7, 38, 39):
            i.mn = {6: "psq_lx", 7: "psq_stx", 38: "psq_lux", 39: "psq_stux"}[x6]
            if A:
                U.append(A)
            U.append(B)
            i.mem = (ra0(), B, 0, 8, x6 >= 38)
            if x6 in (6, 38):
                i.kind = "load"; Df.append(F + D)
            else:
                i.kind = "store"; U.append(F + D); i.stval = [F + D]
            if x6 >= 38:
                Df.append(A)
        elif xo10 in (0, 32, 64, 96):
            i.mn, i.kind = "ps_cmp", "cmp"; U += [F + A, F + B]; Df += crf(D >> 2)
        elif xo10 in (40, 72, 136, 264):
            i.mn = {40: "ps_neg", 72: "ps_mr", 136: "ps_nabs", 264: "ps_abs"}[xo10]; U.append(F + B); Df.append(F + D)
        elif xo10 in (528, 560, 592, 624):
            i.mn = {528: "ps_merge00", 560: "ps_merge01", 592: "ps_merge10", 624: "ps_merge11"}[xo10]
            U += [F + A, F + B]; Df.append(F + D)
        elif xo10 == 1014:
            i.mn, i.kind = "dcbz_l", "other"
            if A:
                U.append(A)
            U.append(B)
        else:
            i.mn, i.kind = "op4_other", "other"
    elif op == 31:
        x9 = (w >> 1) & 0x1FF
        if x9 in XO_ARITH and xo10 not in X_LOGIC and xo10 not in LOADS_X and xo10 not in STORES_X:
            n = XO_ARITH[x9]
            i.mn = n
            U.append(A)
            if n not in ("neg", "subfze", "addze", "subfme", "addme"):
                U.append(B)
            if n in ("subfe", "adde", "subfze", "addze", "subfme", "addme"):
                U.append(CA)
            Df.append(D)
            if n in ("subfc", "addc", "subfe", "adde", "subfze", "addze", "subfme", "addme"):
                Df.append(CA)
            if rc:
                Df += crf(0)
        elif xo10 in X_LOGIC:
            n = X_LOGIC[xo10]
            i.mn = n
            U.append(D)
            if n not in ("srawi", "cntlzw", "extsh", "extsb"):
                if not (n == "or" and B == D):
                    U.append(B)
                if n == "or" and B == D:
                    i.mn = "mr"
            Df.append(A)
            if n in ("sraw", "srawi"):
                Df.append(CA)
            if rc:
                Df += crf(0)
        elif xo10 in (0, 32):
            i.mn, i.kind = ("cmp", "cmpl")[xo10 == 32], "cmp"; U += [A, B]; Df += crf(D >> 2)
        elif xo10 in LOADS_X or xo10 in FLOADS_X:
            mn, n, upd = LOADS_X.get(xo10) or FLOADS_X[xo10]
            i.mn, i.kind = mn, "load"
            i.mem = (ra0(), B, 0, n, upd)
            if A:
                U.append(A)
            U.append(B)
            Df.append(D if xo10 in LOADS_X else F + D)
            if upd:
                Df.append(A)
        elif xo10 in STORES_X or xo10 in FSTORES_X:
            mn, n, upd = STORES_X.get(xo10) or FSTORES_X[xo10]
            i.mn, i.kind = mn, "store"
            i.mem = (ra0(), B, 0, n, upd)
            v = D if xo10 in STORES_X else F + D
            U.append(v); i.stval = [v]
            if A:
                U.append(A)
            U.append(B)
            if upd:
                Df.append(A)
            if xo10 == 150:
                Df += crf(0)
        elif xo10 == 19:
            i.mn = "mfcr"; Df.append(D)   # CR save: not checked as a read
        elif xo10 == 144:
            i.mn = "mtcrf"; U.append(D)
            fxm = (w >> 12) & 0xFF
            for n in range(8):
                if fxm & (0x80 >> n):
                    Df += crf(n)
        elif xo10 in (339, 467, 371):
            spr = ((w >> 16) & 31) | (((w >> 11) & 31) << 5)
            if xo10 == 339 or xo10 == 371:
                i.mn = "mf" + {1: "xer", 8: "lr", 9: "ctr"}.get(spr, "spr")
                if spr == 9:
                    U.append(CTR)
                if spr == 1:
                    U.append(CA)
                Df.append(D)
                if spr not in (1, 8, 9):
                    i.kind = "other"
            else:
                i.mn = "mt" + {1: "xer", 8: "lr", 9: "ctr"}.get(spr, "spr")
                U.append(D)
                if spr == 9:
                    Df.append(CTR)
                elif spr == 1:
                    Df.append(CA)
                else:
                    i.kind = "other"
        elif xo10 in (83, 595, 659):
            i.mn, i.kind = "mfmsr/sr", "other"; Df.append(D)
        elif xo10 in (146, 210, 242):
            i.mn, i.kind = "mtmsr/sr", "other"; U.append(D)
        elif xo10 in (86, 54, 278, 246, 470, 1014, 982):
            i.mn, i.kind = "dcache", "other"
            if A:
                U.append(A)
            U.append(B)
        elif xo10 in (598, 854, 566, 306, 370):
            i.mn, i.kind = "sync", "other"
        elif xo10 == 4:
            i.mn, i.kind = "tw", "other"; U += [A, B]
        else:
            i.mn, i.kind = "x31_%d" % xo10, "other"
    else:
        i.mn, i.kind = "op%d" % op, "other"
    return i


def decode_function(dol, f):
    out = []
    for p in range(f.addr, f.end, 4):
        w = dol.word(p)
        out.append(decode(p, w if w is not None else 0))
    return out


def fused_counts(dol, f):
    from collections import Counter
    c = Counter()
    for p in range(f.addr, f.end, 4):
        w = dol.word(p)
        if not w:
            continue
        op = w >> 26
        if op in (59, 63) and ((w >> 1) & 0x1F) in (28, 29, 30, 31):
            c[decode(p, w).mn] += 1
        elif op == 4:
            mn = decode(p, w).mn
            if mn in PS_FUSED:
                c[mn] += 1
    return c


# ---------------------------------------------------------------- C prototypes

FLOAT_T = {"f32", "f64", "float", "double", "Float"}
INT64_T = {"s64", "u64", "long long", "unsigned long long", "signed long long", "OSTime", "OSTick64"}
SCALAR_T = {"void", "int", "s32", "u32", "s16", "u16", "s8", "u8", "char", "short", "long", "unsigned", "signed",
            "bool", "BOOL", "size_t", "uintptr_t", "intptr_t", "ptrdiff_t", "vu32", "vu16", "vu8", "vs32", "vs16",
            "vs8", "uint", "u_int"}

_typedef_cache = None


def _strip_comments(s):
    s = re.sub(r"/\*.*?\*/", lambda m: "\n" * m.group(0).count("\n"), s, flags=re.S)
    s = re.sub(r"//[^\n]*", "", s)
    return s


def _iter_sources():
    for top in SRC_ROOTS:
        for root, _, files in os.walk(top):
            for fn in files:
                if fn.endswith((".c", ".h", ".cpp", ".inc")):
                    yield os.path.join(root, fn), top


def typedef_classes():
    """name -> 'float' | 'int64' | 'scalar' | 'struct' from typedefs across src/."""
    global _typedef_cache
    if _typedef_cache is not None:
        return _typedef_cache
    td = {}
    for path, _top in _iter_sources():
        try:
            s = _strip_comments(open(path, encoding="utf-8", errors="replace").read())
        except OSError:
            continue
        for m in re.finditer(r"typedef\s+(struct|union)\s*\w*\s*\{", s):
            # find matching close then the name
            depth, j = 0, m.end() - 1
            while j < len(s):
                if s[j] == "{":
                    depth += 1
                elif s[j] == "}":
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            mm = re.match(r"\s*(\w+)", s[j + 1:j + 200])
            if mm:
                td[mm.group(1)] = "struct"
        for m in re.finditer(r"typedef\s+(struct|union)\s+(\w+)\s+(\w+)\s*;", s):
            td[m.group(3)] = "struct"
        for m in re.finditer(r"typedef\s+enum\s*\w*\s*\{[^}]*\}\s*(\w+)\s*;", s):
            td[m.group(1)] = "scalar"
        for m in re.finditer(r"typedef\s+enum\s+\w+\s+(\w+)\s*;", s):
            td[m.group(1)] = "scalar"
        for m in re.finditer(r"typedef\s+([\w ]+?)\s+(\w+)\s*;", s):
            base, name = m.group(1).strip(), m.group(2)
            if base.startswith(("struct", "union")):
                continue
            base = base.replace("volatile", "").replace("const", "").strip()
            if base in FLOAT_T:
                td.setdefault(name, "float")
            elif base in INT64_T or "long long" in base:
                td.setdefault(name, "int64")
            elif base.startswith("enum") or base in SCALAR_T or base.split()[-1] in SCALAR_T:
                td.setdefault(name, "scalar")
            else:
                td.setdefault(name, ("alias", base))
    # resolve aliases
    def res(n, depth=0):
        v = td.get(n)
        if isinstance(v, tuple):
            if depth > 8:
                return "scalar"
            b = v[1]
            if b in FLOAT_T:
                return "float"
            if b in INT64_T:
                return "int64"
            r = res(b, depth + 1)
            return r if r else "scalar"
        return v
    _typedef_cache = {n: res(n) for n in td}
    return _typedef_cache


def classify_type(t):
    """Return 'void', 'float', 'int64', 'struct', 'int' (GPR) for a C type text (no name)."""
    t = re.sub(r"\b(const|volatile|static|inline|extern|register|signed|__attribute__\(\(.*?\)\)|AT_ADDRESS\(.*?\)|"
               r"ATTRIBUTE_\w+|WEAK|DECL_WEAK|asm|__declspec\(\w+\)|NORETURN|SECTION_\w+)\b", " ", t)
    t = " ".join(t.split())
    if "*" in t or "[" in t or "(" in t:
        return "int"
    if t in ("void", ""):
        return "void" if t == "void" else "int"
    if t in FLOAT_T:
        return "float"
    if t in INT64_T or "long long" in t:
        return "int64"
    words = t.split()
    last = words[-1] if words else t
    if words and words[0] in ("struct", "union"):
        return "struct"
    if words and words[0] == "enum":
        return "int"
    td = typedef_classes().get(last)
    if td == "float":
        return "float"
    if td == "int64":
        return "int64"
    if td == "struct":
        return "struct"
    return "int"


def _split_params(p):
    out, depth, cur = [], 0, ""
    for ch in p:
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return [x.strip() for x in out]


def _param_type(p):
    """Drop the parameter name from a declarator (keeps * and function-pointer shape)."""
    p = p.strip()
    if "(" in p:
        return p  # function pointer -> GPR
    p = re.sub(r"\[[^\]]*\]", "*", p)
    m = re.match(r"(.*?)(\w+)\s*$", p)
    if m and m.group(1).strip() and not m.group(1).strip().endswith(("struct", "union", "enum")):
        base = m.group(1).strip()
        # a bare type like "f32" or "u8" has no name; m.group(1) would be '' then
        return base
    return p


class Proto:
    __slots__ = ("name", "ret", "params", "varargs", "path", "line", "is_asm", "body")

    def __init__(self):
        self.body = None

    def gpr_args(self):
        """GPR indices holding parameters (r3..r10), floats f1..f8, as register lists."""
        g, f = [], []
        nxt = 3
        if self.ret == "struct":
            g.append(3)
            nxt = 4
        nf = 1
        for c in self.params:
            if c == "float":
                if nf <= 8:
                    f.append(F + nf)
                nf += 1
            elif c == "int64":
                if nxt % 2 == 0:
                    nxt += 1
                if nxt + 1 <= 10:
                    g += [nxt, nxt + 1]
                nxt += 2
            else:
                if nxt <= 10:
                    g.append(nxt)
                nxt += 1
        return g, f


_proto_cache = None
_DEF_RE = re.compile(r"(?m)^([A-Za-z_][\w \t\*]*?[\s\*])\s*(\w+)\s*\(")
_KEYWORDS = {"if", "for", "while", "switch", "return", "sizeof", "do", "else", "case"}


def _match_paren(s, i):
    depth = 0
    for j in range(i, len(s)):
        if s[j] == "(":
            depth += 1
        elif s[j] == ")":
            depth -= 1
            if depth == 0:
                return j
    return -1


def _match_brace(s, i, quotes=True):
    depth = 0
    j = i
    n = len(s)
    while j < n:
        ch = s[j]
        if quotes and (ch == '"' or ch == "'"):
            q = ch
            j += 1
            while j < n and s[j] != q:
                if s[j] == "\\":
                    j += 1
                j += 1
        elif ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return j
        j += 1
    return -1


def _parse_file(path, top, out, decls):
    try:
        raw = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        return
    s = _strip_comments(raw)
    rel = os.path.relpath(path, top).replace("\\", "/")
    for m in _DEF_RE.finditer(s):
        name = m.group(2)
        if name in _KEYWORDS:
            continue
        ret = m.group(1)
        if ret.strip().startswith(("#", "return", "else", "typedef")):
            continue
        po = s.index("(", m.end() - 1)
        pc = _match_paren(s, po)
        if pc < 0:
            continue
        after = s[pc + 1:pc + 400].lstrip()
        params = s[po + 1:pc]
        is_def = after.startswith("{")
        if not is_def and not after.startswith(";"):
            # K&R or attributes between ) and {
            mm = re.match(r"(?:\w+(?:\([^)]*\))?\s*)*\{", after)
            if mm:
                is_def = True
            else:
                continue
        p = Proto()
        p.name = name
        p.is_asm = bool(re.search(r"\basm\b|\bASM\b", ret))
        rt = classify_type(re.sub(r"\b(asm|ASM|static|inline)\b", " ", ret))
        p.ret = rt
        plist = _split_params(params)
        p.varargs = any(x.strip() == "..." for x in plist)
        cls = []
        for x in plist:
            if x.strip() in ("void", "...", ""):
                continue
            cls.append(classify_type(_param_type(x)))
        p.params = cls
        p.path = rel
        p.line = s.count("\n", 0, m.start()) + 1
        if is_def:
            b0 = s.index("{", pc)
            b1 = _match_brace(s, b0)
            p.body = s[b0:b1 + 1] if b1 > 0 else ""
            out.setdefault(name, []).append(p)
        else:
            decls.setdefault(name, []).append(p)


def load_prototypes():
    """name -> list of Proto (definitions), and name -> list of declarations."""
    global _proto_cache
    if _proto_cache is None:
        defs, decls = {}, {}
        for path, top in _iter_sources():
            _parse_file(path, top, defs, decls)
        _proto_cache = (defs, decls)
    return _proto_cache


def proto_for(func, defs, decls):
    """Best prototype for a DOL function: definition in its split TU, else any definition, else a declaration."""
    cands = defs.get(func.name) or []
    if func.tu:
        base = os.path.splitext(func.tu)[0]
        for p in cands:
            if os.path.splitext(p.path)[0] == base:
                return p
    if cands:
        return cands[0]
    d = decls.get(func.name)
    if d:
        return d[0]
    return None


# ---------------------------------------------------------------- Rust functions

_rust_cache = None


def _rust_strip(s):
    """Blank out comments and string/char literals (keeps offsets)."""
    out = list(s)
    n = len(s)
    i = 0
    while i < n:
        c = s[i]
        if c == "/" and i + 1 < n and s[i + 1] == "/":
            j = s.find("\n", i)
            j = n if j < 0 else j
            for k in range(i, j):
                out[k] = " "
            i = j
        elif c == "/" and i + 1 < n and s[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if s.startswith("/*", j):
                    depth += 1; j += 2
                elif s.startswith("*/", j):
                    depth -= 1; j += 2
                else:
                    j += 1
            for k in range(i, j):
                if out[k] != "\n":
                    out[k] = " "
            i = j
        elif c == '"' or (c == "r" and i + 1 < n and s[i + 1] in '#"' and (i == 0 or not (s[i - 1].isalnum() or s[i - 1] == "_"))):
            if c == "r":
                m = re.match(r'r(#*)"', s[i:])
                if not m:
                    i += 1
                    continue
                close = '"' + m.group(1)
                j = s.find(close, i + m.end())
                j = n if j < 0 else j + len(close)
            else:
                j = i + 1
                while j < n and s[j] != '"':
                    if s[j] == "\\":
                        j += 1
                    j += 1
                j += 1
            for k in range(i + 1, min(j, n) - 1):
                if out[k] != "\n":
                    out[k] = " "
            i = j
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", s[i:])
            if m:
                for k in range(i + 1, i + m.end() - 1):
                    out[k] = " "
                i += m.end()
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


class RustFn:
    __slots__ = ("name", "path", "line", "body", "inline", "raw")


def load_rust_functions():
    """name -> list of RustFn from rust/melee_*/src/tu/*.rs (and the rest of each crate's src for helpers)."""
    global _rust_cache
    if _rust_cache is not None:
        return _rust_cache
    fns = {}
    for crate in sorted(os.listdir(RUST_DIR)):
        if not crate.startswith("melee_"):
            continue
        src = os.path.join(RUST_DIR, crate, "src")
        for root, _, files in os.walk(src):
            for fn in files:
                if not fn.endswith(".rs"):
                    continue
                path = os.path.join(root, fn)
                raw = open(path, encoding="utf-8", errors="replace").read()
                s = _rust_strip(raw)
                rel = os.path.relpath(path, REPO).replace("\\", "/")
                for m in re.finditer(r"\bfn\s+(\w+)\s*(<[^>{]*>)?\s*\(", s):
                    po = m.end() - 1
                    pc = _match_paren(s, po)
                    if pc < 0:
                        continue
                    b0 = s.find("{", pc)
                    semi = s.find(";", pc)
                    if b0 < 0 or (0 <= semi < b0) or "=>" in s[pc:b0] or "=" in s[pc:b0].replace("->", ""):
                        continue  # declaration, or an extern-shim macro arm `fn X(..) => unsafe { .. }`
                    b1 = _match_brace(s, b0, quotes=False)  # literals already blanked; ' also starts labels
                    if b1 < 0:
                        continue
                    r = RustFn()
                    r.name = m.group(1)
                    r.path = rel
                    r.line = s.count("\n", 0, m.start()) + 1
                    r.body = s[b0:b1 + 1]
                    r.raw = raw[m.start():b1 + 1]
                    pre = s[max(0, m.start() - 300):m.start()]
                    last = pre.rsplit("}", 1)[-1].rsplit(";", 1)[-1]
                    r.inline = "#[inline" in last
                    fns.setdefault(r.name, []).append(r)
    _rust_cache = fns
    return fns


_MACRO_RE = re.compile(r"(?:melee_core::)?(?:mod_hook|ct)!\s*\((?:[^()]|\((?:[^()]|\([^()]*\))*\))*\)\s*;?")


_FWD_CACHE = {}


def _is_forwarder(r, name):
    key = (r.path, r.line, name)
    v = _FWD_CACHE.get(key)
    if v is None:
        v = _FWD_CACHE[key] = _is_forwarder_uncached(r, name)
    return v


def _is_forwarder_uncached(r, name):
    if len(r.body) > 600:
        return False
    b = _MACRO_RE.sub(" ", r.body)
    b = re.sub(r"#\[[^\]]*\]", " ", b)
    return re.fullmatch(r"\{\s*(?:unsafe\s*\{\s*)?(?:return\s+)?[\w:]*\b%s\s*\([^;{}]*\)\s*;?\s*\}?\s*\}" % re.escape(name), b) is not None


def rust_tu_fn(fns, name, prefer=None, module=None):
    """The port's function of this name. Order: a path matching the call's module (`crate::pure::f` -> pure.rs),
    the caller's own file, the caller's crate (tu/ first), melee_core / melee_runtime, any tu/ file, anything.
    Forwarders (`fn X(..) { other::X(..) }`) lose against real bodies."""
    lst = fns.get(name) or []
    real = [r for r in lst if not _is_forwarder(r, name)]
    if real:
        lst = real
    if not lst:
        return None
    if module and module not in ("crate", "self", "super", "ext", "e", "st"):
        for r in lst:
            if r.path.endswith("/%s.rs" % module) or ("/%s/" % module) in r.path:
                return r
    if prefer:
        for r in lst:
            if r.path == prefer:
                return r
        crate = prefer.split("/")[1] if prefer.startswith("rust/") else None
        same = [r for r in lst if crate and r.path.split("/")[1] == crate]
        for r in same:
            if "/src/tu/" in r.path:
                return r
        if same:
            return same[0]
        for core in ("melee_core", "melee_runtime"):
            for r in lst:
                if r.path.split("/")[1] == core:
                    return r
    for r in lst:
        if "/src/tu/" in r.path:
            return r
    return lst[0]


def rust_port_of(fns, func):
    """The port of a DOL function: a same-named Rust function in the file named after the function's TU
    (melee/mp/mplib.c -> mplib.rs), else the largest same-named body that is not a forwarder."""
    lst = list(fns.get(func.name) or [])
    low = func.name.lower()
    if low != func.name:
        lst += fns.get(low) or []  # some ports are snake-case bodies behind a hook shim of the DOL name
    if not lst:
        return None
    if func.tu:
        stem = os.path.splitext(os.path.basename(func.tu))[0].lower()
        for r in lst:
            if os.path.splitext(os.path.basename(r.path))[0].lower() == stem and not _is_forwarder(r, r.name):
                return r
    real = [r for r in lst if not _is_forwarder(r, r.name) and "Some(f) => f(" not in r.body] or lst
    return max(real, key=lambda r: (len(r.body), "/src/tu/" in r.path))
