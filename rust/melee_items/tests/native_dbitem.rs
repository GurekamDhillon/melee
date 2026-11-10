//! src/tu/dbitem.rs vs the C of db_80225DD8 / fn_8022697C (src/melee/db/dbitem.c, dbanim.c): all (bubbles, owner kind, flag, byte) combos.
#![allow(non_snake_case, static_mut_refs)]
use melee_items::tu::dbitem::*;
use melee_types::*;
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn db_80225dd8_matches_c() {
    let _g = LOCK.lock().unwrap();
    unsafe {
        for bubbles in 0..4 {
            for is_ft in [false, true] {
                for flag in 0..=255u8 {
                    for start in [0u8, 0xFF, 0x5A, 0x03] {
                        db_ShowItemCollisionBubbles = bubbles;
                        let mut it: Box<Item> = Box::new(core::mem::zeroed());
                        it.xDAA.xDAA_byte = start;
                        let mut ft: Box<Fighter> = Box::new(core::mem::zeroed());
                        ft.x21FC_flag.byte = flag;
                        let mut ig: HSD_GObj = core::mem::zeroed();
                        ig.user_data = &mut *it as *mut Item as *mut _;
                        let mut og: HSD_GObj = core::mem::zeroed();
                        og.user_data = &mut *ft as *mut Fighter as *mut _;
                        og.classifier = if is_ft { HSD_GOBJ_CLASS_FIGHTER as _ } else { 0 };
                        db_80225DD8(&mut ig, &mut og);
                        // the C, written out
                        let mut want = start;
                        if !is_ft {
                            want |= bubbles as u8;
                        } else {
                            want &= 0xFC;
                            let x = flag as i32;
                            if x != 0 {
                                want |= (x & 3) as u8;
                            }
                        }
                        assert_eq!(it.xDAA.xDAA_byte, want);
                    }
                }
            }
        }
    }
}

#[test]
fn db_menu_accessors_and_80225d64() {
    let _g = LOCK.lock().unwrap();
    unsafe {
        db_ItemAndPokemonMenu.ShowEnemyStompRange = 1;
        db_ItemAndPokemonMenu.ShowItemPickupRange = 0;
        db_ItemAndPokemonMenu.ShowCoinPickupRange = 1;
        db_ItemAndPokemonMenu.CurrentlySelectedPokemon = 17;
        db_ItemAndPokemonMenu.ItemSpawnsEnabled = -1;
        assert_eq!((db_ShowEnemyStompRange(), db_ShowItemPickupRange(), db_ShowCoinPickupRange()), (1, 0, 1));
        assert_eq!(db_GetCurrentlySelectedPokemon(), 17);
        assert_eq!(db_AreItemSpawnsEnabled(), -1);
        db_ShowItemCollisionBubbles = 2;
        let mut it: Box<Item> = Box::new(core::mem::zeroed());
        it.xDAA.xDAA_byte = 0x11;
        let mut ig: HSD_GObj = core::mem::zeroed();
        ig.user_data = &mut *it as *mut Item as *mut _;
        db_80225D64(&mut ig, core::ptr::null_mut());
        assert_eq!(it.xDAA.xDAA_byte, 0x13);
    }
}
