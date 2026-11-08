//! src/melee/ft/ftcpuattack.c: CPU attack selection, grab/escape/guard reactions and the
//! incoming-hit predictor. Bit-exact float semantics of the shipping build (fma_sites.py):
//!   * every `a * t + b` in f32 (motion prediction, `count`-frame lookahead, `3.0f * d + p`)
//!     is fused: `melee_core::fp::fmadds`;
//!   * `halfRange = 0.5 * x570 + 0.5` and each `X - 0.5 * (f64)(g * sq)` are f64 fma
//!     (`f64::mul_add`);
//!   * `sqrtf` is math_ppc.h's (`melee_runtime::gekko::sqrtf`), float->int casts saturate
//!     (`melee_runtime::cabi`), `%` by zero follows gwtool (`cabi::srem`).
//! `ABS(x)` is `x < 0 ? -x : x` (ABS(-0.0) == -0.0), kept. `sdata2_order` (MUST_MATCH only) and
//! the MUST_MATCH-only `sqrtf_store`/`ftCo_CpuAttackValue` shims have no PC body.
//! `ftCo_800B9704` is the existing `crate::ftcpu::ftco_800b9704` (verified), drawn here.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::too_many_arguments)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
use melee_core::fp::Fmadd as _;
#[allow(unused_imports)]
use melee_types::{Fighter, FighterKind, Ft_Kind_Donkey, Ft_Kind_GKoops, Ft_Kind_Koopa, Ft_Kind_Nana};
use crate::tu::ftcmdscript::{self as cs, ftco_800b463c as cmd0, ftco_800b46b8 as cmd1};
use crate::types::*;
use melee_core::fp::fmadds;
use melee_runtime::cabi;
use melee_runtime::gekko::sqrtf;

pub type G = *mut HSD_GObj;

/// Callees / globals in TUs not ported yet (settable; unset panics). TODO rewire.
pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! stubs {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO stub: ", stringify!($name))),
                }
            }
        )*};
    }
    stubs! {
        HSD_RANDF HSD_Randf() -> f32;
        GENO_DEFINE_AI_LIKE Geno_DefineAiLike(kind: i32) -> i32;
        FTKB_MEX_KIRBY_LIKE ftKb_MexKirbyLike(kind: i32) -> i32;
        SCRIPT_GAME_CAPS_MAX_JUMPS ScriptGame_CapsMaxJumps(fp: *mut Fighter) -> i32;
        STAGE_INFO_GRKIND stage_info_grkind() -> i32;
        PLINK_HEAD plink_head(link: i32) -> G;
        FTCO_800A0C8C ftCo_800A0C8C(fp: *mut Fighter);
        FTCO_800A0098 ftCo_800A0098(fp: *mut Fighter);
        FTCO_800A05F4 ftCo_800A05F4(fp: *mut Fighter);
        FTCO_800A0798 ftCo_800A0798(fp: *mut Fighter);
        FTCO_800A0FB0 ftCo_800A0FB0(p: *mut Vec3, line: *mut i32, flags: *mut u32, n: *mut Vec3, a: i32, b: i32, c: i32, x0: f32, y0: f32, x1: f32, y1: f32, z: f32) -> i32;
        FTCO_800A1AB4 ftCo_800A1AB4(fp: *mut Fighter, t: *mut Fighter) -> f32;
        FTCO_800A1BA8 ftCo_800A1BA8(fp: *mut Fighter) -> i32;
        FTCO_800A1C44 ftCo_800A1C44(fp: *mut Fighter) -> i32;
        FTCO_800A1CA8 ftCo_800A1CA8(fp: *mut Fighter) -> i32;
        FTCO_800A2170 ftCo_800A2170(fp: *mut Fighter, t: *mut Fighter) -> i32;
        FTCO_800A28D0 ftCo_800A28D0(fp: *mut Fighter, f: f32) -> i32;
        FTCO_800A2998 ftCo_800A2998(fp: *mut Fighter, f: f32) -> i32;
        FTCO_800A2A70 ftCo_800A2A70(fp: *mut Fighter, b: i32) -> f32;
        FTCO_800A2BD4 ftCo_800A2BD4(fp: *mut Fighter) -> i32;
        FTCO_800A2C08 ftCo_800A2C08(fp: *mut Fighter) -> i32;
        FTCO_800A3134 ftCo_800A3134(fp: *mut Fighter) -> i32;
        FTCO_800A3200 ftCo_800A3200(fp: *mut Fighter) -> i32;
        FTCO_800A4E8C ftCo_800A4E8C(fp: *mut Fighter, v: *mut Vec3) -> *mut Fighter;
        FTCO_800A5908 ftCo_800A5908(it: *mut Item) -> i32;
        FTCO_800A5980 ftCo_800A5980(fp: *mut Fighter) -> i32;
        FTCO_800A59C0 ftCo_800A59C0(it: *mut Item) -> i32;
        FTCO_800A59E4 ftCo_800A59E4(it: *mut Item) -> i32;
        FTCO_800A5A28 ftCo_800A5A28(it: *mut Item) -> i32;
        FTCO_800A5A5C ftCo_800A5A5C(it: *mut Item) -> i32;
        FTCO_800A5A90 ftCo_800A5A90(fp: *mut Fighter) -> i32;
        FTCO_IS_GRABBING ftCo_IsGrabbing(fp: *mut Fighter) -> i32;
        FTCO_IS_ALLY ftCo_IsAlly(a: *mut Fighter, b: *mut Fighter) -> i32;
        MPISLAND_8005AC14 mpIsland_8005AC14(pos: *mut Vec3, f: f32) -> *mut mp_UnkStruct0;
        MPISLAND_8005ACE8 mpIsland_8005ACE8(isl: *mut mp_UnkStruct0, a: *mut Vec3, b: *mut Vec3);
        MP_CHECK_CEILING mpCheckCeiling(x0: f32, y0: f32, x1: f32, y1: f32, p: *mut Vec3, line: *mut i32, flags: *mut u32, n: *mut Vec3, a: i32, b: i32) -> i32;
        MP_CHECK_ALL mpCheckAll(p: *mut Vec3, line: *mut i32, flags: *mut u32, n: *mut Vec3, a: i32, b: i32, x0: f32, y0: f32, x1: f32, y1: f32) -> i32;
        IT_8026B960 it_8026B960(g: *mut HSD_GObj) -> f32;
        IT_8026B924 it_8026B924(g: *mut HSD_GObj) -> i32;
        IT_8026C1B4 it_8026C1B4(g: *mut HSD_GObj) -> i32;
        FTLIB_ISFIGHTER ftLib_IsFighter(g: *mut HSD_GObj) -> i32;
        LBCOLL_8000ACFC lbColl_8000ACFC(fp: *mut Fighter, h: *mut HitCapsule) -> i32;
        LBCOLL_80006094 lbColl_80006094(a: *mut Vec3, b: *mut Vec3, c: *mut Vec3, d: *mut Vec3, e: *mut Vec3, f: *mut Vec3, s0: f32, s1: f32) -> i32;
    }
}

/// `ftCo_AttackEntry` (ftcpuattack.c-local; GC 0x24, all 4-byte fields).
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct ftCo_AttackEntry {
    pub cmd: i32,
    pub x04: i32,
    pub x08: f32,
    pub x0C: f32,
    pub x10: f32,
    pub x14: f32,
    pub weight: f32,
    pub x1C: i32,
    pub x20: i32,
}

/// `ftCo_CollData` (ftcpuattack.c-local scratch for the mp probes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ftCo_CollData {
    pub pad0: [u8; 4],
    pub p: Vec3,
    pub n: Vec3,
    pub line: i32,
    pub flags: u32,
}
impl ftCo_CollData {
    fn new() -> Self {
        Self { pad0: [0; 4], p: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, n: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, line: 0, flags: 0 }
    }
}

const PI_DEG5: f64 = 0.08726646192371845;
const PI_DEG15: f64 = 0.26179938577115536;
const PI_DEG40: f64 = 0.6981316953897476;
const PI_DEG45: f64 = 0.7853981573134661;
const PI_DEG50: f64 = 0.8726646192371845;
const PI_DEG60: f64 = 1.0471975430846214;
const PI_DEG75: f64 = 1.3089969288557768;

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}
#[inline(always)]
fn abs_d(x: f64) -> f64 {
    if x < 0.0 { -x } else { x }
}
#[inline(always)]
fn near_zero(x: f32) -> bool {
    x < 0.00001f32 && x > -0.00001f32
}

#[inline(always)]
fn fpm<'a>(p: *mut Fighter) -> &'a mut Fighter {
    unsafe { &mut *p }
}

/// `FTAI_KIND(fp)` (ftaikind.h, TARGET_PC).
pub fn ftai_kind(fp: &Fighter) -> i32 {
    melee_core::ct!("ftai_kind");
    let kind = fp.kind as i32;
    if kind < Ft_Kind_Mex0 as i32 {
        return kind;
    }
    let like = ext::Geno_DefineAiLike(kind);
    if like >= 0 { like } else { kind }
}
/// `FTKB_IS_KIRBY` (TARGET_PC).
fn is_kirby(kind: i32) -> bool {
    kind == Ft_Kind_Kirby as i32
        || (kind >= Ft_Kind_Mex0 as i32 && kind < Ft_Kind_Max as i32 && ext::ftKb_MexKirbyLike(kind) != 0)
}
fn canon_kind(kind: i32) -> i32 {
    if is_kirby(kind) { Ft_Kind_Kirby as i32 } else { kind }
}

fn tables<'a>() -> &'a Fighter_804D64FC_t {
    unsafe { &*cs::ext::Fighter_804D64FC }
}
fn table_list(t: *mut *mut core::ffi::c_void, kind: i32) -> *const ftCo_AttackEntry {
    unsafe { *t.offset(kind as isize) as *const ftCo_AttackEntry }
}

