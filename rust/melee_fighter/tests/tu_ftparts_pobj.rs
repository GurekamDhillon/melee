//! src/tu/ftparts_render.rs ftParts_PObjSetupMtx (the z-scale Rigid / SharedVtx / Envelope matrix setups) against the C
//! (gen/cref/tu_ftparts_pobj_ref.c: the game TU with the real class.c, mtx.c, pobj.c marks, mtx_pc.c and the two index
//! tables cut verbatim; expected rows in tests/data/tu_ftparts_pobj.txt, regenerate with
//! `python rust/melee_fighter/gen/cref/tu_run.py tu_ftparts_pobj`). The GX loads, perf counters and a few HSD callees are
//! recording stand-ins on both sides (melee_runtime's pobj ext hooks here).
#![allow(static_mut_refs, non_snake_case, unused_unsafe)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_fighter::tu::ftparts_render as fr;
use melee_fighter::types::*;
use melee_runtime::tu::pobj as po;

static mut RS: u32 = 0;
static mut H: u32 = 0;
static mut TR: u32 = 0;
fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
fn rg() -> f32 {
    match rnd() % 12 {
        0 => 0.0,
        1 => 1.0,
        2 => -1.0,
        3 => 2.0,
        4 => 0.5,
        _ => ((rnd() % 41) as i32 - 20) as f32 / 7.0,
    }
}
fn hm(v: u32) {
    unsafe { H = (H.wrapping_mul(0x01000193)) ^ v }
}
fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}

const NJ: usize = 6;
static mut JO: [HSD_JObj; NJ] = unsafe { core::mem::zeroed() };
static mut ENVM: [[[f32; 4]; 3]; NJ] = [[[0.0; 4]; 3]; NJ];
static mut TOB: HSD_TObj = unsafe { core::mem::zeroed() };
static mut PO: HSD_PObj = unsafe { core::mem::zeroed() };
static mut SL: [HSD_SList; 4] = unsafe { core::mem::zeroed() };
static mut EV: [HSD_Envelope; 6] = unsafe { core::mem::zeroed() };
static mut CUR: *mut HSD_JObj = null_mut();

unsafe fn tm(m: *const f32) {
    for i in 0..12 {
        t((*m.add(i)).to_bits());
    }
}
fn ji(p: *const HSD_JObj) -> u32 {
    if p.is_null() {
        return 0;
    }
    unsafe {
        for i in 0..NJ {
            if p == &raw const JO[i] {
                return i as u32 + 1;
            }
        }
    }
    0xFFFF_FFFE
}

unsafe extern "C" fn cbps(p: *mut HSD_PObj, v: *mut [f32; 4], m: *mut [f32; 4], r: u32) {
    t(9112);
    t((p == &raw mut PO) as u32);
    tm(v.cast());
    tm(m.cast());
    t(r);
}

fn install() {
    unsafe {
        po::ext::hook::GXLoadPosMtxImm = Some(|m, id| unsafe { t(9100); tm(m.cast()); t(id) });
        po::ext::hook::GXLoadNrmMtxImm = Some(|m, id| unsafe { t(9101); tm(m.cast()); t(id) });
        po::ext::hook::GXLoadTexMtxImm = Some(|m, id, ty| unsafe { t(9102); tm(m.cast()); t(id); t(ty) });
        po::ext::hook::GXSetCurrentMtx = Some(|id| { t(9103); t(id) });
        po::ext::hook::HSD_PerfCountEnvelopeBlending = Some(|n| { t(9105); t(n as u32) });
        po::ext::hook::_HSD_TObjGetCurrentByType = Some(|from, mapping| {
            t(9106);
            t((!from.is_null()) as u32);
            t(mapping);
            if rnd() & 1 != 0 { unsafe { &raw mut TOB } } else { null_mut() }
        });
        po::ext::hook::_HSD_mkEnvelopeModelNodeMtx = Some(|j, out| unsafe {
            t(9107);
            t(ji(j));
            if rnd() & 1 != 0 {
                let o = out as *mut f32;
                for i in 0..12 {
                    *o.add(i) = rg();
                }
                return out;
            }
            null_mut()
        });
        po::ext::hook::HSD_JObjGetCurrent = Some(|| { t(9108); unsafe { CUR } });
        po::ext::hook::HSD_JObjSetupMatrixSub = Some(|j| unsafe {
            t(9109);
            t(ji(j));
            for i in 0..3 {
                for k in 0..4 {
                    (*j).mtx[i][k] = rg();
                }
            }
            (*j).flags &= !(JOBJ_MTX_DIRTY as u32);
        });
        po::hsdPObj.setup_mtx = Some(cbps);
    }
}

