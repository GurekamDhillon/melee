#![allow(non_snake_case)]
use melee_runtime::tu::dbeffect::pad;
use melee_runtime::tu::dbitem::{DbItem, DbItemHost};
use melee_types::{Article, DevText, GXColor, HSD_GObj, HSD_ObjAllocUnk, Item, ItemCommonData, SpawnItem, Vec3};

struct H {
    down: u32,
    pressed: u32,
    repeat: u32,
    head: *mut HSD_GObj,
    text: DevText,
    log: Vec<String>,
    alloc: HSD_ObjAllocUnk,
    common: ItemCommonData,
    spawned: *mut HSD_GObj,
}
impl DbItemHost for H {
    fn DbLevel(&mut self) -> i32 { 4 }
    fn db_ButtonsDown(&mut self, _: i32) -> u32 { self.down }
    fn db_ButtonsPressed(&mut self, _: i32) -> u32 { self.pressed }
    fn db_ButtonsRepeat(&mut self, _: i32) -> u32 { self.repeat }
    fn HSD_GObjPLinkHead(&mut self, i: usize) -> *mut HSD_GObj { assert_eq!(i, 9); self.head }
    fn Item_804A0C64(&mut self) -> *mut HSD_ObjAllocUnk { &mut self.alloc }
    fn it_804D6D28(&mut self) -> *mut ItemCommonData { &mut self.common }
    fn it_804A0F60(&mut self, _: usize) -> *mut Article { core::ptr::null_mut() }
    fn ftLib_IsFighter(&mut self, _: *mut HSD_GObj) -> i32 { 0 }
    fn fn_8022697C(&mut self, _: *mut HSD_GObj) -> u8 { 0 }
    fn DevText_GetGObj(&mut self) -> *mut HSD_GObj { core::ptr::null_mut() }
    fn DevText_Create(&mut self, _: i32, _: i32, _: i32, _: i32, _: i32, _: *mut i8) -> *mut DevText { &mut self.text }
    fn DevText_Show(&mut self, _: *mut HSD_GObj, _: *mut DevText) {}
    fn DevText_HideCursor(&mut self, _: *mut DevText) {}
    fn DevText_SetBGColor(&mut self, _: *mut DevText, _: GXColor) {}
    fn DevText_SetTextColor(&mut self, _: *mut DevText, _: GXColor) {}
    fn DevText_SetScale(&mut self, _: *mut DevText, _: f32, _: f32) {}
    fn DevText_ShowBackground(&mut self, _: *mut DevText) {}
    fn DevText_ShowText(&mut self, _: *mut DevText) {}
    fn DevText_HideBackground(&mut self, _: *mut DevText) { self.log.push("hide".into()) }
    fn DevText_HideText(&mut self, _: *mut DevText) {}
    fn DevText_Erase(&mut self, _: *mut DevText) {}
    fn DevText_SetCursorXY(&mut self, _: *mut DevText, _: i32, _: i32) {}
    fn DevText_Printf(&mut self, _: *mut DevText, s: &str) { self.log.push(s.into()) }
    fn gm_GetDbPauseFlag(&mut self, _: i32) -> i32 { 0 }
    fn Item_80266F3C(&mut self) -> i32 { 1 }
    fn Player_LoadPlayerCoords(&mut self, _: i32, out: &mut Vec3) { *out = Vec3 { x: 1.0, y: 2.0, z: 3.0 } }
    fn it_8026B684(&mut self, _: &mut Vec3) -> f32 { -1.0 }
    fn it_8026C704(&mut self) -> i32 { 0 }
    fn Item_80268B18(&mut self, s: &mut SpawnItem) -> *mut HSD_GObj {
        self.log.push(format!("spawn {} {} {} {}", s.kind, s.pos.y, s.pos.z, s.facing_dir));
        self.spawned
    }
    fn efSync_Spawn(&mut self, id: i32, _: *mut HSD_GObj, p: *mut Vec3) { self.log.push(format!("ef {id:x} {}", unsafe { (*p).y })) }
    fn OSReport(&mut self, s: &str) { self.log.push(s.into()) }
}

