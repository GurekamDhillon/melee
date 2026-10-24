//! src/tu/ftcamera.rs vs the C (gen/cref/tu/ftcamera.c, shipping contraction), 5000 cases.
#![allow(static_mut_refs)]
mod tu_s25;
use melee_fighter::hooks;
use melee_fighter::tu::ftcamera as c;
use melee_fighter::types::*;
use tu_s25::*;

static mut ZOOM: f32 = 0.0;
static mut BT: f32 = 0.0;
static mut CT: f32 = 0.0;
static mut TC: f32 = 0.0;
static mut GOP: usize = 0;

fn zoom() -> f32 { t(1); unsafe { ZOOM } }
fn bt() -> f32 { t(2); unsafe { BT } }
fn ct() -> f32 { t(3); unsafe { CT } }
fn tcam(v: &mut Vec3) { t(4); v.x = 0.0; v.y = unsafe { TC }; v.z = 0.0; }
// ftLib_800866DC itself is ported; the C stubs it, so stub its only callee to emit the same.
fn lb(_j: *mut HSD_JObj, _o: &Vec3, v: &mut Vec3) {
    t(5);
    t(1);
    unsafe { v.x = TC; v.y = ZOOM; v.z = BT; }
}

#[test]
fn ftcamera_vs_c() {
    let exp = expected("tu_ftcamera");
    unsafe {
        c::ext::Stage_GetCamFixedZoom = Some(zoom);
        c::ext::Stage_GetBlastZoneTopOffset = Some(bt);
        c::ext::Stage_GetCamBoundsTopOffset = Some(ct);
        c::ext::Stage_UnkSetVec3TCam_Offset = Some(tcam);
        hooks::LB_8000B1CC = Some(lb);
    }
    let mut n = 0;
    for op in 0..5u32 {
        for k in 0..1000u32 {
            let mut h = H::case(op, k);
            unsafe { TR = 0 };
            let mut f: Box<Fighter> = zeroed();
            let mut cb: Box<CmSubject> = zeroed();
            let mut inp: Box<UnkFloat6_Camera> = zeroed();
            let mut out: Box<UnkFloat6_Camera> = zeroed();
            let mut fd: Box<ftData> = zeroed();
            let mut attrs: Box<ftCo_DatAttrs> = zeroed();
            let mut parts: Box<[FighterBone; 4]> = zeroed();
            let mut go: Box<HSD_GObj> = zeroed();
            inp.x0.x = h.rw(); inp.x0.y = h.rw(); inp.x0.z = h.rw();
            inp.xC.x = h.rw(); inp.xC.y = h.rw(); inp.xC.z = h.rw();
            unsafe {
                ZOOM = h.rw(); BT = h.rw(); CT = h.rw(); TC = h.rg();
            }
            f.x34_scale.y = h.rw();
            f.facing_dir = if h.rnd() & 1 != 0 { 1.0 } else { h.rg() };
            f.cur_pos.x = h.rw(); f.cur_pos.y = h.rw(); f.cur_pos.z = h.rw();
            cb.pos.x = h.rw();
            cb.state = h.rnd() & 3;
            cb.set_on_ledge((h.rnd() & 1) as u8);
            fd.x3C = &mut *inp;
            fd.x0 = &mut *attrs;
            f.ft_data = &mut *fd;
            f.parts = parts.as_mut_ptr();
            f.x890_cameraBox = &mut *cb;
            go.user_data = &mut *f as *mut Fighter as *mut _;
            let g: *mut HSD_GObj = &mut *go;
            unsafe { GOP = g as usize };
            match op {
                0 => c::ftCamera_80076018(&inp, &mut out, f.x34_scale.y),
                1 => c::ftCamera_80076064(&mut f),
                2 => c::ftCamera_UpdateCameraBox(g),
                3 => c::ftCamera_800762F4(g),
                _ => unsafe {
                    if BT - TC != 0.0 {
                        c::ftCamera_80076320(g)
                    }
                },
            }
            h.h = 0x811C9DC5;
            h.hm(cb.state); h.hm(cb.on_ledge() as u32);
            let (p, bp) = (cb.pos, cb.bone_pos);
            h.hv(&p); h.hv(&bp); h.hf(cb.facing_dir);
            h.hf(cb.ext.h.x); h.hf(cb.ext.h.y); let v = cb.ext.v; h.hv(&v);
            h.hf(cb.target_ext.h.x); h.hf(cb.target_ext.h.y); let v = cb.target_ext.v; h.hv(&v);
            h.hv(&out.x0); h.hv(&out.xC);
            let got = format!("{} {} {:08X} {:08X}", op, k, h.h, unsafe { TR });
            assert_eq!(got, exp[n], "case {op} {k}");
            n += 1;
        }
    }
    assert_eq!(n, exp.len());
}
