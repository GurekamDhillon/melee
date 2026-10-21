//! src/melee/ft/ftcmdscript.c: the CPU command-script interpreter (ftCo_800B3E04) and the
//! script buffer writers. Float->s8 stores follow gwtool's lowering: saturate to i32, then
//! truncate (`(x as i32) as i8`). No fused sites.
#![allow(non_snake_case, static_mut_refs, non_upper_case_globals)]

use crate::types::*;
use melee_runtime::gekko::{cosf, sinf};

pub mod ext {
    use crate::types::*;
    /// TODO(move-to-core ftdata) `Fighter_804D64FC` (PlCo CPU tables; set by Fighter_LoadCommonData).
    pub static mut Fighter_804D64FC: *const Fighter_804D64FC_t = core::ptr::null();
    pub static mut LB_8000D008: Option<fn(f32, f32) -> f32> = None;
    /// `lb_8000D008` (atan2 with dead zone), ported in melee_modes::lb (upward crate): hook.
    pub fn lb_8000D008(y: f32, x: f32) -> f32 {
        match unsafe { LB_8000D008 } {
            Some(f) => f(y, x),
            None => panic!("TODO: lb_8000D008 not wired"),
        }
    }
}

#[inline(always)]
fn s8f(x: f32) -> i8 {
    (x as i32) as i8
}

const BUF_LEN: usize = 256;

