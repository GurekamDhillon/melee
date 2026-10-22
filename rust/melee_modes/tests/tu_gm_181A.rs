//! tu::gm_181A against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_gm_181A_ref.c, data tests/data/tu_gm_181A.txt; regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard529.py gm_181A`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::*;
use melee_modes::tu::gm_181A as t;
use melee_modes::tu_ext as ext;
use melee_types::*;

fn bt() -> u64 {
    let r = rnd();
    (if r & 1 != 0 { 0x1000u64 } else { 0 }) | (if r & 2 != 0 { 0x100u64 } else { 0 })
}
fn gv() -> i32 {
    const V: [i32; 5] = [0x190, 0x280, 0x370, 0x4B0, 0x4B1];
    let r = rnd() % 8;
    if r < 5 { V[r as usize] } else { (rnd() % 0x500) as i32 }
}

macro_rules! pid_fields {
    ($m:ident) => {
        $m!(rumble_enabled, set_rumble_enabled); $m!(xC_b1, set_xC_b1); $m!(vs_metal, set_vs_metal);
        $m!(xC_b3, set_xC_b3); $m!(vs_invisible, set_vs_invisible); $m!(xC_b5, set_xC_b5);
        $m!(xC_b6, set_xC_b6); $m!(xC_b7, set_xC_b7); $m!(xD_b0, set_xD_b0); $m!(xD_b1, set_xD_b1);
        $m!(xD_b2, set_xD_b2); $m!(xD_b3, set_xD_b3); $m!(xD_b4, set_xD_b4); $m!(xD_b5, set_xD_b5);
        $m!(xD_b6, set_xD_b6); $m!(xD_b7, set_xD_b7);
    };
}
unsafe fn pid_randomize(p: *mut PlayerInitData) {
    let p = &mut *p;
    p.ckind = rnd() as i8;
    p.slot_type = rnd() as u8;
    p.stocks = rnd() as i8;
    p.color = rnd() as u8;
    p.slot = rnd() as u8;
    p.spawn_pos = rnd() as i8;
    p.spawn_dir = rnd() as i8;
    p.sub_color = rnd() as u8;
    p.handicap = rnd() as i8;
    p.team = rnd() as u8;
    p.nametag = rnd() as u8;
    p.xB = rnd() as u8;
    macro_rules! seed {
        ($g:ident, $s:ident) => {
            p.$s(rnd() as _);
        };
    }
    pid_fields!(seed);
    p.cpu_kind = rnd() as u8;
    p.cpu_level = rnd() as u8;
    p.damage = rnd() as u16;
    p.damage1 = rnd() as u16;
    p.hp = rnd() as u16;
    p.attack_ratio = rf();
    p.defense_ratio = rf();
    p.model_scale = rf();
}
unsafe fn pid_fp(p: *const PlayerInitData) -> u64 {
    let p = &*p;
    let mut h: u64 = 1469598103934665603;
    macro_rules! m {
        ($v:expr) => {
            h = h.wrapping_mul(1000003).wrapping_add($v as u64);
        };
    }
    m!(p.ckind as u8); m!(p.slot_type); m!(p.stocks as u8); m!(p.color); m!(p.slot);
    m!(p.spawn_pos as u8); m!(p.spawn_dir as u8); m!(p.sub_color); m!(p.handicap as u8);
    m!(p.team); m!(p.nametag); m!(p.xB);
    macro_rules! fp {
        ($g:ident, $s:ident) => {
            m!(p.$g());
        };
    }
    pid_fields!(fp);
    m!(p.cpu_kind); m!(p.cpu_level); m!(p.damage); m!(p.damage1); m!(p.hp);
    m!(p.attack_ratio.to_bits()); m!(p.defense_ratio.to_bits()); m!(p.model_scale.to_bits());
    h
}
unsafe fn pid_print(p: *mut PlayerInitData) {
    out(&format!(" pid {:016x}", pid_fp(p)));
}

include!("stubs/gm_181A.rs");