#[test]
fn ftparts_pobj_matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftparts_pobj.txt")).unwrap();
    let mut lines = want.lines();
    install();
    for k in 0..4000u32 {
        let mut vm = [[0.0f32; 4]; 3];
        let mut pm = [[0.0f32; 4]; 3];
        unsafe {
            RS = 0x2468Au32.wrapping_add(k.wrapping_mul(104729));
            H = 0x811C_9DC5;
            TR = 0;
            JO = core::mem::zeroed();
            ENVM = [[[0.0; 4]; 3]; NJ];
            TOB = core::mem::zeroed();
            PO = core::mem::zeroed();
            SL = core::mem::zeroed();
            EV = core::mem::zeroed();
            po::hsdPObj.setup_mtx = Some(cbps);
            for i in 0..NJ {
                for a in 0..3 {
                    for b in 0..4 {
                        JO[i].mtx[a][b] = rg();
                    }
                }
                for a in 0..3 {
                    for b in 0..4 {
                        ENVM[i][a][b] = rg();
                    }
                }
                JO[i].envelopemtx = ENVM[i].as_mut_ptr();
                JO[i].flags = (if rnd() & 1 != 0 { JOBJ_LIGHTING as u32 } else { 0 }) | (if rnd() % 3 == 0 { JOBJ_MTX_DIRTY as u32 } else { 0 });
            }
            CUR = &raw mut JO[(rnd() % NJ as u32) as usize];
            for a in 0..3 {
                for b in 0..4 {
                    vm[a][b] = rg();
                    pm[a][b] = rg();
                }
            }
            let mut rm = rnd();
            if rnd() % 3 == 0 {
                rm &= !(RENDER_SHADOW as u32);
            }
            fr::ft_jobj_scale.has_z_scale = (rnd() % 4) != 0;
            for a in 0..3 {
                for b in 0..4 {
                    fr::ft_jobj_scale.mtx[a][b] = rg();
                }
            }
            let mark_obj: *mut c_void = if rnd() & 1 != 0 { CUR.cast() } else { (&raw mut JO[(rnd() % NJ as u32) as usize]).cast() };
            po::HSD_PObjClearMtxMark(mark_obj, if rnd() & 1 != 0 { HSD_MTX_RIGID as u32 } else { HSD_MTX_ENVELOPE as u32 });
            match rnd() % 3 {
                0 => {
                    PO.flags = 0;
                    PO.u.jobj = if rnd() & 1 != 0 { &raw mut JO[(rnd() % NJ as u32) as usize] } else { null_mut() };
                }
                1 => {
                    PO.flags = 0x1000;
                    PO.u.jobj = if rnd() & 1 != 0 { &raw mut JO[(rnd() % NJ as u32) as usize] } else { null_mut() };
                }
                _ => {
                    PO.flags = 0x2000;
                    let nenv = (rnd() % 4) as usize;
                    for i in 0..nenv.min(4) {
                        let depth = rnd() % 3;
                        SL[i].data = (&raw mut EV[i]).cast();
                        SL[i].next = if i + 1 < nenv { &raw mut SL[i + 1] } else { null_mut() };
                        EV[i].weight = if rnd() % 2 != 0 { 1.0 } else { (rnd() % 4) as f32 / 4.0 };
                        EV[i].jobj = &raw mut JO[(rnd() % NJ as u32) as usize];
                        EV[i].next = if depth != 0 { &raw mut EV[4 + (i & 1)] } else { null_mut() };
                        if depth != 0 {
                            EV[4 + (i & 1)].weight = (rnd() % 4) as f32 / 4.0;
                            EV[4 + (i & 1)].jobj = &raw mut JO[(rnd() % NJ as u32) as usize];
                            EV[4 + (i & 1)].next = null_mut();
                        }
                    }
                    PO.u.envelope_list = if nenv != 0 { &raw mut SL[0] } else { null_mut() };
                }
            }
            fr::ftParts_PObjSetupMtx(&raw mut PO, vm.as_mut_ptr(), pm.as_mut_ptr(), rm);
            for i in 0..NJ {
                tm(JO[i].mtx.as_ptr().cast());
                hm(JO[i].flags);
            }
            for j in 0..2 {
                let mut o: *mut c_void = null_mut();
                let mut mk: u32 = 0;
                po::HSD_PObjGetMtxMark(j, &mut o, &mut mk);
                hm(ji(o.cast()));
                hm((o.is_null()) as u32);
                hm(mk);
            }
            hm(melee_runtime::tu::perf::HSD_PerfCurrentStat.nb_mtx_load);
            melee_runtime::tu::perf::HSD_PerfCurrentStat.nb_mtx_load = 0;
        }
        let got = format!("{} {:08X} {:08X}", k, unsafe { H }, unsafe { TR });
        assert_eq!(got, lines.next().unwrap(), "case {k}");
    }
    assert!(lines.next().is_none());
}
