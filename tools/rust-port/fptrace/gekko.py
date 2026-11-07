#!/usr/bin/env python3
"""A small Gekko (GameCube PowerPC 750CL) interpreter for float tracing (docs/rust-port/notes/semantics.md, round 3).

  python tools/rust-port/fptrace/gekko.py DUMP.bin -o trace.tsv [--max N]

Runs the function a Dolphin oracle dump stopped at (MELEE_DOLPHIN_NHV1_DUMP; header: 'GDMP', version, hooked pc, frame,
gpr[32], ps0/ps1 bits [32][2], cr lr ctr xer fpscr msr, gqr[8], MEM1) until it returns (pc == the entry lr with the
entry stack pointer), interpreting every callee too, and logs each floating-point instruction as one TSV row:
  seq  pc  func  op  prec  inputs  out0  out1  flags
inputs are the double bit patterns of the operands in Gekko operand order (a, c, b for the multiply-add family), out0 /
out1 the destination's ps0 / ps1 bits after the instruction, prec `s` (single, rounded by ForceSingle), `d` (double),
`ps` (paired single), `cvt` (frsp, fctiw[z], loads/stores), flags `f25` when Force25Bit changed the frC operand.

Float semantics follow Dolphin's interpreter (Source/Core/Core/PowerPC/Interpreter: Interpreter_FPUtils.h,
Interpreter_FloatingPoint.cpp, Interpreter_Paired.cpp, Interpreter_LoadStorePaired.cpp; the estimate tables are the ones
melee_core::fp already ports from Common/FloatUtils.cpp). This file is a port of those parts and so is
GPL-2.0-or-later, like Dolphin. Exceptions (FPSCR enable bits) are not modelled: the game runs with them off.
Integer, branch and load/store instructions cover what the game's compiled code uses; anything else stops with
`unimplemented` and the pc (no disc bytes are printed)."""
import argparse, struct, sys
from fractions import Fraction
from pathlib import Path

import numpy as np

M64 = (1 << 64) - 1
SIGN = 1 << 63
EXP = 0x7FF << 52
FRAC = (1 << 52) - 1
QBIT = 1 << 51


def b2d(b):
    return struct.unpack("<d", struct.pack("<Q", b & M64))[0]


def d2b(d):
    return struct.unpack("<Q", struct.pack("<d", d))[0]


def isnan_b(b):
    return (b & EXP) == EXP and (b & FRAC) != 0


def is_snan(b):
    return isnan_b(b) and not (b & QBIT)


def quiet(b):
    return b | QBIT


def single_bits_to_double(f32b):
    return d2b(float(np.frombuffer(struct.pack("<I", f32b), dtype=np.float32)[0]))


def force_single(fpscr, b):
    """ForceSingle: round the double to single (RNE), as a double bit pattern of the float."""
    if fpscr & 4:  # NI
        if (b & ~SIGN & M64) < 0x3810000000000000:
            return b & SIGN
    with np.errstate(all="ignore"):
        x = np.float64(b2d(b)).astype(np.float32)
    if np.isnan(x):  # keep the payload the way the C++ cast does (truncate the low bits, keep the quiet bit)
        f = ((b >> 32) & 0x80000000) | 0x7F800000 | ((b >> 29) & 0x7FFFFF)
        return single_bits_to_double(f | 0x400000)
    return d2b(float(x))


def force25(b):
    e, f = b & EXP, b & FRAC
    if e == 0 and f != 0:
        shift = (64 - f.bit_length()) - (63 - 52)
        keep = (0xFFFFFFFFF8000000 >> shift) | (((1 << shift) - 1) << (64 - shift)) if shift else 0xFFFFFFFFF8000000
        keep &= M64
        rnd = 0x8000000 >> shift
        return ((b & keep) + (b & rnd)) & M64
    return ((b & 0xFFFFFFFFF8000000) + (b & 0x8000000)) & M64


def fma_exact(a, c, b):
    """IEEE fused multiply-add on doubles (std::fma): one rounding."""
    A, C, B = b2d(a), b2d(c), b2d(b)
    if not (np.isfinite(A) and np.isfinite(C) and np.isfinite(B)):
        with np.errstate(all="ignore"):
            return d2b(float(np.float64(A) * np.float64(C) + np.float64(B)))
    p = Fraction(A) * Fraction(C)
    s = p + Fraction(B)
    if s == 0:
        if p == 0 and B == 0:
            neg = (((a ^ c) & SIGN) != 0) and ((b & SIGN) != 0)
            return SIGN if neg else 0
        return 0  # exact cancellation, round to nearest: +0
    try:
        return d2b(float(s))
    except OverflowError:
        return EXP | (SIGN if s < 0 else 0)


def ni_arith(op, a, b):
    """NI_add / NI_sub / NI_mul / NI_div (NaN propagation as Dolphin's)."""
    A, B = b2d(a), b2d(b)
    with np.errstate(all="ignore"):
        r = {"+": np.float64(A) + np.float64(B), "-": np.float64(A) - np.float64(B),
             "*": np.float64(A) * np.float64(B), "/": np.float64(A) / np.float64(B)}[op]
    rb = d2b(float(r))
    if isnan_b(rb):
        if isnan_b(a):
            return quiet(a)
        if isnan_b(b):
            return quiet(b)
        return 0x7FF8000000000000
    return rb


def ni_madd(a, c, b, sub=False):
    bb = b ^ SIGN if sub else b
    r = fma_exact(a, c, bb)
    if isnan_b(r):
        for x in (a, b, c):
            if isnan_b(x):
                return quiet(x)
        return 0x7FF8000000000000
    return r


def neg_unless_nan(r):
    return r if isnan_b(r) else r ^ SIGN


def convert_to_single(x):
    e = (x >> 52) & 0x7FF
    if e > 896 or (x & ~SIGN & M64) == 0:
        return ((x >> 32) & 0xC0000000) | ((x >> 29) & 0x3FFFFFFF)
    if e >= 874:
        t = 0x80000000 | ((x & FRAC) >> 21)
        t >>= (905 - e)
        return t | ((x >> 32) & 0x80000000)
    return ((x >> 32) & 0xC0000000) | ((x >> 29) & 0x3FFFFFFF)


def convert_to_single_ftz(x):
    e = (x >> 52) & 0x7FF
    if e > 896 or (x & ~SIGN & M64) == 0:
        return ((x >> 32) & 0xC0000000) | ((x >> 29) & 0x3FFFFFFF)
    return (x >> 32) & 0x80000000


