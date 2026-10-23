//! Port of src/melee/it/it_279C.c: the Pokemon item behaviour (the spawn kind selection from the common list
//! or the stage's list, the spawn scale animation, the attack-id/attack-instance copies from the owner, the
//! facing/rotation helpers and the spawn driver `it_8027AB64`). The item logic tables at the top of the C file
//! (`it_803F23CC` and friends) are data, not functions, and are not part of this port.
//! `it_8027B4A4` is ported in melee_fighter and is not repeated. Callees of other TUs go through `ext` (unset
//! hooks panic "TODO"); callees ported in this crate are called directly.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_mut, unused_assignments, unused_variables, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

use crate::tu::it_26B1::{it_8026B3A8, it_8026B6A8, it_8026B6C8, it_8026BB88, it_8026BDB4};
use crate::tu::it_2725::{it_80272DE4, it_80272F7C, it_80272980};
use crate::tu::itzako::it_8027B964;

// BEGIN GENERATED EXT (the callees of the C TU that are not ported in this crate)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjGetScaleX(a0: *mut HSD_JObj) -> f32;
    fn HSD_JObjGetTranslation(a0: *mut HSD_JObj, a1: *mut Vec3);
    fn HSD_JObjSetRotation(a0: *mut HSD_JObj, a1: *mut Quaternion);
    fn HSD_JObjSetScale(a0: *mut HSD_JObj, a1: *mut Vec3);
    fn HSD_JObjSetTranslate(a0: *mut HSD_JObj, a1: *mut Vec3);
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn db_GetCurrentlySelectedPokemon() -> i32;
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut f32) -> *mut c_void;
    fn ft_800898A8(a0: *mut HSD_GObj) -> *mut Struct2074;
    fn ft_80089768(a0: *mut Vec);
    fn ft_80089884(a0: *mut HSD_GObj) -> *mut Struct2070;
    fn it_8026B3C0(a0: i32) -> i32;
    fn ftLib_8008627C(a0: *mut Vec3, a1: *mut HSD_GObj) -> *mut HSD_GObj;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec3);
    fn ftLib_80086960(a0: *mut HSD_GObj) -> bool;
    fn ftLib_80086BE0(a0: *mut HSD_GObj) -> u8;
    fn ftLib_800860C4() -> i32;
    fn ftLib_800874BC(a0: *mut HSD_GObj) -> i32;
    fn gm_801BEC54() -> *mut c_void;
    fn gm_80164ABC() -> bool;
    fn gm_80165084() -> bool;
    fn gm_80172BC4();
    fn gm_80172C04();
    fn gm_8016AEB8() -> u32;
    fn gm_8018841C() -> bool;
    fn gm_GetCurrentGameMode() -> i32;
    fn lbVector_Mirror(a0: *mut Vec3, a1: *mut Vec3);
    fn it_8026E4D0(a0: *mut HSD_GObj, a1: i32);
    fn it_802756D0(a0: *mut HSD_GObj);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_802787B4(a0: *mut HSD_GObj, a1: i32);
    fn pl_8003E70C(a0: *mut HSD_GObj);
    fn pl_8003EB30(a0: f32, a1: u16, a2: i32, a3: i32, a4: i32, a5: u8);
    fn pl_800384DC(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn pl_80037DF4(a0: *mut HSD_GObj, a1: *mut Struct2070);
    fn plAttack_80037B08() -> u8;
    fn plStale_IncrementAttackInstance() -> u32;
    fn Item_80268B9C(a0: *mut SpawnItem);
    }
    /// `common_pokemon` is a `static` table in the C file; the crate's init copies it here.
    pub static mut common_pokemon: *mut ItemKind = core::ptr::null_mut();
    /// `Item_804A0E24` (PokemonSelectionState) is defined in another TU.
    pub static mut Item_804A0E24: PokemonSelectionState = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `Pokemon_ID_*` (the C file's enum): the index of each Pokemon in the selection list.
pub const Pokemon_ID_Tosakinto: i32 = 0;
pub const Pokemon_ID_Metamon: i32 = 19;

/// `IT_M_PI_2` (the C double constant).
const IT_M_PI_2: f64 = 1.5707963267948966;

/// `it_803B8588`: the zero quaternion used as the rotation template.
pub const it_803B8588: Quaternion = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
/// `it_803B8598`: the second zero quaternion template.
pub const it_803B8598: Quaternion = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };

