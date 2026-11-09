"""archive.c spec (see lbspecs.py): a synthetic archive image (header, data, relocs, publics, externs, symbols) in AB."""
from lbspecs import add

POOLS = [("HSD_Archive", "ARC", 1)]
_SETUP_C = (
    "{ u32* w = (u32*) AB; int i, D, R, P, E, pw, ew, rw, sb; static const unsigned char SY[24] = { 'a', 'b', 0, 'a', 'b', 'c', 0, 'x', 0, 's', 'y', 'm', 0 }; static const int SO[4] = { 0, 3, 7, 9 }; "
    "for (i = 0; i < 1024; i++) AB[i] = 0; D = 4 * (2 + fv() % 6); R = fv() % 3; P = fv() % 3; E = fv() % 3; "
    "rw = (0x20 + D) / 4; pw = rw + R; ew = pw + 2 * P; sb = 0x20 + D + 4 * R + 8 * P + 8 * E; TOT = sb + 24; "
    "w[0] = TOT; w[1] = D; w[2] = R; w[3] = P; w[4] = E; "
    "for (i = 0; i < D / 4; i++) { u32 c = fv(); w[8 + i] = (c & 3) ? 4 * (fv() % (D / 4)) : 0xFFFFFFFFu; } "
    "{ int x = fv() % (D / 4); for (i = 0; i < R; i++) w[rw + i] = 4 * ((x + i) % (D / 4)); } "
    "for (i = 0; i < P; i++) { w[pw + 2 * i] = 4 * (fv() % (D / 4)); w[pw + 2 * i + 1] = SO[fv() % 4]; } "
    "for (i = 0; i < E; i++) { w[ew + 2 * i] = 4 * (fv() % (D / 4)); w[ew + 2 * i + 1] = SO[fv() % 4]; } "
    "for (i = 0; i < 24; i++) AB[sb + i] = SY[i]; "
    "if (op != OP_HSD_ArchiveParse && op != OP_HSD_ArchiveParse_profile_body) HSD_ArchiveParse(&ARC[0], AB, TOT); }")
_SETUP_R = (
    "{ let w = (&raw mut AB) as *mut u32; const SY: [u8; 24] = [b'a', b'b', 0, b'a', b'b', b'c', 0, b'x', 0, b's', b'y', b'm', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; const SO: [u32; 4] = [0, 3, 7, 9]; "
    "for i in 0..1024usize { AB[i] = 0; } let d = (4 * (2 + fv() % 6)) as usize; let r = (fv() % 3) as usize; let p = (fv() % 3) as usize; let e = (fv() % 3) as usize; "
    "let rw = (0x20 + d) / 4; let pw = rw + r; let ew = pw + 2 * p; let sb = 0x20 + d + 4 * r + 8 * p + 8 * e; TOT = (sb + 24) as u32; "
    "*w.add(0) = TOT; *w.add(1) = d as u32; *w.add(2) = r as u32; *w.add(3) = p as u32; *w.add(4) = e as u32; "
    "for i in 0..d / 4 { let c = fv(); *w.add(8 + i) = if c & 3 != 0 { 4 * (fv() % (d as u32 / 4)) } else { 0xFFFF_FFFF }; } "
    "{ let x = fv() % (d as u32 / 4); for i in 0..r { *w.add(rw + i) = 4 * ((x + i as u32) % (d as u32 / 4)); } } "
    "for i in 0..p { *w.add(pw + 2 * i) = 4 * (fv() % (d as u32 / 4)); *w.add(pw + 2 * i + 1) = SO[(fv() % 4) as usize]; } "
    "for i in 0..e { *w.add(ew + 2 * i) = 4 * (fv() % (d as u32 / 4)); *w.add(ew + 2 * i + 1) = SO[(fv() % 4) as usize]; } "
    "for i in 0..24usize { AB[sb + i] = SY[i]; } "
    "if op != OP_HSD_ArchiveParse && op != OP_HSD_ArchiveParse_profile_body { m::HSD_ArchiveParse(&raw mut ARC[0], (&raw mut AB) as *mut u8, TOT as usize); } }")
_HASH_C = ("{ int i; u32 bs = (u32) (unsigned long long) AB; for (i = 0; i < 1024; i += 4) { u32 v = *(u32*) (AB + i); u32 d = v - bs; HM((d < 4096u || d > 0xFFFFFF00u) ? (0xF0000000u ^ d) : v); } }")
_HASH_R = ("{ let bs = (&raw const AB) as usize as u32; for i in (0..1024usize).step_by(4) { let v = ((&raw const AB) as *const u8).add(i).cast::<u32>().read_unaligned(); let d = v.wrapping_sub(bs); hm(if d < 4096 || d > 0xFFFF_FF00 { 0xF000_0000 ^ d } else { v }); } }")
PARSE = {1: ("AB", "(&raw mut AB) as *mut u8"), 2: ("(size_t) ((fv() & 7) ? TOT : TOT + 1)", "if fv() & 7 != 0 { TOT as usize } else { TOT as usize + 1 }")}
add("archive", "src/sysdolphin/baselib/archive.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), hash=(_HASH_C, _HASH_R), ncases=400,
    c_decl="static unsigned char AB[1024] __attribute__((aligned(8))); static u32 TOT; static const char SYMS[4][8] = { \"ab\", \"abc\", \"x\", \"sym\" };",
    rs_extra="#[repr(align(8))] struct Al([u8; 1024]); static mut AB: [u8; 1024] = [0; 1024]; static mut TOT: u32 = 0; static SYMS: [[u8; 8]; 4] = [*b\"ab\0\0\0\0\0\0\", *b\"abc\0\0\0\0\0\", *b\"x\0\0\0\0\0\0\0\", *b\"sym\0\0\0\0\0\"];",
    pregs=("preg(AB, sizeof AB);", "preg(&raw const AB, 1024); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"HSD_ArchiveParse": PARSE, "HSD_ArchiveParse_profile_body": PARSE,
        "HSD_ArchiveGetPublicAddress": {1: ("SYMS[fv() % 4]", "(&raw const SYMS[(fv() % 4) as usize]) as *const i8")},
        "HSD_ArchiveGetExtern": {1: ("(int) (fv() % 5) - 1", "(fv() % 5) as i32 - 1")},
        "HSD_ArchiveLocateExtern": {1: ("SYMS[fv() % 4]", "(&raw const SYMS[(fv() % 4) as usize]) as *const i8"), 2: ("(void*) (AB + 4 * (fv() % 64))", "(&raw mut AB as *mut u8).add(4 * (fv() % 64) as usize) as *mut c_void")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], skip_stubs=["memset", "memcpy", "strcmp"],
    # the console's uintptr_t is 32 bits: the extern chain is made of u32 words (a native 64-bit uintptr_t would read 8 bytes)
    # the console's uintptr_t is 32 bits: the extern chain is made of u32 words (a native 64-bit uintptr_t would read 8 bytes)
    # (the extern chain is u32 words upstream since the 363720e12 sync: no patch needed)
    patch=[])