def convert_to_double(v):
    x = v
    e = (x >> 23) & 0xFF
    f = x & 0x7FFFFF
    if 0 < e < 255:
        y = 0 if (e >> 7) else 1
        z = (y << 61) | (y << 60) | (y << 59)
        return ((x & 0xC0000000) << 32) | z | ((x & 0x3FFFFFFF) << 29)
    if e == 0 and f != 0:
        e = 1023 - 126
        while True:
            f <<= 1
            e -= 1
            if f & 0x800000:
                break
        return ((x & 0x80000000) << 32) | (e << 52) | ((f & 0x7FFFFF) << 29)
    y = e >> 7
    z = (y << 61) | (y << 60) | (y << 59)
    return ((x & 0xC0000000) << 32) | z | ((x & 0x3FFFFFFF) << 29)


FRSQRTE = [(0x1a7e800, -0x568), (0x17cb800, -0x4f3), (0x1552800, -0x48d), (0x130c000, -0x435), (0x10f2000, -0x3e7),
           (0x0eff000, -0x3a2), (0x0d2e000, -0x365), (0x0b7c000, -0x32e), (0x09e5000, -0x2fc), (0x0867000, -0x2d0),
           (0x06ff000, -0x2a8), (0x05ab800, -0x283), (0x046a000, -0x261), (0x0339800, -0x243), (0x0218800, -0x226),
           (0x0105800, -0x20b), (0x3ffa000, -0x7a4), (0x3c29000, -0x700), (0x38aa000, -0x670), (0x3572000, -0x5f2),
           (0x3279000, -0x584), (0x2fb7000, -0x524), (0x2d26000, -0x4cc), (0x2ac0000, -0x47e), (0x2881000, -0x43a),
           (0x2665000, -0x3fa), (0x2468000, -0x3c2), (0x2287000, -0x38e), (0x20c1000, -0x35e), (0x1f12000, -0x332),
           (0x1d79000, -0x30a), (0x1bf4000, -0x2e6)]
FRES = [(0x7ff800, 0x3e1), (0x783800, 0x3a7), (0x70ea00, 0x371), (0x6a0800, 0x340), (0x638800, 0x313),
        (0x5d6200, 0x2ea), (0x579000, 0x2c4), (0x520800, 0x2a0), (0x4cc800, 0x27f), (0x47ca00, 0x261),
        (0x430800, 0x245), (0x3e8000, 0x22a), (0x3a2c00, 0x212), (0x360800, 0x1fb), (0x321400, 0x1e5),
        (0x2e4a00, 0x1d1), (0x2aa800, 0x1be), (0x272c00, 0x1ac), (0x23d600, 0x19b), (0x209e00, 0x18b),
        (0x1d8800, 0x17c), (0x1a9000, 0x16e), (0x17ae00, 0x15b), (0x14f800, 0x15b), (0x124400, 0x143),
        (0x0fbe00, 0x143), (0x0d3800, 0x12d), (0x0ade00, 0x12d), (0x088400, 0x11a), (0x065000, 0x11a),
        (0x041c00, 0x108), (0x020c00, 0x106)]


