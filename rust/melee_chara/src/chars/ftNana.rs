//! Nana (src/melee/ft/kinds/ftNana).

pub mod ftnana {
    use melee_types::Fighter;

    pub fn ftNn_Init_8012309C(fp: &Fighter) -> bool {
        if fp.x221F_b3() == 0 {
            let m = fp.motion_id as i32;
            if (362..=366).contains(&m) {
                return true;
            }
        }
        false
    }
}
