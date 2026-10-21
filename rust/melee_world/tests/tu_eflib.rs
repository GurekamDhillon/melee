//! Smoke tests for tu/eflib.rs: the effect callbacks that only touch jobj SRT (jobjs are made
//! MTX_DIRTY so the setters never reach baselib), the param table, and the generator link pick.
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;
use melee_world::tu::eflib as m;

fn jobj() -> HSD_JObj {
    let mut j: HSD_JObj = unsafe { core::mem::zeroed() };
    j.flags = JOBJ_MTX_DIRTY as u32;
    j
}

#[test]
fn fall_and_offset_callbacks() {
    let mut j = jobj();
    j.translate = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    j.rotate.x = 0.25;
    j.rotate.y = 0.7;
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    g.hsd_obj = &mut j as *mut _ as *mut _;
    let mut e = EF_Effect::zeroed();
    e.gobj = &mut g;
    e.params = Vec3 { x: 0.5, y: 1.0, z: -1.0 };
    unsafe { m::efLib_Cb_Fall_FromParamY(&mut e) };
    let py = 1.0f32 - 0.2f32;
    assert_eq!(e.params.y.to_bits(), py.to_bits());
    assert_eq!(j.translate.x.to_bits(), (1.0f32 + sinf(0.7)).to_bits());
    assert_eq!(j.translate.y.to_bits(), (2.0f32 + py).to_bits());
    assert_eq!(j.translate.z.to_bits(), (3.0f32 + cosf(0.7)).to_bits());
    assert_eq!(j.rotate.x, 0.75);

    unsafe { m::efLib_Cb_SetScale_FromParamX(&mut e) };
    unsafe { m::efLib_Cb_SetOffset_FromParams(&mut e) };
    assert_eq!(e.params.y.to_bits(), (py - 0.1f32).to_bits());
}

#[test]
fn rot_y_from_fighter_dir_and_transition() {
    let mut j = jobj();
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    g.hsd_obj = &mut j as *mut _ as *mut _;
    let mut f: Fighter = unsafe { core::mem::zeroed() };
    f.facing_dir = -1.0;
    let mut pg: HSD_GObj = unsafe { core::mem::zeroed() };
    pg.user_data = &mut f as *mut _ as *mut _;
    let mut e = EF_Effect::zeroed();
    e.gobj = &mut g;
    e.parent_gobj = &mut pg;
    unsafe { m::efLib_Cb_SetRotY_FromFighterDir(&mut e) };
    assert_eq!(j.rotate.y, -(core::f64::consts::FRAC_PI_2 as f32));
    unsafe { m::efLib_Cb_SetRotYZ_FromParamZ_FighterDir(&mut e) };
    e.lifetime = 3;
    unsafe { m::efLib_Cb_SetRotYAndTransition(&mut e) }; // no user_data, not expiring, lifetime != 1
    assert_eq!(e.lifetime, 3);
    e.lifetime = 5;
    unsafe { m::efLib_Cb_LifetimeEndSpawn(&mut e) }; // only acts at lifetime 1
    assert_eq!(e.lifetime, 5);
}

#[test]
fn param_table_and_link() {
    let a = 0x1000usize as *mut HSD_GObj;
    let b = 0x2000usize as *mut HSD_GObj;
    unsafe {
        m::efLib_SetParamAlpha(a, 0x80);
        m::efLib_SetParamGfxId(a, 0x417);
        m::efLib_SetParamGfxId(b, 0x419);
        let t = &*core::ptr::addr_of!(m::efLib_ParamTable);
        assert_eq!(t[0].gobj as *mut HSD_GObj, a);
        assert_eq!((t[0].alpha, t[0].gfx_id), (0x80, 0x417));
        assert_eq!(t[1].gobj as *mut HSD_GObj, b);
    }
    assert_eq!(m::create_generator_link(0x121), 2);
    assert_eq!(m::create_generator_link(0x7918), 1);
    assert_eq!(m::create_generator_link(5), 0);
    assert_eq!(m::f2u32(10.0 * 2.5), 25);
}