#[inline(always)]
unsafe fn item_of(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

/// `it_80279C48`: faces the item toward the nearest fighter of its owner (or a random side when there is none).
pub unsafe extern "C" fn it_80279C48(item_gobj: *mut HSD_GObj) {
    let mut v: Vec3 = core::mem::zeroed();
    let item = item_of(item_gobj);
    let gobj = ext::ftLib_8008627C(&raw mut (*item).pos, (*item).owner);
    if !gobj.is_null() {
        ext::ftLib_80086644(gobj, &raw mut v);
        if (*item).pos.x - v.x > 0.0 {
            (*item).facing_dir = -1.0;
            return;
        }
        (*item).facing_dir = 1.0;
        return;
    }
    let dir = if ext::HSD_Randi(2) != 0 { 1.0f32 } else { -1.0f32 };
    (*item).facing_dir = dir;
}

/// `it_80279CDC`: resets the item's state and scale, clears its flags.
pub unsafe extern "C" fn it_80279CDC(item_gobj: *mut HSD_GObj, scale: f32) {
    it_8027A13C(item_gobj);
    it_80279D5C(item_gobj, scale);
    it_8026B3A8(item_gobj);
    ext::it_802756D0(item_gobj);
    it_8026BDB4(item_gobj);
}

/// `it_80279D38`: the knockback path (`it_8027B964` with `chk` false).
pub unsafe extern "C" fn it_80279D38(item_gobj: *mut HSD_GObj) {
    it_8027B964(item_gobj, 0);
}

/// `it_80279D5C`: sets the Pokemon's scale on all three axes and resets its spawn counter.
pub unsafe extern "C" fn it_80279D5C(item_gobj: *mut HSD_GObj, scale: f32) {
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let item = item_of(item_gobj);
    let mut sp18 = Vec3 { x: scale, y: scale, z: scale };
    (*item).xDD4_itemVar.pokemon_spawn.x4C = scale;
    (*item).xDD4_itemVar.pokemon_spawn.x5C = 0;
    ext::HSD_JObjSetScale(item_jobj, &raw mut sp18);
}

/// `it_80279E24_inline` (static inline): one step of the spawn scale animation (the next keyframe).
unsafe fn it_80279E24_inline(item_gobj: *mut HSD_GObj) {
    let jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let item = (*item_gobj).user_data as *mut Item;
    let scale = ext::HSD_JObjGetScaleX(jobj);
    let ps = &raw mut (*item).xDD4_itemVar.pokemon_spawn;
    it_80272F7C(jobj, (*ps).x2C[(*ps).x58 as usize]);
    (*ps).x58 += 1;
    (*ps).x4C = ((*ps).x2C[(*ps).x58 as usize] - scale) / (*ps).x44;
}

/// `it_80279E24`: advances the Pokemon's scale animation one frame and sets the next keyframe when due.
pub unsafe extern "C" fn it_80279E24(item_gobj: *mut HSD_GObj) {
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let item = (*item_gobj).user_data as *mut Item;
    let scale = ext::HSD_JObjGetScaleX(item_jobj);
    let ps = &raw mut (*item).xDD4_itemVar.pokemon_spawn;

    if (*ps).x50 <= 0.0 {
        (*ps).x50 = (*ps).x44;
        if (*ps).x58 & 1 != 0 {
            if (*ps).x2C[(*ps).x58 as usize] <= scale {
                it_80279E24_inline(item_gobj);
            }
        } else if (*ps).x2C[(*ps).x58 as usize] >= scale {
            it_80279E24_inline(item_gobj);
        }
        if (*ps).x54 <= (*ps).x58 {
            (*ps).x5C = 2;
            return;
        }
    }
    (*ps).x50 -= 1.0;
    it_80272DE4(item_jobj, (*ps).x4C);
}

/// `it_80279FF8`: grows the Pokemon to its attribute scale, then runs the scale animation.
pub unsafe extern "C" fn it_80279FF8(item_gobj: *mut HSD_GObj) {
    let item = (*item_gobj).user_data as *mut Item;
    if (*item).xDD4_itemVar.pokemon_spawn.x5C == 0 {
        let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
        if (*(*item).xCC_item_attr).x60_scale > (*item).xDD4_itemVar.pokemon_spawn.x48 + ext::HSD_JObjGetScaleX(item_jobj) {
            it_80272DE4(item_jobj, (*item).xDD4_itemVar.pokemon_spawn.x48);
            return;
        }
        it_80272F7C(item_jobj, (*(*item).xCC_item_attr).x60_scale);
        (*item).xDD4_itemVar.pokemon_spawn.x5C = 1;
        return;
    }
    if (*item).xDD4_itemVar.pokemon_spawn.x5C != 2 {
        it_80279E24(item_gobj);
    }
}

/// `it_8027A09C`: runs the Pokemon's timed movement; true when it finishes (the scale is then restored).
pub unsafe extern "C" fn it_8027A09C(item_gobj: *mut HSD_GObj) -> bool {
    let item = item_of(item_gobj);
    ext::it_80272860(item_gobj, (*item).xDD4_itemVar.pokemon_spawn.x1C, (*item).xDD4_itemVar.pokemon_spawn.x20);
    if (*item).xDD4_itemVar.pokemon_spawn.x24 <= 0.0 {
        it_80272F7C((*item_gobj).hsd_obj as *mut HSD_JObj, (*(*item).xCC_item_attr).x60_scale);
        return true;
    }
    (*item).xDD4_itemVar.pokemon_spawn.x24 -= 1.0;
    false
}

/// `it_8027A118`: forwards the event to `it_8026E4D0`; never reports a removal.
pub unsafe extern "C" fn it_8027A118(item_gobj: *mut HSD_GObj, arg1: i32) -> bool {
    ext::it_8026E4D0(item_gobj, arg1);
    false
}

/// `it_8027A13C`: clears the Pokemon's two vectors (x4 and x10).
pub unsafe extern "C" fn it_8027A13C(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).xDD4_itemVar.pokemon_spawn.x10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    (*item).xDD4_itemVar.pokemon_spawn.x4 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
}

