//! Luigi (src/melee/ft/kinds/ftLuigi).

pub mod ftluigi {
    pub fn ftLg_Init_UnkDemoCallbacks0(arg0: i32, arg1: &mut i32, arg2: &mut i32) {
        if arg0 != 10 {
            if arg0 < 10 && arg0 >= 9 {
                *arg2 = 14;
                *arg1 = 14;
            }
        } else {
            *arg2 = 15;
            *arg1 = 15;
        }
    }
}

pub mod ftluigispecials {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftLuigiAttributes, Fighter};

    pub fn ftLg_SpecialAirSStart_Phys_fp(fp: &mut Fighter) {
        let a: &ftLuigiAttributes = da(fp);
        if fp.cmd_vars[0] != 0 {
            co::fall(fp, a.x20_LUIGI_GREENMISSILE_FALLING_SPEED, fp.co_attrs.terminal_velocity);
        }
        co::calc_self_accel_deaccel(fp, a.x1C_LUIGI_GREENMISSILE_UNK2);
    }
}
