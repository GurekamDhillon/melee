//! Mario (src/melee/ft/kinds/ftMario).

pub mod ftmario {
    pub fn ftMr_Init_UnkDemoCallbacks0(arg0: i32, arg1: &mut i32, arg2: &mut i32) {
        if arg0 != 10 {
            if arg0 >= 10 {
                return;
            }
            if arg0 < 9 {
                return;
            }
            *arg2 = 14;
            *arg1 = 14;
        } else {
            *arg2 = 15;
            *arg1 = 15;
        }
    }
}

pub mod ftmariospecials {
    use melee_types::{ftMr_MS_SpecialAirS, ftMr_MS_SpecialS, Fighter};
    pub fn ftMr_SpecialS_CheckItemCapeRemove(fp: &Fighter) -> bool {
        let msid = fp.motion_id as i32;
        !(msid >= ftMr_MS_SpecialS as i32 && msid <= ftMr_MS_SpecialAirS as i32)
    }
}