#[test]
fn menu_cursor_wraps_through_the_ranges_and_spawns() {
    let mut items: [Item; 2] = unsafe { core::mem::zeroed() };
    let mut gobjs: [HSD_GObj; 2] = unsafe { core::mem::zeroed() };
    gobjs[0].user_data = (&mut items[0] as *mut Item).cast();
    gobjs[1].user_data = (&mut items[1] as *mut Item).cast();
    let g1: *mut HSD_GObj = &mut gobjs[1];
    gobjs[0].next = g1;
    let mut h = H {
        down: 0, pressed: 0, repeat: 0, head: &mut gobjs[0], text: unsafe { core::mem::zeroed() }, log: vec![],
        alloc: unsafe { core::mem::zeroed() }, common: unsafe { core::mem::zeroed() }, spawned: g1,
    };
    let mut s = Box::new(DbItem::default());
    s.fn_SetupItemAndPokemonMenu();
    assert_eq!(s.menu.currently_selected_item, 0x22);
    h.down = pad::L;
    h.repeat = pad::DPADUP | pad::DPADRIGHT;
    let mut seen = vec![];
    for _ in 0..6 {
        s.db_HandleItemPokemonMenuInput(&mut h, 0);
        seen.push(s.menu.currently_selected_item);
    }
    assert_eq!(seen, [0x2B, 0x2C, 0x2D, 0x2E, 0xD0, 0xD1]);
    assert_eq!(s.menu.currently_selected_pokemon, 6);
    h.repeat = pad::DPADDOWN;
    for _ in 0..2 {
        s.db_HandleItemPokemonMenuInput(&mut h, 0);
    }
    assert_eq!(s.menu.currently_selected_item, 0x2E);
    // the display path
    s.menu.currently_selected_item = 0x2B;
    h.repeat = pad::DPADRIGHT;
    s.fn_UpdateItemAndPokemonMenu(&mut h, 1);
    assert_eq!(h.log.last().unwrap(), "Item-> Kuriboh   Pokemon-> Fire");
    assert_eq!((s.menu.display_status, s.menu.player, s.menu.display_fade_timer), (1, 1, 0x78));
    // bubbles 1 -> 2 on both items, low two bits only
    unsafe {
        items[0].xDAA.xDAA_byte = 0xF1;
        s.fn_ToggleItemCollisionBubbles(&mut h);
        assert_eq!(items[0].xDAA.xDAA_byte, 0xF2);
        assert_eq!(items[1].xDAA.xDAA_byte, 0x02);
        s.fn_EnableShowItemPickupRange(&mut h);
        assert_eq!(items[1].xDAA.xDAA_flag.x0.b4(), 1);
        assert_eq!(s.db_ShowItemPickupRange(), 1);
    }
    // spawn: d-down alone, selected item a common one (Capsule) under the limit
    s.menu.currently_selected_item = 0;
    h.down = 0;
    h.pressed = pad::DPADDOWN;
    h.common.x0 = 10;
    h.common.x14 = 10;
    h.log.clear();
    unsafe { s.db_CheckAndSpawnItem(&mut h, 0) };
    assert_eq!(h.log, ["spawn 0 62 0 -1", "ef 420 62"]);
    h.alloc.x0 = 10;
    h.log.clear();
    unsafe { s.db_CheckAndSpawnItem(&mut h, 0) };
    assert_eq!(h.log, ["Item Max Over.\n"]);
    // fn_80225A54 (Develop): B + d-right snapshots the counters
    h.down = pad::B;
    h.pressed = pad::DPADRIGHT;
    h.alloc.x60 = 9;
    s.fn_80225A54(&mut h, 0);
    assert_eq!((h.alloc.x4, h.alloc.x64), (10, 9));
}

/// `db_EnableItemSpawns` / `db_DisableItemSpawns` / `db_AreItemSpawnsEnabled` (dbitem.c: the three accessors of
/// `db_ItemAndPokemonMenu.ItemSpawnsEnabled`; checked against the C source, no oracle build of the TU).
#[test]
fn item_spawn_flag_accessors() {
    let mut s = Box::new(DbItem::default());
    s.menu.item_spawns_enabled = 7;
    s.db_EnableItemSpawns();
    assert_eq!(s.db_AreItemSpawnsEnabled(), 1);
    s.db_DisableItemSpawns();
    assert_eq!(s.db_AreItemSpawnsEnabled(), 0);
    s.db_EnableItemSpawns();
    assert_eq!(s.menu.item_spawns_enabled, 1);
}