/// `it_8027A160`: the bone's translation (scaled, with |v| < 0.001 zeroed) becomes the per-frame delta.
pub unsafe extern "C" fn it_8027A160(bone: *mut HSD_JObj, item: *mut Item) {
    if !bone.is_null() {
        let mut sp1c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut sp10: Vec3 = core::mem::zeroed();
        ext::HSD_JObjGetTranslation(bone, &raw mut sp10);
        let s = (*(*item).xCC_item_attr).x60_scale;
        sp10.x *= s;
        sp10.y *= s;
        sp10.z *= s;
        if sp10.x.abs() < 0.001 {
            sp10.x = 0.0;
        }
        if sp10.y.abs() < 0.001 {
            sp10.y = 0.0;
        }
        if sp10.z.abs() < 0.001 {
            sp10.z = 0.0;
        }
        let ps = &raw mut (*item).xDD4_itemVar.pokemon_spawn;
        (*ps).x4.x = sp10.x - (*ps).x10.x;
        (*ps).x4.y = sp10.y - (*ps).x10.y;
        (*ps).x4.z = sp10.z - (*ps).x10.z;
        (*ps).x10 = sp10;
        ext::HSD_JObjSetTranslate(bone, &raw mut sp1c);
    }
}

/// `it_8027A344`: the Pokemon's velocity from its per-frame delta.
pub unsafe extern "C" fn it_8027A344(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).x40_vel.x = (*item).xDD4_itemVar.pokemon_spawn.x4.z * (*item).facing_dir;
    (*item).x40_vel.y = (*item).xDD4_itemVar.pokemon_spawn.x4.y;
}