unsafe fn mktable() -> *mut t::RegClearSpawnEntry {
    let tb = az(core::mem::size_of::<t::RegClearSpawnEntry>() * 102) as *mut t::RegClearSpawnEntry;
    let term = if rnd() % 5 == 0 { 200 } else { 3 + (rnd() % 95) as i32 };
    for i in 0..102i32 {
        let e = &mut *tb.add(i as usize);
        e.x0 = if i == term { 0x3E7 } else { (rnd() % 100) as i32 };
        e.x4 = (rnd() % 7) as u8;
        e.x5 = rnd() as u8;
        e.x6 = rnd() as u8;
        e.x7 = rnd() as u8;
        e.x8 = rf();
        e.xC = rf();
    }
    tb
}
fn t_lbArchive_80016DBC(file: *const core::ffi::c_char, rest: &[*mut core::ffi::c_void]) -> *mut HSD_Archive {
    unsafe {
        out("lbArchive_80016DBC");
        trs(file);
        for pair in rest.chunks(2) {
            trp(pair[0] as *const u8);
            trs(pair[1] as *const core::ffi::c_char);
            *(pair[0] as *mut *mut core::ffi::c_void) = mktable() as *mut _;
        }
        out("\n");
        az_t::<HSD_Archive>()
    }
}

