//! The hand-ported character functions (src/chars) vs the game's C, compiled the way the
//! shipping build compiles it (clang contraction on + gwtool lowering). Expected values:
//! tests/data/chars.txt from gen/chref.py (C driver gen/chref_main.c). This file repeats the
//! C's fill order exactly; each function runs on 4000 random fighters and the per-function FNV
//! hash over the whole Fighter, its dat_attrs block, the common data, the return value and the
//! out-params must match. Set CHARS_VERBOSE=<index> to print one function's per-case lines in
//! the format of `chref.py -v` (rust/target/chref/chars_verbose.txt).
#![allow(static_mut_refs)]

mod chars_dispatch;

use melee_chara::stubs as st;
use melee_chara::types_gen::{ftCommonData, Fighter, HSD_GObj, Vec2, Vec3};
use melee_fighter::hooks;

const CASES: u32 = 4000;
/// Round-2 callbacks (index >= N_R1): thin wrappers checked through the stub trace.
const CASES_R2: u32 = 256;
const ATTR_WORDS: usize = 1024;
const FNV0: u64 = 0xCBF2_9CE4_8422_2325;

static mut G_FRIC: f32 = 0.0;
static mut G_SFX: u32 = 0;

// Round 2: recorders for the group-3 stubs, the same as gen/chref_main.c's (id + non-pointer
// args; gobj as 0 = NULL / 1 = the fighter's own / 2 = other; returns from an LCG on G_SEED).
static mut G_TR: u32 = 0;
static mut G_SEED: u32 = 0;
static mut G_SELF: *mut HSD_GObj = core::ptr::null_mut();
fn tr(x: u32) {
    unsafe { G_TR = (G_TR ^ x).wrapping_mul(16777619) }
}
fn trf(f: f32) {
    tr(f.to_bits())
}
fn trg(p: *mut HSD_GObj) {
    tr(if p.is_null() { 0 } else if p == unsafe { G_SELF } { 1 } else { 2 })
}
fn trfp(fp: &Fighter) {
    tr((fp as *const Fighter as *mut core::ffi::c_void == unsafe { (*G_SELF).user_data }) as u32)
}
fn pick() -> i32 {
    unsafe {
        G_SEED = G_SEED.wrapping_mul(1103515245).wrapping_add(12345);
        ((G_SEED >> 16) & 1) as i32
    }
}
fn install_recorders() {
    unsafe {
        st::FT_80084F3C = Some(|g| { tr(1); trg(g) });
        st::FT_80084EEC = Some(|g| { tr(2); trg(g) });
        st::FT_80085134 = Some(|g| { tr(3); trg(g) });
        st::FT_80084DB0 = Some(|g| { tr(4); trg(g) });
        st::FT_80084FA8 = Some(|g| { tr(5); trg(g) });
        st::FT_80083F88 = Some(|g| { tr(6); trg(g) });
        st::FTANIM_ISFRAMESREMAINING = Some(|g| { tr(7); trg(g); pick() });
        st::FTANIM_8006EBA4 = Some(|g| { tr(8); trg(g) });
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|g, msid, flags, a, b, c, g3| {
            tr(9); trg(g); tr(msid as u32); tr(flags); trf(a); trf(b); trf(c); trg(g3)
        });
        st::FT_8008A2BC = Some(|g| { tr(10); trg(g) });
        st::FTCO_FALL_ENTER = Some(|g| { tr(11); trg(g) });
        st::FT_80081D0C = Some(|g| { tr(12); trg(g); pick() });
        st::FT_80082708 = Some(|g| { tr(13); trg(g); pick() });
        st::FT_800827A0 = Some(|g| { tr(14); trg(g); pick() });
        st::FTCOMMON_8007D7FC = Some(|fp| { tr(15); trfp(fp) });
        st::FTCO_80096900 = Some(|g, a, b, c, d, e| {
            tr(16); trg(g); tr(a as u32); tr(b as u32); tr(c as u32); trf(d); trf(e)
        });
        hooks::GENO_GROUND_EDGE = Some(|fp, landing| { tr(17); trfp(fp); tr(landing as u32) });
        hooks::PL_8003FC44 = Some(|slot, sub| { tr(18); tr(slot as u32); tr(sub as u32) });
        // round 5 (gen/port_r5.py)
        st::FT_80082C74 = Some(|g, cb| { tr(19); trg(g); trc(cb) });
        st::FT_8008403C = Some(|g, cb| { tr(20); trg(g); trc(cb) });
        st::FT_800841B8 = Some(|g, cb| { tr(21); trg(g); trc(cb) });
        st::FT_80084104 = Some(|g| { tr(22); trg(g) });
        st::FTPARTS_80074A4C = Some(|g, a, b| { tr(24); trg(g); tr(a as u32); tr(b as u32) });
        st::FTPARTS_80074B0C = Some(|g, a, b| { tr(25); trg(g); tr(a as u32); tr(b as u32) });
        st::FT_CHECKGROUNDANDLEDGE = Some(|g, d| { tr(26); trg(g); tr(d as u32); pick() });
        st::FTCOMMON_8007DB24 = Some(|g| { tr(28); trg(g) });
        st::FTCOMMON_8007D92C = Some(|g| { tr(29); trg(g) });
        st::FTCO_WAIT_IASA = Some(|g| { tr(30); trg(g) });
        st::FTCOMMON_8007EBAC = Some(|fp, a, b| { tr(32); trfp(fp); tr(a); tr(b) });
        st::FTANIM_SETANIMRATE = Some(|g, r| { tr(33); trg(g); trf(r) });
        st::FTANIM_800704F0 = Some(|g, i, f| { tr(34); trg(g); tr(i as u32); trf(f) });
        st::FTCO_LANDINGFALLSPECIAL_ENTER = Some(|g, c, f| { tr(35); trg(g); tr(c as u32); trf(f) });
        st::FIGHTER_UNKSETFLAG_8006CFBC = Some(|g| { tr(36); trg(g) });
        st::FTCO_800957F4 = Some(|g, m| { tr(37); trg(g); tr(m as u32) });
        // round 6 (gen/port_r6.py)
        st::FTCOLL_8007AEF8 = Some(|g| { tr(38); trg(g) });
        st::FTCO_AIRCATCHHIT_COLL = Some(|g| { tr(39); trg(g) });
        st::FTANIM_APPLYPARTANIM = Some(|g, a, b, f| { tr(40); trg(g); tr(a as u32); tr(b as u32); trf(f) });
        st::FTANIM_80070C48 = Some(|g, a| { tr(41); trg(g); tr(a as u32) });
        st::FTANIM_80070CC4 = Some(|g, a| { tr(42); trg(g); tr(a as u32) });
        st::EFLIB_DESTROYALL = Some(|g| { tr(43); trg(g) });
        st::ITISHEAVY = Some(|item| { tr(44); tr(!item.is_null() as u32); pick() });
        // round 8 (gen/port_r8.py)
        st::PLAYER_GETPLAYERSLOTTYPE = Some(|slot| { tr(45); tr(slot as u32); pick() & 3 });
    }
}