/// `it_8027A364`: the total weight of the kinds that may be picked now (1 plus the weights of the kinds
/// other than the last two picked).
pub unsafe extern "C" fn it_8027A364(item: *mut Item) -> i32 {
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itPokemonSpawn_DatAttrs;
    let mut ret_val = 1i32;
    let start = It_PKind_Start;
    let end = It_PKind_Terminate;
    let mut i = start;
    while i < end {
        if ext::Item_804A0E24.last_kind != i && ext::Item_804A0E24.previous_kind != i {
            ret_val += (*attr).pokemon_spawn_weights[(i - It_PKind_Start) as usize];
        }
        i += 1;
    }
    ret_val
}

/// `it_8027A4D4`: a random Pokemon by weight (with the two rare spawns checked first). Returns the index.
pub unsafe extern "C" fn it_8027A4D4(item: *mut Item) -> i32 {
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itPokemonSpawn_DatAttrs;

    if ext::HSD_Randi(251) == 0 && ext::Item_804A0E24.rare_spawned == 0 && ext::gm_80165084() {
        ext::Item_804A0E24.rare_spawned = 1;
        ext::gm_80172C04();
        return 23;
    }
    if ext::HSD_Randi(251) == 0 && ext::Item_804A0E24.rare_spawned == 0 && ext::gm_80164ABC() {
        ext::Item_804A0E24.rare_spawned = 1;
        ext::gm_80172BC4();
        return 22;
    }
    let rand_int = ext::HSD_Randi(it_8027A364(item));
    let mut var_r3 = 0i32;
    let mut i = It_PKind_Start;
    while i < It_PKind_Terminate {
        let previous_kind = ext::Item_804A0E24.previous_kind;
        let last_kind = ext::Item_804A0E24.last_kind;
        if last_kind != i && previous_kind != i {
            var_r3 += (*attr).pokemon_spawn_weights[(i - It_PKind_Start) as usize];
            if var_r3 >= rand_int {
                ext::Item_804A0E24.previous_kind = last_kind;
                ext::Item_804A0E24.last_kind = i;
                return i - It_PKind_Start;
            }
        }
        i += 1;
    }
    0
}

/// `struct PokemonSpawnWeight` (it_8027A780's local): a kind and its weight.
#[derive(Clone, Copy)]
struct PokemonSpawnWeight {
    kind: ItemKind,
    weight: i32,
}

/// `it_8027A780`: picks a Pokemon from the kind list `arg1` (terminated by `It_PKind_Terminate`), by weight,
/// skipping the last two picks. Returns the index.
pub unsafe extern "C" fn it_8027A780(item: *mut Item, arg1: *mut c_void) -> i32 {
    let mut weights: [PokemonSpawnWeight; 30] = [PokemonSpawnWeight { kind: 0, weight: 0 }; 30];
    let mut cnt = 0i32;
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itPokemonSpawn_DatAttrs;
    let mut total = 0i32;
    let last_kind = ext::Item_804A0E24.last_kind;
    let previous_kind = ext::Item_804A0E24.previous_kind;
    let list = arg1 as *const ItemKind;
    let mut buf = 0usize;
    for i in 0..30 {
        let kind = *list.add(i);
        if kind == It_PKind_Terminate {
            break;
        }
        if last_kind != kind && previous_kind != kind {
            weights[buf].kind = kind;
            cnt += 1;
            weights[buf].weight = (*attr).pokemon_spawn_weights[(kind - It_PKind_Start) as usize];
            total += weights[buf].weight;
            buf += 1;
        }
    }
    let rand_int = ext::HSD_Randi(total);
    let mut accum = 0i32;
    let mut result: ItemKind = It_PKind_Sonans;
    let mut idx = 0usize;
    let mut remaining = cnt;
    while remaining > 0 {
        accum += weights[idx].weight;
        if rand_int < accum {
            result = weights[idx].kind;
            break;
        }
        idx += 1;
        remaining -= 1;
    }
    ext::Item_804A0E24.previous_kind = ext::Item_804A0E24.last_kind;
    ext::Item_804A0E24.last_kind = result;
    result - It_PKind_Start
}

