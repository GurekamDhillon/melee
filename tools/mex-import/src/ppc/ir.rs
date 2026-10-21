//! PPC32 (Gekko subset) -> IR decoder.
//!
//! The subset and every decode quirk are pc/platform/gw_ppc.c's (`gw_ppc_execute`, `_x`, `_cr`,
//! `_fp`), which is what the C build runs m-ex code on: a word gw_ppc.c refuses decodes to
//! [`Op::Bad`] here, so "decodes" means exactly "the C interpreter would execute it". Notable
//! inherited behaviour: `cmpi`/`cmp` with L=1 and any OE=1 arithmetic form are refused; the FP
//! families decode A-form (5-bit XO) first and fall back to X-form (10-bit XO) for opcode 59 and
//! 63 alike; the Rc bit of FP ops is ignored (CR1 is never written); `fsel` is never rounded.
//! Branch targets are resolved to absolute addresses at decode time (`decode(word, ip)`).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ea {
    /// d(rA): rA = 0 reads as 0 for the non-update forms.
    Disp(i32),
    /// rA + rB.
    Idx(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithOp {
    Add, Subf, Neg, Mullw, Mulhw, Mulhwu, Divw, Divwu, Subfc, Subfe, Subfze, Subfme, Addze, Addc, Adde, Addme,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicOp {
    And, Andc, Or, Orc, Xor, Nor, Nand, Eqv, Slw, Srw, Sraw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Cntlzw, Extsb, Extsh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrOp {
    Nor, Andc, Xor, Nand, And, Eqv, Orc, Or,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FOp {
    Div, Sub, Add, Sqrt, Res, Mul, Rsqrte, Msub, Madd, Nmsub, Nmadd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FUn {
    Mr, Neg, Abs, Nabs, Rsp, Ctiwz, Ctiw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spr {
    Xer,
    Lr,
    Ctr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// addi / addis (imm already shifted for addis); rA = 0 reads as 0.
    AddI { d: u8, a: u8, imm: u32 },
    MulI { d: u8, a: u8, imm: i32 },
    /// subfic: rA = 0 reads as 0 (as gw_ppc.c has it).
    SubfIc { d: u8, a: u8, imm: u32 },
    /// addic / addic.: rA = 0 reads as 0 (as gw_ppc.c has it).
    AddIc { d: u8, a: u8, imm: u32, rc: bool },
    /// ori/oris/xori/xoris (rc false), andi./andis. (rc true); imm already shifted.
    OrI { a: u8, s: u8, imm: u32 },
    XorI { a: u8, s: u8, imm: u32 },
    AndI { a: u8, s: u8, imm: u32 },
    Rlwinm { a: u8, s: u8, sh: u8, mask: u32, rc: bool },
    Rlwimi { a: u8, s: u8, sh: u8, mask: u32, rc: bool },
    Rlwnm { a: u8, s: u8, b: u8, mask: u32, rc: bool },
    CmpI { crf: u8, a: u8, imm: u32, unsigned: bool },
    Cmp { crf: u8, a: u8, b: u8, unsigned: bool },
    Arith { op: ArithOp, d: u8, a: u8, b: u8, rc: bool },
    Logic { op: LogicOp, a: u8, s: u8, b: u8, rc: bool },
    Srawi { a: u8, s: u8, sh: u8, rc: bool },
    Un { op: UnOp, a: u8, s: u8, rc: bool },
    /// Integer load: size 1/2/4, sign-extend (lha), update rA, byte-reversed.
    Load { d: u8, a: u8, ea: Ea, size: u8, sign: bool, update: bool, rev: bool },
    Store { s: u8, a: u8, ea: Ea, size: u8, update: bool, rev: bool },
    LoadF { d: u8, a: u8, ea: Ea, single: bool, update: bool },
    StoreF { s: u8, a: u8, ea: Ea, single: bool, update: bool },
    Stfiwx { s: u8, a: u8, b: u8 },
    Lmw { d: u8, a: u8, disp: i32 },
    Stmw { s: u8, a: u8, disp: i32 },
    Dcbz { a: u8, b: u8 },
    /// Cache/sync no-ops.
    Nop,
    /// b/bl/ba (target absolute).
    B { target: u32, lk: bool },
    Bc { bo: u8, bi: u8, target: u32, lk: bool },
    Bclr { bo: u8, bi: u8, lk: bool },
    Bcctr { bo: u8, bi: u8, lk: bool },
    Mcrf { d: u8, s: u8 },
    CrLog { op: CrOp, t: u8, a: u8, b: u8 },
    Mfcr { d: u8 },
    Mtcrf { mask: u32, s: u8 },
    Mfspr { d: u8, spr: Spr },
    Mtspr { spr: Spr, s: u8 },
    /// mfspr/mtspr of an SPR gw_ppc.c does not model: a gw_panic there.
    BadSpr { spr: u32 },
    FArith { op: FOp, d: u8, a: u8, b: u8, c: u8, single: bool },
    Fsel { d: u8, a: u8, b: u8, c: u8 },
    FUnary { op: FUn, d: u8, b: u8 },
    Fcmp { crf: u8, a: u8, b: u8 },
    /// Outside the subset: gw_ppc_bad_opcode.
    Bad,
}

fn mask(mb: u32, me: u32) -> u32 {
    if mb <= me {
        (0xFFFF_FFFFu32 << (31 - me)) & (0xFFFF_FFFFu32 >> mb)
    } else {
        (0xFFFF_FFFFu32 << (31 - me)) | (0xFFFF_FFFFu32 >> mb)
    }
}

/// Decode one instruction word at address `ip`.
pub fn decode(w: u32, ip: u32) -> Op {
    let op = w >> 26;
    let rd = ((w >> 21) & 31) as u8;
    let ra = ((w >> 16) & 31) as u8;
    let rb = ((w >> 11) & 31) as u8;
    let simm = w as u16 as i16 as i32;
    let uimm = w & 0xFFFF;
    let rc = w & 1 != 0;
    let disp = Ea::Disp(simm);
    match op {
        14 => Op::AddI { d: rd, a: ra, imm: simm as u32 },
        15 => Op::AddI { d: rd, a: ra, imm: (simm as u32) << 16 },
        7 => Op::MulI { d: rd, a: ra, imm: simm },
        8 => Op::SubfIc { d: rd, a: ra, imm: simm as u32 },
        12 | 13 => Op::AddIc { d: rd, a: ra, imm: simm as u32, rc },
        24 => Op::OrI { a: ra, s: rd, imm: uimm },
        25 => Op::OrI { a: ra, s: rd, imm: uimm << 16 },
        26 => Op::XorI { a: ra, s: rd, imm: uimm },
        27 => Op::XorI { a: ra, s: rd, imm: uimm << 16 },
        28 => Op::AndI { a: ra, s: rd, imm: uimm },
        29 => Op::AndI { a: ra, s: rd, imm: uimm << 16 },
        21 | 20 | 23 => {
            let m = mask((w >> 6) & 31, (w >> 1) & 31);
            match op {
                21 => Op::Rlwinm { a: ra, s: rd, sh: rb, mask: m, rc },
                20 => Op::Rlwimi { a: ra, s: rd, sh: rb, mask: m, rc },
                _ => Op::Rlwnm { a: ra, s: rd, b: rb, mask: m, rc },
            }
        }
        11 | 10 => {
            if (w >> 21) & 1 != 0 {
                return Op::Bad;
            }
            let crf = ((w >> 23) & 7) as u8;
            if op == 11 {
                Op::CmpI { crf, a: ra, imm: simm as u32, unsigned: false }
            } else {
                Op::CmpI { crf, a: ra, imm: uimm, unsigned: true }
            }
        }
        32 => Op::Load { d: rd, a: ra, ea: disp, size: 4, sign: false, update: false, rev: false },
        33 => Op::Load { d: rd, a: ra, ea: disp, size: 4, sign: false, update: true, rev: false },
        40 => Op::Load { d: rd, a: ra, ea: disp, size: 2, sign: false, update: false, rev: false },
        41 => Op::Load { d: rd, a: ra, ea: disp, size: 2, sign: false, update: true, rev: false },
        42 => Op::Load { d: rd, a: ra, ea: disp, size: 2, sign: true, update: false, rev: false },
        43 => Op::Load { d: rd, a: ra, ea: disp, size: 2, sign: true, update: true, rev: false },
        34 => Op::Load { d: rd, a: ra, ea: disp, size: 1, sign: false, update: false, rev: false },
        35 => Op::Load { d: rd, a: ra, ea: disp, size: 1, sign: false, update: true, rev: false },
        36 => Op::Store { s: rd, a: ra, ea: disp, size: 4, update: false, rev: false },
        37 => Op::Store { s: rd, a: ra, ea: disp, size: 4, update: true, rev: false },
        44 => Op::Store { s: rd, a: ra, ea: disp, size: 2, update: false, rev: false },
        45 => Op::Store { s: rd, a: ra, ea: disp, size: 2, update: true, rev: false },
        38 => Op::Store { s: rd, a: ra, ea: disp, size: 1, update: false, rev: false },
        39 => Op::Store { s: rd, a: ra, ea: disp, size: 1, update: true, rev: false },
        48 => Op::LoadF { d: rd, a: ra, ea: disp, single: true, update: false },
        49 => Op::LoadF { d: rd, a: ra, ea: disp, single: true, update: true },
        50 => Op::LoadF { d: rd, a: ra, ea: disp, single: false, update: false },
        51 => Op::LoadF { d: rd, a: ra, ea: disp, single: false, update: true },
        52 => Op::StoreF { s: rd, a: ra, ea: disp, single: true, update: false },
        53 => Op::StoreF { s: rd, a: ra, ea: disp, single: true, update: true },
        54 => Op::StoreF { s: rd, a: ra, ea: disp, single: false, update: false },
        55 => Op::StoreF { s: rd, a: ra, ea: disp, single: false, update: true },
        46 => Op::Lmw { d: rd, a: ra, disp: simm },
        47 => Op::Stmw { s: rd, a: ra, disp: simm },
        18 => {
            let li = ((w << 6) as i32) >> 8;
            let base = if w & 2 != 0 { 0 } else { ip };
            Op::B { target: base.wrapping_add((li << 2) as u32), lk: rc }
        }
        16 => {
            let bd = ((w << 16) as i32) >> 18;
            let base = if w & 2 != 0 { 0 } else { ip };
            Op::Bc { bo: rd, bi: ra, target: base.wrapping_add((bd << 2) as u32), lk: rc }
        }
        19 => {
            let xo = (w >> 1) & 0x3FF;
            match xo {
                // gw_ppc.c tests `insn & 0x400` for bcctr; xo 528 is the only one with it here.
                16 => Op::Bclr { bo: rd, bi: ra, lk: rc },
                528 => Op::Bcctr { bo: rd, bi: ra, lk: rc },
                0 => Op::Mcrf { d: ((w >> 23) & 7) as u8, s: ((w >> 18) & 7) as u8 },
                33 => Op::CrLog { op: CrOp::Nor, t: rd, a: ra, b: rb },
                129 => Op::CrLog { op: CrOp::Andc, t: rd, a: ra, b: rb },
                193 => Op::CrLog { op: CrOp::Xor, t: rd, a: ra, b: rb },
                225 => Op::CrLog { op: CrOp::Nand, t: rd, a: ra, b: rb },
                257 => Op::CrLog { op: CrOp::And, t: rd, a: ra, b: rb },
                289 => Op::CrLog { op: CrOp::Eqv, t: rd, a: ra, b: rb },
                417 => Op::CrLog { op: CrOp::Orc, t: rd, a: ra, b: rb },
                449 => Op::CrLog { op: CrOp::Or, t: rd, a: ra, b: rb },
                _ => Op::Bad,
            }
        }
        31 => decode_x(w, rd, ra, rb, rc),
        59 | 63 => decode_fp(w, op == 59, rd, ra, rb),
        _ => Op::Bad,
    }
}

fn decode_x(w: u32, rd: u8, ra: u8, rb: u8, rc: bool) -> Op {
    let xo = (w >> 1) & 0x3FF;
    let ar = |op| Op::Arith { op, d: rd, a: ra, b: rb, rc };
    let lg = |op| Op::Logic { op, a: ra, s: rd, b: rb, rc };
    let un = |op| Op::Un { op, a: ra, s: rd, rc };
    let ld = |size, sign, update, rev| Op::Load { d: rd, a: ra, ea: Ea::Idx(rb), size, sign, update, rev };
    let st = |size, update, rev| Op::Store { s: rd, a: ra, ea: Ea::Idx(rb), size, update, rev };
    let lf = |single, update| Op::LoadF { d: rd, a: ra, ea: Ea::Idx(rb), single, update };
    let sf = |single, update| Op::StoreF { s: rd, a: ra, ea: Ea::Idx(rb), single, update };
    let spr_of = |w: u32| ((w >> 16) & 31) | (((w >> 11) & 31) << 5);
    match xo {
        0 | 32 => {
            if (w >> 21) & 1 != 0 {
                return Op::Bad;
            }
            Op::Cmp { crf: ((w >> 23) & 7) as u8, a: ra, b: rb, unsigned: xo == 32 }
        }
        266 => ar(ArithOp::Add),
        40 => ar(ArithOp::Subf),
        104 => ar(ArithOp::Neg),
        235 => ar(ArithOp::Mullw),
        75 => ar(ArithOp::Mulhw),
        11 => ar(ArithOp::Mulhwu),
        491 => ar(ArithOp::Divw),
        459 => ar(ArithOp::Divwu),
        8 => ar(ArithOp::Subfc),
        136 => ar(ArithOp::Subfe),
        200 => ar(ArithOp::Subfze),
        232 => ar(ArithOp::Subfme),
        202 => ar(ArithOp::Addze),
        10 => ar(ArithOp::Addc),
        138 => ar(ArithOp::Adde),
        234 => ar(ArithOp::Addme),
        60 => lg(LogicOp::Andc),
        412 => lg(LogicOp::Orc),
        476 => lg(LogicOp::Nand),
        284 => lg(LogicOp::Eqv),
        28 => lg(LogicOp::And),
        444 => lg(LogicOp::Or),
        316 => lg(LogicOp::Xor),
        124 => lg(LogicOp::Nor),
        24 => lg(LogicOp::Slw),
        536 => lg(LogicOp::Srw),
        792 => lg(LogicOp::Sraw),
        824 => Op::Srawi { a: ra, s: rd, sh: rb, rc },
        26 => un(UnOp::Cntlzw),
        954 => un(UnOp::Extsb),
        922 => un(UnOp::Extsh),
        23 => ld(4, false, false, false),
        87 => ld(1, false, false, false),
        279 => ld(2, false, false, false),
        343 => ld(2, true, false, false),
        790 => ld(2, false, false, true),
        534 => ld(4, false, false, true),
        55 => ld(4, false, true, false),
        119 => ld(1, false, true, false),
        311 => ld(2, false, true, false),
        375 => ld(2, true, true, false),
        215 => st(1, false, false),
        407 => st(2, false, false),
        151 => st(4, false, false),
        662 => st(4, false, true),
        918 => st(2, false, true),
        183 => st(4, true, false),
        247 => st(1, true, false),
        439 => st(2, true, false),
        535 => lf(true, false),
        567 => lf(true, true),
        599 => lf(false, false),
        631 => lf(false, true),
        663 => sf(true, false),
        695 => sf(true, true),
        727 => sf(false, false),
        759 => sf(false, true),
        983 => Op::Stfiwx { s: rd, a: ra, b: rb },
        19 => Op::Mfcr { d: rd },
        144 => {
            let crm = (w >> 12) & 0xFF;
            let mut m = 0u32;
            for i in 0..8 {
                if crm & (0x80 >> i) != 0 {
                    m |= 0xF << (28 - 4 * i);
                }
            }
            Op::Mtcrf { mask: m, s: rd }
        }
        339 | 467 => {
            let n = spr_of(w);
            let spr = match n {
                1 => Spr::Xer,
                8 => Spr::Lr,
                9 => Spr::Ctr,
                _ => return Op::BadSpr { spr: n },
            };
            if xo == 339 { Op::Mfspr { d: rd, spr } } else { Op::Mtspr { spr, s: rd } }
        }
        54 | 86 | 150 | 246 | 278 | 470 | 598 | 854 | 982 => Op::Nop,
        1014 | 1015 => Op::Dcbz { a: ra, b: rb },
        _ => Op::Bad,
    }
}

fn decode_fp(w: u32, single: bool, d: u8, a: u8, b: u8) -> Op {
    let c = ((w >> 6) & 31) as u8;
    let xo5 = (w >> 1) & 31;
    let fa = |op| Op::FArith { op, d, a, b, c, single };
    match xo5 {
        18 => return fa(FOp::Div),
        20 => return fa(FOp::Sub),
        21 => return fa(FOp::Add),
        22 => return fa(FOp::Sqrt),
        24 => return fa(FOp::Res),
        25 => return fa(FOp::Mul),
        26 => return fa(FOp::Rsqrte),
        28 => return fa(FOp::Msub),
        29 => return fa(FOp::Madd),
        30 => return fa(FOp::Nmsub),
        31 => return fa(FOp::Nmadd),
        23 => return Op::Fsel { d, a, b, c },
        _ => {}
    }
    let un = |op| Op::FUnary { op, d, b };
    match (w >> 1) & 0x3FF {
        72 => un(FUn::Mr),
        40 => un(FUn::Neg),
        264 => un(FUn::Abs),
        136 => un(FUn::Nabs),
        12 => un(FUn::Rsp),
        15 => un(FUn::Ctiwz),
        14 => un(FUn::Ctiw),
        0 | 32 => Op::Fcmp { crf: ((w >> 23) & 7) as u8, a, b },
        _ => Op::Bad,
    }
}

impl Op {
    /// Mnemonic family, for coverage tables.
    pub fn family(&self) -> &'static str {
        match self {
            Op::AddI { .. } | Op::MulI { .. } | Op::SubfIc { .. } | Op::AddIc { .. } => "int-imm",
            Op::OrI { .. } | Op::XorI { .. } | Op::AndI { .. } => "logic-imm",
            Op::Rlwinm { .. } | Op::Rlwimi { .. } | Op::Rlwnm { .. } | Op::Srawi { .. } => "rotate",
            Op::CmpI { .. } | Op::Cmp { .. } | Op::Fcmp { .. } => "compare",
            Op::Arith { .. } | Op::Logic { .. } | Op::Un { .. } => "int-x",
            Op::Load { .. } | Op::Store { .. } | Op::Lmw { .. } | Op::Stmw { .. } => "int-mem",
            Op::LoadF { .. } | Op::StoreF { .. } | Op::Stfiwx { .. } => "fp-mem",
            Op::Dcbz { .. } | Op::Nop => "cache",
            Op::B { .. } | Op::Bc { .. } | Op::Bclr { .. } | Op::Bcctr { .. } => "branch",
            Op::Mcrf { .. } | Op::CrLog { .. } | Op::Mfcr { .. } | Op::Mtcrf { .. } => "cr",
            Op::Mfspr { .. } | Op::Mtspr { .. } => "spr",
            Op::FArith { .. } | Op::Fsel { .. } | Op::FUnary { .. } => "fp",
            Op::BadSpr { .. } => "bad-spr",
            Op::Bad => "bad",
        }
    }
    pub fn executable(&self) -> bool {
        !matches!(self, Op::Bad | Op::BadSpr { .. })
    }
}
