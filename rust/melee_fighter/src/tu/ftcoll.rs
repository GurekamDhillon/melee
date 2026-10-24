//! src/melee/ft/ftcoll.c: the hit/hurt/shield/reflect/absorb capsule setters the character code
//! calls (`ftColl_8007ABD0` .. `ftColl_HurtboxInit`). The leaf functions of this file are in
//! `crate::ftcoll`; the rest (damage log, hit processing) is not ported. No fused sites
//! (fma_sites.py).
#![allow(non_snake_case)]

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fpg<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ftColl_8007ABD0(hit, damage, gobj)`: set a capsule's damage from the move's base damage
/// (size scaling, smash charge, staling) and let the LAB's hit rules adjust it.
pub fn ftColl_8007ABD0(hit: *mut HitCapsule, damage: u32, gobj: G) {
    let fp = fpg(gobj);
    let mut dmg = damage as f32;
    if fp.x34_scale.y != 1.0 {
        let x4 = crate::tu::ftchangeparam::ext::d6524().x4;
        dmg = crate::tu::ftchangeparam::ftco_calc_y_scaled_knockback(dmg, fp.x34_scale.y, x4);
    }
    let scaled = crate::ft0df::ftco_800deeb8(fp, dmg);
    unsafe {
        (*hit).unk_count = (scaled as i32) as u32;
        (*hit).damage = crate::stale::apply_staling(fp, fp.x2068_attackID, fp.x206C_attack_instance as i32, scaled);
        // TARGET_PC: ScriptGame_HitRuleDamage(hit)
        melee_runtime::tu::script_hit::ScriptGame_HitRuleDamage(hit);
    }
}

/// `ftColl_8007AFF8(gobj)`: disable every hit capsule.
pub fn ftColl_8007AFF8(gobj: G) {
    let fp = fpg(gobj);
    for i in 0..fp.x914.len() {
        unsafe { melee_runtime::tu::lbcollision::lbColl_80008428(&mut fp.x914[i]) };
    }
    fp.set_x2219_b3(0);
}

/// `ftColl_8007B064(gobj, idx)`: enable hit capsule `idx` and clear its victim lists.
pub fn ftColl_8007B064(gobj: G, idx: i32) {
    let fp = fpg(gobj);
    let capsule: *mut HitCapsule = &mut fp.x914[idx as usize];
    unsafe {
        melee_runtime::tu::lbcollision::lbColl_80008434(capsule);
        melee_runtime::tu::lbcollision::lbColl_80008440(capsule);
    }
    fp.set_x2219_b3(1);
}

/// `ftColl_8007B0C0(gobj, state)`: set every hurt capsule's state.
pub fn ftColl_8007B0C0(gobj: G, state: HurtCapsuleState) {
    let fp = fpg(gobj);
    for i in 0..fp.hurt_capsules_len as usize {
        fp.hurt_capsules[i].capsule.state = state;
        fp.hurt_capsules[i].capsule.set_skip_update_pos(0);
    }
    if state == HurtCapsule_Enabled {
        fp.set_x221A_b5(0);
    } else {
        fp.set_x221A_b5(1);
    }
}

/// `ftColl_8007B128(gobj, bone_id, state)`: set the state of the hurt capsule on a bone.
pub fn ftColl_8007B128(gobj: G, bone_id: i32, state: HurtCapsuleState) {
    let fp = fpg(gobj);
    for i in 0..fp.hurt_capsules_len as usize {
        let hurt = &mut fp.hurt_capsules[i];
        if hurt.capsule.bone_idx == bone_id {
            hurt.capsule.state = state;
            if state != HurtCapsule_Enabled {
                fp.set_x221A_b5(1);
            }
            return;
        }
    }
    // TARGET_PC: a base "none" define has its own skeleton, so a script word it borrowed from the
    // donor can name a joint it has no hurtbox on. Skip it (retail data and every other define assert).
    if crate::hooks::geno_define_is_none(fp.kind as i32) != 0 && crate::hooks::geno_define_template_skeleton(fp.kind as i32) == 0 {
        return;
    }
    panic!("ftcoll.c:0x888 in ftCollisionSetHitStatus illegal parts!");
}

/// `ftColl_8007B1B8(gobj, shield, cb)`: set up the shield hit sphere.
pub fn ftColl_8007B1B8(gobj: G, shield: *const ShieldDesc, cb: HSD_GObjEvent) {
    let fp = fpg(gobj);
    fp.anon_221b.set_x221B_b0(1);
    fp.anon_221b.set_x221B_b1(0);
    fp.anon_221b.set_x221B_b2(0);
    fp.anon_221b.set_x221B_b3(0);
    fp.anon_221b.set_x221B_b4(0);
    fp.shield_hit_cb = cb;
    unsafe {
        fp.shield_hit.bone = (*fp.parts.add((*shield).bone as usize)).joint;
        fp.shield_hit.size = (*shield).radius;
        fp.shield_hit.offset = (*shield).pos;
    }
}

/// `ftColl_8007B62C(gobj, kind)`.
pub fn ftColl_8007B62C(gobj: G, arg1: i32) {
    let fp = fpg(gobj);
    fp.x1988 = arg1;
    match arg1 {
        0 => {
            crate::tu::ftcolanim::ftco_800bffd0(fp, 1, 0);
        }
        1 => {
            crate::tu::ftcolanim::ftco_800bffd0(fp, 3, 0);
        }
        2 => {
            crate::tu::ftcolanim::ftco_800bffd0(fp, 2, 0);
        }
        _ => {}
    }
}

/// `ftColl_8007B760(gobj, x)`.
pub fn ftColl_8007B760(gobj: G, arg1: i32) {
    let fp = fpg(gobj);
    if arg1 > fp.x1990 {
        fp.x1990 = arg1;
    }
    fp.x198C = 2;
    crate::tu::ftcolanim::ftco_800bffd0(fp, 9, 0);
}

/// `ftColl_CreateAbsorbHit(gobj, absorb)`.
pub fn ftColl_CreateAbsorbHit(gobj: G, absorb: *const AbsorbDesc) {
    let fp = fpg(gobj);
    fp.set_x2218_b6(1);
    fp.set_x2218_b7(0);
    unsafe {
        fp.absorb_hit.bone = (*fp.parts.add((*absorb).x0_bone_id as usize)).joint;
        fp.absorb_hit.size = (*absorb).x10_size;
        fp.absorb_hit.offset = (*absorb).x4_offset;
    }
}

/// `ftColl_CreateReflectHit(gobj, reflect, cb)`.
pub fn ftColl_CreateReflectHit(gobj: G, reflect: *const ReflectDesc, cb: HSD_GObjEvent) {
    let fp = fpg(gobj);
    fp.set_reflecting(1);
    fp.set_x2218_b4(0);
    fp.reflect_hit_cb = cb;
    unsafe {
        fp.ReflectAttr.x1A30_maxDamage = (*reflect).x4_max_damage;
        fp.ReflectAttr.x1A34_damageMul = (*reflect).x18_damage_mul;
        fp.ReflectAttr.x1A38_speedMul = (*reflect).x1C_speed_mul;
        fp.set_x2218_b5((*reflect).x20_behavior);
        fp.reflect_hit.bone = (*fp.parts.add((*reflect).x0_bone_id as usize)).joint;
        fp.reflect_hit.size = (*reflect).x14_size;
        fp.reflect_hit.offset = (*reflect).x8_offset;
    }
}

/// `ftColl_HurtboxInit(fp, hurt, init)`.
pub fn ftColl_HurtboxInit(fp: *mut Fighter, hurt: *mut FighterHurtCapsule, init: *const ftHurtboxInit) {
    unsafe {
        (*hurt).capsule.bone_idx = (*init).bone_idx as i32;
        (*hurt).height = (*init).height;
        (*hurt).is_grabbable = (*init).is_grabbable as i32;
        (*hurt).capsule.state = HurtCapsule_Enabled;
        (*hurt).capsule.bone = (*(*fp).parts.add((*hurt).capsule.bone_idx as usize)).joint;
        (*hurt).capsule.a_offset = (*init).a_offset;
        (*hurt).capsule.b_offset = (*init).b_offset;
        (*hurt).capsule.scale = (*init).scale;
        (*fp).set_x221A_b6(1);
    }
}
