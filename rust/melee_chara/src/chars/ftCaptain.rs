//! Captain Falcon (src/melee/ft/kinds/ftCaptain).

pub mod ftcaptainspecialhi {
    use crate::chars::{caps_max_jumps, da};
    use melee_types::{ftCaptain_DatAttrs, Fighter};

    /// `ftCa_SpecialLw_800E49FC` (static): Falcon Dive state reset.
    pub fn ftCa_SpecialLw_800E49FC(fp: &mut Fighter) {
        let da: &ftCaptain_DatAttrs = da(fp);
        fp.x1968_jumpsUsed = caps_max_jumps(fp) as u8;
        unsafe {
            fp.mv.ca.specialhi.x0 = da.specialhi_air_var as _;
        }
        fp.cmd_vars[0] = 0;
        fp.cmd_vars[1] = da.specialhi_unk2 as u32;
        unsafe {
            fp.mv.ca.specialhi.vel.x = 0.0;
            fp.mv.ca.specialhi.vel.y = 0.0;
            fp.mv.ca.specialhi.set_x2_b0(0);
            fp.mv.ca.specialhi.set_x2_b1(0);
        }
    }
}

pub mod ftcaptainspeciallw {
    use melee_types::Fighter;

    /// `resetCmdAndThrow` (static).
    pub fn resetCmdAndThrow(fp: &mut Fighter) {
        fp.cmd_vars[2] = 0;
        fp.cmd_vars[1] = 0;
        fp.cmd_vars[0] = 0;
        fp.anon_2210.throw_flags = 0;
    }
}
