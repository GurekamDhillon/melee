//! CPU controller init: `ftCo_800A101C` (kinds/ftCommon/ftCo_0A01.c) and `ftCo_800B9704`
//! (ftcpuattack.c). `Fighter_UnkProcessDeath_80068354` runs them for every fighter, human or
//! CPU, so their two `HSD_Randf` draws are part of every match start (and every respawn).
//!
//! Not ported here: `ftCo_800A0FB0`'s floor probe (`mpCheckFloor` under the fighter, which needs
//! the stage collision; TODO(group2 mp)). The caller passes its result as `floor` (`None` = no
//! floor found, the C's else branch). `FTAI_KIND` is the identity (vanilla kinds; the TARGET_PC
//! `ftAi_Kind` maps only m-ex kinds).

use melee_types::{Fighter, FighterKind, Ft_Kind_Donkey, Ft_Kind_GKoops, Ft_Kind_Koopa, Ft_Kind_Nana};

/// `ftCo_800B9704`: the CPU's reaction delay.
pub fn ftco_800b9704(fp: &mut Fighter, randf: f32) {
    let cpu = &mut fp.cpu;
    let rand = randf;
    // ftcpuattack.c: (10 - level) * (rand * 15.0F + 15.0F) + 10.0F, stored to an int field.
    let t = melee_core::fp::fmadds(rand, 15.0, 15.0);
    let v = melee_core::fp::fmadds((10 - cpu.level) as f32, t, 10.0);
    cpu.x34 = v as i32;
    if cpu.kind == 7 {
        cpu.x34 /= 2;
    }
}

/// `ftCo_800A101C(fp, kind, level, x14)`. `rand` is `HSD_Randf` (called twice, in C order:
/// once for `x7C`/`x80`, once inside `ftCo_800B9704`). `nana_partner` is `ftCo_800A589C(fp)`'s
/// position and facing (only read for Nana).
pub fn ftco_800a101c(
    fp: &mut Fighter,
    kind: i32,
    level: i32,
    x14: i32,
    floor: Option<(f32, f32)>,
    nana_partner: Option<(melee_types::Vec, f32)>,
    rand: &mut dyn FnMut() -> f32,
) {
    let ai_kind: FighterKind = fp.kind;
    let cur_pos = fp.cur_pos;
    {
        let c = &mut fp.cpu;
        if ai_kind == Ft_Kind_Nana {
            c.set_xF9_b2(1);
            c.kind = 6;
            c.x3C = 15.0;
        } else {
            c.kind = kind as u32;
            c.x3C = 40.0;
        }
        c.level = level;
        c.x14 = x14;
        match c.kind {
            1 | 25 => {
                c.x18 = 0xC;
                c.x1C = 0xC;
                c.x20 = 0;
            }
            3 => {
                c.x18 = 1;
                c.x1C = 1;
                c.x20 = 0xB;
            }
            15 => {
                c.x18 = 0;
                c.x1C = 0;
                c.x20 = 0;
            }
            _ => {
                c.x18 = 1;
                c.x1C = 1;
                c.x20 = 0xA;
            }
        }
        c.x24 = 0x12C;
        c.x28 = 0;
        c.x2C = 5;
        c.x30 = 0;
        c.set_xF9_b0(0);
        c.x38 = 5.0;
        c.x40 = 50.0;
        c.x44 = core::ptr::null_mut();
        c.x48 = core::ptr::null_mut();
        c.x50 = 0;
        c.x98 = cur_pos;
        let (fx, fy) = floor.unwrap_or((cur_pos.x, cur_pos.y));
        c.x54.x = fx;
        c.x64.x = fx;
        c.x54.y = fy;
        c.x64.y = fy;
        c.x60 = 0;
        c.x74.y = 0.0;
        c.x74.x = 0.0;
        c.x6C.y = 0.0;
        c.x6C.x = 0.0;
        c.set_xF8_b0(0);
        c.set_xF8_b12(0);
    }
    // `temp_r6 = 10.0 * HSD_Randf()`: a double product truncated to int.
    let r = rand();
    let t = (10.0f64 * r as f64) as i32;
    {
        let c = &mut fp.cpu;
        c.x7C = t;
        c.x80 = t;
        c.x84 = 0;
        c.x90 = 0;
        c.x88 = 0;
        c.x8C = 0;
        c.csP = core::ptr::null_mut();
        c.command_duration = 0;
        c.write_pos = c.buffer.as_mut_ptr();
        c.xA4 = 0;
        c.xC8 = 0;
        c.xEC = 0;
        c.set_xF8_b5(0);
        c.set_xF8_b6(0);
        c.set_xF8_b7(0);
        c.set_xF9_b1(0);
        c.buttons = 0;
        c.lstick.x = 0;
        c.lstick.y = 0;
        c.cstick.x = 0;
        c.cstick.y = 0;
        c.rtrigger = 0;
        c.ltrigger = 0;
        c.set_xF9_b2(0);
        c.set_xF9_b3(0);
        c.set_xF9_b4(0);
        c.set_xF9_b6(0);
        c.set_xF9_b5(0);
        c.set_xF9_b7(0);
        c.set_xFA_b7(0);
        c.set_xFB_b0(0);
        c.x94 = 0;
        c.set_xFA_b1(0);
        c.set_xFA_b2(0);
        c.set_xFA_b34(1);
        c.set_xFA_b5(0);
        c.set_xFA_b6(0);
        c.x55C = 1.0;
        c.x560 = 1.0;
        c.x564 = 0.5 * (c.x55C + c.x560);
        c.x568 = 2.0;
        c.x444 = &mut c.xFC[5];
        c.x448 = c.xFC.as_mut_ptr();
        for e in c.xFC.iter_mut() {
            e.x0 = 0;
            e.lstick.x = 0;
            e.lstick.y = 0;
            e.x4 = 0;
            e.x5 = 0;
            match (ai_kind == Ft_Kind_Nana, nana_partner) {
                (true, Some((pos, facing))) => {
                    e.cur_pos = pos;
                    e.facing_dir = facing;
                }
                (true, None) => panic!("ftCo_800A101C on Nana needs ftCo_800A589C's partner (TODO(group3 nana))"),
                _ => {
                    e.cur_pos.x = 0.0;
                    e.cur_pos.y = 0.0;
                    e.cur_pos.z = 0.0;
                    e.facing_dir = 1.0;
                }
            }
        }
    }
    let g = fp.co_attrs.gravity;
    let v = fp.co_attrs.jump_v_initial_velocity * fp.co_attrs.air_jump_v_multiplier;
    let zero_g = g < 0.00001 && g > -0.00001;
    fp.cpu.x558 = if zero_g { 10.0 } else { ((v * v) as f64 / (2.0 * g as f64)) as f32 };
    fp.cpu.half_width = 10.0;
    fp.cpu.half_height = 10.0;
    fp.cpu.x570 = 1.0;
    let r = rand();
    ftco_800b9704(fp, r);
    fp.cpu.x56C = match ai_kind {
        k if k == Ft_Kind_Donkey || k == Ft_Kind_Koopa => 8.5,
        k if k == Ft_Kind_GKoops => 17.0,
        k if k == Ft_Kind_Nana => 1.0,
        _ => 3.5,
    };
}