/// `ftCo_800B3E04`: step the CPU command script by one frame.
pub fn ftco_800b3e04(fp: &mut Fighter) {
    let pos = fp.cur_pos;
    let facing = fp.facing_dir;
    let motion_id = fp.motion_id;
    let data = &mut fp.cpu;
    if data.csP.is_null() || data.command_duration == 0 {
        return;
    }
    data.command_duration -= 1;
    if data.command_duration != 0 {
        return;
    }
    let base = data.buffer.as_mut_ptr();
    let mut cur = data.csP;
    if (data.csP as usize) < base as usize || (data.csP as usize) >= base as usize + BUF_LEN {
        panic!("csP is bad address");
    }
    macro_rules! next {
        () => {{
            let v = unsafe { *cur };
            cur = unsafe { cur.add(1) };
            v
        }};
    }
    let pa = HSD_PAD_A as u32;
    let pb = HSD_PAD_B as u32;
    let px = HSD_PAD_X as u32;
    let py = HSD_PAD_Y as u32;
    while data.command_duration == 0 {
        let c = next!() as u8 as u32;
        match c {
            CpuCmd_PressA => data.buttons |= pa,
            CpuCmd_ReleaseA => data.buttons &= !pa,
            CpuCmd_PressB => data.buttons |= pb,
            CpuCmd_ReleaseB => data.buttons &= !pb,
            CpuCmd_PressX => data.buttons |= px,
            CpuCmd_ReleaseX => data.buttons &= !px,
            CpuCmd_PressY => data.buttons |= py,
            CpuCmd_ReleaseY => data.buttons &= !py,
            CpuCmd_PressStart => data.buttons |= HSD_PAD_START as u32,
            CpuCmd_ReleaseStart => data.buttons &= !(HSD_PAD_START as u32),
            CpuCmd_PressR => {
                data.rtrigger = 0xFF;
                data.buttons |= HSD_PAD_R as u32;
            }
            CpuCmd_ReleaseR => {
                data.rtrigger = 0;
                data.buttons &= !(HSD_PAD_R as u32);
            }
            CpuCmd_PressL => data.buttons |= HSD_PAD_L as u32,
            CpuCmd_ReleaseL => data.buttons &= !(HSD_PAD_L as u32),
            CpuCmd_PressZ => data.buttons |= HSD_PAD_Z as u32,
            CpuCmd_ReleaseZ => data.buttons &= !(HSD_PAD_Z as u32),
            CpuCmd_PressUp => data.buttons |= HSD_PAD_DPADUP as u32,
            CpuCmd_ReleaseUp => data.buttons &= !(HSD_PAD_DPADUP as u32),
            CpuCmd_PressDown => data.buttons |= HSD_PAD_DPADDOWN as u32,
            CpuCmd_ReleaseDown => data.buttons &= !(HSD_PAD_DPADDOWN as u32),
            CpuCmd_PressRight => data.buttons |= HSD_PAD_DPADRIGHT as u32,
            CpuCmd_ReleaseRight => data.buttons &= !(HSD_PAD_DPADRIGHT as u32),
            CpuCmd_PressLeft => data.buttons |= HSD_PAD_DPADLEFT as u32,
            CpuCmd_ReleaseLeft => data.buttons &= !(HSD_PAD_DPADLEFT as u32),
            CpuCmd_PressAFor => {
                data.buttons |= pa;
                data.command_duration = next!() as u8 as u32;
            }
            CpuCmd_ReleaseAFor => {
                data.buttons &= !pa;
                data.command_duration = next!() as u8 as u32;
            }
            CpuCmd_PressBFor => {
                data.buttons |= pb;
                data.command_duration = next!() as u8 as u32;
            }
            CpuCmd_ReleaseBFor => {
                data.buttons &= !pb;
                data.command_duration = next!() as u8 as u32;
            }
            CpuCmd_PressXFor => {
                data.buttons |= px;
                data.command_duration = next!() as u8 as u32;
            }
            CpuCmd_ReleaseXFor => {
                data.buttons &= !px;
                data.command_duration = next!() as u8 as u32;
            }
            CpuCmd_PressYFor => {
                data.buttons |= py;
                data.command_duration = next!() as u8 as u32;
            }
            CpuCmd_ReleaseYFor => {
                data.buttons &= !py;
                data.command_duration = next!() as u8 as u32;
            }
            CpuCmd_SetLstickX => data.lstick.x = next!(),
            CpuCmd_SetLstickY => data.lstick.y = next!(),
            CpuCmd_SetCstickX => data.cstick.x = next!(),
            CpuCmd_SetCstickY => data.cstick.y = next!(),
            CpuCmd_SetRtrigger => data.rtrigger = next!() as u8,
            CpuCmd_SetLtrigger => data.ltrigger = next!() as u8,
            CpuCmd_ReleaseAll => data.buttons = 0,
            CpuCmd_WaitFor => data.command_duration = next!() as u8 as u32,
            CpuCmd_LstickTowardDestination => {
                let magnitude = next!() as i32;
                let angle = ext::lb_8000D008(data.x54.y - pos.y, data.x54.x - pos.x);
                data.lstick.x = s8f(magnitude as f32 * cosf(angle));
                data.lstick.y = s8f(magnitude as f32 * sinf(angle));
            }
            CpuCmd_LstickXTowardDestination => {
                let v = next!();
                data.lstick.x = if data.x54.x > pos.x { v } else { v.wrapping_neg() };
            }
            CpuCmd_LstickTowardFighter => {
                let magnitude = next!() as i32;
                let target = data.x44;
                if !target.is_null() {
                    let tp = unsafe { (*target).cur_pos };
                    let angle = ext::lb_8000D008(tp.y - pos.y, tp.x - pos.x);
                    data.lstick.x = s8f(magnitude as f32 * cosf(angle));
                    data.lstick.y = s8f(magnitude as f32 * sinf(angle));
                }
            }
            CpuCmd_LstickXTowardFighter => {
                let target = data.x44;
                let mut stick_x = next!();
                if !target.is_null() {
                    if unsafe { (*target).cur_pos.x } < pos.x {
                        stick_x = stick_x.wrapping_neg();
                    }
                    data.lstick.x = stick_x;
                }
            }
            CpuCmd_LstickXForward => {
                let v = next!();
                data.lstick.x = if facing as f64 >= 0.0 { v } else { v.wrapping_neg() };
            }
            CpuCmd_WaitIfMotionId => {
                let m = next!() as u8;
                if motion_id as i64 == m as i64 {
                    data.command_duration = 1;
                    return;
                }
            }
            CpuCmd_Unk0x93 => data.x18 = next!() as u8 as i32,
            CpuCmd_LstickTowardDestinationClamped => {
                let a = next!() as i32;
                let b = next!() as i32;
                let angle = ext::lb_8000D008(data.x54.y - pos.y, data.x54.x - pos.x);
                let mut x = data.lstick.x as i32 + s8f(a as f32 * cosf(angle)) as i32;
                let cx = s8f(b as f32 * cosf(angle)) as i32;
                if x > cx {
                    x = cx;
                } else if x < -cx {
                    x = -cx;
                }
                data.lstick.x = x as i8;
                let mut y = data.lstick.y as i32 + s8f(a as f32 * sinf(angle)) as i32;
                let cy = s8f(b as f32 * sinf(angle)) as i32;
                if y > cy {
                    y = cy;
                }
                if y < -cy {
                    y = -cy;
                }
                data.lstick.y = y as i8;
            }
            CpuCmd_LstickXTowardDestinationClamped | CpuCmd_LstickForwardClamped => {
                let dx = next!() as i32;
                let clamp = next!() as i32;
                let fwd = if c == CpuCmd_LstickForwardClamped { facing as f64 > 0.0 } else { data.x54.x > pos.x };
                let mut x = if fwd { data.lstick.x as i32 + dx } else { data.lstick.x as i32 - dx };
                if x > clamp {
                    x = clamp;
                } else if x < -clamp {
                    x = -clamp;
                }
                data.lstick.x = x as i8;
            }
            CpuCmd_Done => {
                data.command_duration = 0;
                data.csP = core::ptr::null_mut();
                return;
            }
            _ => {}
        }
    }
    data.csP = cur;
}

