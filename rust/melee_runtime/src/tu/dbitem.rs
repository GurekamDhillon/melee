//! Port of src/melee/db/dbitem.c (debug item / Pokemon spawn menu: L + d-pad picks, d-down
//! spawns; R + d-up cycles the item collision-bubble display).
//!
//! The C globals (`db_ItemAndPokemonMenu`, `db_ShowItemCollisionBubbles`, the DevText and its
//! buffer) live in [`DbItem`]; calls out of the TU go through [`DbItemHost`], whose default
//! implementation [`ext::Ext`] is the unported-callee stub set (panics, named like the C).
#![allow(non_snake_case)]

use melee_types::{
    Article, DevText, GXColor, HSD_GObj, HSD_ObjAllocUnk, Item, ItemCommonData, SpawnItem, Vec3, HSD_GOBJ_PLINK_ITEM,
    It_Kind_Arwing_Laser, It_Kind_Kuriboh, It_Kind_L_Gun_Ray, It_Kind_M_Ball, It_Kind_Octarock_Stone, It_Kind_Old_Kuri,
    It_Kind_Unk4,
};

use super::dbeffect::pad;
use super::dbinit::dblkind;

/// `It_Common_End` (it/forward.h) = `It_Kind_L_Gun_Ray`.
pub const IT_COMMON_END: i32 = It_Kind_L_Gun_Ray as i32;
const PAD_Z: u32 = 1 << 4;
const PAD_START: u32 = 1 << 12;