/// Round 5: an HSD_GObjEvent argument as its non-NULL bit + FNV-1a of its chars::cb key (C `trc`).
fn trc(cb: melee_types::HSD_GObjEvent) {
    let k = match cb {
        None => 0,
        Some(f) => melee_chara::chars::cb::CALLBACKS.iter().find(|(e, _)| *e as usize == f as usize)
            .map(|(_, key)| fnv(FNV0, key.as_bytes())).unwrap_or(0),
    };
    tr(cb.is_some() as u32);
    tr(k as u32);
    tr((k >> 32) as u32);
}

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        let k = self.rnd() % 16;
        if k == 0 {
            return 0.0;
        }
        if k == 1 {
            return -0.0;
        }
        ((self.rnd() % 2001) as i32 - 1000) as f32 / 7.0
    }
    fn slot(&mut self, small: u32) -> u32 {
        let k = self.rnd() % 8;
        if k < 3 {
            return self.rnd() % small;
        }
        if k == 3 {
            return 0;
        }
        self.rf().to_bits()
    }
}

fn fnv(mut h: u64, b: &[u8]) -> u64 {
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

fn bytes<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(v as *const T as *const u8, core::mem::size_of::<T>()) }
}
fn bytes_mut<T>(v: &mut T) -> &mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(v as *mut T as *mut u8, core::mem::size_of::<T>()) }
}