/// `it_8027A9B8`: a Pokemon for the stage's own list (gm_801BEC54), with the rare spawns checked first.
pub unsafe extern "C" fn it_8027A9B8(item: *mut Item) -> i32 {
    let vec = ext::gm_801BEC54();
    if vec.is_null() {
        return It_PKind_Sonans;
    }
    if ext::HSD_Randi(251) == 0 && ext::Item_804A0E24.rare_spawned == 0 && ext::gm_80165084() {
        ext::Item_804A0E24.rare_spawned = 1;
        ext::gm_80172C04();
        return 23;
    }
    if ext::HSD_Randi(251) == 0 && ext::Item_804A0E24.rare_spawned == 0 && ext::gm_80164ABC() {
        ext::Item_804A0E24.rare_spawned = 1;
        ext::gm_80172BC4();
        return 22;
    }
    it_8027A780(item, vec)
}

/// `it_8027AAA0`: sets up the spawned Pokemon's timers, scales and motion table from its attributes.
pub unsafe extern "C" fn it_8027AAA0(item1_gobj: *mut HSD_GObj, item2: *mut Item, arg2: i32) {
    let it = item_of(item1_gobj);
    let attr = (*(*it).xC4_article_data).x4_specialAttributes as *mut itPokemonSpawn_DatAttrs;
    let ps = &raw mut (*item2).xDD4_itemVar.pokemon_spawn;
    if arg2 == It_PKind_Marumine {
        (*ps).x1C = 0.0;
        (*ps).x20 = 0.0;
    } else {
        (*ps).x1C = (*attr).xC;
        (*ps).x20 = (*attr).x10;
    }
    (*ps).x24 = (*attr).x14;
    (*ps).x28 = (*attr).x18 as _;
    (*ps).x48 = ((*(*item2).xCC_item_attr).x60_scale - (*ps).x4C) / (*ps).x28;
    (*ps).x44 = (*attr).x34;
    (*ps).x54 = (*attr).x38;
    (*ps).x5C = 0;
    (*ps).x58 = 0;
    (*ps).x50 = 0.0;
    for i in 0..(*attr).x38 {
        (*ps).x2C[i as usize] = (*attr).x1C[i as usize] * (*(*item2).xCC_item_attr).x60_scale;
    }
}

/// `selectPokemonForOpening` (static inline): a weighted pick for the opening movie.
unsafe fn selectPokemonForOpening(item: *mut Item) -> i32 {
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itPokemonSpawn_DatAttrs;
    let rand_int = ext::HSD_Randi(it_8027A364(item));
    let mut total = 0i32;
    let mut index = It_PKind_Start;
    while index < It_PKind_Terminate {
        let previous_kind = ext::Item_804A0E24.previous_kind;
        let last_kind = ext::Item_804A0E24.last_kind;
        if last_kind != index && previous_kind != index {
            total += (*attr).pokemon_spawn_weights[(index - It_PKind_Start) as usize];
            if total >= rand_int {
                ext::Item_804A0E24.previous_kind = last_kind;
                ext::Item_804A0E24.last_kind = index;
                return index - It_PKind_Start;
            }
        }
        index += 1;
    }
    0
}

/// `selectPokemonFromList` (static inline): `it_8027A780` over `kinds`, or Sonans when there is no list.
unsafe fn selectPokemonFromList(item: *mut Item, kinds: *mut ItemKind) -> i32 {
    if kinds.is_null() {
        return It_PKind_Sonans;
    }
    it_8027A780(item, kinds as *mut c_void)
}