fn neutral(fp: &mut Fighter) {
    cmd1(fp, CpuCmd_SetLstickX as u8, 0);
    cmd1(fp, CpuCmd_SetLstickY as u8, 0);
}

fn clear_target_and_finish(fp: &mut Fighter) {
    fp.cpu.x44 = core::ptr::null_mut();
    fp.cpu.x18 = fp.cpu.x1C;
    ext::ftCo_800A0C8C(fp);
}

fn select_attack(fp: &mut Fighter, e: &ftCo_AttackEntry) -> i32 {
    let s = fp.x34_scale.y;
    fp.cpu.x6C.x = s * e.x08;
    fp.cpu.x6C.y = s * e.x10;
    fp.cpu.x74.x = s * e.x0C;
    fp.cpu.x74.y = s * e.x14;
    e.cmd
}

fn push_cc(fp: &mut Fighter, v: i32) -> bool {
    let c = &mut fp.cpu;
    if (c.xEC as u32) < 8 {
        c.xCC_array[c.xEC as usize] = v;
        c.xEC = c.xEC.wrapping_add(1);
        return true;
    }
    false
}
fn push_a8(fp: &mut Fighter, v: i32) {
    let c = &mut fp.cpu;
    if (c.xC8 as u32) < 8 {
        c.xA8_array[c.xC8 as usize] = v;
        c.xC8 = c.xC8.wrapping_add(1);
    }
}

/// One mover's state for the prediction (position, velocity, -terminal velocity, gravity).
#[derive(Clone, Copy)]
pub struct Mover {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub term_neg: f32,
    pub grav: f32,
}

/// The attacker's predicted y after `t` frames (`air` is the TARGET's ground_or_air, as in C).
pub fn pred_self_y(m: &Mover, t: f32, air: bool) -> f32 {
    melee_core::ct!("pred_self_y");
    if !air {
        return fmadds(m.vy, t, m.y);
    }
    // C uses fpY (not fpVy) in the fall-time estimate; kept.
    let v = if near_zero(m.grav) { 1000.0f32 } else { -(m.term_neg - m.y) / m.grav };
    if v <= 0.0 {
        fmadds(m.vy, t, m.y)
    } else if t < v {
        let sq = sqrtf(t);
        (m.y as f64 + (-0.5f64).fmadd((m.grav * sq) as f64, (m.vy * t) as f64)) as f32
    } else {
        let sq = sqrtf(v);
        (m.y as f64 + ((m.term_neg * (t - v)) as f64 + (-0.5f64).fmadd((m.grav * sq) as f64, (m.vy * t) as f64))) as f32
    }
}

/// The target's predicted y after `t` frames, relative to `self_y`.
pub fn pred_rel_y(m: &Mover, t: f32, air: bool, self_y: f32) -> f32 {
    melee_core::ct!("pred_rel_y");
    if !air {
        return fmadds(m.vy, t, m.y) - self_y;
    }
    let v = if near_zero(m.grav) { 1000.0f32 } else { -(m.term_neg - m.vy) / m.grav };
    if v <= 0.0 {
        fmadds(m.vy, t, m.y) - self_y
    } else if t < v {
        let sq = sqrtf(t);
        ((-0.5f64).fmadd((m.grav * sq) as f64, fmadds(m.vy, t, m.y) as f64) - self_y as f64) as f32
    } else {
        let sq = sqrtf(v);
        (((m.term_neg * (t - v)) as f64 + (-0.5f64).fmadd((m.grav * sq) as f64, fmadds(m.vy, t, m.y) as f64))
            - self_y as f64) as f32
    }
}

fn mover_of(f: &Fighter) -> Mover {
    Mover {
        x: f.cur_pos.x,
        y: f.cur_pos.y,
        vx: f.pos_delta.x,
        vy: f.pos_delta.y,
        term_neg: -f.co_attrs.terminal_velocity,
        grav: f.co_attrs.gravity,
    }
}

/// Is `cmd` excluded by the CPU's xCC list?
fn excluded(cpu: &CpuFighter, cmd: i32) -> bool {
    (0..cpu.xEC as usize).any(|j| cpu.xCC_array[j] == cmd)
}

/// Collect a candidate (xA8 whitelist, else the `x80 % x1C` cadence).
fn collect(cpu: &CpuFighter, e: &ftCo_AttackEntry, out: &mut std::vec::Vec<ftCo_AttackEntry>) {
    if cpu.xC8 != 0 {
        for j in 0..cpu.xC8 as usize {
            if cpu.xA8_array[j] == e.cmd {
                out.push(*e);
            }
        }
    } else if cabi::srem(cpu.x80, e.x1C) == 0 {
        out.push(*e);
    }
}

/// The weighted random pick shared by every selector; `None` = zero total weight.
fn weighted_pick(cands: &[ftCo_AttackEntry], line: u32) -> Option<usize> {
    let r = ext::HSD_Randf();
    let mut sum = 0.0f32;
    for c in cands {
        sum += c.weight;
    }
    if near_zero(sum) {
        return None;
    }
    let inv = (1.0f64 / sum as f64) as f32;
    let mut acc = 0.0f32;
    for (i, c) in cands.iter().enumerate() {
        acc += c.weight;
        if acc * inv >= r {
            return Some(i);
        }
    }
    panic!("ftcpuattack.c:{line:#X}");
}

fn reset_ranges(cpu: &mut CpuFighter) -> f32 {
    cpu.x74.y = 0.0;
    cpu.x74.x = 0.0;
    cpu.x6C.y = 0.0;
    cpu.x6C.x = 0.0;
    (0.5f64.fmadd(cpu.x570 as f64, 0.5)) as f32
}

/// `ftCo_800B4AB0` / `ftCo_800B52AC` body (`reach` = None for 4AB0).
fn select_vs_fighter(fp: &mut Fighter, target: &Fighter, list: *const ftCo_AttackEntry, reach: Option<f32>, line: u32) -> i32 {
    let half = reset_ranges(&mut fp.cpu);
    if list.is_null() {
        return 0;
    }
    let me = mover_of(fp);
    let tg = mover_of(target);
    let (range_f, range_b) = if target.facing_dir as f64 > 0.0 {
        (target.cpu.x55C, target.cpu.x560)
    } else {
        (target.cpu.x560, target.cpu.x55C)
    };
    let x568 = target.cpu.x568;
    let air = target.ground_or_air == GA_Air;
    let mut cands = std::vec::Vec::new();
    let mut p = list;
    unsafe {
        while (*p).cmd != 0 {
            let e = &*p;
            p = p.add(1);
            if e.x20 > fp.cpu.level || excluded(&fp.cpu, e.cmd) {
                continue;
            }
            let t = e.x04 as f32;
            let relx = fmadds(tg.vx, t, tg.x) - fmadds(me.vx, t, me.x);
            let self_y = pred_self_y(&me, t, air);
            let rel_y = pred_rel_y(&tg, t, air, self_y);
            let s = fp.x34_scale.y;
            let (mut dirx, mut diry, lower, upper);
            match reach {
                None => {
                    if fp.facing_dir > 0.0 {
                        dirx = e.x08 * s;
                        diry = e.x0C * s;
                    } else {
                        dirx = -e.x0C * s;
                        diry = -e.x08 * s;
                    }
                    dirx *= half;
                    diry *= half;
                    upper = (e.x14 * s) * half;
                    lower = (e.x10 * s) * half;
                }
                Some(reach) => {
                    if fp.facing_dir > 0.0 {
                        dirx = e.x08 * s;
                        diry = fmadds(e.x0C, s, reach);
                    } else {
                        dirx = fmadds(-e.x0C, s, -reach);
                        diry = -e.x08 * s;
                    }
                    dirx *= half;
                    diry *= half;
                    lower = (e.x10 * s) * half;
                    upper = fmadds(e.x14, s, reach) * half;
                }
            }
            if upper > rel_y && lower < rel_y + x568 && dirx < relx + range_f && diry > relx - range_b {
                collect(&fp.cpu, e, &mut cands);
            }
        }
    }
    if cands.is_empty() {
        return 0;
    }
    match weighted_pick(&cands, line) {
        None => 0,
        Some(i) => select_attack(fp, &cands[i]),
    }
}

/// `ftCo_800B4AB0`: pick an attack that reaches `target` (predicted `x04` frames ahead).
pub fn ftCo_800B4AB0(fp: *mut Fighter, target: *mut Fighter, arg2: *const ftCo_AttackEntry) -> i32 {
    melee_core::ct!("ftCo_800B4AB0");
    select_vs_fighter(fpm(fp), unsafe { &*target }, arg2, None, 0xFA)
}

/// `ftCo_800B52AC`: as ftCo_800B4AB0 with a held weapon's extra `reach`.
pub fn ftCo_800B52AC(fp: *mut Fighter, target: *mut Fighter, arg2: *const ftCo_AttackEntry, reach: f32) -> i32 {
    melee_core::ct!("ftCo_800B52AC");
    select_vs_fighter(fpm(fp), unsafe { &*target }, arg2, Some(reach), 0x1C5)
}