/// The bitfields compared by value (see gen/chref_main.c: C packs LSB-first, melee_types keeps
/// GC order, so their bytes differ by design).
const NBF: usize = 17;
fn bf_get(fp: &Fighter, i: usize) -> u32 {
    (unsafe {
        match i {
            0 => fp.fall_fast(),
            1 => fp.x2219_b5(),
            2 => fp.x221F_b3(),
            3 => fp.x2223_b4(),
            4 => fp.anon_2210.anon_0.throw_flags_b0(),
            5 => fp.x2070.anon_0.x2071_b6(),
            6 => fp.mv.ca.specialhi.x2_b0(),
            7 => fp.mv.ca.specialhi.x2_b1(),
            8 => fp.is_sub_fighter(),
            9 => fp.x2227_b0(),
            10 => fp.allow_interrupt(),
            11 => fp.invisible(),
            12 => fp.reflecting(),
            14 => fp.x2222_b2(),
            15 => fp.x2220_b3(),
            16 => fp.x221E_b6(),
            _ => fp.x2218_b4(),
        }
    }) as u32
}
fn bf_set(fp: &mut Fighter, i: usize, v: u8) {
    unsafe {
        match i {
            0 => fp.set_fall_fast(v),
            1 => fp.set_x2219_b5(v),
            2 => fp.set_x221F_b3(v),
            3 => fp.set_x2223_b4(v),
            4 => fp.anon_2210.anon_0.set_throw_flags_b0(v),
            5 => fp.x2070.anon_0.set_x2071_b6(v),
            6 => fp.mv.ca.specialhi.set_x2_b0(v),
            7 => fp.mv.ca.specialhi.set_x2_b1(v),
            8 => fp.set_is_sub_fighter(v),
            9 => fp.set_x2227_b0(v),
            10 => fp.set_allow_interrupt(v),
            11 => fp.set_invisible(v),
            12 => fp.set_reflecting(v),
            14 => fp.set_x2222_b2(v),
            15 => fp.set_x2220_b3(v),
            16 => fp.set_x221E_b6(v),
            _ => fp.set_x2218_b4(v),
        }
    }
}
/// Round 7: whole-word `throw_flags` writers (gen/chref_main.c THROW_WORD): bf 4 is hashed as bytes.
const THROW_WORD: &[&str] = &[
    "ftCa_SpecialAirLw_Anim",
    "ftCa_SpecialAirLw_Anim_inline",
    "ftKb_SpecialNLg_800F9614",
    "ftKb_SpecialNLg_800F9684",
    "ftLg_SpecialAirN_Enter",
    "ftLg_SpecialHi_Enter",
    "ftLg_SpecialN_Enter",
    "ftMr_SpecialAirN_Enter",
    "ftMr_SpecialHi_Enter",
    "ftMr_SpecialN_Enter",
    "ftMs_SpecialHi_Enter",
    "ftMt_SpecialAirLw_Enter",
    "ftMt_SpecialLw_Enter",
    "ftPk_SpecialAirLw_Enter",
    "ftPk_SpecialLw_Enter",
    "ftPp_SpecialN_Enter",
    "ftSamus_ClearThrowFlagsUnk",
    "ftYs_SpecialAirHi_Enter",
    "ftYs_SpecialHi_Enter",
    "resetCmdAndThrow",
];

fn bf_probe(fp: &mut Fighter) -> [usize; NBF] {
    let mut out = [0; NBF];
    for (i, o) in out.iter_mut().enumerate() {
        bytes_mut(fp).fill(0);
        bf_set(fp, i, 1);
        *o = bytes(fp).iter().position(|&b| b != 0).unwrap();
    }
    out
}