def frsqrte(b):
    mant, sign, e = b & FRAC, b & SIGN, b & EXP
    if mant == 0 and e == 0:
        return EXP | sign
    if e == EXP:
        if mant == 0:
            return 0x7FF8000000000000 if sign else 0
        return quiet(b)
    if sign:
        return 0x7FF8000000000000
    if e == 0:
        while True:
            e -= 1 << 52
            mant <<= 1
            if mant & (1 << 52):
                break
        mant &= FRAC
        e += 1 << 52
    lsb = e & (1 << 52)
    d = e - (0x3FE << 52)
    half = abs(d) // 2 * (1 if d >= 0 else -1)  # i64 division truncates toward zero
    e = ((0x3FF << 52) - half) & EXP
    r = sign | e
    i = (lsb | mant) >> 37
    base, dec = FRSQRTE[i // 2048]
    return (r | (((base + dec * (i % 2048)) & 0xFFFFFFFF) << 26)) & M64


def fres(b):
    mant, sign, e = b & FRAC, b & SIGN, b & EXP
    if mant == 0 and e == 0:
        return EXP | sign
    if e == EXP:
        if mant == 0:
            return sign
        return quiet(b)
    if e < (895 << 52):
        return sign | 0x47EFFFFFE0000000
    if e >= (1149 << 52):
        return sign
    e = (0x7FD << 52) - e
    i = mant >> 37
    base, dec = FRES[i // 1024]
    return (sign | e | ((base - (dec * (i % 1024) + 1) // 2) << 29)) & M64


def s32(x):
    x &= 0xFFFFFFFF
    return x - (1 << 32) if x & 0x80000000 else x


def rotl(x, n):
    n &= 31
    x &= 0xFFFFFFFF
    return ((x << n) | (x >> (32 - n))) & 0xFFFFFFFF if n else x


def mask(mb, me):
    m = 0
    i = mb
    while True:
        m |= 1 << (31 - i)
        if i == me:
            break
        i = (i + 1) & 31
    return m


class Unimplemented(Exception):
    pass


class Gekko:
    def __init__(self, dump, symbols=None):
        d = Path(dump).read_bytes()
        hdr = struct.unpack("<8I", d[:32])
        if hdr[0] != 0x504D4447:
            raise SystemExit("not a GDMP dump")
        self.entry, self.frame = hdr[2], s32(hdr[3])
        o = 32
        self.gpr = list(struct.unpack("<32I", d[o:o + 128])); o += 128
        self.ps0, self.ps1 = [], []
        for i in range(32):
            a, b = struct.unpack("<QQ", d[o:o + 16]); o += 16
            self.ps0.append(a); self.ps1.append(b)
        sr = struct.unpack("<16I", d[o:o + 64]); o += 64
        self.cr, self.lr, self.ctr, xer, self.fpscr, self.msr = sr[:6]
        self.ca = (xer >> 29) & 1
        self.xer_rest = xer & ~(1 << 29)
        self.gqr = list(sr[8:16])
        n = struct.unpack("<I", d[o:o + 4])[0]; o += 4
        self.mem = bytearray(d[o:o + n]); o += n
        self.ram = n
        self.aram = None
        if hdr[1] >= 2:  # version 2 dumps carry ARAM after MEM1
            an = struct.unpack("<I", d[o:o + 4])[0]; o += 4
            self.aram = bytearray(d[o:o + an])
        # hardware registers (0xCC000000: GX FIFO, DSP, PI...) and the locked cache (0xE0000000) as plain scratch
        # bytes: reads see 0 or the last write. Float results never come from them; a call that polls a register
        # for a state change would spin, which --max catches.
        self.mem += bytearray(0x10000 + 0x4000)
        self.mmio = 0
        self.pc = self.entry
        self.trace, self.seq = [], 0
        self.syms = symbols or []

    # ---- memory (big-endian guest memory, MEM1 only; cached 0x8 and uncached 0xC mirrors)
    def ea(self, a):
        a &= 0xFFFFFFFF
        if 0x80000000 <= a < 0x80000000 + self.ram:
            return a - 0x80000000
        if 0xC0000000 <= a < 0xC0000000 + self.ram:
            return a - 0xC0000000
        if 0xCC000000 <= a < 0xCC010000:
            self.mmio += 1
            return self.ram + (a - 0xCC000000)
        if 0xE0000000 <= a < 0xE0004000:
            return self.ram + 0x10000 + (a - 0xE0000000)
        raise Unimplemented(f"memory access outside MEM1 at {a:08X} (pc {self.pc:08X})")

    def r8(self, a): return self.mem[self.ea(a)]
    def r16(self, a): i = self.ea(a); return (self.mem[i] << 8) | self.mem[i + 1]
    def r32(self, a): i = self.ea(a); return int.from_bytes(self.mem[i:i + 4], "big")
    def r64(self, a): i = self.ea(a); return int.from_bytes(self.mem[i:i + 8], "big")
    def w8(self, a, v): self.mem[self.ea(a)] = v & 0xFF
    def w16(self, a, v): i = self.ea(a); self.mem[i:i + 2] = (v & 0xFFFF).to_bytes(2, "big")
    def w32(self, a, v): i = self.ea(a); self.mem[i:i + 4] = (v & 0xFFFFFFFF).to_bytes(4, "big")
    def w64(self, a, v): i = self.ea(a); self.mem[i:i + 8] = (v & M64).to_bytes(8, "big")

    # ---- CR helpers
    def setcr(self, field, v4):
        sh = 28 - 4 * field
        self.cr = (self.cr & ~(0xF << sh)) | ((v4 & 0xF) << sh)

    def cr0(self, r):
        r = s32(r)
        so = (self.xer_rest >> 31) & 1
        self.setcr(0, (8 if r < 0 else 4 if r > 0 else 2) | so)

    def crbit(self, i):
        return (self.cr >> (31 - i)) & 1

    # ---- float logging
    def log(self, op, prec, ins, fd, flags=""):
        self.seq += 1
        self.trace.append((self.seq, self.pc, op, prec, ins, self.ps0[fd], self.ps1[fd], flags))

    def fill(self, fd, b):
        self.ps0[fd] = b; self.ps1[fd] = b

    def func_of(self, pc):
        lo, hi = 0, len(self.syms)
        while lo < hi:
            m = (lo + hi) // 2
            if self.syms[m][0] <= pc:
                lo = m + 1
            else:
                hi = m
        if lo and pc < self.syms[lo - 1][0] + self.syms[lo - 1][1]:
            return self.syms[lo - 1][2]
        return "?"

    # ---- run
    def run(self, max_steps=50_000_000):
        ret_pc, ret_sp = self.lr, self.gpr[1]
        steps = 0
        arq = next((a for a, _n, name in self.syms if name == "ARQPostRequest"), None) if self.syms and             len(self.syms[0]) > 2 else None
        while not (self.pc == ret_pc and self.gpr[1] == ret_sp):
            steps += 1
            if steps > max_steps:
                raise Unimplemented(f"step budget exhausted at {self.pc:08X}")
            if self.pc == arq:
                self.hle_arq()
                continue
            self.step(self.r32(self.pc))
        return steps

    def hle_arq(self):
        """ARQPostRequest(req, owner, type, prio, src, dst, len, cb): do the ARAM DMA at once and enter the
        callback with the caller's lr (on the console the DMA-done interrupt calls it later; callers that need the
        data spin on the state the callback sets, as lbArq_80014BD0 does)."""
        g = self.gpr
        req, typ, src, dst, n, cb = g[3], g[5], g[7], g[8], g[9], g[10]
        if self.aram is None:
            raise Unimplemented(f"ARAM DMA at {self.lr:08X} with a version 1 dump (no ARAM)")
        if typ == 1:  # ARAM -> MRAM
            m = (dst & 0x01FFFFFF)
            self.mem[m:m + n] = self.aram[src:src + n]
        else:
            m = (src & 0x01FFFFFF)
            self.aram[dst:dst + n] = self.mem[m:m + n]
        if cb:
            g[3] = req
            self.pc = cb
        else:
            self.pc = self.lr

    def step(self, w):
        op = w >> 26
        rd, ra, rb = (w >> 21) & 31, (w >> 16) & 31, (w >> 11) & 31
        simm = w & 0xFFFF
        simm = simm - 0x10000 if simm & 0x8000 else simm
        uimm = w & 0xFFFF
        g = self.gpr
        npc = (self.pc + 4) & 0xFFFFFFFF
        def ra0():
            return g[ra] if ra else 0
        if op == 14:  # addi
            g[rd] = (ra0() + simm) & 0xFFFFFFFF
        elif op == 15:  # addis
            g[rd] = (ra0() + (simm << 16)) & 0xFFFFFFFF
        elif op == 32: g[rd] = self.r32(ra0() + simm)
        elif op == 33: a = g[ra] + simm; g[rd] = self.r32(a); g[ra] = a & 0xFFFFFFFF
        elif op == 34: g[rd] = self.r8(ra0() + simm)
        elif op == 35: a = g[ra] + simm; g[rd] = self.r8(a); g[ra] = a & 0xFFFFFFFF
        elif op == 40: g[rd] = self.r16(ra0() + simm)
        elif op == 41: a = g[ra] + simm; g[rd] = self.r16(a); g[ra] = a & 0xFFFFFFFF
        elif op == 42: v = self.r16(ra0() + simm); g[rd] = (v - 0x10000 if v & 0x8000 else v) & 0xFFFFFFFF
        elif op == 43:
            a = g[ra] + simm; v = self.r16(a); g[rd] = (v - 0x10000 if v & 0x8000 else v) & 0xFFFFFFFF; g[ra] = a & 0xFFFFFFFF
        elif op == 36: self.w32(ra0() + simm, g[rd])
        elif op == 37: a = (g[ra] + simm) & 0xFFFFFFFF; self.w32(a, g[rd]); g[ra] = a
        elif op == 38: self.w8(ra0() + simm, g[rd])
        elif op == 39: a = (g[ra] + simm) & 0xFFFFFFFF; self.w8(a, g[rd]); g[ra] = a
        elif op == 44: self.w16(ra0() + simm, g[rd])
        elif op == 45: a = (g[ra] + simm) & 0xFFFFFFFF; self.w16(a, g[rd]); g[ra] = a
        elif op == 46:  # lmw
            a = ra0() + simm
            for r in range(rd, 32):
                g[r] = self.r32(a); a += 4
        elif op == 47:  # stmw
            a = ra0() + simm
            for r in range(rd, 32):
                self.w32(a, g[r]); a += 4
        elif op == 24: g[ra] = g[rd] | uimm
        elif op == 25: g[ra] = g[rd] | (uimm << 16)
        elif op == 26: g[ra] = g[rd] ^ uimm
        elif op == 27: g[ra] = g[rd] ^ (uimm << 16)
        elif op == 28: g[ra] = g[rd] & uimm; self.cr0(g[ra])
        elif op == 29: g[ra] = g[rd] & (uimm << 16); self.cr0(g[ra])
        elif op == 7: g[rd] = (s32(g[ra]) * simm) & 0xFFFFFFFF
        elif op == 8:  # subfic
            r = (simm & 0xFFFFFFFF) + ((~g[ra]) & 0xFFFFFFFF) + 1
            g[rd] = r & 0xFFFFFFFF; self.ca = r >> 32
        elif op in (12, 13):  # addic(.)
            r = g[ra] + (simm & 0xFFFFFFFF)
            g[rd] = r & 0xFFFFFFFF; self.ca = r >> 32
            if op == 13: self.cr0(g[rd])
        elif op == 11:  # cmpi
            a, b = s32(g[ra]), simm
            self.setcr(rd >> 2, (8 if a < b else 4 if a > b else 2) | ((self.xer_rest >> 31) & 1))
        elif op == 10:  # cmpli
            a, b = g[ra], uimm
            self.setcr(rd >> 2, (8 if a < b else 4 if a > b else 2) | ((self.xer_rest >> 31) & 1))
        elif op == 18:  # b
            li = w & 0x03FFFFFC
            if li & 0x02000000: li -= 0x04000000
            tgt = (li if (w & 2) else self.pc + li) & 0xFFFFFFFF
            if w & 1: self.lr = npc
            npc = tgt
        elif op == 16:  # bc
            bd = w & 0xFFFC
            if bd & 0x8000: bd -= 0x10000
            if self.cond(rd, ra):
                npc = (bd if (w & 2) else self.pc + bd) & 0xFFFFFFFF
            if w & 1: self.lr = (self.pc + 4) & 0xFFFFFFFF
        elif op == 19: npc = self.op19(w, rd, ra, rb, npc)
        elif op == 20:  # rlwimi
            sh, mb, me = rb, (w >> 6) & 31, (w >> 1) & 31
            m = mask(mb, me)
            g[ra] = (rotl(g[rd], sh) & m) | (g[ra] & ~m & 0xFFFFFFFF)
            if w & 1: self.cr0(g[ra])
        elif op == 21:  # rlwinm
            g[ra] = rotl(g[rd], rb) & mask((w >> 6) & 31, (w >> 1) & 31)
            if w & 1: self.cr0(g[ra])
        elif op == 23:  # rlwnm
            g[ra] = rotl(g[rd], g[rb] & 31) & mask((w >> 6) & 31, (w >> 1) & 31)
            if w & 1: self.cr0(g[ra])
        elif op == 31: self.op31(w, rd, ra, rb)
        elif op in (48, 49, 50, 51, 52, 53, 54, 55): self.fls(op, rd, ra, simm)
        elif op in (56, 57, 60, 61): self.psq(op, w, rd, ra)
        elif op == 59: self.op59(w, rd, ra, rb)
        elif op == 63: self.op63(w, rd, ra, rb)
        elif op == 4: self.op4(w, rd, ra, rb)
        else:
            raise Unimplemented(f"primary opcode {op} at {self.pc:08X}")
        self.pc = npc

    def cond(self, bo, bi):
        if not (bo & 4):
            self.ctr = (self.ctr - 1) & 0xFFFFFFFF
        ctr_ok = (bo & 4) or ((self.ctr != 0) ^ bool(bo & 2))
        cond_ok = (bo & 16) or (self.crbit(bi) == ((bo >> 3) & 1))
        return ctr_ok and cond_ok

    def op19(self, w, bo, bi, rb, npc):
        xo = (w >> 1) & 0x3FF
        if xo == 16:  # bclr
            tgt = self.lr & ~3
            if self.cond(bo, bi):
                npc = tgt
            if w & 1: self.lr = (self.pc + 4) & 0xFFFFFFFF
        elif xo == 528:  # bcctr
            if (bo & 16) or (self.crbit(bi) == ((bo >> 3) & 1)):
                npc = self.ctr & ~3
            if w & 1: self.lr = (self.pc + 4) & 0xFFFFFFFF
        elif xo in (257, 449, 193, 129, 225, 33, 289, 417):  # crand cror crxor crandc crnand crnor creqv crorc
            a, b = self.crbit(bi), self.crbit(rb)
            r = {257: a & b, 449: a | b, 193: a ^ b, 129: a & (1 - b), 225: 1 - (a & b), 33: 1 - (a | b),
                 289: 1 - (a ^ b), 417: a | (1 - b)}[xo]
            self.cr = (self.cr & ~(1 << (31 - bo))) | (r << (31 - bo))
        elif xo == 0:  # mcrf
            self.setcr(bo >> 2, (self.cr >> (28 - 4 * (bi >> 2))) & 0xF)
        elif xo in (150, 50):  # isync, rfi
            pass
        else:
            raise Unimplemented(f"op19 xo {xo} at {self.pc:08X}")
        return npc

    def op31(self, w, rd, ra, rb):
        g = self.gpr
        xo = (w >> 1) & 0x3FF
        rc = w & 1
        xo9 = xo & 0x1FF
        so = (self.xer_rest >> 31) & 1
        def ea():
            return ((g[ra] if ra else 0) + g[rb]) & 0xFFFFFFFF
        def ea_u():
            return (g[ra] + g[rb]) & 0xFFFFFFFF
        res = None
        if xo == 0 or xo == 32:  # cmp / cmpl
            a, b = (s32(g[ra]), s32(g[rb])) if xo == 0 else (g[ra], g[rb])
            self.setcr(rd >> 2, (8 if a < b else 4 if a > b else 2) | so); return
        if xo9 == 266: res = (g[ra] + g[rb]) & 0xFFFFFFFF; g[rd] = res
        elif xo9 == 40: res = (g[rb] - g[ra]) & 0xFFFFFFFF; g[rd] = res
        elif xo9 == 10: r = g[ra] + g[rb]; res = r & 0xFFFFFFFF; self.ca = r >> 32; g[rd] = res
        elif xo9 == 8: r = g[rb] + ((~g[ra]) & 0xFFFFFFFF) + 1; res = r & 0xFFFFFFFF; self.ca = r >> 32; g[rd] = res
        elif xo9 == 138: r = g[ra] + g[rb] + self.ca; res = r & 0xFFFFFFFF; self.ca = r >> 32; g[rd] = res
        elif xo9 == 136: r = ((~g[ra]) & 0xFFFFFFFF) + g[rb] + self.ca; res = r & 0xFFFFFFFF; self.ca = r >> 32; g[rd] = res
        elif xo9 == 202: r = g[ra] + self.ca; res = r & 0xFFFFFFFF; self.ca = r >> 32; g[rd] = res
        elif xo9 == 200: r = ((~g[ra]) & 0xFFFFFFFF) + self.ca; res = r & 0xFFFFFFFF; self.ca = r >> 32; g[rd] = res
        elif xo9 == 234: r = g[ra] + self.ca + 0xFFFFFFFF; res = r & 0xFFFFFFFF; self.ca = r >> 32; g[rd] = res
        elif xo9 == 232: r = ((~g[ra]) & 0xFFFFFFFF) + self.ca + 0xFFFFFFFF; res = r & 0xFFFFFFFF; self.ca = r >> 32; g[rd] = res
        elif xo9 == 104: res = (-g[ra]) & 0xFFFFFFFF; g[rd] = res
        elif xo9 == 235: res = (s32(g[ra]) * s32(g[rb])) & 0xFFFFFFFF; g[rd] = res
        elif xo9 == 75: res = ((s32(g[ra]) * s32(g[rb])) >> 32) & 0xFFFFFFFF; g[rd] = res
        elif xo9 == 11: res = ((g[ra] * g[rb]) >> 32) & 0xFFFFFFFF; g[rd] = res
        elif xo9 == 491:  # divw (Dolphin: 0 or -1 on overflow / divide by zero)
            a, b = s32(g[ra]), s32(g[rb])
            if b == 0 or (a == -0x80000000 and b == -1):
                res = 0xFFFFFFFF if a < 0 else 0
            else:
                q = abs(a) // abs(b)
                res = (q if (a < 0) == (b < 0) else -q) & 0xFFFFFFFF
            g[rd] = res
        elif xo9 == 459:
            res = 0 if g[rb] == 0 else g[ra] // g[rb]; g[rd] = res
        else:
            res = None
        if res is not None:
            if rc: self.cr0(res)
            return
        # logical / shifts: rS = rd, result in ra
        s = g[rd]
        if xo == 28: g[ra] = s & g[rb]
        elif xo == 60: g[ra] = s & ~g[rb] & 0xFFFFFFFF
        elif xo == 444: g[ra] = s | g[rb]
        elif xo == 412: g[ra] = (s | ~g[rb]) & 0xFFFFFFFF
        elif xo == 316: g[ra] = s ^ g[rb]
        elif xo == 124: g[ra] = ~(s | g[rb]) & 0xFFFFFFFF
        elif xo == 476: g[ra] = ~(s & g[rb]) & 0xFFFFFFFF
        elif xo == 284: g[ra] = ~(s ^ g[rb]) & 0xFFFFFFFF
        elif xo == 24: n = g[rb]; g[ra] = 0 if n & 0x20 else (s << (n & 31)) & 0xFFFFFFFF
        elif xo == 536: n = g[rb]; g[ra] = 0 if n & 0x20 else s >> (n & 31)
        elif xo == 792:
            n = g[rb]
            if n & 0x20:
                g[ra] = 0xFFFFFFFF if s & 0x80000000 else 0; self.ca = 1 if s & 0x80000000 else 0
            else:
                n &= 31; v = s32(s)
                g[ra] = (v >> n) & 0xFFFFFFFF
                self.ca = 1 if (v < 0 and n and (s << (32 - n)) & 0xFFFFFFFF) else 0
        elif xo == 824:
            n = rb; v = s32(s)
            g[ra] = (v >> n) & 0xFFFFFFFF
            self.ca = 1 if (v < 0 and n and (s << (32 - n)) & 0xFFFFFFFF) else 0
        elif xo == 26: g[ra] = 32 - s.bit_length()
        elif xo == 922: g[ra] = ((s & 0xFFFF) - 0x10000 if s & 0x8000 else s & 0xFFFF) & 0xFFFFFFFF
        elif xo == 954: g[ra] = ((s & 0xFF) - 0x100 if s & 0x80 else s & 0xFF) & 0xFFFFFFFF
        # loads / stores
        elif xo == 23: g[rd] = self.r32(ea())
        elif xo == 55: a = ea_u(); g[rd] = self.r32(a); g[ra] = a
        elif xo == 87: g[rd] = self.r8(ea())
        elif xo == 119: a = ea_u(); g[rd] = self.r8(a); g[ra] = a
        elif xo == 279: g[rd] = self.r16(ea())
        elif xo == 311: a = ea_u(); g[rd] = self.r16(a); g[ra] = a
        elif xo == 343: v = self.r16(ea()); g[rd] = (v - 0x10000 if v & 0x8000 else v) & 0xFFFFFFFF
        elif xo == 375: a = ea_u(); v = self.r16(a); g[rd] = (v - 0x10000 if v & 0x8000 else v) & 0xFFFFFFFF; g[ra] = a
        elif xo == 534: g[rd] = int.from_bytes(self.r32(ea()).to_bytes(4, "big"), "little")
        elif xo == 790: g[rd] = int.from_bytes(self.r16(ea()).to_bytes(2, "big"), "little")
        elif xo == 151: self.w32(ea(), g[rd])
        elif xo == 183: a = ea_u(); self.w32(a, g[rd]); g[ra] = a
        elif xo == 215: self.w8(ea(), g[rd])
        elif xo == 247: a = ea_u(); self.w8(a, g[rd]); g[ra] = a
        elif xo == 407: self.w16(ea(), g[rd])
        elif xo == 439: a = ea_u(); self.w16(a, g[rd]); g[ra] = a
        elif xo == 662: self.w32(ea(), int.from_bytes((g[rd] & 0xFFFFFFFF).to_bytes(4, "little"), "big"))
        elif xo == 918: self.w16(ea(), int.from_bytes((g[rd] & 0xFFFF).to_bytes(2, "little"), "big"))
        elif xo in (535, 567, 599, 631, 663, 695, 727, 759, 983):
            a = ea_u() if xo in (567, 631, 695, 759) else ea()
            if xo in (535, 567): self.fill(rd, convert_to_double(self.r32(a)))
            elif xo in (599, 631): self.ps0[rd] = self.r64(a)
            elif xo in (663, 695): self.w32(a, convert_to_single(self.ps0[rd]))
            elif xo in (727, 759): self.w64(a, self.ps0[rd])
            elif xo == 983: self.w32(a, self.ps0[rd] & 0xFFFFFFFF)
            if xo in (567, 631, 695, 759): g[ra] = a
            return
        elif xo == 339:  # mfspr
            spr = ((w >> 16) & 31) | (((w >> 11) & 31) << 5)
            g[rd] = self.mfspr(spr)
        elif xo == 467:
            spr = ((w >> 16) & 31) | (((w >> 11) & 31) << 5)
            self.mtspr(spr, g[rd])
        elif xo == 371: g[rd] = 0  # mftb: time base is not part of the replayed state
        elif xo == 19: g[rd] = self.cr
        elif xo == 144:
            crm = (w >> 12) & 0xFF
            m = 0
            for i in range(8):
                if crm & (0x80 >> i): m |= 0xF << (28 - 4 * i)
            self.cr = (self.cr & ~m) | (g[rd] & m)
        elif xo == 83: g[rd] = self.msr
        elif xo == 146: self.msr = g[rd]
        elif xo in (86, 54, 278, 246, 470, 982, 598, 854, 1014):  # dcbf dcbst dcbt dcbtst dcbi icbi sync eieio dcbz
            if xo == 1014:
                a = ea() & ~31
                for k in range(0, 32, 4): self.w32(a + k, 0)
            return
        else:
            raise Unimplemented(f"op31 xo {xo} at {self.pc:08X}")
        if rc: self.cr0(g[ra])

    def mfspr(self, spr):
        if spr == 8: return self.lr
        if spr == 9: return self.ctr
        if spr == 1: return self.xer_rest | (self.ca << 29)
        if 912 <= spr <= 919: return self.gqr[spr - 912]
        if spr in (268, 269): return 0
        return 0

    def mtspr(self, spr, v):
        if spr == 8: self.lr = v
        elif spr == 9: self.ctr = v
        elif spr == 1: self.ca = (v >> 29) & 1; self.xer_rest = v & ~(1 << 29)
        elif 912 <= spr <= 919: self.gqr[spr - 912] = v

    def fls(self, op, fd, ra, simm):
        base = self.gpr[ra] if (ra or op in (49, 51, 53, 55)) else 0
        a = (base + simm) & 0xFFFFFFFF
        if op in (48, 49):
            self.fill(fd, convert_to_double(self.r32(a)))
        elif op in (50, 51):
            self.ps0[fd] = self.r64(a)
        elif op in (52, 53):
            self.w32(a, convert_to_single(self.ps0[fd]))
        else:
            self.w64(a, self.ps0[fd])
        if op in (49, 51, 53, 55):
            self.gpr[ra] = a

    # ---- paired-single quantised loads / stores
    def psq(self, op, w, fd, ra):
        simm = w & 0xFFF
        if simm & 0x800: simm -= 0x1000
        W, I = (w >> 15) & 1, (w >> 12) & 7
        a = ((self.gpr[ra] if (ra or op in (57, 61)) else 0) + simm) & 0xFFFFFFFF
        if op in (56, 57): self.dequant(a, I, fd, W)
        else: self.quant(a, I, fd, W)
        if op in (57, 61): self.gpr[ra] = a

    def dequant(self, a, I, fd, W):
        gq = self.gqr[I]
        typ, scale = (gq >> 16) & 7, (gq >> 24) & 0x3F
        if typ == 0:
            if W: p0, p1 = convert_to_double(self.r32(a)), d2b(1.0)
            else: p0, p1 = convert_to_double(self.r32(a)), convert_to_double(self.r32(a + 4))
        else:
            size = {4: 1, 6: 1, 5: 2, 7: 2}[typ]
            signed = typ in (6, 7)
            mul = np.float32(2.0 ** (-scale if scale < 32 else 64 - scale))
            def rd(x):
                v = self.r8(x) if size == 1 else self.r16(x)
                if signed and v >> (8 * size - 1): v -= 1 << (8 * size)
                return d2b(float(np.float32(v) * mul))
            p0 = rd(a)
            p1 = d2b(1.0) if W else rd(a + size)
        self.ps0[fd], self.ps1[fd] = p0, p1
        self.log("psq_l", "cvt", (), fd)

    def quant(self, a, I, fs, W):
        gq = self.gqr[I]
        typ, scale = gq & 7, (gq >> 8) & 0x3F
        if typ == 0:
            self.w32(a, convert_to_single_ftz(self.ps0[fs]))
            if not W: self.w32(a + 4, convert_to_single_ftz(self.ps1[fs]))
            return
        size = {4: 1, 6: 1, 5: 2, 7: 2}[typ]
        signed = typ in (6, 7)
        lo, hi = ((-(1 << (8 * size - 1)), (1 << (8 * size - 1)) - 1) if signed else (0, (1 << (8 * size)) - 1))
        mul = np.float32(2.0 ** (scale if scale < 32 else scale - 64))
        def cv(b):
            with np.errstate(all="ignore"):
                f = np.float32(b2d(b)) * mul
            f = min(max(f, np.float32(lo)), np.float32(hi))
            v = int(f) & ((1 << (8 * size)) - 1)
            return v
        v0 = cv(self.ps0[fs])
        (self.w8 if size == 1 else self.w16)(a, v0)
        if not W:
            (self.w8 if size == 1 else self.w16)(a + size, cv(self.ps1[fs]))

    # ---- float arithmetic
    def op59(self, w, fd, fa, fb):
        fc, xo = (w >> 6) & 31, (w >> 1) & 31
        P, F = self.ps0, self.fpscr
        a, b, c = P[fa], P[fb], P[fc]
        flags = ""
        if xo in (25, 28, 29, 30, 31):
            c2 = force25(c)
            if c2 != c: flags = "f25"
        if xo == 21: r = force_single(F, ni_arith("+", a, b)); ins = (a, b); name = "fadds"
        elif xo == 20: r = force_single(F, ni_arith("-", a, b)); ins = (a, b); name = "fsubs"
        elif xo == 25: r = force_single(F, ni_arith("*", a, c2)); ins = (a, c); name = "fmuls"
        elif xo == 18: r = force_single(F, ni_arith("/", a, b)); ins = (a, b); name = "fdivs"
        elif xo == 29: r = force_single(F, ni_madd(a, c2, b)); ins = (a, c, b); name = "fmadds"
        elif xo == 28: r = force_single(F, ni_madd(a, c2, b, True)); ins = (a, c, b); name = "fmsubs"
        elif xo == 31: r = neg_unless_nan(force_single(F, ni_madd(a, c2, b))); ins = (a, c, b); name = "fnmadds"
        elif xo == 30: r = neg_unless_nan(force_single(F, ni_madd(a, c2, b, True))); ins = (a, c, b); name = "fnmsubs"
        elif xo == 24: r = fres(b); ins = (b,); name = "fres"
        else:
            raise Unimplemented(f"op59 xo {xo} at {self.pc:08X}")
        self.fill(fd, r)
        self.log(name, "s", ins, fd, flags)

    def fcmp(self, crf, a, b):
        A, B = b2d(a), b2d(b)
        if isnan_b(a) or isnan_b(b): v = 1
        elif A < B: v = 8
        elif A > B: v = 4
        else: v = 2
        self.fpscr = (self.fpscr & ~0xF000) | (v << 12)
        self.setcr(crf, v)

    def op63(self, w, fd, fa, fb):
        fc, xo5, xo = (w >> 6) & 31, (w >> 1) & 31, (w >> 1) & 0x3FF
        P = self.ps0
        a, b, c = P[fa], P[fb], P[fc]
        if xo5 in (18, 20, 21, 23, 25, 26, 28, 29, 30, 31):
            if xo5 == 21: r = ni_arith("+", a, b); ins = (a, b); name = "fadd"
            elif xo5 == 20: r = ni_arith("-", a, b); ins = (a, b); name = "fsub"
            elif xo5 == 25: r = ni_arith("*", a, c); ins = (a, c); name = "fmul"
            elif xo5 == 18: r = ni_arith("/", a, b); ins = (a, b); name = "fdiv"
            elif xo5 == 29: r = ni_madd(a, c, b); ins = (a, c, b); name = "fmadd"
            elif xo5 == 28: r = ni_madd(a, c, b, True); ins = (a, c, b); name = "fmsub"
            elif xo5 == 31: r = neg_unless_nan(ni_madd(a, c, b)); ins = (a, c, b); name = "fnmadd"
            elif xo5 == 30: r = neg_unless_nan(ni_madd(a, c, b, True)); ins = (a, c, b); name = "fnmsub"
            elif xo5 == 23:
                r = c if (b2d(a) >= -0.0 and not isnan_b(a)) else b; ins = (a, c, b); name = "fsel"
            elif xo5 == 26: r = frsqrte(b); ins = (b,); name = "frsqrte"
            P[fd] = r
            self.log(name, "d", ins, fd)
            return
        if xo in (0, 32): self.fcmp(fd >> 2, a, b); return
        if xo == 12:
            r = force_single(self.fpscr, b); self.fill(fd, r); self.log("frsp", "cvt", (b,), fd); return
        if xo in (14, 15):
            B = b2d(b)
            if isnan_b(b): v = 0x80000000
            else:
                if xo == 15: rnd = float(np.trunc(B))
                else:
                    rn = self.fpscr & 3
                    rnd = {0: float(np.rint(B)), 1: float(np.trunc(B)), 2: float(np.ceil(B)), 3: float(np.floor(B))}[rn]
                if rnd >= 2147483648.0: v = 0x7FFFFFFF
                elif rnd < -2147483648.0: v = 0x80000000
                else: v = int(rnd) & 0xFFFFFFFF
            r = 0xFFF8000000000000 | v
            if v == 0 and (b & SIGN): r |= 0x100000000
            P[fd] = r
            self.log("fctiwz" if xo == 15 else "fctiw", "cvt", (b,), fd)
            return
        if xo == 72: P[fd] = b; return
        if xo == 40: P[fd] = b ^ SIGN; return
        if xo == 264: P[fd] = b & ~SIGN & M64; return
        if xo == 136: P[fd] = b | SIGN; return
        if xo == 583: P[fd] = 0xFFF8000000000000 | self.fpscr; return  # mffs
        if xo == 711:  # mtfsf
            fm = (w >> 17) & 0xFF
            m = 0
            for i in range(8):
                if fm & (0x80 >> i): m |= 0xF << (28 - 4 * i)
            self.fpscr = (self.fpscr & ~m) | (b & 0xFFFFFFFF & m); return
        if xo in (38, 70):  # mtfsb1 / mtfsb0
            bit = 1 << (31 - fd)
            self.fpscr = (self.fpscr | bit) if xo == 38 else (self.fpscr & ~bit); return
        if xo == 134:  # mtfsfi
            sh = 28 - 4 * (fd >> 2)
            self.fpscr = (self.fpscr & ~(0xF << sh)) | (((w >> 12) & 0xF) << sh); return
        if xo == 64:  # mcrfs
            self.setcr(fd >> 2, (self.fpscr >> (28 - 4 * (fa >> 2))) & 0xF); return
        raise Unimplemented(f"op63 xo {xo} at {self.pc:08X}")

    def op4(self, w, fd, fa, fb):
        fc, xo5, xo, xo6 = (w >> 6) & 31, (w >> 1) & 31, (w >> 1) & 0x3FF, (w >> 1) & 0x3F
        P0, P1, F = self.ps0, self.ps1, self.fpscr
        if xo6 in (6, 7, 38, 39):  # psq_lx / psq_stx / psq_lux / psq_stux
            W, I = (w >> 10) & 1, (w >> 7) & 7
            g = self.gpr
            a = ((g[fa] if (fa or xo6 in (38, 39)) else 0) + g[fb]) & 0xFFFFFFFF
            if xo6 in (6, 38): self.dequant(a, I, fd, W)
            else: self.quant(a, I, fd, W)
            if xo6 in (38, 39): g[fa] = a
            return
        a0, a1, b0, b1, c0, c1 = P0[fa], P1[fa], P0[fb], P1[fb], P0[fc], P1[fc]
        fs = lambda x: force_single(F, x)
        flags = ""
        name = None
        if xo5 == 21: r0, r1 = fs(ni_arith("+", a0, b0)), fs(ni_arith("+", a1, b1)); name = "ps_add"; ins = (a0, a1, b0, b1)
        elif xo5 == 20: r0, r1 = fs(ni_arith("-", a0, b0)), fs(ni_arith("-", a1, b1)); name = "ps_sub"; ins = (a0, a1, b0, b1)
        elif xo5 == 18: r0, r1 = fs(ni_arith("/", a0, b0)), fs(ni_arith("/", a1, b1)); name = "ps_div"; ins = (a0, a1, b0, b1)
        elif xo5 in (25, 28, 29, 30, 31, 12, 13, 14, 15):
            if xo5 in (12, 14): k0 = k1 = force25(c0)
            elif xo5 in (13, 15): k0 = k1 = force25(c1)
            else: k0, k1 = force25(c0), force25(c1)
            if (xo5 in (12, 14) and k0 != c0) or (xo5 in (13, 15) and k0 != c1) or \
                    (xo5 not in (12, 13, 14, 15) and (k0 != c0 or k1 != c1)):
                flags = "f25"
            ins = (a0, a1, c0, c1, b0, b1)
            if xo5 in (25, 12, 13): r0, r1 = fs(ni_arith("*", a0, k0)), fs(ni_arith("*", a1, k1)); ins = (a0, a1, c0, c1)
            elif xo5 in (29, 14, 15): r0, r1 = fs(ni_madd(a0, k0, b0)), fs(ni_madd(a1, k1, b1))
            elif xo5 == 28: r0, r1 = fs(ni_madd(a0, k0, b0, True)), fs(ni_madd(a1, k1, b1, True))
            elif xo5 == 31: r0, r1 = neg_unless_nan(fs(ni_madd(a0, k0, b0))), neg_unless_nan(fs(ni_madd(a1, k1, b1)))
            else: r0, r1 = neg_unless_nan(fs(ni_madd(a0, k0, b0, True))), neg_unless_nan(fs(ni_madd(a1, k1, b1, True)))
            name = {25: "ps_mul", 12: "ps_muls0", 13: "ps_muls1", 29: "ps_madd", 14: "ps_madds0", 15: "ps_madds1",
                    28: "ps_msub", 31: "ps_nmadd", 30: "ps_nmsub"}[xo5]
        elif xo5 == 10: r0, r1 = fs(ni_arith("+", a0, b1)), fs(c1); name = "ps_sum0"; ins = (a0, b1, c1)
        elif xo5 == 11: r0, r1 = fs(c0), fs(ni_arith("+", a0, b1)); name = "ps_sum1"; ins = (a0, b1, c0)
        elif xo5 == 23:
            r0 = c0 if (b2d(a0) >= -0.0 and not isnan_b(a0)) else b0
            r1 = c1 if (b2d(a1) >= -0.0 and not isnan_b(a1)) else b1
            name = "ps_sel"; ins = (a0, a1, b0, b1, c0, c1)
        elif xo5 == 24: r0, r1 = fres(b0), fres(b1); name = "ps_res"; ins = (b0, b1)
        elif xo5 == 26: r0, r1 = fs(frsqrte(b0)), fs(frsqrte(b1)); name = "ps_rsqrte"; ins = (b0, b1)
        if name:
            P0[fd], P1[fd] = r0, r1
            self.log(name, "ps", ins, fd, flags)
            return
        if xo == 40: P0[fd], P1[fd] = P0[fb] ^ SIGN, P1[fb] ^ SIGN; return
        if xo == 72: P0[fd], P1[fd] = P0[fb], P1[fb]; return
        if xo == 136: P0[fd], P1[fd] = P0[fb] | SIGN, P1[fb] | SIGN; return
        if xo == 264: P0[fd], P1[fd] = P0[fb] & ~SIGN & M64, P1[fb] & ~SIGN & M64; return
        if xo in (528, 560, 592, 624):
            x = P0[fa] if xo in (528, 560) else P1[fa]
            y = P0[fb] if xo in (528, 592) else P1[fb]
            P0[fd], P1[fd] = x, y
            return
        if xo in (0, 32): self.fcmp(fd >> 2, P0[fa], P0[fb]); return
        if xo in (64, 96): self.fcmp(fd >> 2, P1[fa], P1[fb]); return
        if xo == 1014:  # dcbz_l
            return
        raise Unimplemented(f"op4 xo {xo} at {self.pc:08X}")


def load_syms():
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    import int_audit
    return sorted(int_audit.load_symbols())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dump")
    ap.add_argument("-o", required=True)
    ap.add_argument("--max", type=int, default=50_000_000)
    a = ap.parse_args()
    syms = load_syms()
    m = Gekko(a.dump, syms)
    try:
        n = m.run(a.max)
        status = f"returned after {n} instructions" + (f" ({m.mmio} hardware register accesses)" if m.mmio else "")
    except Unimplemented as e:
        status = f"stopped: {e}"
    with open(a.o, "w") as f:
        f.write(f"# entry {m.entry:08X} ({m.func_of(m.entry)}) frame {m.frame}: {status}\n")
        f.write("seq\tpc\tfunc\top\tprec\tinputs\tout0\tout1\tflags\n")
        for seq, pc, op, prec, ins, o0, o1, fl in m.trace:
            f.write(f"{seq}\t{pc:08X}\t{m.func_of(pc)}\t{op}\t{prec}\t{','.join(f'{x:016X}' for x in ins)}\t"
                    f"{o0:016X}\t{o1:016X}\t{fl}\n")
    print(f"gekko: {m.func_of(m.entry)} frame {m.frame}: {status}; {len(m.trace)} float ops -> {a.o}")


if __name__ == "__main__":
    main()