/// `it_8027AB64`: spawns a Pokemon for the item (the kind from the mode, the list or the selection) and
/// returns true when a spawn happened.
pub unsafe extern "C" fn it_8027AB64(item_gobj: *mut HSD_GObj) -> bool {
    let mut spawn: SpawnItem = core::mem::zeroed();
    let item = item_of(item_gobj);
    let temp_r3 = ext::gm_8016AEB8();
    if temp_r3 != 0 {
        if temp_r3 == 1 {
            spawn.kind = it_8027A9B8(item);
        } else {
            spawn.kind = temp_r3 as i32 - It_PKind_Start;
        }
    } else if ext::gm_8018841C() {
        spawn.kind = selectPokemonFromList(item, ext::common_pokemon);
    } else if ext::gm_GetCurrentGameMode() == GM_OPENING_MV as i32 {
        spawn.kind = selectPokemonForOpening(item);
    } else {
        spawn.kind = ext::db_GetCurrentlySelectedPokemon();
        if spawn.kind == Pokemon_ID_Tosakinto {
            spawn.kind = it_8027A4D4(item);
        } else {
            spawn.kind -= 1;
        }
        if spawn.kind == Pokemon_ID_Metamon && (ext::it_8026B3C0(It_PKind_Metamon) != 0 || ext::ftLib_800860C4() == 4) {
            spawn.kind = Pokemon_ID_Tosakinto;
        }
    }

    let item2 = item_of(item_gobj);
    {
        let attrs = (*(*item2).xC4_article_data).x4_specialAttributes as *mut itPokemonSpawn_DatAttrs;
        let prev_pos = &raw mut spawn.prev_pos;
        spawn.vel.z = 0.0;
        spawn.vel.x = 0.0;
        spawn.vel.y = (*attrs).x8;
        *prev_pos = (*item2).pos;
        (*prev_pos).z = 0.0;
        it_8026BB88(item_gobj, &raw mut spawn.pos);
        spawn.x3C_damage = 0;
        spawn.facing_dir = it_8026B6A8(prev_pos, (*item2).owner);
        spawn.kind += It_PKind_Start;
        spawn.x0_parent_gobj = (*item2).owner;
        spawn.x4_parent_gobj2 = item_gobj;
        spawn.x44_flag.anon_0.set_b0(1);
        spawn.x40 = 0;
        if spawn.kind == It_PKind_Cerebi {
            ext::gm_80172C04();
        }
        if spawn.kind == It_PKind_Mew {
            ext::gm_80172BC4();
        }
    }
    {
        let spawn_gobj = ext::Item_80268B18(&raw mut spawn);
        if !spawn_gobj.is_null() {
            it_8027AAA0(item_gobj, item_of(spawn_gobj), spawn.kind);
            ext::pl_8003E70C(spawn_gobj);
            ext::db_80225DD8(spawn_gobj, (*item).owner);
            it_8027B288(spawn_gobj, 0x440060);
            it_8027B564(spawn_gobj);
            return true;
        }
    }
    false
}

/// `it_8027ADEC`: spawns effect `arg0` on the jobj `arg2`, scaled by the item's scale over `arg3`.
pub unsafe extern "C" fn it_8027ADEC(arg0: i32, item_gobj: *mut HSD_GObj, arg2: *mut HSD_JObj, arg3: f32) {
    let item = item_of(item_gobj);
    let mut sp1c: f32 = (*(*item).xCC_item_attr).x60_scale * (1.0f32 / arg3);
    ext::efSync_Spawn(arg0, item_gobj, arg2, &raw mut sp1c);
}

/// `it_8027AE34`: reverses the item's velocity and facing, resets its life timer and rotates it to face.
pub unsafe extern "C" fn it_8027AE34(item_gobj: *mut HSD_GObj) -> bool {
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let item = item_of(item_gobj);
    let mut rotate = it_803B8588;
    (*item).x40_vel.x = -(*item).x40_vel.x * (*item).xC70;
    (*item).facing_dir = -(*item).facing_dir;
    (*item).xD44_lifeTimer = (*item).xD48_halfLifeTimer;
    rotate.y = (IT_M_PI_2 * (*item).facing_dir as f64) as f32;
    ext::HSD_JObjSetRotation(item_jobj, &raw mut rotate);
    false
}