/// Callees of dbitem.c.
pub trait DbItemHost {
    fn DbLevel(&mut self) -> i32;
    fn db_ButtonsDown(&mut self, player: i32) -> u32;
    fn db_ButtonsPressed(&mut self, player: i32) -> u32;
    fn db_ButtonsRepeat(&mut self, player: i32) -> u32;
    fn HSD_GObjPLinkHead(&mut self, i: usize) -> *mut HSD_GObj;
    /// `&Item_804A0C64`.
    fn Item_804A0C64(&mut self) -> *mut HSD_ObjAllocUnk;
    fn it_804D6D28(&mut self) -> *mut ItemCommonData;
    fn it_804A0F60(&mut self, i: usize) -> *mut Article;
    fn ftLib_IsFighter(&mut self, owner: *mut HSD_GObj) -> i32;
    fn fn_8022697C(&mut self, owner: *mut HSD_GObj) -> u8;
    fn DevText_GetGObj(&mut self) -> *mut HSD_GObj;
    fn DevText_Create(&mut self, id: i32, x: i32, y: i32, w: i32, h: i32, buf: *mut i8) -> *mut DevText;
    fn DevText_Show(&mut self, gobj: *mut HSD_GObj, t: *mut DevText);
    fn DevText_HideCursor(&mut self, t: *mut DevText);
    fn DevText_SetBGColor(&mut self, t: *mut DevText, c: GXColor);
    fn DevText_SetTextColor(&mut self, t: *mut DevText, c: GXColor);
    fn DevText_SetScale(&mut self, t: *mut DevText, x: f32, y: f32);
    fn DevText_ShowBackground(&mut self, t: *mut DevText);
    fn DevText_ShowText(&mut self, t: *mut DevText);
    fn DevText_HideBackground(&mut self, t: *mut DevText);
    fn DevText_HideText(&mut self, t: *mut DevText);
    fn DevText_Erase(&mut self, t: *mut DevText);
    fn DevText_SetCursorXY(&mut self, t: *mut DevText, x: i32, y: i32);
    /// `DevText_Printf` with the C format already applied.
    fn DevText_Printf(&mut self, t: *mut DevText, s: &str);
    fn gm_GetDbPauseFlag(&mut self, i: i32) -> i32;
    fn Item_80266F3C(&mut self) -> i32;
    fn Player_LoadPlayerCoords(&mut self, player: i32, out: &mut Vec3);
    fn it_8026B684(&mut self, pos: &mut Vec3) -> f32;
    fn it_8026C704(&mut self) -> i32;
    fn Item_80268B18(&mut self, spawn: &mut SpawnItem) -> *mut HSD_GObj;
    /// `efSync_Spawn(gfx_id, gobj, Vec3*)`.
    fn efSync_Spawn(&mut self, gfx_id: i32, gobj: *mut HSD_GObj, pos: *mut Vec3);
    fn OSReport(&mut self, s: &str);
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl DbItemHost for Ext {
        fn DbLevel(&mut self) -> i32 { todo!("DbLevel (db/dbinit.c)") }
        fn db_ButtonsDown(&mut self, _: i32) -> u32 { todo!("db_ButtonsDown (db/dbinit.c)") }
        fn db_ButtonsPressed(&mut self, _: i32) -> u32 { todo!("db_ButtonsPressed (db/dbinit.c)") }
        fn db_ButtonsRepeat(&mut self, _: i32) -> u32 { todo!("db_ButtonsRepeat (db/dbinit.c)") }
        fn HSD_GObjPLinkHead(&mut self, _: usize) -> *mut HSD_GObj { todo!("HSD_GObjPLinkHead (baselib/gobj.c)") }
        fn Item_804A0C64(&mut self) -> *mut HSD_ObjAllocUnk { todo!("Item_804A0C64 (it/item.c)") }
        fn it_804D6D28(&mut self) -> *mut ItemCommonData { todo!("it_804D6D28 (it/it_3F14.c)") }
        fn it_804A0F60(&mut self, _: usize) -> *mut Article { todo!("it_804A0F60 (it/it_3F14.c)") }
        fn ftLib_IsFighter(&mut self, _: *mut HSD_GObj) -> i32 { todo!("ftLib_IsFighter (ft/ftlib.c)") }
        fn fn_8022697C(&mut self, _: *mut HSD_GObj) -> u8 { todo!("fn_8022697C (db)") }
        fn DevText_GetGObj(&mut self) -> *mut HSD_GObj { todo!("DevText_GetGObj (if/textdraw.c)") }
        fn DevText_Create(&mut self, _: i32, _: i32, _: i32, _: i32, _: i32, _: *mut i8) -> *mut DevText { todo!("DevText_Create (if/textdraw.c)") }
        fn DevText_Show(&mut self, _: *mut HSD_GObj, _: *mut DevText) { todo!("DevText_Show (if/textdraw.c)") }
        fn DevText_HideCursor(&mut self, _: *mut DevText) { todo!("DevText_HideCursor (if/textdraw.c)") }
        fn DevText_SetBGColor(&mut self, _: *mut DevText, _: GXColor) { todo!("DevText_SetBGColor (if/textdraw.c)") }
        fn DevText_SetTextColor(&mut self, _: *mut DevText, _: GXColor) { todo!("DevText_SetTextColor (if/textdraw.c)") }
        fn DevText_SetScale(&mut self, _: *mut DevText, _: f32, _: f32) { todo!("DevText_SetScale (if/textdraw.c)") }
        fn DevText_ShowBackground(&mut self, _: *mut DevText) { todo!("DevText_ShowBackground (if/textdraw.c)") }
        fn DevText_ShowText(&mut self, _: *mut DevText) { todo!("DevText_ShowText (if/textdraw.c)") }
        fn DevText_HideBackground(&mut self, _: *mut DevText) { todo!("DevText_HideBackground (if/textdraw.c)") }
        fn DevText_HideText(&mut self, _: *mut DevText) { todo!("DevText_HideText (if/textdraw.c)") }
        fn DevText_Erase(&mut self, _: *mut DevText) { todo!("DevText_Erase (if/textdraw.c)") }
        fn DevText_SetCursorXY(&mut self, _: *mut DevText, _: i32, _: i32) { todo!("DevText_SetCursorXY (if/textdraw.c)") }
        fn DevText_Printf(&mut self, _: *mut DevText, _: &str) { todo!("DevText_Printf (if/textdraw.c)") }
        fn gm_GetDbPauseFlag(&mut self, _: i32) -> i32 { todo!("gm_GetDbPauseFlag (gm)") }
        fn Item_80266F3C(&mut self) -> i32 { todo!("Item_80266F3C (it/item.c)") }
        fn Player_LoadPlayerCoords(&mut self, _: i32, _: &mut Vec3) { todo!("Player_LoadPlayerCoords (pl/player.c)") }
        fn it_8026B684(&mut self, _: &mut Vec3) -> f32 { todo!("it_8026B684 (it/it_26B1.c)") }
        fn it_8026C704(&mut self) -> i32 { todo!("it_8026C704 (it/it_26B1.c)") }
        fn Item_80268B18(&mut self, _: &mut SpawnItem) -> *mut HSD_GObj { todo!("Item_80268B18 (it/item.c)") }
        fn efSync_Spawn(&mut self, _: i32, _: *mut HSD_GObj, _: *mut Vec3) { todo!("efSync_Spawn (ef/efsync.c)") }
        fn OSReport(&mut self, _: &str) { todo!("OSReport (pc/platform)") }
    }
}