/// `ftCo_800B462C`: reset the script write position.
pub fn ftco_800b462c(fp: &mut Fighter) {
    fp.cpu.write_pos = fp.cpu.buffer.as_mut_ptr();
}

/// `ftCo_800B463C`: append one byte to the script buffer.
pub fn ftco_800b463c(fp: &mut Fighter, cmd: u8) {
    let data = &mut fp.cpu;
    if data.write_pos as usize >= data.buffer.as_ptr() as usize + BUF_LEN {
        panic!("command script buffer over flow!");
    }
    unsafe {
        *data.write_pos = cmd as i8;
        data.write_pos = data.write_pos.add(1);
    }
}

/// `ftCo_800B46B8`.
pub fn ftco_800b46b8(fp: &mut Fighter, cmd: u8, arg: u8) {
    ftco_800b463c(fp, cmd);
    ftco_800b463c(fp, arg);
}

/// `ftCo_800B4778`.
pub fn ftco_800b4778(fp: &mut Fighter, cmd: u8, arg1: u8, arg2: u8) {
    ftco_800b463c(fp, cmd);
    ftco_800b463c(fp, arg1);
    ftco_800b463c(fp, arg2);
}

/// `ftCo_800B4880`: copy script `script_idx` from the loaded table into the buffer.
pub fn ftco_800b4880(fp: &mut Fighter, script_idx: i32) {
    let t = unsafe { ext::Fighter_804D64FC };
    assert!(!t.is_null(), "TODO Fighter_804D64FC not loaded");
    unsafe {
        let mut cmd = *(*t).cmdscripts.offset(script_idx as isize);
        while *cmd as u32 != CpuCmd_Done {
            ftco_800b463c(fp, *cmd);
            if *cmd as u32 > CpuCmd_OneArgEnd {
                cmd = cmd.add(1);
                ftco_800b463c(fp, *cmd);
            }
            if *cmd as u32 > CpuCmd_ZeroArgEnd {
                cmd = cmd.add(1);
                ftco_800b463c(fp, *cmd);
            }
            cmd = cmd.add(1);
        }
        ftco_800b463c(fp, *cmd);
    }
}

/// `ftCo_800B49F4`: terminate the script and start running it.
pub fn ftco_800b49f4(fp: &mut Fighter) {
    ftco_800b463c(fp, CpuCmd_Done as u8);
    fp.cpu.csP = fp.cpu.buffer.as_mut_ptr();
    fp.cpu.command_duration = 1;
}

/// `ftCo_800B4A78`: clear the CPU inputs and the script.
pub fn ftco_800b4a78(fp: &mut Fighter) {
    let data = &mut fp.cpu;
    data.buttons = 0;
    data.lstick.x = 0;
    data.lstick.y = 0;
    data.cstick.x = 0;
    data.cstick.y = 0;
    data.rtrigger = 0;
    data.ltrigger = 0;
    data.csP = core::ptr::null_mut();
    data.command_duration = 0;
    ftco_800b462c(fp);
}