/// `it_8027AF50`: mirrors the velocity off the collision, re-derives the facing and rotates to face.
pub unsafe extern "C" fn it_8027AF50(item_gobj: *mut HSD_GObj) -> bool {
    let item = item_of(item_gobj);
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let mut rotate = it_803B8598;
    ext::lbVector_Mirror(&raw mut (*item).x40_vel, &raw mut (*item).xC58);
    it_80272980(item_gobj);
    rotate.y = (IT_M_PI_2 * (*item).facing_dir as f64) as f32;
    ext::HSD_JObjSetRotation(item_jobj, &raw mut rotate);
    false
}

/// `it_8027B070`: copies the owner's attack id, attack instance and attack data onto the item.
pub unsafe extern "C" fn it_8027B070(item_gobj: *mut HSD_GObj, owner_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    let fighter = (*owner_gobj).user_data as *mut Fighter;
    (*item).xD88_attackID = (*fighter).x2068_attackID;
    (*item).xD8C_attack_instance = (*fighter).x206C_attack_instance;
    (*item).xD90 = (*fighter).x2070;
    (*item).xD94 = (*fighter).x2074.x2074_vec;
    (*item).xD9C = (*fighter).x2074.x207C;
    (*item).xDA4_word = (*fighter).x2074.anon_10.x2084;
    (*item).xDA8_short = (*fighter).x2074.x2088;
}

/// `it_8027B0C4`: the spawned item's attack data, from its parent or its secondary parent.
pub unsafe extern "C" fn it_8027B0C4(item_gobj: *mut HSD_GObj, spawn: *mut SpawnItem) {
    if ext::ftLib_80086960((*spawn).x0_parent_gobj) {
        if ext::ftLib_80086960((*spawn).x4_parent_gobj2) {
            it_8027B070(item_gobj, (*spawn).x4_parent_gobj2);
        } else {
            let owner_item = item_of((*spawn).x4_parent_gobj2);
            let spawn_item = item_of(item_gobj);
            (*spawn_item).xD88_attackID = (*owner_item).xD88_attackID;
            (*spawn_item).xD8C_attack_instance = (*owner_item).xD8C_attack_instance;
            (*spawn_item).xD90 = (*owner_item).xD90;
            (*spawn_item).xD94 = (*owner_item).xD94;
            (*spawn_item).xD9C = (*owner_item).xD9C;
            (*spawn_item).xDA4_word = (*owner_item).xDA4_word;
            (*spawn_item).xDA8_short = (*owner_item).xDA8_short;
        }
    } else {
        let spawn_item = item_of(item_gobj);
        let mut sp24: Struct2070 = core::mem::zeroed();
        sp24.x2070_int = 0;
        (*spawn_item).xD88_attackID = 1;
        (*spawn_item).xD8C_attack_instance = 0;
        (*spawn_item).xD90 = sp24;
        ext::ft_80089768(&raw mut (*spawn_item).xD94 as *mut Vec);
        (*spawn_item).xDA8_short = 0;
    }
}

/// `it_8027B1F4`: the item's attack data, from its owner or the default when it has none.
pub unsafe extern "C" fn it_8027B1F4(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    let mut sp10: Struct2070 = core::mem::zeroed();
    sp10.x2070_int = 0;
    (*item).xD88_attackID = 1;
    (*item).xD8C_attack_instance = 0;
    (*item).xD90 = sp10;
    (*item).xDA8_short = 0;
    if ext::ftLib_80086960((*item).owner) {
        let temp_r3 = ext::ft_800898A8((*item).owner);
        (*item).xD94 = (*temp_r3).x2074_vec;
        (*item).xD9C = (*temp_r3).x207C;
        (*item).xDA4_word = (*temp_r3).anon_10.x2084;
        return;
    }
    ext::ft_80089768(&raw mut (*item).xD94 as *mut Vec);
}