/// `db_ItemNames` (0x803EA94C).
pub const DB_ITEM_NAMES: [&str; 35] = [
    "Capsule ", "Box     ", "Taru    ", "Egg     ", "Kusudama", "TaruCann", "BombHei ", "Dosei   ", "Heart   ",
    "Tomato  ", "Star    ", "Bat     ", "Sword   ", "Parasol ", "G Shell ", "R Shell ", "L Gun   ", "Freeze  ",
    "Foods   ", "MSBomb  ", "Flipper ", "S Scope ", "StarRod ", "LipStick", "Harisen ", "F Flower", "Kinoko  ",
    "DKinoko ", "Hammer  ", "WStar   ", "ScBall  ", "RabbitC ", "MetalB  ", "Spycloak", "M Ball  ",
];
/// `db_PokemonNames` (0x803EAA50).
pub const DB_POKEMON_NAMES: [&str; 31] = [
    "Random", "Tosakinto", "Chicorita", "Kabigon", "Kamex", "Matadogas", "Lizardon", "Fire", "Thunder", "Freezer",
    "Sonans", "Hassam", "Unknown", "Entei", "Raikou", "Suikun", "Kireihana", "Marumine", "Lugia", "Houou", "Metamon",
    "Pippi", "Togepy", "Mew", "Cerebi", "Hitodeman", "Lucky", "Porygon2", "Hinoarashi", "Maril", "Fushigibana",
];
/// `db_BarrelEnemies` (0x803EAAFC).
pub const DB_BARREL_ENEMIES: [&str; 4] = ["Kuriboh ", "Leadead ", "Octarock", "Ottosei "];
/// `db_AdventureEnemies[26]` (0x803EABA8); the C leaves entries 13..25 NULL.
pub const DB_ADVENTURE_ENEMIES: [Option<&str>; 26] = {
    let mut a = [None; 26];
    let n = ["old-Kuri", "Mato    ", "Heiho   ", "Nokonoko", "Patapata", "likelike", "old-lead", "old-octa",
             "old-otto", "whitebea", "klap    ", "zgshell ", "zrshell "];
    let mut i = 0;
    while i < n.len() {
        a[i] = Some(n[i]);
        i += 1;
    }
    a
};

/// The anonymous `db_ItemAndPokemonMenu` struct.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbItemMenu {
    /// 0 = uninitialised, 1 = visible, 2 = hidden.
    pub display_status: u32,
    pub display_fade_timer: u32,
    pub item_spawns_enabled: i32,
    pub player: i32,
    pub currently_selected_item: i32,
    pub currently_selected_pokemon: i32,
    pub last_selected_item: i32,
    pub last_selected_pokemon: i32,
    pub show_enemy_stomp_range: bool,
    pub show_item_pickup_range: bool,
    pub show_coin_pickup_range: bool,
}

/// The globals of dbitem.c. The DevText keeps a pointer into `text_buf`, so keep this in place
/// (a static or a box) once [`DbItem::fn_ShowOrCreateItemAndPokemonMenu`] has run.
#[derive(Clone, Debug)]
pub struct DbItem {
    pub menu: DbItemMenu,
    /// `db_ItemAndPokemonMenuText` (0x804D6B38).
    pub text: *mut DevText,
    /// `db_ShowItemCollisionBubbles` (0x804D6B3C).
    pub show_item_collision_bubbles: i32,
    /// `db_ItemAndPokemonMenuText_buf` (0x8049FA50).
    pub text_buf: [i8; 0x50],
}

impl Default for DbItem {
    fn default() -> Self {
        Self { menu: DbItemMenu::default(), text: core::ptr::null_mut(), show_item_collision_bubbles: 0, text_buf: [0; 0x50] }
    }
}