/// Round 3: stored callbacks -> FNV-1a of their `<setter tu>:<C name>` key (gen/chref_main.c `cb_normalise`).
fn cb_normalise(fp: &mut Fighter) {
    let b = bytes_mut(fp);
    for &off in chars_dispatch::CB_OFF {
        let v = usize::from_le_bytes(b[off..off + 8].try_into().unwrap());
        if let Some((_, key)) = melee_chara::chars::cb::CALLBACKS.iter().find(|(f, _)| *f as usize == v) {
            b[off..off + 8].copy_from_slice(&fnv(FNV0, key.as_bytes()).to_le_bytes());
        }
    }
}

#[test]
fn character_functions_match_c() {
    unsafe {
        hooks::GROUND_FRICTION_MUL = Some(|_| G_FRIC);
        hooks::PLAY_SFX = Some(|_, id, vol, pan| G_SFX = G_SFX.wrapping_add((id + vol + pan) as u32));
    }
    install_recorders();
    let mut gobj: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    let expected = include_str!("data/chars.txt");
    let mut lines = expected.lines();
    let verbose: Option<usize> = std::env::var("CHARS_VERBOSE").ok().and_then(|s| s.parse().ok());

    let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut attrs: Box<[u32; ATTR_WORDS]> = Box::new([0; ATTR_WORDS]);
    // round 9: fp->ft_data->ext_attr, the attrs block reversed (C `ext_attrs` / `ftd`)
    let mut ext_attrs: Box<[u32; ATTR_WORDS]> = Box::new([0; ATTR_WORDS]);
    let mut ftd: Box<melee_types::ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    let bf_byte = bf_probe(&mut fp);

    // header: native sizes and the bitfield bytes must agree with the C, or nothing below can
    let mut hdr = lines.next().unwrap().split_whitespace().skip(1);
    assert_eq!(hdr.next().unwrap().parse::<usize>().unwrap(), core::mem::size_of::<Fighter>(), "sizeof(Fighter)");
    assert_eq!(hdr.next().unwrap().parse::<usize>().unwrap(), core::mem::size_of::<ftCommonData>(), "sizeof(ftCommonData)");
    for (i, b) in bf_byte.iter().enumerate() {
        assert_eq!(hdr.next().unwrap().parse::<usize>().unwrap(), *b, "byte of bitfield {i}");
    }

    let mut bad = Vec::new();
    for (f, name) in chars_dispatch::NAMES.iter().enumerate() {
        let mut h = FNV0;
        // mv.ca.specialhi.x2_b0/b1 share their byte with the other characters' motion vars
        let mut bmask: u32 = if *name == "ftCa_SpecialLw_800E49FC" { (1 << NBF) - 1 } else { ((1 << NBF) - 1) & !0xC0 };
        if THROW_WORD.contains(name) {
            bmask &= !0x10;
        }
        let on = |i: usize| bmask >> i & 1 != 0;
        for k in 0..(if f < chars_dispatch::N_R1 { CASES } else { CASES_R2 }) {
            let mut r = Rng(0x9E37_79B9u32 ^ ((f as u32).wrapping_mul(7919).wrapping_add(k.wrapping_mul(104729)).wrapping_add(1)));
            {
                let w = unsafe {
                    core::slice::from_raw_parts_mut(&mut *fp as *mut Fighter as *mut u32, core::mem::size_of::<Fighter>() / 4)
                };
                for x in w.iter_mut() {
                    *x = r.slot(400);
                }
            }
            fp.motion_id = (if r.rnd() % 4 != 0 { 330 + r.rnd() % 300 } else { r.rnd() % 1000 }) as _;
            for (i, &b) in bf_byte.iter().enumerate() {
                if on(i) {
                    bytes_mut(&mut *fp)[b] = 0;
                }
            }
            for i in 0..NBF {
                let v = (r.rnd() & 1) as u8;
                if on(i) {
                    bf_set(&mut fp, i, v);
                }
            }
            for x in attrs.iter_mut() {
                *x = r.slot(8);
            }
            {
                let w = unsafe {
                    core::slice::from_raw_parts_mut(&mut *cd as *mut ftCommonData as *mut u32, core::mem::size_of::<ftCommonData>() / 4)
                };
                for x in w.iter_mut() {
                    *x = r.slot(8);
                }
            }
            fp.dat_attrs = attrs.as_mut_ptr() as *mut _;
            for i in 0..ATTR_WORDS {
                ext_attrs[i] = attrs[ATTR_WORDS - 1 - i];
            }
            ftd.ext_attr = ext_attrs.as_mut_ptr() as *mut _;
            let ft_data_saved = fp.ft_data;
            fp.ft_data = &mut *ftd;
            unsafe {
                hooks::P_FT_COMMON_DATA = &*cd;
                G_SFX = 0;
                G_FRIC = r.rf();
            }
            let a = r.rf();
            let mut ia = (r.rnd() % 4) as i32;
            if r.rnd() % 2 != 0 {
                ia = (r.rnd() % 20) as i32;
            }
            let nul = r.rnd() % 10 == 0;
            let mut o1 = r.rnd() as i32;
            let mut o2 = r.rnd() as i32;
            let mut v3 = Vec3 { x: r.rf(), y: r.rf(), z: r.rf() };
            let mut v2 = Vec2 { x: r.rf(), y: r.rf() };
            unsafe {
                G_SEED = r.rnd();
                G_TR = 0;
                G_SELF = &mut *gobj;
            }
            gobj.user_data = &mut *fp as *mut Fighter as *mut _;

            let ret = chars_dispatch::call(f, &mut *gobj, &mut fp, a, ia, nul, &mut v3, &mut v2, &mut o1, &mut o2);

            fp.dat_attrs = core::ptr::null_mut();
            fp.ft_data = ft_data_saved;
            let mut bits = 0u32;
            for i in 0..NBF {
                if on(i) {
                    bits |= bf_get(&fp, i) << i;
                }
            }
            for (i, &b) in bf_byte.iter().enumerate() {
                if on(i) {
                    bytes_mut(&mut *fp)[b] = 0;
                }
            }
            h = fnv(h, &bits.to_le_bytes());
            cb_normalise(&mut fp);
            let sfx = unsafe { G_SFX };
            let trace = unsafe { G_TR };
            if verbose == Some(f) {
                println!(
                    "b{:02X} {} {} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {} {} {:08X} {:016X} {:016X} {:016X} {:08X}",
                    bits, f, k, ret, v3.x.to_bits(), v3.y.to_bits(), v3.z.to_bits(), v2.x.to_bits(), v2.y.to_bits(),
                    o1, o2, sfx, fnv(FNV0, bytes(&*fp)), fnv(FNV0, bytes(&*attrs)), fnv(FNV0, bytes(&*cd)), trace
                );
            }
            h = fnv(h, bytes(&*fp));
            h = fnv(h, bytes(&*attrs));
            h = fnv(h, bytes(&*cd));
            h = fnv(h, &ret.to_le_bytes());
            h = fnv(h, bytes(&v3));
            h = fnv(h, bytes(&v2));
            h = fnv(h, &o1.to_le_bytes());
            h = fnv(h, &o2.to_le_bytes());
            h = fnv(h, &sfx.to_le_bytes());
            h = fnv(h, &trace.to_le_bytes());
        }
        let line = lines.next().expect("chars.txt shorter than the dispatch table");
        let (cname, chash) = line.split_once(' ').unwrap();
        assert_eq!(cname, *name, "chars.txt and gen/ported.txt out of step");
        let want = u64::from_str_radix(chash, 16).unwrap();
        if want != h {
            bad.push(format!("{f} {name}"));
        }
    }
    assert!(bad.is_empty(), "{} of {} functions differ from the C:\n{}", bad.len(), chars_dispatch::NAMES.len(), bad.join("\n"));
}