/// `ftCo_800B5AB0`: pick an attack that reaches item `arg1`.
pub fn ftCo_800B5AB0(fp: *mut Fighter, arg1: *mut Item, arg2: *const ftCo_AttackEntry) -> i32 {
    melee_core::ct!("ftCo_800B5AB0");
    let fp = fpm(fp);
    let x50 = unsafe { &*arg1 };
    let list = arg2;
    if list.is_null() {
        return 0;
    }
    let me = mover_of(fp);
    let attr = unsafe { &*x50.xCC_item_attr };
    let it = Mover {
        x: x50.pos.x,
        y: x50.pos.y,
        vx: x50.x40_vel.x,
        vy: x50.x40_vel.y,
        term_neg: -attr.x14_fall_speed_max,
        grav: attr.x10_fall_speed,
    };
    let size_half = (0.5f64 * (x50.xC1C.right + x50.xC1C.left) as f64) as f32;
    let y_bound = x50.xC1C.top + x50.xC1C.bottom;
    let air = x50.ground_or_air == GA_Air;
    let mut cands = std::vec::Vec::new();
    let mut p = list;
    unsafe {
        while (*p).cmd != 0 {
            let e = &*p;
            p = p.add(1);
            if e.x20 > fp.cpu.level || excluded(&fp.cpu, e.cmd) {
                continue;
            }
            let t = e.x04 as f32;
            let relx = fmadds(it.vx, t, it.x) - fmadds(me.vx, t, me.x);
            let self_y = pred_self_y(&me, t, air);
            let rel_y = pred_rel_y(&it, t, air, self_y);
            let s = fp.x34_scale.y;
            let (dirx, diry) = if fp.facing_dir > 0.0 { (e.x08 * s, e.x0C * s) } else { (-e.x0C * s, -e.x08 * s) };
            let v = e.x14 * s;
            let lower = e.x10 * s;
            if v > rel_y && lower < rel_y + y_bound && dirx < relx + size_half && diry > relx - size_half {
                collect(&fp.cpu, e, &mut cands);
            }
        }
    }
    if cands.is_empty() {
        return 0;
    }
    match weighted_pick(&cands, 0x26A) {
        None => 0,
        Some(i) => cands[i].cmd,
    }
}

/// `ftCo_800B6208`: weighted pick over a 0-terminated list.
pub fn ftCo_800B6208(arr: *const ftCo_AttackEntry) -> i32 {
    melee_core::ct!("ftCo_800B6208");
    let r = ext::HSD_Randf();
    let mut sum = 0.0f32;
    let mut n = 0usize;
    unsafe {
        while (*arr.add(n)).cmd != 0 {
            sum += (*arr.add(n)).weight;
            n += 1;
        }
    }
    if n == 0 || near_zero(sum) {
        return 0;
    }
    let inv = (1.0f64 / sum as f64) as f32;
    let mut acc = 0.0f32;
    for i in 0..n {
        acc += unsafe { (*arr.add(i)).weight };
        if acc * inv >= r {
            return unsafe { (*arr.add(i)).cmd };
        }
    }
    panic!("ftcpuattack.c:0x28A");
}

/// `ftCo_800B630C`: is the fighter in any attacking motion?
pub fn ftCo_800B630C(fp: *mut Fighter) -> bool {
    melee_core::ct!("ftCo_800B630C");
    let f = fpm(fp);
    let m = f.motion_id as i32;
    if m >= ftCo_MS_Attack11 as i32 && m <= ftCo_MS_AttackAirLw as i32 {
        return true;
    }
    if ext::ftCo_800A5A90(fp) != 0 {
        return true;
    }
    let k = ftai_kind(f);
    if k == Ft_Kind_Donkey as i32 {
        if m <= ftDk_MS_ThrowAirFLw as i32 {
            return false;
        }
    } else if is_kirby(k) {
        if m <= ftKb_MS_AttackDashAir as i32 {
            return false;
        }
    } else if k == Ft_Kind_Peach as i32 {
        if m <= ftPe_MS_FloatFallB as i32 {
            return false;
        }
    } else if m < ftCo_MS_Count as i32 {
        return false;
    }
    true
}

/// `ftCo_800B63D8`: wait a random, level-scaled number of frames.
pub fn ftCo_800B63D8(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B63D8");
    let f = fpm(fp);
    let r = ext::HSD_Randf();
    let (mul, add) = match f.cpu.level {
        0 => (40, 5),
        1 => (35, 5),
        2 => (30, 5),
        3 => (25, 5),
        4 => (25, 0),
        5 => (20, 0),
        6 => (15, 0),
        7 => (10, 0),
        8 => (5, 0),
        _ => return,
    };
    let n = cabi::f2i32((mul as f32 * r) as f64).wrapping_add(add);
    cmd1(f, CpuCmd_WaitFor as u8, n as u8);
}

