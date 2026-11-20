//! The Thunder's hit flag and pointer (ftpikachuspeciallw.rs thunder_flag_set; tools/rust-port/mvshadow sites #1 / #2).
//! GameCube word fp+2340 is the Thunder's GObj pointer and fp+2344 the flag; natively the pointer is 8 bytes and
//! covers the flag's other view, so a hit used to rewrite the pointer (0x81f8f7a0 -> 0x3_81f8f7a0).
use melee_chara::tu::ftpikachuspeciallw as pk;
use melee_types::*;

#[test]
fn hit_keeps_the_thunder_pointer() {
    let mut fp = Box::new(Fighter::zeroed());
    fp.motion_id = ftPk_MS_SpecialLwLoop0 as _;
    let thunder = 0x81f8_f7a0usize as *mut HSD_GObj;
    unsafe {
        fp.mv.pk.speciallw.x0 = thunder;
        fp.mv.pk.speciallw.x4 = 1;
    }
    pk::ftPk_SpecialLw_SetState_Unk0_fp(&mut fp);
    unsafe {
        assert_eq!(fp.mv.pk.speciallw.x0, thunder, "the hit flag landed in the Thunder's pointer");
        assert_eq!(fp.mv.pk.speciallw.x4, 3, "SpecialLwLoop0_Anim reads the flag here");
    }
    pk::ftPk_SpecialLw_SetState_Unk1_fp(&mut fp);
    unsafe {
        assert_eq!(fp.mv.pk.speciallw.x0, thunder);
        assert_eq!(fp.mv.pk.speciallw.x4, 0);
    }
}

#[test]
fn thunder_ending_outside_down_b_leaves_turn_alone() {
    let mut fp = Box::new(Fighter::zeroed());
    fp.motion_id = ftCo_MS_Turn as _;
    unsafe {
        fp.mv.co.turn.x8 = 5.0;
        fp.mv.co.turn.facing_after = 1.0;
    }
    pk::ftPk_SpecialLw_SetState_Unk1_fp(&mut fp);
    unsafe {
        // the GameCube writes fp+2344 (turn.facing_after's word) and leaves fp+2348 (turn.x8, the Turn IASA gate)
        assert_eq!(fp.mv.co.turn.x8, 5.0, "the flag write reached co.turn.x8");
        assert_eq!(fp.mv.co.turn.facing_after.to_bits(), 0, "the GameCube word fp+2344 holds the flag (0)");
    }
}