/// Walk `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]`.
///
/// # Safety
/// The list is well formed and every `user_data` is a live `Item`.
unsafe fn for_items<H: DbItemHost + ?Sized>(h: &mut H, mut f: impl FnMut(*mut HSD_GObj, &mut Item)) {
    let mut g = h.HSD_GObjPLinkHead(HSD_GOBJ_PLINK_ITEM as usize);
    while !g.is_null() {
        let it = (*g).user_data as *mut Item;
        f(g, &mut *it);
        g = (*g).next;
    }
}

impl DbItem {
    /// `fn_SetupItemAndPokemonMenu` (0x80225A00).
    pub fn fn_SetupItemAndPokemonMenu(&mut self) {
        let m = &mut self.menu;
        m.display_status = 0;
        m.currently_selected_item = 0x22;
        m.last_selected_item = m.currently_selected_item;
        m.currently_selected_pokemon = 0;
        m.last_selected_pokemon = m.currently_selected_pokemon;
        self.show_item_collision_bubbles = 1;
        m.show_enemy_stomp_range = false;
        m.show_item_pickup_range = false;
    }

    /// `fn_80225A54`: in a Develop build, B + d-right snapshots the item alloc counters.
    pub fn fn_80225A54<H: DbItemHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if h.DbLevel() == dblkind::DEVELOP
            && h.db_ButtonsDown(player) & pad::B != 0
            && h.db_ButtonsPressed(player) & pad::DPADRIGHT != 0
        {
            // SAFETY: the host returns the live global.
            let u = unsafe { &mut *h.Item_804A0C64() };
            u.x4 = u.x0;
            u.xC = u.x8;
            u.x14 = u.x10;
            u.x20 = u.x1C;
            u.x28 = u.x24;
            u.x30 = u.x2C;
            u.x38 = u.x34;
            u.x44 = u.x40;
            u.x4C = u.x48;
            u.x54 = u.x50;
            u.x5C = u.x58;
            u.x64 = u.x60;
        }
    }

    /// `db_ShowEnemyStompRange` (0x80225B0C).
    pub fn db_ShowEnemyStompRange(&self) -> u32 {
        self.menu.show_enemy_stomp_range as u32
    }
    /// `db_ShowItemPickupRange` (0x80225B20).
    pub fn db_ShowItemPickupRange(&self) -> u32 {
        self.menu.show_item_pickup_range as u32
    }
    /// `db_ShowCoinPickupRange` (0x80225B34).
    pub fn db_ShowCoinPickupRange(&self) -> u32 {
        self.menu.show_coin_pickup_range as u32
    }

    /// `fn_EnableShowCoinPickupRange` (0x80225B48).
    ///
    /// # Safety
    /// See [`for_items`].
    pub unsafe fn fn_EnableShowCoinPickupRange<H: DbItemHost + ?Sized>(&mut self, h: &mut H) {
        for_items(h, |_, it| {
            if it.kind == It_Kind_Unk4 {
                it.xDAA.xDAA_flag.x0.set_b0(1);
            }
        });
        self.menu.show_coin_pickup_range = true;
    }
    /// `fn_DisableShowCoinPickupRange` (0x80225B9C).
    ///
    /// # Safety
    /// See [`for_items`].
    pub unsafe fn fn_DisableShowCoinPickupRange<H: DbItemHost + ?Sized>(&mut self, h: &mut H) {
        for_items(h, |_, it| {
            if it.kind == It_Kind_Unk4 {
                it.xDAA.xDAA_flag.x0.set_b0(0);
            }
        });
        self.menu.show_coin_pickup_range = false;
    }
    /// `fn_EnableShowEnemyStompRange` (0x80225BF0).
    ///
    /// # Safety
    /// See [`for_items`].
    pub unsafe fn fn_EnableShowEnemyStompRange<H: DbItemHost + ?Sized>(&mut self, h: &mut H) {
        for_items(h, |_, it| {
            if it.xDD0_flag.x0.b0() != 0 {
                it.xDAA.xDAA_flag.x0.set_b3(1);
            }
        });
        self.menu.show_enemy_stomp_range = true;
    }
    /// `fn_DisableShowEnemyStompRange` (0x80225C44).
    ///
    /// # Safety
    /// See [`for_items`].
    pub unsafe fn fn_DisableShowEnemyStompRange<H: DbItemHost + ?Sized>(&mut self, h: &mut H) {
        for_items(h, |_, it| it.xDAA.xDAA_flag.x0.set_b3(0));
        self.menu.show_enemy_stomp_range = false;
    }
    /// `fn_EnableShowItemPickupRange` (0x80225C8C).
    ///
    /// # Safety
    /// See [`for_items`].
    pub unsafe fn fn_EnableShowItemPickupRange<H: DbItemHost + ?Sized>(&mut self, h: &mut H) {
        for_items(h, |_, it| it.xDAA.xDAA_flag.x0.set_b4(1));
        self.menu.show_item_pickup_range = true;
    }
    /// `fn_DisableShowItemPickupRange` (0x80225CD4).
    ///
    /// # Safety
    /// See [`for_items`].
    pub unsafe fn fn_DisableShowItemPickupRange<H: DbItemHost + ?Sized>(&mut self, h: &mut H) {
        for_items(h, |_, it| it.xDAA.xDAA_flag.x0.set_b4(0));
        self.menu.show_item_pickup_range = false;
    }

    /// `db_GetCurrentlySelectedPokemon` (0x80225D1C).
    pub fn db_GetCurrentlySelectedPokemon(&self) -> i32 {
        self.menu.currently_selected_pokemon
    }
    /// `db_DisableItemSpawns` (0x80225D2C).
    pub fn db_DisableItemSpawns(&mut self) {
        self.menu.item_spawns_enabled = 0;
    }
    /// `db_EnableItemSpawns` (0x80225D40).
    pub fn db_EnableItemSpawns(&mut self) {
        self.menu.item_spawns_enabled = 1;
    }
    /// `db_AreItemSpawnsEnabled` (0x80225D54).
    pub fn db_AreItemSpawnsEnabled(&self) -> i32 {
        self.menu.item_spawns_enabled
    }

    /// `db_80225D64`.
    ///
    /// # Safety
    /// `item` is a live item GObj.
    pub unsafe fn db_80225D64(&self, item: *mut HSD_GObj, _owner: *mut HSD_GObj) {
        let it = &mut *((*item).user_data as *mut Item);
        it.xDAA.xDAA_byte |= self.show_item_collision_bubbles as u8;
    }

    /// `fn_ToggleItemCollisionBubbles` (0x80225D7C).
    ///
    /// # Safety
    /// See [`for_items`].
    pub unsafe fn fn_ToggleItemCollisionBubbles<H: DbItemHost + ?Sized>(&mut self, h: &mut H) {
        self.show_item_collision_bubbles += 1;
        if self.show_item_collision_bubbles > 3 {
            self.show_item_collision_bubbles = 1;
        }
        let v = self.show_item_collision_bubbles as u8;
        for_items(h, |_, it| {
            it.xDAA.xDAA_byte &= 0xFC;
            it.xDAA.xDAA_byte |= v;
        });
    }

    /// `db_80225DD8`.
    ///
    /// # Safety
    /// `item` is a live item GObj.
    pub unsafe fn db_80225DD8<H: DbItemHost + ?Sized>(&self, h: &mut H, item: *mut HSD_GObj, owner: *mut HSD_GObj) {
        let it = (*item).user_data as *mut Item;
        if h.ftLib_IsFighter(owner) == 0 {
            (*it).xDAA.xDAA_byte |= self.show_item_collision_bubbles as u8;
        } else {
            (*it).xDAA.xDAA_byte &= 0xFC;
            let x = h.fn_8022697C(owner) as i32;
            if x != 0 {
                (*it).xDAA.xDAA_byte |= (x & 3) as u8;
            }
        }
    }

    /// `fn_80225E6C`: re-apply the bubble display to every item `owner` owns.
    ///
    /// # Safety
    /// See [`for_items`].
    pub unsafe fn fn_80225E6C<H: DbItemHost + ?Sized>(&self, h: &mut H, owner: *mut HSD_GObj) {
        let mut g = h.HSD_GObjPLinkHead(HSD_GOBJ_PLINK_ITEM as usize);
        while !g.is_null() {
            let it = (*g).user_data as *mut Item;
            if (*it).owner == owner {
                self.db_80225DD8(h, g, owner);
            }
            g = (*g).next;
        }
    }

    /// `db_HandleItemPokemonMenuInput` (0x80225F20).
    pub fn db_HandleItemPokemonMenuInput<H: DbItemHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        let m = &mut self.menu;
        if h.db_ButtonsDown(player) & pad::L != 0 && h.db_ButtonsRepeat(player) & pad::DPADUP != 0 {
            if m.currently_selected_item < 0x23 {
                m.currently_selected_item += 1;
                if m.currently_selected_item == 0x23 {
                    m.currently_selected_item = 0x2B;
                }
            } else if m.currently_selected_item < 0x2F {
                m.currently_selected_item += 1;
                if m.currently_selected_item == 0x2F {
                    m.currently_selected_item = It_Kind_Old_Kuri as i32;
                }
            } else if m.currently_selected_item < 0xE9 {
                m.currently_selected_item += 1;
            }
        }
        if h.db_ButtonsDown(player) & pad::L != 0 && h.db_ButtonsRepeat(player) & pad::DPADDOWN != 0 {
            if m.currently_selected_item >= 0xD0 {
                m.currently_selected_item -= 1;
                if m.currently_selected_item < 0xD0 {
                    m.currently_selected_item = 0x2E;
                }
            } else if m.currently_selected_item >= 0x2B {
                m.currently_selected_item -= 1;
                if m.currently_selected_item < 0x2B {
                    m.currently_selected_item = 0x22;
                }
            } else if m.currently_selected_item > 0 {
                m.currently_selected_item -= 1;
            }
        }
        if h.db_ButtonsDown(player) & pad::L != 0
            && h.db_ButtonsRepeat(player) & pad::DPADRIGHT != 0
            && m.currently_selected_pokemon < 0x1E
        {
            m.currently_selected_pokemon += 1;
        }
        if h.db_ButtonsDown(player) & pad::L != 0
            && h.db_ButtonsRepeat(player) & pad::DPADLEFT != 0
            && m.currently_selected_pokemon > 0
        {
            m.currently_selected_pokemon -= 1;
        }
    }

    /// `fn_ShowOrCreateItemAndPokemonMenu` (0x802260D4).
    pub fn fn_ShowOrCreateItemAndPokemonMenu<H: DbItemHost + ?Sized>(&mut self, h: &mut H, _player: i32) {
        let gobj = h.DevText_GetGObj();
        if self.menu.display_status == 2 {
            h.DevText_ShowBackground(self.text);
            h.DevText_ShowText(self.text);
        } else {
            self.text = h.DevText_Create(8, 20, 20, 40, 1, self.text_buf.as_mut_ptr());
            if !self.text.is_null() {
                let bg = GXColor { r: 0x80, g: 0x40, b: 0xFF, a: 0xFF };
                let fg = GXColor { r: 0x00, g: 0x00, b: 0x00, a: 0xFF };
                h.DevText_Show(gobj, self.text);
                h.DevText_HideCursor(self.text);
                h.DevText_SetBGColor(self.text, bg);
                h.DevText_SetTextColor(self.text, fg);
                h.DevText_SetScale(self.text, 12.0, 16.0);
            }
        }
        self.menu.display_status = 1;
    }

    /// `fn_UpdateItemAndPokemonMenu` (0x802261BC).
    pub fn fn_UpdateItemAndPokemonMenu<H: DbItemHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        self.menu.last_selected_item = self.menu.currently_selected_item;
        self.menu.last_selected_pokemon = self.menu.currently_selected_pokemon;
        self.db_HandleItemPokemonMenuInput(h, player);
        if self.menu.last_selected_item != self.menu.currently_selected_item
            || self.menu.last_selected_pokemon != self.menu.currently_selected_pokemon
        {
            self.menu.player = player;
            self.menu.display_fade_timer = 0x78;
            if self.menu.display_status != 1 {
                self.fn_ShowOrCreateItemAndPokemonMenu(h, player);
            }
            h.DevText_Erase(self.text);
            h.DevText_SetCursorXY(self.text, 0, 0);
            let cur = self.menu.currently_selected_item;
            let item: &str = if cur < 0x23 {
                DB_ITEM_NAMES[cur as usize]
            } else if cur < It_Kind_Octarock_Stone as i32 {
                DB_BARREL_ENEMIES[(cur - It_Kind_Kuriboh as i32) as usize]
            } else if cur < It_Kind_Arwing_Laser as i32 {
                // The C prints "(null)"-style garbage for the NULL tail; this is printf's "(null)".
                DB_ADVENTURE_ENEMIES[(cur - It_Kind_Old_Kuri as i32) as usize].unwrap_or("(null)")
            } else {
                loop {
                    core::hint::spin_loop();
                }
            };
            let s = format!("Item-> {}  Pokemon-> {}", item, DB_POKEMON_NAMES[self.menu.currently_selected_pokemon as usize]);
            h.DevText_Printf(self.text, &s);
        }
    }

    /// `db_CheckAndSpawnItem` (0x802262E0).
    ///
    /// # Safety
    /// The host's item globals are live; a spawned GObj's `user_data` is an `Item`.
    pub unsafe fn db_CheckAndSpawnItem<H: DbItemHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        let mask = pad::DPADLEFT | pad::DPADRIGHT | pad::DPADUP | PAD_Z | pad::R | pad::L | pad::A | pad::B | pad::X
            | pad::Y | PAD_START;
        if h.db_ButtonsDown(player) & mask != 0 {
            return;
        }
        if h.db_ButtonsPressed(player) & pad::DPADDOWN == 0 {
            return;
        }
        let mut si = SpawnItem::zeroed();
        si.kind = self.menu.currently_selected_item as _;
        let kind = si.kind as i32;
        if h.Item_80266F3C() == 0 && kind < IT_COMMON_END {
            return;
        }
        h.Player_LoadPlayerCoords(player, &mut si.prev_pos);
        si.prev_pos.y += 60.0;
        si.prev_pos.z = 0.0;
        si.pos = si.prev_pos;
        si.facing_dir = h.it_8026B684(&mut si.prev_pos);
        si.x3C_damage = 0;
        si.vel.x = 0.0;
        si.vel.y = 0.0;
        si.vel.z = 0.0;
        si.x0_parent_gobj = core::ptr::null_mut();
        si.x4_parent_gobj2 = si.x0_parent_gobj;
        si.x44_flag.x0.set_b0(1);
        si.x40 = 0;
        let alloc = &*h.Item_804A0C64();
        let common = &*h.it_804D6D28();
        if kind < IT_COMMON_END && (alloc.x0 as u32) >= common.x0 as u32 {
            h.OSReport("Item Max Over.\n");
            return;
        }
        if kind < It_Kind_Octarock_Stone as i32 && (alloc.x2C as u32) >= common.x14 as u32 {
            h.OSReport("couldn't get Item struct.(CZako)\n");
            return;
        }
        if (kind < It_Kind_Old_Kuri as i32
            || kind >= It_Kind_Arwing_Laser as i32
            || !h.it_804A0F60((kind - It_Kind_Old_Kuri as i32) as usize).is_null())
            && (kind != It_Kind_M_Ball as i32 || h.it_8026C704() == 0)
        {
            let gobj = h.Item_80268B18(&mut si);
            if !gobj.is_null() {
                let it = (*gobj).user_data as *mut Item;
                (*it).xDAA.xDAA_flag.byte |= self.show_item_collision_bubbles as u8;
                h.efSync_Spawn(0x420, gobj, &mut si.prev_pos);
            }
        }
    }

    /// `fn_CheckItemAndPokemonMenu` (0x802264C4).
    ///
    /// # Safety
    /// As [`Self::db_CheckAndSpawnItem`] and [`for_items`].
    pub unsafe fn fn_CheckItemAndPokemonMenu<H: DbItemHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if self.menu.display_status == 1 && self.menu.player == player {
            if self.menu.display_fade_timer == 0 {
                self.menu.display_status = 2;
                h.DevText_HideBackground(self.text);
                h.DevText_HideText(self.text);
            } else {
                self.menu.display_fade_timer -= 1;
            }
        }
        self.fn_UpdateItemAndPokemonMenu(h, player);
        if h.gm_GetDbPauseFlag(1) == 0 && h.gm_GetDbPauseFlag(0) == 0 {
            self.db_CheckAndSpawnItem(h, player);
        }
        if h.db_ButtonsDown(player) & pad::R != 0 && h.db_ButtonsPressed(player) & pad::DPADUP != 0 {
            self.fn_ToggleItemCollisionBubbles(h);
        }
        self.fn_80225A54(h, player);
    }
}