fn tap_r_wait5(f: &mut Fighter) {
    cmd0(f, CpuCmd_PressR as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd0(f, CpuCmd_ReleaseR as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 5);
}

/// `ftCo_800B658C`: attack state step.
pub fn ftCo_800B658C(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B658C");
    let f = fpm(fp);
    let target = f.cpu.x44;
    if target.is_null() {
        clear_target_and_finish(f);
        return;
    }
    if f.cpu.xF8_b6() != 0 && f.ground_or_air == GA_Ground && ext::HSD_Randf() < 0.05f32 * f.cpu.level as f32 {
        f.cpu.set_xF8_b6(0);
        if ftCo_800BA2E8(fp, target) {
            return;
        }
    }
    let k = ftai_kind(f);
    let m = f.motion_id as i32;
    if k == Ft_Kind_Ness as i32 {
        if m >= ftNs_MS_SpecialNStart as i32 && m <= ftNs_MS_SpecialNEnd as i32 {
            ftCo_800B920C(fp);
            return;
        }
        if m >= ftNs_MS_SpecialHiStart as i32 && m <= ftNs_MS_SpecialAirHiRebound as i32 {
            ftCo_800B92D4(fp);
            return;
        }
    } else if k == Ft_Kind_Yoshi as i32 {
        if m >= ftYs_MS_SpecialAirSStart_0 as i32 && m <= ftYs_MS_SpecialAirSEnd as i32 {
            ftCo_800B9340(fp);
            return;
        }
    } else if k == Ft_Kind_Samus as i32 {
        if m == ftSs_MS_SpecialNHold as i32 {
            tap_r_wait5(f);
        }
    } else if k == Ft_Kind_Donkey as i32 {
        if m == ftDk_MS_SpecialNLoop as i32 {
            tap_r_wait5(f);
        }
    } else if k == Ft_Kind_Zelda as i32 && m >= ftZd_MS_SpecialSLoop as i32 && m <= ftZd_MS_SpecialAirSEnd as i32 {
        ftCo_800B9504(fp);
        return;
    }
    if ftCo_800B630C(fp) {
        ext::ftCo_800A0C8C(fp);
        f.cpu.xA4 = 0;
        return;
    }
    if f.cpu.xA4 == 0 {
        if f.ground_or_air == GA_Ground && ext::ftCo_800A2C08(fp) == 0 {
            ext::ftCo_800A0098(fp);
            return;
        }
        if !ftCo_800B8A9C(fp) {
            clear_target_and_finish(f);
            return;
        }
    }
    ftCo_800B63D8(fp);
    let a4 = f.cpu.xA4;
    cs::ftco_800b4880(f, a4);
    f.cpu.xA4 = 0;
}

fn flick_lstick_x(f: &mut Fighter, half: i8, full: i8) {
    cmd1(f, CpuCmd_SetLstickX as u8, 0);
    cmd1(f, CpuCmd_SetLstickY as u8, 0);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd1(f, CpuCmd_SetLstickX as u8, half as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd1(f, CpuCmd_SetLstickX as u8, full as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd1(f, CpuCmd_SetLstickX as u8, 0);
    cmd0(f, CpuCmd_Done as u8);
}

fn press_a_with_lstick_x(f: &mut Fighter, x: i8) {
    cmd1(f, CpuCmd_SetLstickY as u8, 0);
    cmd1(f, CpuCmd_SetLstickX as u8, 0);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd1(f, CpuCmd_SetLstickY as u8, 0);
    cmd1(f, CpuCmd_SetLstickX as u8, x as u8);
    cmd0(f, CpuCmd_PressA as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd1(f, CpuCmd_SetLstickX as u8, 0);
    cmd1(f, CpuCmd_SetLstickY as u8, 0);
    cmd0(f, CpuCmd_ReleaseA as u8);
    cmd0(f, CpuCmd_Done as u8);
}

/// Island ledge distances from the fighter: (|x - right|, |x - left|) of `mpIsland_8005ACE8`.
fn island_dists(f: &mut Fighter) -> Option<(f32, f32)> {
    let isl = ext::mpIsland_8005AC14(&mut f.cur_pos, -10.0);
    if isl.is_null() {
        return None;
    }
    let mut a = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::mpIsland_8005ACE8(isl, &mut a, &mut b);
    Some((abs_c(f.cur_pos.x - b.x), abs_c(f.cur_pos.x - a.x)))
}

fn level_roll(f: &Fighter) -> bool {
    ext::HSD_Randf() < 0.05f32 * f.cpu.level as f32
}

/// `ftCo_800B683C`: grab follow-up (pummel / throw choice).
pub fn ftCo_800B683C(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B683C");
    let f = fpm(fp);
    if f.victim_gobj.is_null() {
        clear_target_and_finish(f);
        return;
    }
    let mut koopa = false;
    let k = ftai_kind(f);
    let m = f.motion_id as i32;
    if k == Ft_Kind_Donkey as i32 && (0x15F..=0x164).contains(&m) {
        let Some((d2, d1)) = island_dists(f) else {
            cmd0(f, CpuCmd_Done as u8);
            return;
        };
        if d2 < d1 {
            if d2 as f64 > 10.0 {
                cmd1(f, CpuCmd_SetLstickY as u8, 0);
                cmd1(f, CpuCmd_SetLstickX as u8, 0x7F);
                cmd0(f, CpuCmd_Done as u8);
            } else {
                press_a_with_lstick_x(f, 0x7F);
            }
        } else if d1 as f64 > 10.0 {
            cmd1(f, CpuCmd_SetLstickY as u8, 0);
            cmd1(f, CpuCmd_SetLstickX as u8, 0x81);
            cmd0(f, CpuCmd_Done as u8);
        } else {
            press_a_with_lstick_x(f, 0x81u8 as i8);
        }
        return;
    } else if (k == Ft_Kind_Koopa as i32 || k == Ft_Kind_GKoops as i32) && m == 0x15E {
        koopa = true;
    } else if is_kirby(k) && (0x164..=0x17E).contains(&m) {
        let v = if f.victim_gobj.is_null() { core::ptr::null_mut() } else { unsafe { (*f.victim_gobj).user_data as *mut Fighter } };
        if !v.is_null() && is_kirby(unsafe { (*v).kind } as i32) {
            cmd1(f, CpuCmd_SetLstickY as u8, 0);
            cmd1(f, CpuCmd_SetLstickX as u8, 0);
            cmd1(f, CpuCmd_WaitFor as u8, 1);
            cmd0(f, CpuCmd_PressA as u8);
            cmd1(f, CpuCmd_WaitFor as u8, 1);
            cmd0(f, CpuCmd_ReleaseA as u8);
            cmd0(f, CpuCmd_Done as u8);
        } else {
            neutral(f);
            cmd1(f, CpuCmd_WaitFor as u8, 1);
            cmd1(f, CpuCmd_SetLstickY as u8, (-0x7Fi8) as u8);
            cmd1(f, CpuCmd_WaitFor as u8, 1);
            cmd1(f, CpuCmd_SetLstickY as u8, 0);
            cmd0(f, CpuCmd_Done as u8);
        }
        return;
    } else if k == Ft_Kind_Popo as i32 {
        if f.cpu.x94 == 0 && f.cpu.level > 4 && level_roll(f) {
            cs::ftco_800b4880(f, 0x3B);
            return;
        }
    } else if k == Ft_Kind_Mario as i32 {
        if f.cpu.x94 == 0 && f.cpu.level < 5 && level_roll(f) {
            cs::ftco_800b4880(f, 0x35);
            return;
        }
    } else if k == Ft_Kind_Fox as i32 {
        if f.cpu.x94 == 0 && f.cpu.level > 4 && level_roll(f) {
            cs::ftco_800b4880(f, 0x3D);
            return;
        }
    } else if k == Ft_Kind_Captain as i32 && f.cpu.x94 == 0 && f.cpu.level > 4 && level_roll(f) {
        let pct = unsafe { (*((*f.victim_gobj).user_data as *mut Fighter)).dmg.x1830_percent };
        cs::ftco_800b4880(f, if pct < 100.0 { 0x39 } else { 0x3A });
        return;
    }
    if ext::ftCo_IsGrabbing(fp) != 0 {
        let tmp = ext::HSD_Randf();
        if ((tmp as f64 * (1.0 + f.cpu.x94 as f64)) as f32) < 0.5 {
            cs::ftco_800b4880(f, if koopa { 0x11 } else { 2 });
            f.cpu.x94 += 1;
            return;
        }
        if let Some((d2, d1)) = island_dists(f) {
            if d2 > d1 {
                if (d1 as f64) < 30.0 {
                    flick_lstick_x(f, -0x50, -0x7F);
                    return;
                }
            } else if (d2 as f64) < 30.0 {
                flick_lstick_x(f, 0x50, 0x7F);
                return;
            }
        }
        let r = ext::HSD_Randf() as f64;
        neutral(f);
        cmd1(f, CpuCmd_WaitFor as u8, 1);
        let (c, a, b) = if r < 0.25 {
            (CpuCmd_SetLstickY, 0x50, 0x7F)
        } else if r < 0.5 {
            (CpuCmd_SetLstickY, 0xB0, 0x81)
        } else if r < 0.75 {
            (CpuCmd_LstickXForward, 0x50, 0x7F)
        } else {
            (CpuCmd_LstickXForward, 0xB0, 0x81)
        };
        cmd1(f, c as u8, a);
        cmd1(f, CpuCmd_WaitFor as u8, 1);
        cmd1(f, c as u8, b);
        cmd1(f, CpuCmd_WaitFor as u8, 1);
        neutral(f);
        cmd0(f, CpuCmd_Done as u8);
    } else {
        ext::ftCo_800A0C8C(fp);
    }
}

fn reset_b34(f: &mut Fighter) {
    f.cpu.x18 = f.cpu.x1C;
    f.cpu.set_xF8_b34(0);
    cmd0(f, CpuCmd_Done as u8);
}

/// `ftCo_800B7180`: jump-toward-target sequencing.
pub fn ftCo_800B7180(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B7180");
    let f = fpm(fp);
    let t = f.cpu.x44;
    if t.is_null() {
        reset_b34(f);
    }
    match f.cpu.xF8_b34() {
        1 => {
            ext::ftCo_800A05F4(fp);
            f.cpu.set_xF8_b34(3);
        }
        2 => {
            ext::ftCo_800A0798(fp);
            f.cpu.set_xF8_b34(3);
        }
        3 => {
            if f.ground_or_air == GA_Air {
                if f.pos_delta.y as f64 > 0.0 {
                    cmd1(f, CpuCmd_SetLstickY as u8, 0);
                    cmd1(f, CpuCmd_LstickXTowardFighter as u8, 0x7F);
                    cmd0(f, CpuCmd_Done as u8);
                } else if unsafe { (*t).cur_pos.y } > f.cur_pos.y {
                    ext::ftCo_800A05F4(fp);
                } else {
                    reset_b34(f);
                }
            } else {
                reset_b34(f);
            }
        }
        _ => reset_b34(f),
    }
}

fn ceiling_between(f: &Fighter, t: &Fighter) -> bool {
    let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut n = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let (mut line, mut flags) = (0i32, 0u32);
    ext::mpCheckCeiling(f.cur_pos.x, f.cur_pos.y, t.cur_pos.x, t.cur_pos.y, &mut p, &mut line, &mut flags, &mut n, -1, -1) != 0
}

fn angle_to(f: &Fighter, t: &Fighter) -> f32 {
    cs::ext::lb_8000D008(t.cur_pos.y - f.cur_pos.y, abs_c(t.cur_pos.x - f.cur_pos.x))
}

/// `ftCo_800B732C`: should the CPU jump/up-attack toward its target? (0 no, 1 up, 2 level).
pub fn ftCo_800B732C(fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800B732C");
    let f = fpm(fp);
    let t0 = f.cpu.x44;
    if t0.is_null() || f.ground_or_air == GA_Air {
        return 0;
    }
    let t = unsafe { &mut *t0 };
    if ext::ftCo_800A3200(t0) != 0 {
        if t.cur_pos.y < f.cur_pos.y || ceiling_between(f, t) {
            return 0;
        }
        ext::ftCo_800A1AB4(fp, t0);
        return (angle_to(f, t) > 1.3089969f32) as i32;
    }
    if f.cpu.xF9_b3() == 0 {
        return 0;
    }
    if f.cpu.kind == 0x10 {
        return ftCo_800B7638(fp);
    }
    if ext::ftCo_800A2BD4(fp) == 0 || f.cpu.level < 4 || ext::ftCo_800A28D0(fp, 4.0) != 0 {
        return 0;
    }
    if t.ground_or_air == GA_Air {
        if t.cur_pos.y < f.cur_pos.y || ceiling_between(f, t) {
            return 0;
        }
        let d = ext::ftCo_800A1AB4(fp, t0);
        if angle_to(f, t) as f64 > PI_DEG45 && (d as f64) < 30.0 {
            return 1;
        }
    } else {
        if cabi::srem(f.cpu.x7C, 60) != 0 {
            return 0;
        }
        if ext::HSD_Randf() as f64 > 0.1 {
            return 0;
        }
        let d = ext::ftCo_800A1AB4(fp, t0) as f64;
        let a = angle_to(f, t) as f64;
        if d > 30.0 && d < 40.0 && a > -PI_DEG5 && a < PI_DEG5 {
            return 2;
        }
    }
    0
}

/// `ftCo_800B7638`.
pub fn ftCo_800B7638(fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800B7638");
    let f = fpm(fp);
    let t0 = f.cpu.x44;
    if t0.is_null() {
        return 0;
    }
    let t = unsafe { &mut *t0 };
    if f.ground_or_air == GA_Air {
        if ext::ScriptGame_CapsMaxJumps(fp) <= f.x1968_jumpsUsed as i32 {
            return 0;
        }
        if f.pos_delta.y as f64 > 0.0 {
            return 0;
        }
    }
    if ((t.cur_pos.y - f.cur_pos.y) as f64) < 0.0 {
        return 0;
    }
    if t.ground_or_air == GA_Air {
        if ceiling_between(f, t) {
            return 0;
        }
        ext::ftCo_800A1AB4(fp, t0);
        if angle_to(f, t) as f64 > PI_DEG45 {
            return 1;
        }
    } else {
        let d = ext::ftCo_800A1AB4(fp, t0) as f64;
        let a = angle_to(f, t) as f64;
        if d > 30.0 && d < 40.0 && a > -PI_DEG5 && a < PI_DEG5 {
            return 2;
        }
    }
    0
}

/// `ftCo_800A0FB0(...)` under a point: no floor within 1000 below `x` (from `y + 5`).
fn no_floor_below(c: &mut ftCo_CollData, x: f32, y: f32) -> bool {
    ext::ftCo_800A0FB0(&mut c.p, &mut c.line, &mut c.flags, &mut c.n, -1, -1, -1, x, (5.0 + y as f64) as f32, x,
                       (y as f64 - 1000.0) as f32, 0.0) == 0
}

/// `ftCo_CpuCanUseRangedAttack`: line of sight to the target, or no floor `range` ahead.
fn can_use_ranged(f: &Fighter, target: *mut Fighter, c: &mut ftCo_CollData, range: f32) -> bool {
    if !target.is_null() {
        let t = unsafe { &*target };
        if ext::mpCheckAll(&mut c.p, &mut c.line, &mut c.flags, &mut c.n, -1, -1, f.cur_pos.x, f.cur_pos.y, t.cur_pos.x, t.cur_pos.y) != 0 {
            return true;
        }
    }
    let y = f.cur_pos.y;
    let x = if f.facing_dir as f64 > 0.0 { f.cur_pos.x + range } else { f.cur_pos.x - range };
    no_floor_below(c, x, y)
}

/// `ftCo_800B77E8`: exclude ranged/unsafe attacks for this kind (fills xCC).
pub fn ftCo_800B77E8(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B77E8");
    let f = fpm(fp);
    let cpu_target = f.cpu.x44;
    if f.cpu.kind == 7 || ext::ftCo_800A1C44(fp) != 0 {
        push_cc(f, 0x34);
    }
    let mut c = ftCo_CollData::new();
    let k = canon_kind(ftai_kind(f)) as u32;
    let ranged = |f: &mut Fighter, c: &mut ftCo_CollData, r: f32| {
        let t = f.cpu.x44;
        can_use_ranged(f, t, c, r)
    };
    match k {
        Ft_Kind_Mario => {
            if ranged(f, &mut c, 22.0) {
                push_cc(f, 0x1F);
            }
        }
        Ft_Kind_Fox | Ft_Kind_Falco => {
            if ranged(f, &mut c, 62.0) {
                push_cc(f, 0x1B);
            }
        }
        Ft_Kind_Captain | Ft_Kind_Ganon => {
            if ranged(f, &mut c, 102.0) {
                push_cc(f, 0x26);
            }
            if ranged(f, &mut c, 58.0) {
                push_cc(f, 0x1B);
                push_cc(f, 0x37);
                push_cc(f, 0x38);
            }
        }
        Ft_Kind_Pikachu | Ft_Kind_Pichu => {
            if ranged(f, &mut c, 57.0) {
                push_cc(f, 0x1B);
            }
        }
        Ft_Kind_Koopa | Ft_Kind_GKoops | Ft_Kind_Yoshi => {
            let r = match k {
                Ft_Kind_Koopa => 27.0,
                Ft_Kind_GKoops => 53.0,
                _ => 16.0,
            };
            if f.ground_or_air == GA_Ground {
                if ranged(f, &mut c, r) {
                    push_cc(f, 0x26);
                }
            } else if f.cpu.xFA_b5() == 0 {
                push_cc(f, 0x26);
            }
        }
        Ft_Kind_Luigi => {
            if ranged(f, &mut c, 60.0) {
                push_cc(f, 0x26);
            }
        }
        Ft_Kind_Peach => {
            if ranged(f, &mut c, 32.0) {
                push_cc(f, 0x26);
            }
        }
        Ft_Kind_Kirby => {
            if f.ground_or_air == GA_Air && f.cpu.xFA_b5() == 0 {
                push_cc(f, 0x26);
            }
            // retail reads fp+0x2238 (u.gw.x2238_panicCharge in the source), on GC Kirby's hat.kind; the 64-bit
            // union puts hat.kind at a different offset than gw.x2238, so read the field itself
            let charge = unsafe { f.u.kb.hat.kind } as i32;
            if charge == 3 || charge == 0xD || charge == 0x10 {
                push_cc(f, 0x11);
            } else {
                push_cc(f, 0x12);
                if f.x34_scale.y < unsafe { (*cpu_target).x34_scale.y } {
                    push_cc(f, 0x11);
                }
            }
        }
        _ => {}
    }
}

/// `ftCo_800B885C`: level-based attack cooldown window (true = hold off).
pub fn ftCo_800B885C(fp: *mut Fighter) -> bool {
    melee_core::ct!("ftCo_800B885C");
    let c = &fpm(fp).cpu;
    let (m, lim) = match c.level {
        0 => (300, 240),
        1 => (300, 180),
        2 => (300, 120),
        3 => (240, 120),
        4 => (240, 60),
        _ => return false,
    };
    cabi::srem(c.x80, m) <= lim
}

/// `ftCo_800B89CC`: may the CPU attack in the air?
pub fn ftCo_800B89CC(fp: *mut Fighter) -> bool {
    melee_core::ct!("ftCo_800B89CC");
    let f = fpm(fp);
    let tmp = f.cpu.x44;
    if ext::stage_info_grkind() == Gr_Kind_Kongo as i32 && (f.cur_pos.x > 25.0 || f.cur_pos.x < -25.0) {
        return false;
    }
    if ext::ftCo_800A3200(tmp) != 0 {
        return true;
    }
    if f.cpu.xF9_b4() == 0 {
        return false;
    }
    if f.cpu.xFA_b5() == 0 && ftai_kind(f) == Ft_Kind_Koopa as i32 {
        return false;
    }
    f.cpu.level >= 4
}

fn weapon_reach(item: *mut Item) -> f32 {
    if ext::ftCo_800A59E4(item) == 0 {
        return 0.0;
    }
    let r = tables().x24 as *const f32;
    let i = match unsafe { (*item).kind } {
        It_Kind_Harisen => 0,
        It_Kind_LipStick => 1,
        It_Kind_StarRod => 2,
        It_Kind_Sword => 3,
        It_Kind_Bat => 4,
        It_Kind_Parasol => 5,
        _ => return 0.0,
    };
    unsafe { *r.add(i) }
}

/// `ftCo_800B8A9C`: choose the CPU's next attack (into cpu.xA4); true when one was chosen.
pub fn ftCo_800B8A9C(fp: *mut Fighter) -> bool {
    melee_core::ct!("ftCo_800B8A9C");
    let f = fpm(fp);
    if f.cpu.xF9_b2() == 0 {
        return false;
    }
    let target = f.cpu.x44;
    f.cpu.xEC = 0;
    f.cpu.xC8 = 0;
    if target.is_null() || f.motion_id as i32 == ftCo_MS_Pass as i32 || ext::ftCo_800A1C44(target) != 0
        || ftCo_800B885C(fp) || ext::ftCo_800A1CA8(fp) != 0
    {
        f.cpu.xA4 = 0;
        return false;
    }
    {
        let t2 = unsafe { &*target };
        let m = t2.motion_id as i32;
        if m >= ftCo_MS_Catch as i32 && m <= ftCo_MS_EscapeAir as i32 {
            push_cc(f, 0x28);
            push_cc(f, 0x29);
        } else if t2.x34_scale.y > f.x34_scale.y {
            push_cc(f, 0x28);
            push_cc(f, 0x29);
            if is_kirby(ftai_kind(f)) {
                push_cc(f, 0x11);
            }
        }
    }
    ftCo_800B77E8(fp);
    let k = ftai_kind(f);
    let tb = tables();
    let done = |f: &mut Fighter, r: i32| -> bool {
        if r != 0 {
            f.cpu.xA4 = r;
            true
        } else {
            false
        }
    };
    if f.ground_or_air == GA_Air {
        if ftCo_800B89CC(fp) {
            let r = ftCo_800B4AB0(fp, target, table_list(tb.x8, k));
            if done(f, r) {
                return true;
            }
        }
        f.cpu.xA4 = 0;
        return false;
    }
    if ext::ftCo_800A3134(target) != 0 || ext::ftCo_800A3200(target) != 0 {
        push_a8(f, 0xA);
        push_a8(f, 0xE);
        let r = ftCo_800B4AB0(fp, target, table_list(tb.x4, k));
        if done(f, r) {
            return true;
        }
        f.cpu.xA4 = 0;
        return false;
    }
    if !f.item_gobj.is_null() && ext::ftCo_800A59E4(unsafe { (*f.item_gobj).user_data } as *mut Item) != 0 {
        let item = unsafe { (*f.item_gobj).user_data } as *mut Item;
        let reach = weapon_reach(item);
        let r = ftCo_800B52AC(fp, target, table_list(tb.x18, k), reach);
        if done(f, r) {
            return true;
        }
    }
    if f.cpu.level > 5 && ftCo_800B9F6C(target) {
        let r = ftCo_800B4AB0(fp, target, table_list(tb.x10, k));
        if done(f, r) {
            return true;
        }
    }
    let t3 = f.cpu.x44;
    let open_floor = if !f.item_gobj.is_null() || ext::ftCo_800A3200(t3) != 0 {
        false
    } else {
        let mut c = ftCo_CollData::new();
        let t = unsafe { &*t3 };
        no_floor_below(&mut c, t.cur_pos.x, t.cur_pos.y)
    };
    if open_floor {
        let r = ftCo_800B4AB0(fp, target, table_list(tb.x1C, k));
        if r != 0 {
            f.cpu.xA4 = r;
            f.cpu.set_xF8_b7(1);
            return true;
        }
    }
    let r = ftCo_800B4AB0(fp, target, table_list(tb.x4, k));
    if done(f, r) {
        return true;
    }
    if f.cpu.x50 != 0 {
        let item = f.cpu.x50 as usize as *mut Item;
        let r = ftCo_800B5AB0(fp, item, table_list(tb.x14, k));
        if done(f, r) {
            return true;
        }
    }
    f.cpu.xA4 = 0;
    false
}

/// `ftCo_800B9020`: special-move state step.
pub fn ftCo_800B9020(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B9020");
    let f = fpm(fp);
    if f.cpu.x44.is_null() {
        clear_target_and_finish(f);
        return;
    }
    let k = ftai_kind(f);
    let m = f.motion_id as i32;
    if k == Ft_Kind_Ness as i32 {
        if m >= ftNs_MS_SpecialNStart as i32 && m <= ftNs_MS_SpecialNEnd as i32 {
            ftCo_800B920C(fp);
            f.cpu.x90 += 1;
            return;
        }
        if m >= ftNs_MS_SpecialHiStart as i32 && m <= ftNs_MS_SpecialAirHiRebound as i32 {
            ftCo_800B92D4(fp);
            f.cpu.x90 += 1;
            return;
        }
    } else if k == Ft_Kind_Yoshi as i32 {
        if m >= ftYs_MS_SpecialAirSStart_0 as i32 && m <= ftYs_MS_SpecialAirSEnd as i32 {
            ftCo_800B9340(fp);
            f.cpu.x90 += 1;
            return;
        }
    } else if k == Ft_Kind_Zelda as i32 && m >= ftZd_MS_SpecialSLoop as i32 && m <= ftZd_MS_SpecialAirSEnd as i32 {
        ftCo_800B9504(fp);
        return;
    }
    if ftCo_800B630C(fp) {
        neutral(f);
        ext::ftCo_800A0C8C(fp);
        f.cpu.xA4 = 0;
    } else if f.cpu.xA4 != 0 {
        let a = f.cpu.xA4;
        cs::ftco_800b4880(f, a);
        f.cpu.xA4 = 0;
    } else if ftCo_800B9CBC(fp) {
        if f.cpu.xA4 != 0 {
            let a = f.cpu.xA4;
            cs::ftco_800b4880(f, a);
            f.cpu.xA4 = 0;
            f.cpu.x90 += 1;
        } else {
            clear_target_and_finish(f);
        }
    } else {
        clear_target_and_finish(f);
    }
}

/// `ftCo_800B920C`: Ness PK Flash release when level with the target.
pub fn ftCo_800B920C(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B920C");
    let f = fpm(fp);
    let t = f.cpu.x44;
    if t.is_null() {
        clear_target_and_finish(f);
    } else if (abs_c(unsafe { (*t).cur_pos.y } - f.cur_pos.y) as f64) < 25.0 {
        cmd0(f, CpuCmd_ReleaseB as u8);
        cmd0(f, CpuCmd_Done as u8);
        clear_target_and_finish(f);
    } else {
        cmd1(f, CpuCmd_LstickTowardFighter as u8, 0x7F);
        cmd0(f, CpuCmd_Done as u8);
    }
}

/// `ftCo_800B92D4`: PK Thunder steering.
pub fn ftCo_800B92D4(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B92D4");
    let f = fpm(fp);
    if f.cpu.x44.is_null() {
        clear_target_and_finish(f);
    } else {
        cmd1(f, 0x94, 0x7F);
        cmd0(f, 0x7F);
    }
}

/// `ftCo_800B9340`: Yoshi egg roll steering.
pub fn ftCo_800B9340(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B9340");
    let f = fpm(fp);
    let t = f.cpu.x44;
    let isl = ext::mpIsland_8005AC14(&mut f.cur_pos, -100.0);
    if isl.is_null() || t.is_null() || isl != ext::mpIsland_8005AC14(unsafe { &mut (*t).cur_pos }, -100.0) {
        f.cpu.x44 = core::ptr::null_mut();
        f.cpu.x18 = f.cpu.x1C;
        cmd0(f, CpuCmd_ReleaseB as u8);
        cmd1(f, CpuCmd_WaitFor as u8, 1);
        cmd0(f, CpuCmd_PressB as u8);
        cmd1(f, CpuCmd_WaitFor as u8, 5);
        cmd0(f, CpuCmd_ReleaseB as u8);
        cmd1(f, CpuCmd_WaitFor as u8, 1);
        cmd0(f, CpuCmd_Done as u8);
    } else if (abs_c(unsafe { (*isl).x14.x } - f.cur_pos.x) as f64) < 10.0 {
        cmd1(f, CpuCmd_SetLstickY as u8, 0);
        cmd1(f, CpuCmd_SetLstickX as u8, (-0x7Fi8) as u8);
        cmd0(f, CpuCmd_Done as u8);
    } else if (abs_c(unsafe { (*isl).x8.x } - f.cur_pos.x) as f64) < 10.0 {
        cmd1(f, CpuCmd_SetLstickY as u8, 0);
        cmd1(f, CpuCmd_SetLstickX as u8, 0x7F);
        cmd0(f, CpuCmd_Done as u8);
    } else {
        cmd1(f, CpuCmd_SetLstickY as u8, 0);
        cmd1(f, CpuCmd_LstickXTowardFighter as u8, 0x7F);
        cmd0(f, CpuCmd_Done as u8);
    }
}

/// `ftCo_800B9504`: Zelda Din's Fire steering.
pub fn ftCo_800B9504(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B9504");
    let f = fpm(fp);
    let g = unsafe { f.u.zd.x222C };
    if g.is_null() {
        ext::ftCo_800A0C8C(fp);
        return;
    }
    let j = unsafe { (*g).hsd_obj } as *mut HSD_JObj;
    assert!(!j.is_null(), "jobj.h: jobj");
    let mut sp14 = unsafe { (*j).translate };
    let t = ext::ftCo_800A4E8C(fp, &mut sp14);
    if t.is_null() {
        clear_target_and_finish(f);
        return;
    }
    let tp = unsafe { (*t).cur_pos };
    let d1 = abs_c(tp.x - f.cur_pos.x);
    let d2 = abs_c(sp14.x - f.cur_pos.x);
    if d2 > d1 {
        cmd0(f, CpuCmd_ReleaseB as u8);
        neutral(f);
        cmd1(f, CpuCmd_WaitFor as u8, 1);
    } else if (abs_c(sp14.x - tp.x) as f64) < 15.0 && abs_d(15.0 + tp.y as f64 - sp14.y as f64) < 15.0 {
        cmd0(f, CpuCmd_ReleaseB as u8);
        neutral(f);
        cmd1(f, CpuCmd_WaitFor as u8, 1);
    } else if 15.0f32 + tp.y > sp14.y {
        cmd1(f, CpuCmd_SetLstickY as u8, 0x7F);
    } else {
        cmd1(f, CpuCmd_SetLstickY as u8, 0x81);
    }
    cmd0(f, CpuCmd_Done as u8);
}

/// `ftCo_800B9704`: reaction delay (one HSD_Randf draw; body in `crate::ftcpu`).
pub fn ftCo_800B9704(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B9704");
    let r = ext::HSD_Randf();
    crate::ftcpu::ftco_800b9704(fpm(fp), r);
}

/// `ftCo_800B9790`: Ness special choice by angle/distance.
pub fn ftCo_800B9790(fp: *mut Fighter, arg1: f32, arg2: f32) -> bool {
    melee_core::ct!("ftCo_800B9790");
    let f = fpm(fp);
    let t = f.cpu.x44;
    let (a, d) = (arg1 as f64, arg2 as f64);
    if f.ground_or_air == GA_Air {
        if a < -PI_DEG40 && a > -PI_DEG50 && d < 53.0 && ext::ftCo_800A2C08(fp) != 0 {
            f.cpu.xA4 = 0x1B;
            return true;
        }
        return false;
    }
    if a < PI_DEG5 && a > -PI_DEG5 && d < 53.0 && ext::ftCo_800A2C08(fp) != 0 {
        f.cpu.xA4 = 0x1B;
        return true;
    }
    if f.cur_pos.y >= unsafe { (*t).cur_pos.y } {
        return false;
    }
    if a > PI_DEG60 {
        f.cpu.xA4 = 0x14;
    } else if (ext::HSD_Randf() as f64) < 0.1 {
        f.cpu.xA4 = 0x1F;
    }
    true
}

/// `ftCo_800B98C8`: Yoshi special choice.
pub fn ftCo_800B98C8(fp: *mut Fighter, arg1: f32, arg2: f32) -> bool {
    melee_core::ct!("ftCo_800B98C8");
    let f = fpm(fp);
    let t = f.cpu.x44;
    let (a, d) = (arg1 as f64, arg2 as f64);
    if f.ground_or_air == GA_Air || ext::ftCo_800A2C08(fp) == 0 || d > 80.0 {
        return false;
    }
    if a < PI_DEG5 && a > -PI_DEG5 {
        if unsafe { (*t).ground_or_air } == GA_Air || ext::ftCo_800A2170(fp, t) == 0 {
            return false;
        }
        f.cpu.xA4 = 0x1B;
    } else {
        if a < PI_DEG45 {
            return false;
        }
        f.cpu.xA4 = if a < PI_DEG60 { 0x31 } else if a < PI_DEG75 { 0x32 } else { 0x33 };
    }
    true
}

/// `ftCo_800B9A04`: held-item attack choice.
pub fn ftCo_800B9A04(fp: *mut Fighter, arg1: *mut Item, arg2: f32, arg3: f32) -> bool {
    melee_core::ct!("ftCo_800B9A04");
    let f = fpm(fp);
    let (a, d) = (arg2 as f64, arg3 as f64);
    if ext::ftCo_800A5908(arg1) != 0 || ext::ftCo_800A5A90(fp) != 0 {
        return false;
    }
    let t1 = ext::it_8026B960(f.item_gobj) as f64;
    if t1 < 60.0 && t1 > 0.0 {
        f.cpu.xA4 = 2;
        return true;
    }
    if ext::ftCo_800A59C0(arg1) != 0 {
        if near_zero(ext::it_8026B924(f.item_gobj) as f32) {
            f.cpu.xA4 = 0x28;
            return true;
        }
        if ext::ftCo_800A2C08(fp) == 0 {
            return false;
        }
        if a < PI_DEG5 && a > -PI_DEG5 && d > 30.0 {
            f.cpu.xA4 = if unsafe { (*arg1).kind } == It_Kind_StarRod { 0xF } else { 2 };
            return true;
        }
        f.cpu.xA4 = 0x2A;
        return true;
    }
    if ext::ftCo_800A5A5C(arg1) != 0 {
        f.cpu.xA4 = 2;
        return true;
    }
    if ext::ftCo_800A5A28(arg1) != 0 {
        if a > PI_DEG75 {
            f.cpu.xA4 = 0xD;
            return true;
        }
        if a < -PI_DEG75 {
            f.cpu.xA4 = 0xE;
            return true;
        }
        if ext::ftCo_800A2C08(fp) == 0 {
            return false;
        }
        if a < PI_DEG15 && a > -PI_DEG15 {
            f.cpu.xA4 = 0xF;
            return true;
        }
        if unsafe { (*arg1).kind } == It_Kind_M_Ball {
            f.cpu.xA4 = 0xE;
            return true;
        }
    } else {
        if ext::ftCo_800A59E4(arg1) != 0 {
            return false;
        }
        if d > 30.0 && a < PI_DEG15 && a > -PI_DEG15 {
            f.cpu.xA4 = 0x2A;
            return true;
        }
        return false;
    }
    false
}

/// `ftCo_800B9CBC`: special/item attack choice (after the reaction delay).
pub fn ftCo_800B9CBC(fp: *mut Fighter) -> bool {
    melee_core::ct!("ftCo_800B9CBC");
    let f = fpm(fp);
    let t = f.cpu.x44;
    f.cpu.xA4 = 0;
    if t.is_null() || f.cpu.xF9_b5() == 0 {
        return false;
    }
    if f.cpu.x34 > 0 {
        f.cpu.x34 -= 1;
        return false;
    }
    ftCo_800B9704(fp);
    if ext::ftCo_800A1BA8(fp) != 0 {
        return false;
    }
    let dist = ext::ftCo_800A1AB4(fp, t);
    let ang = angle_to(f, unsafe { &*t });
    if !f.item_gobj.is_null() {
        return ftCo_800B9A04(fp, unsafe { (*f.item_gobj).user_data } as *mut Item, ang, dist);
    }
    if ext::HSD_Randf() as f64 > 0.75 || (dist as f64) < 30.0 {
        return false;
    }
    let k = ftai_kind(f);
    let r = if k == Ft_Kind_Ness as i32 {
        ftCo_800B9790(fp, ang, dist)
    } else if k == Ft_Kind_Yoshi as i32 {
        ftCo_800B98C8(fp, ang, dist)
    } else if k == Ft_Kind_Samus as i32 {
        if f.ground_or_air == GA_Ground {
            false
        } else if (ang as f64) < -PI_DEG45 {
            f.cpu.xA4 = 0x26;
            true
        } else {
            false
        }
    } else {
        false
    };
    if r {
        return true;
    }
    if ext::ftCo_800A2C08(fp) == 0 {
        return false;
    }
    if (ang as f64) < PI_DEG5 && ang as f64 > -PI_DEG5 {
        f.cpu.xA4 = ftCo_800B6208(table_list(tables().xC, k));
        return f.cpu.xA4 != 0;
    }
    false
}

/// `ftCo_800B9F6C`: shielding?
pub fn ftCo_800B9F6C(fp: *mut Fighter) -> bool {
    melee_core::ct!("ftCo_800B9F6C");
    let m = fpm(fp).motion_id as i32;
    m == ftCo_MS_GuardOn as i32 || m == ftCo_MS_Guard as i32
}

/// `ftCo_800B9F90`: hold shield for a level-scaled time.
pub fn ftCo_800B9F90(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B9F90");
    let f = fpm(fp);
    if f.motion_id as i32 == ftCo_MS_Guard as i32 {
        let n = 9 - f.cpu.level;
        neutral(f);
        cmd0(f, CpuCmd_PressR as u8);
        if n != 0 {
            let w = n.wrapping_mul(cabi::f2i32((4.0f32 * ext::HSD_Randf()) as f64).wrapping_add(4));
            cmd1(f, CpuCmd_WaitFor as u8, w as u8);
        }
        cmd0(f, CpuCmd_Done as u8);
    } else {
        neutral(f);
        cmd0(f, CpuCmd_PressR as u8);
        cmd0(f, CpuCmd_Done as u8);
    }
}

/// `ftCo_800BA080`: down-B escape (Kirby stone etc.).
pub fn ftCo_800BA080(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800BA080");
    let f = fpm(fp);
    let m = f.motion_id as i32;
    neutral(f);
    if (0x16F..=0x178).contains(&m) {
        cmd0(f, CpuCmd_Done as u8);
    } else {
        cmd0(f, CpuCmd_ReleaseB as u8);
        cmd1(f, CpuCmd_WaitFor as u8, 1);
        cmd1(f, CpuCmd_SetLstickX as u8, 0);
        cmd1(f, CpuCmd_SetLstickY as u8, (-0x7Fi8) as u8);
        cmd0(f, CpuCmd_PressB as u8);
        cmd0(f, CpuCmd_Done as u8);
    }
}

fn roll(f: &mut Fighter, x: i8) {
    neutral(f);
    cmd0(f, CpuCmd_ReleaseR as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd1(f, CpuCmd_SetLstickX as u8, x as u8);
    cmd0(f, CpuCmd_PressR as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 2);
    cmd0(f, CpuCmd_ReleaseR as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 5);
    cmd1(f, CpuCmd_SetLstickX as u8, 0);
    cmd0(f, CpuCmd_Done as u8);
}

/// `ftCo_800BA160`: roll right.
pub fn ftCo_800BA160(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800BA160");
    roll(fpm(fp), 0x7F);
}
/// `ftCo_800BA224`: roll left.
pub fn ftCo_800BA224(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800BA224");
    roll(fpm(fp), -0x7F);
}

fn roll_room(fp: *mut Fighter, right: bool) -> f32 {
    ext::ftCo_800A2A70(fp, right as i32)
}
fn x20(fp: *mut Fighter) -> f32 {
    unsafe { *tables().x20.offset(ftai_kind(fpm(fp)) as isize) }
}

/// `ftCo_800BA2E8`: roll away if there is room.
pub fn ftCo_800BA2E8(fp: *mut Fighter, arg1: *mut Fighter) -> bool {
    melee_core::ct!("ftCo_800BA2E8");
    let away_right = ((fpm(fp).cur_pos.x - unsafe { (*arg1).cur_pos.x }) as f64) > 0.0;
    let order: [bool; 2] = if away_right { [false, true] } else { [true, false] };
    for right in order {
        if roll_room(fp, right) > x20(fp) {
            if right { ftCo_800BA224(fp) } else { ftCo_800BA160(fp) }
            return true;
        }
    }
    false
}

/// `ftCo_800BA674`: roll (preferring away when there is twice the room).
pub fn ftCo_800BA674(fp: *mut Fighter, arg1: *mut Fighter) {
    melee_core::ct!("ftCo_800BA674");
    let away_right = ((fpm(fp).cur_pos.x - unsafe { (*arg1).cur_pos.x }) as f64) > 0.0;
    if away_right {
        if roll_room(fp, false) as f64 > 2.0 * x20(fp) as f64 {
            ftCo_800BA160(fp);
        } else {
            ftCo_800BA224(fp);
        }
    } else if roll_room(fp, true) as f64 > 2.0 * x20(fp) as f64 {
        ftCo_800BA224(fp);
    } else {
        ftCo_800BA160(fp);
    }
}

fn fire_blaster(f: &mut Fighter) {
    let delay = 9 - f.cpu.level;
    cmd1(f, CpuCmd_SetLstickX as u8, 0);
    cmd1(f, CpuCmd_SetLstickY as u8, (-0x50i8) as u8);
    cmd0(f, CpuCmd_PressB as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd1(f, CpuCmd_SetLstickY as u8, 0);
    if delay != 0 {
        let w = delay.wrapping_mul(cabi::f2i32((4.0f32 * ext::HSD_Randf()) as f64).wrapping_add(4));
        cmd1(f, CpuCmd_WaitFor as u8, w as u8);
    }
    cmd0(f, CpuCmd_Done as u8);
}

fn retap_r(f: &mut Fighter) {
    cmd0(f, CpuCmd_ReleaseR as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd0(f, CpuCmd_PressR as u8);
    cmd1(f, CpuCmd_WaitFor as u8, 1);
    cmd0(f, CpuCmd_ReleaseR as u8);
    cmd0(f, CpuCmd_Done as u8);
}

/// `ftCo_800BA9A0`: defend against the threat ftCo_800BB9B4 found (xF8_b12: 1 fighter, 2 item
/// or blaster, 3 down-B).
pub fn ftCo_800BA9A0(fp: *mut Fighter) {
    melee_core::ct!("ftCo_800BA9A0");
    let f = fpm(fp);
    let m = f.motion_id as i32;
    if m == ftCo_MS_EscapeF as i32 || m == ftCo_MS_EscapeB as i32 || m == ftCo_MS_EscapeAir as i32 || m == ftCo_MS_EscapeN as i32 {
        ext::ftCo_800A0C8C(fp);
        return;
    }
    if m == 0xB2 {
        ftCo_800B9F90(fp);
        return;
    }
    let r4 = f.cpu.xF8_b12() as u32;
    if r4 == 0 {
        f.cpu.x18 = f.cpu.x1C;
        neutral(f);
        cmd0(f, CpuCmd_ReleaseR as u8);
        cmd0(f, CpuCmd_ReleaseB as u8);
        ext::ftCo_800A0C8C(fp);
        return;
    }
    let k = ftai_kind(f);
    let foxlike = k == Ft_Kind_Fox as i32 || k == Ft_Kind_Falco as i32;
    if f.ground_or_air == GA_Air {
        if foxlike {
            if r4 == 2 { fire_blaster(f) } else { retap_r(f) }
        } else if r4 == 3 {
            ftCo_800BA080(fp);
        } else {
            retap_r(f);
        }
    } else if r4 == 2 {
        let it = f.cpu.xF4;
        let kind = unsafe { (*it).kind };
        let dx = (f.cur_pos.x - unsafe { (*it).pos.x }) as f64;
        if kind == It_Kind_Flipper || kind == It_Kind_TaruCann {
            if ext::ftCo_800A2998(fp, 1.0) != 0 {
                ftCo_800B9F90(fp);
            } else if dx > 0.0 {
                ftCo_800BA160(fp);
            } else {
                ftCo_800BA224(fp);
            }
        } else if kind == It_Kind_DKinoko {
            if dx > 0.0 { ftCo_800BA224(fp) } else { ftCo_800BA160(fp) }
        } else if foxlike {
            if ext::HSD_Randf() as f64 > 0.5 { fire_blaster(f) } else { ftCo_800B9F90(fp) }
        } else {
            ftCo_800B9F90(fp);
        }
    } else if r4 == 3 {
        ftCo_800BA080(fp);
    } else if f.cpu.xF0.is_null() {
        ftCo_800B9F90(fp);
    } else {
        let t2 = f.cur_pos.x - unsafe { (*f.cpu.xF0).cur_pos.x };
        let v = if t2 < 0.0 { -t2 } else { t2 };
        if v as f64 > 30.0 {
            if t2 as f64 > 0.0 { ftCo_800BA160(fp) } else { ftCo_800BA224(fp) }
        } else if t2 as f64 > 0.0 {
            ftCo_800BA224(fp);
        } else {
            ftCo_800BA160(fp);
        }
    }
}

fn hit_live(fp: *mut Fighter, h: &mut HitCapsule, skip_element: u32) -> bool {
    h.state != HitCapsule_Disabled && h.state != HitCapsule_Enabled && h.x43_b2() == 0 && h.element != skip_element
        && ext::lbColl_8000ACFC(fp, h) == 0
}

/// `ftCo_800BB104`: will one of `arg1`'s hitboxes (extrapolated 3 frames) reach `arg2`?
pub fn ftCo_800BB104(fp: *mut Fighter, arg1: *mut Fighter, arg2: *mut Vec3, arg3: f32) -> bool {
    melee_core::ct!("ftCo_800BB104");
    for i in 0..4 {
        let h = unsafe { &mut (*arg1).x914[i] };
        if hit_live(fp, h, HitElement_Catch as u32) {
            let dx = h.x4C.x - h.x58.x;
            let dy = h.x4C.y - h.x58.y;
            let dz = h.x4C.z - h.x58.z;
            let mut sp28 = Vec3 { x: fmadds(3.0, dx, h.x4C.x), y: fmadds(3.0, dy, h.x4C.y), z: fmadds(3.0, dz, h.x4C.z) };
            let mut a = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let sc = h.scale;
            if ext::lbColl_80006094(&mut h.x4C, &mut sp28, arg2, arg2, &mut a, &mut b, sc, arg3) != 0 {
                return true;
            }
        }
    }
    false
}

/// `ftCo_CpuPredictHitboxPosition`.
pub fn predict_hitbox(h: &HitCapsule, frames: i32) -> Vec3 {
    melee_core::ct!("predict_hitbox");
    let fr = frames as f32;
    Vec3 {
        x: fmadds(h.x4C.x - h.x58.x, fr, h.x4C.x),
        y: fmadds(h.x4C.y - h.x58.y, fr, h.x4C.y),
        z: fmadds(h.x4C.z - h.x58.z, fr, h.x4C.z),
    }
}

/// `ftCo_800BB220`: will item `ip` hit us within a level-scaled lookahead? (0, 2, 3 reflect).
pub fn ftCo_800BB220(fp: *mut Fighter, ip: *mut Item, arg2: *mut Vec3, arg3: f32) -> i32 {
    melee_core::ct!("ftCo_800BB220");
    let f = fpm(fp);
    let it = unsafe { &mut *ip };
    if it.owner == f.gobj {
        return 0;
    }
    if ext::ftLib_IsFighter(it.owner) != 0 && ext::ftCo_IsAlly(fp, unsafe { (*it.owner).user_data } as *mut Fighter) != 0 {
        return 0;
    }
    let harmless = match it.kind {
        It_Kind_Star | It_Kind_Unk4 => true,
        _ => ext::ftCo_800A5980(ip as *mut Fighter) != 0,
    };
    if harmless {
        return 0;
    }
    let count: i32 = if f.cpu.level < 3 {
        cabi::f2i32((20.0f32 * ext::HSD_Randf()) as f64).wrapping_add(10)
    } else if f.cpu.level < 6 {
        cabi::f2i32((10.0f32 * ext::HSD_Randf()) as f64).wrapping_add(5)
    } else if f.cpu.level < 9 {
        cabi::f2i32((5.0f32 * ext::HSD_Randf()) as f64).wrapping_add(3)
    } else {
        1
    };
    let c = count as f32;
    let a2 = unsafe { *arg2 };
    let mut dst = Vec3 {
        x: fmadds(f.pos_delta.x, c, a2.x),
        y: fmadds(f.pos_delta.y, c, a2.y),
        z: fmadds(f.pos_delta.z, c, a2.z),
    };
    let ness = ftai_kind(f) == Ft_Kind_Ness as i32 && f.cpu.level > 3;
    for i in 0..4 {
        let h = unsafe { &mut (*ip).x5D4_hitboxes[i].hit };
        let hit = if hit_live(fp, h, HitElement_Inert as u32) {
            let mut p = predict_hitbox(h, count);
            let mut a = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let sc = h.scale;
            ext::lbColl_80006094(&mut h.x4C, &mut p, arg2, &mut dst, &mut a, &mut b, sc, arg3) != 0
        } else {
            false
        };
        if hit {
            if ness && count >= 21 && h.x42_b0() != 0 {
                return 3;
            }
            return 2;
        }
    }
    0
}

/// `ftCo_800BB768`: is `arg1` doing a move that comes down on us (Bowser/Yoshi down-B,
/// Bowser up-smash, Samus up-B)?
pub fn ftCo_800BB768(fp: *mut Fighter, arg1: *mut Fighter) -> bool {
    melee_core::ct!("ftCo_800BB768");
    let f = fpm(fp);
    let a = unsafe { &mut *arg1 };
    let m = a.motion_id as i32;
    let floor_between = |f: &mut Fighter, a: &Fighter| {
        let mut c = ftCo_CollData::new();
        ext::ftCo_800A0FB0(&mut c.p, &mut c.line, &mut c.flags, &mut c.n, -1, -1, -1, a.cur_pos.x, a.cur_pos.y, f.cur_pos.x,
                           (5.0 + f.cur_pos.y as f64) as f32, 0.0) != 0
    };
    let overlap = |f: &Fighter, a: &Fighter| {
        let (t2, t1, t3, t4) = (a.cur_pos.x, a.cpu.x564, f.cpu.x564, f.cur_pos.x);
        t2 + t1 + t3 > t4 && t2 - t1 - t3 < t4
    };
    match a.kind {
        Ft_Kind_GKoops | Ft_Kind_Koopa => {
            if m == ftKp_MS_SpecialLw as i32 || m == ftKp_MS_SpecialAirLw as i32 {
                if floor_between(f, a) {
                    return false;
                }
                if overlap(f, a) {
                    return true;
                }
            } else if m == ftCo_MS_AttackHi4 as i32
                && overlap(f, a)
                && a.cur_pos.y < f.cur_pos.y
                && f.cur_pos.y - a.cur_pos.y < 35.0
            {
                return true;
            }
        }
        Ft_Kind_Yoshi => {
            if m == ftYs_MS_SpecialLw as i32 || m == ftYs_MS_SpecialAirLw as i32 {
                if floor_between(f, a) {
                    return false;
                }
                if overlap(f, a) {
                    return true;
                }
            }
        }
        Ft_Kind_Samus => {
            if m == ftSs_MS_SpecialHi as i32 && a.cur_pos.y < f.cur_pos.y && (ext::ftCo_800A1AB4(fp, arg1) as f64) < 30.0 {
                return true;
            }
        }
        _ => {}
    }
    false
}

/// `ftCo_800BB9B4`: scan fighters then items for an incoming threat; sets xF8_b12/xF0/xF4.
pub fn ftCo_800BB9B4(fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800BB9B4");
    let f = fpm(fp);
    f.cpu.set_xF8_b12(0);
    f.cpu.xF0 = core::ptr::null_mut();
    f.cpu.xF4 = core::ptr::null_mut();
    if ext::ftCo_800A1C44(fp) != 0 || ftai_kind(f) == Ft_Kind_GKoops as i32 {
        return f.cpu.xF8_b12() as i32;
    }
    let half = 0.5f32 * f.cpu.x568;
    let mut sp24 = Vec3 { x: f.cur_pos.x, y: f.cur_pos.y + half, z: f.cur_pos.z };
    let mut g = ext::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !g.is_null() {
        if f.gobj != g {
            let other = unsafe { (*g).user_data } as *mut Fighter;
            if ext::ftCo_IsAlly(fp, other) == 0 {
                f.cpu.set_xF8_b12(ftCo_800BB104(fp, other, &mut sp24, half) as u8);
                if f.cpu.xF8_b12() != 0 {
                    f.cpu.xF0 = other;
                    return f.cpu.xF8_b12() as i32;
                }
                f.cpu.set_xF8_b12(ftCo_800BB768(fp, other) as u8);
                if f.cpu.xF8_b12() != 0 {
                    f.cpu.xF0 = other;
                    return f.cpu.xF8_b12() as i32;
                }
            }
        }
        g = unsafe { (*g).next };
    }
    let mut g = ext::plink_head(HSD_GOBJ_PLINK_ITEM);
    while !g.is_null() {
        let it = unsafe { (*g).user_data } as *mut Item;
        if ext::it_8026C1B4(g) != 0 {
            f.cpu.set_xF8_b12(ftCo_800BB220(fp, it, &mut sp24, half) as u8);
            if f.cpu.xF8_b12() != 0 {
                f.cpu.xF4 = it;
                return f.cpu.xF8_b12() as i32;
            }
        }
        g = unsafe { (*g).next };
    }
    f.cpu.xF8_b12() as i32
}

// ---- folded from ftcpu.rs (consolidation steps 21/22) ----
/// `ftCo_800B9704`: the CPU's reaction delay.
pub fn ftco_800b9704(fp: &mut Fighter, randf: f32) {
    melee_core::ct!("ftco_800b9704");
    let cpu = &mut fp.cpu;
    let rand = randf;
    // ftcpuattack.c: (10 - level) * (rand * 15.0F + 15.0F) + 10.0F, stored to an int field.
    let t = melee_core::fp::fmadds(rand, 15.0, 15.0);
    let v = melee_core::fp::fmadds((10 - cpu.level) as f32, t, 10.0);
    cpu.x34 = melee_core::ppc::fctiwz_f32(v);
    if cpu.kind == 7 {
        cpu.x34 /= 2;
    }
}