/// `it_8027B288`: sets the item's attack data word `arg1`, refreshing the attack id when it changed.
pub unsafe extern "C" fn it_8027B288(item_gobj: *mut HSD_GObj, arg1: u32) {
    let item = item_of(item_gobj);
    let mut sp14: Struct2070 = core::mem::zeroed();
    sp14.x2070_int = arg1 as i32;
    if sp14.anon_0.x2073 == 0 || sp14.anon_0.x2073 != (*item).xD90.anon_0.x2073 {
        (*item).xDA8_short = ext::plAttack_80037B08() as _;
    }
    (*item).xD90 = sp14;
    if ext::ftLib_80086960((*item).owner) {
        let temp_r3 = ext::ft_800898A8((*item).owner);
        (*item).xD94 = (*temp_r3).x2074_vec;
        (*item).xD9C = (*temp_r3).x207C;
        (*item).xDA4_word = (*temp_r3).anon_10.x2084;
        return;
    }
    ext::ft_80089768(&raw mut (*item).xD94 as *mut Vec);
}

/// `it_8027B330`: sets the item's attack id to `arg1` (a new attack instance unless it repeats).
pub unsafe extern "C" fn it_8027B330(item_gobj: *mut HSD_GObj, arg1: u32) {
    let item = item_of(item_gobj);
    if arg1 == 1 || arg1 as i32 != (*item).xD88_attackID {
        (*item).xD88_attackID = arg1 as i32;
        (*item).xD8C_attack_instance = ext::plStale_IncrementAttackInstance() as _;
    }
}

/// `it_8027B378`: the hit sound/rumble for a fighter that picked up the item (for item-class collisions).
pub unsafe extern "C" fn it_8027B378(fighter_gobj: *mut HSD_GObj, item_gobj: *mut HSD_GObj, arg2: f32) {
    if it_8026B6C8(item_gobj) {
        let temp_r30 = (*ext::ft_80089884(fighter_gobj)).anon_0.x2073;
        let temp_r31 = ext::ftLib_800874BC(fighter_gobj);
        let temp_r3 = ext::ftLib_80086BE0(fighter_gobj) as u16;
        ext::pl_8003EB30(arg2, temp_r3, temp_r31, 6, 0, temp_r30);
    }
}

/// `it_8027B408`: as `it_8027B378` for the owner of `item_gobj1`.
pub unsafe extern "C" fn it_8027B408(item_gobj1: *mut HSD_GObj, item_gobj2: *mut HSD_GObj, arg8: f32) {
    if it_8026B6C8(item_gobj2) {
        let item1 = item_of(item_gobj1);
        if ext::ftLib_80086960((*item1).owner) {
            let owner = (*item1).owner;
            let temp_r31 = ext::ftLib_800874BC(owner);
            let temp_r3 = ext::ftLib_80086BE0(owner) as u16;
            ext::pl_8003EB30(arg8, temp_r3, temp_r31, 6, 0, (*item1).xD90.anon_0.x2073);
        }
    }
}

/// `it_8027B508`: the owner's player-state update from the item's attack data.
pub unsafe extern "C" fn it_8027B508(item_gobj1: *mut HSD_GObj, item_gobj2: *mut HSD_GObj) {
    if it_8026B6C8(item_gobj2) {
        let item1 = item_of(item_gobj1);
        if ext::ftLib_80086960((*item1).owner) {
            ext::pl_800384DC((*item1).owner, (*item1).xD90.anon_0.x2073, &raw mut (*item1).xD94 as *mut c_void);
        }
    }
}

/// `it_8027B564`: the owner's attack data pushed to the player state.
pub unsafe extern "C" fn it_8027B564(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    if ext::ftLib_80086960((*item).owner) {
        let mut sp10 = (*item).xD90;
        ext::pl_80037DF4((*item).owner, &raw mut sp10);
    }
}