unsafe fn state() {
    let d = &t::lbl_80472ED8;
    dump_t("d0", &d.x0);
    dump_t("d4", &d.x4);
    dump_t("d8", &d.x8);
    out(&format!("pidC {:016x}\n", pid_fp(&d.xC)));
    dump_t("d30", &d.pad_30);
    dump_t("d54", &d.x54);
    dump_t("d6a4", &d.x6A4);
    dump_t("d6a8", &d.x6A8);
    dump_t("d6ac", &d.x6AC);
    dump_t("d6b0", &d.x6B0);
    dump_t("d6b4", &d.x6B4);
    dump_t("d6b8", &d.x6B8);
    dump_t("rec", &t::lbl_80473594);
    dump_t("blocks", &t::lbl_803D8D08);
    out(&format!(
        "mm {} {} {}\n",
        t::lbl_804D65E0.x0_0() as i32,
        t::lbl_804D65E0.x2 as u32,
        t::lbl_804D65E0.x4
    ));
    dump_t("e8", &t::lbl_804D65E8);
    dump_arena();
    state_end();
}
unsafe fn seed_state() {
    core::ptr::write_bytes(&raw mut t::lbl_80472ED8, 0, 1);
    let d = &mut t::lbl_80472ED8;
    d.x0 = (rnd() % 8) as i32;
    d.x4 = (rnd() % 8) as i32;
    d.x8 = if rnd() % 4 == 0 { 0x5A + (rnd() % 3) as i32 } else { (rnd() % 0x70) as i32 };
    pid_randomize(&mut d.xC);
    for i in 0..101 {
        let r = rnd() % 8;
        let e = &mut d.x54[i];
        e.x0 = if r == 0 {
            -2
        } else if r == 1 {
            -1
        } else if r == 2 {
            0x3E7
        } else {
            (rnd() % 101) as i32
        };
        e.x4 = (rnd() % 7) as u8;
        e.x5 = rnd() as u8;
        e.x6 = rnd() as u8;
        e.x7 = rnd() as u8;
        e.x8 = rf();
        e.xC = rf();
    }
    let r = &mut t::lbl_80473594;
    r.x0 = (rnd() % 3) as u8;
    r.x2 = (rnd() % 400) as u16;
    r.x4 = (rnd() % 2000) as i32;
    const MODES: [i32; 7] = [0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0];
    r.x8 = MODES[(rnd() % 7) as usize];
    r.xC = (rnd() % 27) as i32;
    r.x10 = (rnd() % 4) as i8;
    r.x11 = (rnd() % 100) as u8;
    for k in 0..6 {
        let b = &mut t::lbl_803D8D08[k];
        for i in 0..27 {
            b.scores[i] = rnd() % 2000;
        }
        for i in 0..28 {
            b.icons[i] = (rnd() % 4) as u8;
        }
        for i in 0..28 {
            b.times[i] = (rnd() % 400) as u16;
        }
    }
    t::lbl_804D65E8 = (rnd() % 4) as u8;
}

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        ext::HOOK_lbArchive_80016DBC = Some(t_lbArchive_80016DBC);
        size_t::<t::lbl_80472ED8_t>("lbl_80472ED8_t");
        size_t::<t::RegClearSpawnEntry>("RegClearSpawnEntry");
        size_t::<t::RegClearRecordState>("RegClearRecordState");
        size_t::<t::RecordBlock>("RecordBlock");
        size_t::<PlayerInitData>("PlayerInitData");
        size_t::<UnkMultimanData>("UnkMultimanData");
        reg("rec", &t::lbl_80473594, core::mem::size_of::<t::RegClearRecordState>());
        reg("blocks", t::lbl_803D8D08.as_ptr(), core::mem::size_of_val(&t::lbl_803D8D08));
        reg("d", &t::lbl_80472ED8, core::mem::size_of::<t::lbl_80472ED8_t>());
        fnreg("fn_80181E18", t::fn_80181E18 as usize);
        fnreg("fn_80182F40", t::fn_80182F40 as usize);
        dump_t("blocks0", &t::lbl_803D8D08);
        state_end();

        for c in 0..8usize {
            case(c);
            rseed(100 + c as u32);
            arena_reset();
            seed_state();
            out(&format!("r {} {} {}\n", t::gm_80181A14(), t::gm_80181A24(), t::gm_80181A34()));
            state();
        }
        for c in 0..300usize {
            case(1000 + c);
            rseed(2000 + c as u32);
            arena_reset();
            seed_state();
            let a = (rnd() % 40) as i32;
            let b = 0x20 + (rnd() % 8) as i32;
            let v = rnd() % 2 != 0;
            t::gm_80181A44(a, b, v);
            let a = (rnd() % 40) as i32;
            let b = 0x20 + (rnd() % 8) as i32;
            let v = rnd() as u16;
            t::gm_80181AC8(a, b, v);
            let a = (rnd() % 40) as i32;
            let b = 0x20 + (rnd() % 8) as i32;
            let v = rnd() as i32;
            t::gm_80181B64(a, b, v);
            state();
        }
        for c in 0..200usize {
            case(2000 + c);
            rseed(3000 + c as u32);
            arena_reset();
            seed_state();
            let mut out_i: i32 = -1;
            let r = t::fn_80181BFC(if c % 2 != 0 { &mut out_i } else { core::ptr::null_mut() });
            out(&format!("r {} {}\n", r, out_i));
            state();
        }
        for c in 0..400usize {
            case(3000 + c);
            rseed(4000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80181C80((rnd() % 101) as i32);
            state();
        }
        for c in 0..1500usize {
            case(4000 + c);
            rseed(5000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80181E18_body();
            state();
        }
        for c in 0..60usize {
            case(7000 + c);
            rseed(8000 + c as u32);
            arena_reset();
            seed_state();
            t::gm_80182174();
            state();
        }
        for c in 0..14usize {
            case(8000 + c);
            rseed(9000 + c as u32);
            arena_reset();
            seed_state();
            out(&format!("r {}\n", t::gm_IsMultimanSmashMode() as i32));
            state();
        }
        for c in 0..10usize {
            case(9000 + c);
            rseed(10000 + c as u32);
            arena_reset();
            seed_state();
            let a = (rnd() % 27) as i32;
            let b = 0x21 + (rnd() % 6) as i32;
            t::gm_80182554(a, b);
            state();
        }
        for c in 0..2000usize {
            case(10000 + c);
            rseed(11000 + c as u32);
            arena_reset();
            seed_state();
            t::gm_80182578();
            state();
        }
        for c in 0..800usize {
            case(13000 + c);
            rseed(14000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80182B5C();
            state();
        }
        for c in 0..200usize {
            case(15000 + c);
            rseed(16000 + c as u32);
            arena_reset();
            seed_state();
            t::lbl_804D65E0.set_x0_0((rnd() % 2) as u16);
            t::lbl_804D65E0.x2 = rnd() as u16;
            t::lbl_804D65E0.x4 = rnd() as i32;
            let a = (rnd() % 40) as i32;
            let b = 0x20 + (rnd() % 8) as i32;
            let r = t::gm_80182DF0(a, b);
            out(&format!("r {}\n", (r == &raw mut t::lbl_804D65E0) as i32));
            state();
        }
        for c in 0..2000usize {
            case(16000 + c);
            rseed(17000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80182F40(core::ptr::null_mut());
            state();
        }
        for c in 0..2usize {
            case(19000 + c);
            rseed(20000 + c as u32);
            arena_reset();
            seed_state();
            t::gm_80183218();
            state();
        }
    }
    check("tu_gm_181A");
}
