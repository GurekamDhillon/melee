//! Port of src/melee/it/it_279C.c: the Pokemon item behaviour (the spawn kind selection from the common list
//! or the stage's list, the spawn scale animation, the attack-id/attack-instance copies from the owner, the
//! facing/rotation helpers and the spawn driver `it_8027AB64`). The item logic tables at the top of the C file
//! (`it_803F23CC` and friends) are data, not functions, and are not part of this port.
//! Callees of other TUs go through `ext` (unset hooks panic "TODO"). Checked against the C by the state oracle
//! `tests/tu_it_279C.rs` (gen/mtu).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_mut, unused_assignments, unused_variables, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

use crate::tu::it_26B1::{it_8026B3A8, it_8026B6A8, it_8026B6C8, it_8026BB88, it_8026BDB4};
use crate::tu::it_2725::{it_80272DE4, it_80272F7C, it_80272980};
use crate::tu::itzako::it_8027B964;

/// One argument of a variadic effect spawn (`efSync_Spawn` / `efAsync_Spawn` are C varargs; the port keeps the
/// C type of each extra argument so the eventual implementation can read it back).
#[derive(Clone, Copy)]
pub enum EfArg {
    Int(i32),
    Ptr(*mut c_void),
    F32P(*const f32),
    Vec3P(*const Vec3),
}

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn db_GetCurrentlySelectedPokemon() -> i32;
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[EfArg]) -> *mut c_void;
    fn ftLib_800860C4() -> i32;
    fn ftLib_8008627C(a0: *mut Vec, a1: *mut HSD_GObj) -> *mut HSD_GObj;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::ftlib::ftLib_80086644(a0, a1) };
    fn ftLib_80086960(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_80086960(a0) as _ };
    fn ftLib_80086BE0(a0: *mut HSD_GObj) -> u8 => unsafe { crate::tu::ftlib::ftLib_80086BE0(a0) };
    fn ftLib_800874BC(a0: *mut HSD_GObj) -> i32;
    fn ft_80089768(a0: *mut Vec2);
    fn ft_80089884(a0: *mut HSD_GObj) -> *mut Struct2070;
    fn ft_800898A8(a0: *mut HSD_GObj) -> *mut Struct2074;
    fn gm_80164ABC() -> i32;
    fn gm_80165084() -> i32;
    fn gm_8016AEB8() -> i32;
    fn gm_80172BC4() -> i32;
    fn gm_80172C04() -> i32;
    fn gm_8018841C() -> i32;
    fn gm_801BEC54() -> *mut c_void;
    fn gm_GetCurrentGameMode() -> u8;
    fn itCerebi_Logic23_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itcerebi::itCerebi_Logic23_Spawned(a0 as _) };
    fn itChicoritaLeaf_Logic30_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itchicorita::itChicoritaLeaf_Logic30_EvtUnk(a0 as _, a1) };
    fn itChicoritaLeaf_Logic30_HitShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itchicorita::itChicoritaLeaf_Logic30_HitShield(a0 as _) as _ };
    fn itChicoritaLeaf_Logic30_Reflected(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itchicorita::itChicoritaLeaf_Logic30_Reflected(a0 as _) as _ };
    fn itChicoritaLeaf_Logic30_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itchicorita::itChicoritaLeaf_Logic30_Spawned(a0 as _) };
    fn itChicorita_Logic1_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itchicorita::itChicorita_Logic1_EvtUnk(a0 as _, a1) };
    fn itFire_Logic6_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itfire::itFire_Logic6_EvtUnk(a0 as _, a1 as _) };
    fn itFire_Logic6_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itfire::itFire_Logic6_Spawned(a0 as _) };
    fn itFreezer_802CCF9C(a0: *mut HSD_GObj) => unsafe { crate::tu::itfreezer::itFreezer_802CCF9C(a0 as _) };
    fn itFreezer_802CD000(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itfreezer::itFreezer_802CD000(a0 as _, a1 as _) };
    fn itFushigibana_Logic29_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itfushigibana::itFushigibana_Logic29_Spawned(a0 as _) };
    fn itHassam_802CDBE0(a0: *mut HSD_GObj) => unsafe { crate::tu::ithassam::itHassam_802CDBE0(a0) };
    fn itHassam_802CDC60(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::ithassam::itHassam_802CDC60(a0, a1) };
    fn itHitodeman_Logic43_Absorbed(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ithitodeman::itHitodeman_Logic43_Absorbed(a0 as _) };
    fn itKabigon_Logic2_Destroyed(a0: *mut HSD_GObj) => unsafe { crate::tu::itkabigon::itKabigon_Logic2_Destroyed(a0 as _) };
    fn itKamex_Logic31_DmgDealt(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itkamex::itKamex_Logic31_DmgDealt(a0 as _) };
    fn itKamex_Logic31_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itkamex::itKamex_Logic31_EvtUnk(a0 as _, a1 as _) };
    fn itKamex_Logic31_HitShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itkamex::itKamex_Logic31_HitShield(a0 as _) };
    fn itKireihana_Logic15_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itkireihana::itKireihana_Logic15_EvtUnk(a0 as _, a1 as _) };
    fn itKireihana_Logic15_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itkireihana::itKireihana_Logic15_Spawned(a0 as _) };
    fn itLizardon_Logic34_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itlizardon::itLizardon_Logic34_Spawned(a0 as _) };
    fn itLizardon_Logic35_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itlizardon::itLizardon_Logic35_Spawned(a0 as _) };
    fn itLizardon_Logic36_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itlizardon::itLizardon_Logic36_Spawned(a0 as _) };
    fn itLizardon_Logic37_Absorbed(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itlizardon::itLizardon_Logic37_Absorbed(a0 as _) };
    fn itLizardon_Logic37_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itlizardon::itLizardon_Logic37_EvtUnk(a0 as _, a1 as _) };
    fn itLizardon_Logic37_HitShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itlizardon::itLizardon_Logic37_HitShield(a0 as _) };
    fn itLizardon_Logic37_Reflected(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itlizardon::itLizardon_Logic37_Reflected(a0 as _) };
    fn itLizardon_Logic37_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itlizardon::itLizardon_Logic37_Spawned(a0 as _) };
    fn itLucky_Logic44_Dropped(a0: *mut HSD_GObj) => unsafe { crate::tu::itlucky::itLucky_Logic44_Dropped(a0 as _) };
    fn itLucky_Logic44_EnteredAir(a0: *mut HSD_GObj) => unsafe { crate::tu::itlucky::itLucky_Logic44_EnteredAir(a0 as _) };
    fn itLucky_Logic44_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itlucky::itLucky_Logic44_EvtUnk(a0 as _, a1 as _) };
    fn itLucky_Logic44_PickedUp(a0: *mut HSD_GObj) => unsafe { crate::tu::itlucky::itLucky_Logic44_PickedUp(a0 as _) };
    fn itLucky_Logic44_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itlucky::itLucky_Logic44_Spawned(a0 as _) };
    fn itLugia_Logic17_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itlugia::itLugia_Logic17_EvtUnk(a0 as _, a1 as _) };
    fn itLugia_Logic39_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itlugia::itLugia_Logic39_Spawned(a0 as _) };
    fn itLugia_Logic40_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itlugia::itLugia_Logic40_Spawned(a0 as _) };
    fn itLugia_Logic41_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itlugia::itLugia_Logic41_Spawned(a0 as _) };
    fn itMaril_Logic28_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itmaril::itMaril_Logic28_Spawned(a0 as _) };
    fn itMarumine_Logic16_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itmarumine::itMarumine_Logic16_Spawned(a0 as _) };
    fn itMatadogas_Logic33_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itmatadogas::itMatadogas_Logic33_EvtUnk(a0 as _, a1 as _) };
    fn itMetamon_Logic19_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itmetamon::itMetamon_Logic19_EvtUnk(a0 as _, a1 as _) };
    fn itMew_Logic22_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itmew::itMew_Logic22_Spawned(a0 as _) };
    fn itOldKuri_Logic29_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itoldkuri::itOldKuri_Logic29_EvtUnk(a0 as _, a1 as _) };
    fn itPippi_Logic20_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itpippi::itPippi_Logic20_Spawned(a0 as _) };
    fn itPorygon2_Logic26_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itporygon2::itPorygon2_Logic26_Spawned(a0 as _) };
    fn itSonans_Logic9_DmgDealt(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itsonans::itSonans_Logic9_DmgDealt(a0 as _) as _ };
    fn itSuikun_Logic14_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itsuikun::itSuikun_Logic14_Spawned(a0 as _) };
    fn itThunder_Logic7_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itthunder::itThunder_Logic7_Spawned(a0 as _) };
    fn itTogepy_Logic21_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::ittogepy::itTogepy_Logic21_EvtUnk(a0 as _, a1 as _) };
    fn itTogepy_Logic21_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::ittogepy::itTogepy_Logic21_Spawned(a0 as _) };
    fn itTosakinto_Logic0_Destroyed(a0: *mut HSD_GObj) => unsafe { crate::tu::ittosakinto::itTosakinto_Logic0_Destroyed(a0 as _) };
    fn itUnknown_Logic38_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itunknown::itUnknown_Logic38_EvtUnk(a0 as _, a1 as _) };
    fn it_2725_Logic17_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itlugia::it_2725_Logic17_Spawned(a0 as _) };
    fn it_2725_Logic18_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::ithouou::it_2725_Logic18_Spawned(a0 as _) };
    fn it_2725_Logic24_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::ithitodeman::it_2725_Logic24_Spawned(a0 as _) };
    fn it_2725_Logic27_DmgReceived(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ithinoarashi::it_2725_Logic27_DmgReceived(a0 as _) };
    fn it_2725_Logic31_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itkamex::it_2725_Logic31_Spawned(a0 as _) };
    fn it_2725_Logic32_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itmatadogas::it_2725_Logic32_Spawned(a0 as _) };
    fn it_2725_Logic33_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itmatadogas::it_2725_Logic33_Spawned(a0 as _) };
    fn it_2725_Logic38_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::itunknown::it_2725_Logic38_Spawned(a0 as _) };
    fn it_2725_Logic42_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::ithouou::it_2725_Logic42_Spawned(a0 as _) };
    fn it_2725_Logic43_Spawned(a0: *mut HSD_GObj) => unsafe { crate::tu::ithitodeman::it_2725_Logic43_Spawned(a0 as _) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3C0(a0: i32) -> i32 => unsafe { crate::tu::it_26B1::it_8026B3C0(a0) };
    fn it_8026B6A8(a0: *mut Vec, a1: *mut HSD_GObj) -> f32 => unsafe { crate::tu::it_26B1::it_8026B6A8(a0 as _, a1) };
    fn it_8026B6C8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b6c8(&*((*a0).user_data as *mut Item)) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026BDB4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BDB4(a0) };
    fn it_8026E4D0(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E4D0(a0, a1) };
    fn it_8026EECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itdraw::it_8026EECC(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272980(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272980(a0) };
    fn it_80272DE4(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::it_2725::it_80272DE4(a0, a1) };
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::it_2725::it_80272F7C(a0, a1) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B964(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itzako::it_8027B964(a0, a1) };
    fn it_802C8F4C(a0: *mut HSD_GObj) => unsafe { crate::tu::ittosakinto::it_802C8F4C(a0 as _) };
    fn it_802C8FC4(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802c8fc4(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802C9588(a0: *mut HSD_GObj) => unsafe { crate::tu::itchicorita::it_802C9588(a0 as _) };
    fn it_802C9D40(a0: *mut HSD_GObj) => unsafe { crate::tu::itkabigon::it_802C9D40(a0 as _) };
    fn it_802C9E24(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802c9e24(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802CA49C(a0: *mut HSD_GObj) => unsafe { crate::tu::itkamex::it_802CA49C(a0 as _) };
    fn it_802CA538(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802ca538(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802CAFD4(a0: *mut HSD_GObj) => unsafe { crate::tu::itmatadogas::it_802CAFD4(a0 as _) };
    fn it_802CB0F8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802cb0f8(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802CB8AC(a0: *mut HSD_GObj) => unsafe { crate::tu::itlizardon::it_802CB8AC(a0 as _) };
    fn it_802CB940(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802cb940(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802CCB14(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802ccb14(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802CD44C(a0: *mut HSD_GObj) => unsafe { crate::tu::itsonans::it_802CD44C(a0 as _) };
    fn it_802CD4DC(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802cd4dc(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802CD7D4(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itkinds::it_802cd7d4(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802CE710(a0: *mut HSD_GObj) => unsafe { crate::tu::itunknown::it_802CE710(a0 as _) };
    fn it_802CE7D0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802ce7d0(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802CF3E0(a0: *mut HSD_GObj) => unsafe { crate::tu::itentei::it_802CF3E0(a0 as _) };
    fn it_802CF450(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802cf450(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802CF814(a0: *mut HSD_GObj) => unsafe { crate::tu::itraikou::it_802CF814(a0 as _) };
    fn it_802CF884(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802cf884(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802CFCB8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802cfcb8(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D0AB0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d0ab0(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D0AD0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmarumine::it_802D0AD0(a0 as _) as _ };
    fn it_802D0D18(a0: *mut HSD_GObj) => unsafe { crate::tu::itmarumine::it_802D0D18(a0 as _) };
    fn it_802D0E30(a0: *mut HSD_GObj) => unsafe { crate::tu::itmarumine::it_802D0E30(a0 as _) };
    fn it_802D0E90(a0: *mut HSD_GObj) => unsafe { crate::tu::itmarumine::it_802D0E90(a0 as _) };
    fn it_802D23D4(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d23d4(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D25BC(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d25bc(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D2ED0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d2ed0(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D3008(a0: *mut HSD_GObj) => unsafe { crate::tu::itmetamon::it_802D3008(a0 as _) };
    fn it_802D32DC(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d32dc(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D3B6C(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d3b6c(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D3F4C(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d3f4c(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D43B0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itkinds5::it_802d43b0(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D4EF4(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itkinds::it_802d4ef4(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802D4F08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ithitodeman::it_802D4F08(a0 as _) };
    fn it_802D4F28(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ithitodeman::it_802D4F28(a0 as _) };
    fn it_802D4F50(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd6::it_802d4f50(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802D4F58(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d4f58(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D5050(a0: *mut HSD_GObj) => unsafe { crate::tu::itlucky::it_802D5050(a0 as _) };
    fn it_802D5648(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itlucky::it_802D5648(a0 as _) };
    fn it_802D56F0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d56f0(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D5CD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d5cd8(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D5CF8(a0: *mut HSD_GObj) => unsafe { crate::tu::ithinoarashi::it_802D5CF8(a0 as _) };
    fn it_802D61A8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d61a8(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D6808(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd8::it_802d6808(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802D6810(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd::it_802d6810(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802D6830(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd6::it_802d6830(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802D6838(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd8::it_802d6838(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802D6840(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd6::it_802d6840(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802D6848(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd6::it_802d6848(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802D6850(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d6850(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D6F00(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaril::it_802D6F00(a0 as _) as _ };
    fn it_802D6FB0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::itfwd::it_802d6fb0(&mut *((*a0).user_data as *mut Item), a1) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
    fn plAttack_80037B08() -> u16;
    fn plStale_IncrementAttackInstance() -> u16;
    fn pl_80037DF4(a0: *mut HSD_GObj, a1: *mut Struct2070);
    fn pl_800384DC(a0: *mut HSD_GObj, a1: i32, a2: *mut c_void);
    fn pl_8003E70C(a0: *mut HSD_GObj);
    fn pl_8003EB30(a0: f32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn itCerebi_Logic23_Spawned(a0: *mut HSD_GObj) { super::itCerebi_Logic23_Spawned(a0) }
        pub unsafe extern "C" fn itChicoritaLeaf_Logic30_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itChicoritaLeaf_Logic30_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itChicoritaLeaf_Logic30_HitShield(a0: *mut HSD_GObj) -> i32 { super::itChicoritaLeaf_Logic30_HitShield(a0) }
        pub unsafe extern "C" fn itChicoritaLeaf_Logic30_Reflected(a0: *mut HSD_GObj) -> i32 { super::itChicoritaLeaf_Logic30_Reflected(a0) }
        pub unsafe extern "C" fn itChicoritaLeaf_Logic30_Spawned(a0: *mut HSD_GObj) { super::itChicoritaLeaf_Logic30_Spawned(a0) }
        pub unsafe extern "C" fn itChicorita_Logic1_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itChicorita_Logic1_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itFire_Logic6_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itFire_Logic6_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itFire_Logic6_Spawned(a0: *mut HSD_GObj) { super::itFire_Logic6_Spawned(a0) }
        pub unsafe extern "C" fn itFreezer_802CCF9C(a0: *mut HSD_GObj) { super::itFreezer_802CCF9C(a0) }
        pub unsafe extern "C" fn itFreezer_802CD000(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itFreezer_802CD000(a0, a1) }
        pub unsafe extern "C" fn itFushigibana_Logic29_Spawned(a0: *mut HSD_GObj) { super::itFushigibana_Logic29_Spawned(a0) }
        pub unsafe extern "C" fn itHassam_802CDBE0(a0: *mut HSD_GObj) { super::itHassam_802CDBE0(a0) }
        pub unsafe extern "C" fn itHassam_802CDC60(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itHassam_802CDC60(a0, a1) }
        pub unsafe extern "C" fn itHitodeman_Logic43_Absorbed(a0: *mut HSD_GObj) -> i32 { super::itHitodeman_Logic43_Absorbed(a0) }
        pub unsafe extern "C" fn itKabigon_Logic2_Destroyed(a0: *mut HSD_GObj) { super::itKabigon_Logic2_Destroyed(a0) }
        pub unsafe extern "C" fn itKamex_Logic31_DmgDealt(a0: *mut HSD_GObj) -> i32 { super::itKamex_Logic31_DmgDealt(a0) }
        pub unsafe extern "C" fn itKamex_Logic31_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itKamex_Logic31_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itKamex_Logic31_HitShield(a0: *mut HSD_GObj) -> i32 { super::itKamex_Logic31_HitShield(a0) }
        pub unsafe extern "C" fn itKireihana_Logic15_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itKireihana_Logic15_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itKireihana_Logic15_Spawned(a0: *mut HSD_GObj) { super::itKireihana_Logic15_Spawned(a0) }
        pub unsafe extern "C" fn itLizardon_Logic34_Spawned(a0: *mut HSD_GObj) { super::itLizardon_Logic34_Spawned(a0) }
        pub unsafe extern "C" fn itLizardon_Logic35_Spawned(a0: *mut HSD_GObj) { super::itLizardon_Logic35_Spawned(a0) }
        pub unsafe extern "C" fn itLizardon_Logic36_Spawned(a0: *mut HSD_GObj) { super::itLizardon_Logic36_Spawned(a0) }
        pub unsafe extern "C" fn itLizardon_Logic37_Absorbed(a0: *mut HSD_GObj) -> i32 { super::itLizardon_Logic37_Absorbed(a0) }
        pub unsafe extern "C" fn itLizardon_Logic37_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itLizardon_Logic37_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itLizardon_Logic37_HitShield(a0: *mut HSD_GObj) -> i32 { super::itLizardon_Logic37_HitShield(a0) }
        pub unsafe extern "C" fn itLizardon_Logic37_Reflected(a0: *mut HSD_GObj) -> i32 { super::itLizardon_Logic37_Reflected(a0) }
        pub unsafe extern "C" fn itLizardon_Logic37_Spawned(a0: *mut HSD_GObj) { super::itLizardon_Logic37_Spawned(a0) }
        pub unsafe extern "C" fn itLucky_Logic44_Dropped(a0: *mut HSD_GObj) { super::itLucky_Logic44_Dropped(a0) }
        pub unsafe extern "C" fn itLucky_Logic44_EnteredAir(a0: *mut HSD_GObj) { super::itLucky_Logic44_EnteredAir(a0) }
        pub unsafe extern "C" fn itLucky_Logic44_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itLucky_Logic44_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itLucky_Logic44_PickedUp(a0: *mut HSD_GObj) { super::itLucky_Logic44_PickedUp(a0) }
        pub unsafe extern "C" fn itLucky_Logic44_Spawned(a0: *mut HSD_GObj) { super::itLucky_Logic44_Spawned(a0) }
        pub unsafe extern "C" fn itLugia_Logic17_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itLugia_Logic17_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itLugia_Logic39_Spawned(a0: *mut HSD_GObj) { super::itLugia_Logic39_Spawned(a0) }
        pub unsafe extern "C" fn itLugia_Logic40_Spawned(a0: *mut HSD_GObj) { super::itLugia_Logic40_Spawned(a0) }
        pub unsafe extern "C" fn itLugia_Logic41_Spawned(a0: *mut HSD_GObj) { super::itLugia_Logic41_Spawned(a0) }
        pub unsafe extern "C" fn itMaril_Logic28_Spawned(a0: *mut HSD_GObj) { super::itMaril_Logic28_Spawned(a0) }
        pub unsafe extern "C" fn itMarumine_Logic16_Spawned(a0: *mut HSD_GObj) { super::itMarumine_Logic16_Spawned(a0) }
        pub unsafe extern "C" fn itMatadogas_Logic33_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itMatadogas_Logic33_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itMetamon_Logic19_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itMetamon_Logic19_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itMew_Logic22_Spawned(a0: *mut HSD_GObj) { super::itMew_Logic22_Spawned(a0) }
        pub unsafe extern "C" fn itOldKuri_Logic29_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itOldKuri_Logic29_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itPippi_Logic20_Spawned(a0: *mut HSD_GObj) { super::itPippi_Logic20_Spawned(a0) }
        pub unsafe extern "C" fn itPorygon2_Logic26_Spawned(a0: *mut HSD_GObj) { super::itPorygon2_Logic26_Spawned(a0) }
        pub unsafe extern "C" fn itSonans_Logic9_DmgDealt(a0: *mut HSD_GObj) -> i32 { super::itSonans_Logic9_DmgDealt(a0) }
        pub unsafe extern "C" fn itSuikun_Logic14_Spawned(a0: *mut HSD_GObj) { super::itSuikun_Logic14_Spawned(a0) }
        pub unsafe extern "C" fn itThunder_Logic7_Spawned(a0: *mut HSD_GObj) { super::itThunder_Logic7_Spawned(a0) }
        pub unsafe extern "C" fn itTogepy_Logic21_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itTogepy_Logic21_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn itTogepy_Logic21_Spawned(a0: *mut HSD_GObj) { super::itTogepy_Logic21_Spawned(a0) }
        pub unsafe extern "C" fn itTosakinto_Logic0_Destroyed(a0: *mut HSD_GObj) { super::itTosakinto_Logic0_Destroyed(a0) }
        pub unsafe extern "C" fn itUnknown_Logic38_EvtUnk(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::itUnknown_Logic38_EvtUnk(a0, a1) }
        pub unsafe extern "C" fn it_2725_Logic17_Spawned(a0: *mut HSD_GObj) { super::it_2725_Logic17_Spawned(a0) }
        pub unsafe extern "C" fn it_2725_Logic18_Spawned(a0: *mut HSD_GObj) { super::it_2725_Logic18_Spawned(a0) }
        pub unsafe extern "C" fn it_2725_Logic24_Spawned(a0: *mut HSD_GObj) { super::it_2725_Logic24_Spawned(a0) }
        pub unsafe extern "C" fn it_2725_Logic27_DmgReceived(a0: *mut HSD_GObj) -> i32 { super::it_2725_Logic27_DmgReceived(a0) }
        pub unsafe extern "C" fn it_2725_Logic31_Spawned(a0: *mut HSD_GObj) { super::it_2725_Logic31_Spawned(a0) }
        pub unsafe extern "C" fn it_2725_Logic32_Spawned(a0: *mut HSD_GObj) { super::it_2725_Logic32_Spawned(a0) }
        pub unsafe extern "C" fn it_2725_Logic33_Spawned(a0: *mut HSD_GObj) { super::it_2725_Logic33_Spawned(a0) }
        pub unsafe extern "C" fn it_2725_Logic38_Spawned(a0: *mut HSD_GObj) { super::it_2725_Logic38_Spawned(a0) }
        pub unsafe extern "C" fn it_2725_Logic42_Spawned(a0: *mut HSD_GObj) { super::it_2725_Logic42_Spawned(a0) }
        pub unsafe extern "C" fn it_2725_Logic43_Spawned(a0: *mut HSD_GObj) { super::it_2725_Logic43_Spawned(a0) }
        pub unsafe extern "C" fn it_8026EECC(a0: *mut HSD_GObj, a1: i32) { super::it_8026EECC(a0, a1) }
        pub unsafe extern "C" fn it_802C8F4C(a0: *mut HSD_GObj) { super::it_802C8F4C(a0) }
        pub unsafe extern "C" fn it_802C8FC4(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802C8FC4(a0, a1) }
        pub unsafe extern "C" fn it_802C9588(a0: *mut HSD_GObj) { super::it_802C9588(a0) }
        pub unsafe extern "C" fn it_802C9D40(a0: *mut HSD_GObj) { super::it_802C9D40(a0) }
        pub unsafe extern "C" fn it_802C9E24(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802C9E24(a0, a1) }
        pub unsafe extern "C" fn it_802CA49C(a0: *mut HSD_GObj) { super::it_802CA49C(a0) }
        pub unsafe extern "C" fn it_802CA538(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802CA538(a0, a1) }
        pub unsafe extern "C" fn it_802CAFD4(a0: *mut HSD_GObj) { super::it_802CAFD4(a0) }
        pub unsafe extern "C" fn it_802CB0F8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802CB0F8(a0, a1) }
        pub unsafe extern "C" fn it_802CB8AC(a0: *mut HSD_GObj) { super::it_802CB8AC(a0) }
        pub unsafe extern "C" fn it_802CB940(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802CB940(a0, a1) }
        pub unsafe extern "C" fn it_802CCB14(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802CCB14(a0, a1) }
        pub unsafe extern "C" fn it_802CD44C(a0: *mut HSD_GObj) { super::it_802CD44C(a0) }
        pub unsafe extern "C" fn it_802CD4DC(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802CD4DC(a0, a1) }
        pub unsafe extern "C" fn it_802CD7D4(a0: *mut HSD_GObj) -> i32 { super::it_802CD7D4(a0) }
        pub unsafe extern "C" fn it_802CE710(a0: *mut HSD_GObj) { super::it_802CE710(a0) }
        pub unsafe extern "C" fn it_802CE7D0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802CE7D0(a0, a1) }
        pub unsafe extern "C" fn it_802CF3E0(a0: *mut HSD_GObj) { super::it_802CF3E0(a0) }
        pub unsafe extern "C" fn it_802CF450(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802CF450(a0, a1) }
        pub unsafe extern "C" fn it_802CF814(a0: *mut HSD_GObj) { super::it_802CF814(a0) }
        pub unsafe extern "C" fn it_802CF884(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802CF884(a0, a1) }
        pub unsafe extern "C" fn it_802CFCB8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802CFCB8(a0, a1) }
        pub unsafe extern "C" fn it_802D0AB0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D0AB0(a0, a1) }
        pub unsafe extern "C" fn it_802D0AD0(a0: *mut HSD_GObj) -> i32 { super::it_802D0AD0(a0) }
        pub unsafe extern "C" fn it_802D0D18(a0: *mut HSD_GObj) { super::it_802D0D18(a0) }
        pub unsafe extern "C" fn it_802D0E30(a0: *mut HSD_GObj) { super::it_802D0E30(a0) }
        pub unsafe extern "C" fn it_802D0E90(a0: *mut HSD_GObj) { super::it_802D0E90(a0) }
        pub unsafe extern "C" fn it_802D23D4(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D23D4(a0, a1) }
        pub unsafe extern "C" fn it_802D25BC(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D25BC(a0, a1) }
        pub unsafe extern "C" fn it_802D2ED0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D2ED0(a0, a1) }
        pub unsafe extern "C" fn it_802D3008(a0: *mut HSD_GObj) { super::it_802D3008(a0) }
        pub unsafe extern "C" fn it_802D32DC(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D32DC(a0, a1) }
        pub unsafe extern "C" fn it_802D3B6C(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D3B6C(a0, a1) }
        pub unsafe extern "C" fn it_802D3F4C(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D3F4C(a0, a1) }
        pub unsafe extern "C" fn it_802D43B0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D43B0(a0, a1) }
        pub unsafe extern "C" fn it_802D4EF4(a0: *mut HSD_GObj) -> i32 { super::it_802D4EF4(a0) }
        pub unsafe extern "C" fn it_802D4F08(a0: *mut HSD_GObj) -> i32 { super::it_802D4F08(a0) }
        pub unsafe extern "C" fn it_802D4F28(a0: *mut HSD_GObj) -> i32 { super::it_802D4F28(a0) }
        pub unsafe extern "C" fn it_802D4F50(a0: *mut HSD_GObj) -> i32 { super::it_802D4F50(a0) }
        pub unsafe extern "C" fn it_802D4F58(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D4F58(a0, a1) }
        pub unsafe extern "C" fn it_802D5050(a0: *mut HSD_GObj) { super::it_802D5050(a0) }
        pub unsafe extern "C" fn it_802D5648(a0: *mut HSD_GObj) -> i32 { super::it_802D5648(a0) }
        pub unsafe extern "C" fn it_802D56F0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D56F0(a0, a1) }
        pub unsafe extern "C" fn it_802D5CD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D5CD8(a0, a1) }
        pub unsafe extern "C" fn it_802D5CF8(a0: *mut HSD_GObj) { super::it_802D5CF8(a0) }
        pub unsafe extern "C" fn it_802D61A8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D61A8(a0, a1) }
        pub unsafe extern "C" fn it_802D6808(a0: *mut HSD_GObj) -> i32 { super::it_802D6808(a0) }
        pub unsafe extern "C" fn it_802D6810(a0: *mut HSD_GObj) -> i32 { super::it_802D6810(a0) }
        pub unsafe extern "C" fn it_802D6830(a0: *mut HSD_GObj) -> i32 { super::it_802D6830(a0) }
        pub unsafe extern "C" fn it_802D6838(a0: *mut HSD_GObj) -> i32 { super::it_802D6838(a0) }
        pub unsafe extern "C" fn it_802D6840(a0: *mut HSD_GObj) -> i32 { super::it_802D6840(a0) }
        pub unsafe extern "C" fn it_802D6848(a0: *mut HSD_GObj) -> i32 { super::it_802D6848(a0) }
        pub unsafe extern "C" fn it_802D6850(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D6850(a0, a1) }
        pub unsafe extern "C" fn it_802D6F00(a0: *mut HSD_GObj) -> i32 { super::it_802D6F00(a0) }
        pub unsafe extern "C" fn it_802D6FB0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::it_802D6FB0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut Item_804A0E24: PokemonSelectionState = unsafe { core::mem::zeroed() };
    pub static mut it_803F7A48: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7A98: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7AC8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7AD8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7B08: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7B48: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7B58: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7BB0: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7BC0: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7C00: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7C10: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7C40: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7C70: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7CA0: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7CF8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7D60: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7DC8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7DE8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7E08: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7E28: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7E78: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7EE8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7F48: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7F58: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7FB8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7FC8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F7FF8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F8058: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F80C8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F80F8: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F8128: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F8180: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F8190: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F8200: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F8250: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F8270: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F82A0: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F82B0: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut it_803F82F0: [ItemStateTable; 1] = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

// ---- header inlines of sysdolphin/baselib/jobj.h used by this TU (HSD_ASSERT = assert!) ----

#[inline(always)]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "HSD_ASSERT(564, jobj)");
    let mut result = false;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0 {
        result = true;
    }
    result
}

/// `HSD_JObjSetMtxDirty` as the macro of jobj.h.
#[inline(always)]
unsafe fn HSD_JObjSetMtxDirty_macro(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *mut Vec3) {
    assert!(!jobj.is_null(), "HSD_ASSERT(760, jobj)");
    assert!(!scale.is_null(), "HSD_ASSERT(761, scale)");
    (*jobj).scale = *scale;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty_macro(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjGetScaleX(jobj: *mut HSD_JObj) -> f32 {
    assert!(!jobj.is_null(), "HSD_ASSERT(875, jobj)");
    (*jobj).scale.x
}

#[inline(always)]
unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    assert!(!jobj.is_null(), "HSD_ASSERT(916, jobj)");
    assert!(!translate.is_null(), "HSD_ASSERT(917, translate)");
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty_macro(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    assert!(!jobj.is_null(), "HSD_ASSERT(979, jobj)");
    assert!(!translate.is_null(), "HSD_ASSERT(980, translate)");
    *translate = (*jobj).translate;
}

#[inline(always)]
unsafe fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
    assert!(!jobj.is_null(), "HSD_ASSERT(618, jobj)");
    assert!(!rotate.is_null(), "HSD_ASSERT(619, rotate)");
    (*jobj).rotate = *rotate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty_macro(jobj);
    }
}

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
    ext::it_8026B3A8(item_gobj);
    ext::it_802756D0(item_gobj);
    ext::it_8026BDB4(item_gobj);
}

/// `it_80279D38`: the knockback path (`it_8027B964` with `chk` false).
pub unsafe extern "C" fn it_80279D38(item_gobj: *mut HSD_GObj) {
    ext::it_8027B964(item_gobj, 0);
}

/// `it_80279D5C`: sets the Pokemon's scale on all three axes and resets its spawn counter.
pub unsafe extern "C" fn it_80279D5C(item_gobj: *mut HSD_GObj, scale: f32) {
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let item = item_of(item_gobj);
    let mut sp18 = Vec3 { x: scale, y: scale, z: scale };
    (*item).xDD4_itemVar.pokemon_spawn.x4C = scale;
    (*item).xDD4_itemVar.pokemon_spawn.x5C = 0;
    HSD_JObjSetScale(item_jobj, &raw mut sp18);
}

/// `it_80279E24_inline` (static inline): one step of the spawn scale animation (the next keyframe).
unsafe fn it_80279E24_inline(item_gobj: *mut HSD_GObj) {
    let jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let item = (*item_gobj).user_data as *mut Item;
    let scale = HSD_JObjGetScaleX(jobj);
    let ps = &raw mut (*item).xDD4_itemVar.pokemon_spawn;
    ext::it_80272F7C(jobj, *((&raw const (*ps).x2C) as *const f32).offset((*ps).x58 as isize));
    (*ps).x58 += 1;
    (*ps).x4C = (*((&raw const (*ps).x2C) as *const f32).offset((*ps).x58 as isize) - scale) / (*ps).x44;
}

/// `it_80279E24`: advances the Pokemon's scale animation one frame and sets the next keyframe when due.
pub unsafe extern "C" fn it_80279E24(item_gobj: *mut HSD_GObj) {
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let item = (*item_gobj).user_data as *mut Item;
    let scale = HSD_JObjGetScaleX(item_jobj);
    let ps = &raw mut (*item).xDD4_itemVar.pokemon_spawn;

    if (*ps).x50 <= 0.0 {
        (*ps).x50 = (*ps).x44;
        if (*ps).x58 & 1 != 0 {
            if *((&raw const (*ps).x2C) as *const f32).offset((*ps).x58 as isize) <= scale {
                it_80279E24_inline(item_gobj);
            }
        } else if *((&raw const (*ps).x2C) as *const f32).offset((*ps).x58 as isize) >= scale {
            it_80279E24_inline(item_gobj);
        }
        if (*ps).x54 <= (*ps).x58 {
            (*ps).x5C = 2;
            return;
        }
    }
    (*ps).x50 -= 1.0;
    ext::it_80272DE4(item_jobj, (*ps).x4C);
}

/// `it_80279FF8`: grows the Pokemon to its attribute scale, then runs the scale animation.
pub unsafe extern "C" fn it_80279FF8(item_gobj: *mut HSD_GObj) {
    let item = (*item_gobj).user_data as *mut Item;
    if (*item).xDD4_itemVar.pokemon_spawn.x5C == 0 {
        let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
        if (*(*item).xCC_item_attr).x60_scale > (*item).xDD4_itemVar.pokemon_spawn.x48 + HSD_JObjGetScaleX(item_jobj) {
            ext::it_80272DE4(item_jobj, (*item).xDD4_itemVar.pokemon_spawn.x48);
            return;
        }
        ext::it_80272F7C(item_jobj, (*(*item).xCC_item_attr).x60_scale);
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
        ext::it_80272F7C((*item_gobj).hsd_obj as *mut HSD_JObj, (*(*item).xCC_item_attr).x60_scale);
        return true;
    }
    (*item).xDD4_itemVar.pokemon_spawn.x24 -= 1.0;
    false
}

/// `it_8027A118`: forwards the event to `it_8026E4D0`; never reports a removal.
pub unsafe extern "C" fn it_8027A118(item_gobj: *mut HSD_GObj, arg1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> bool {
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
        HSD_JObjGetTranslation(bone, &raw mut sp10);
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
        HSD_JObjSetTranslate(bone, &raw mut sp1c);
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

    if ext::HSD_Randi(251) == 0 && ext::Item_804A0E24.rare_spawned == 0 && ext::gm_80165084() != 0 {
        ext::Item_804A0E24.rare_spawned = 1;
        ext::gm_80172C04();
        return 23;
    }
    if ext::HSD_Randi(251) == 0 && ext::Item_804A0E24.rare_spawned == 0 && ext::gm_80164ABC() != 0 {
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
    if ext::HSD_Randi(251) == 0 && ext::Item_804A0E24.rare_spawned == 0 && ext::gm_80165084() != 0 {
        ext::Item_804A0E24.rare_spawned = 1;
        ext::gm_80172C04();
        return 23;
    }
    if ext::HSD_Randi(251) == 0 && ext::Item_804A0E24.rare_spawned == 0 && ext::gm_80164ABC() != 0 {
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
        *((&raw mut (*ps).x2C) as *mut f32).offset(i as isize) = (*attr).x1C[i as usize] * (*(*item2).xCC_item_attr).x60_scale;
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

/// `common_pokemon` (static): the kinds `it_8027AB64` draws from in the "common" mode; ends at `It_PKind_Terminate`.
static mut common_pokemon: [ItemKind; 21] = [
    It_PKind_Tosakinto, It_PKind_Chicorita, It_PKind_Kabigon, It_PKind_Kamex, It_PKind_Matadogas, It_PKind_Lizardon,
    It_PKind_Sonans, It_PKind_Hassam, It_PKind_Unknown, It_PKind_Kireihana, It_PKind_Marumine, It_PKind_Metamon,
    It_PKind_Pippi, It_PKind_Togepy, It_PKind_Hitodeman, It_PKind_Lucky, It_PKind_Porygon2, It_PKind_Hinoarashi,
    It_PKind_Maril, It_PKind_Fushigibana, It_PKind_Terminate,
];

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
    } else if ext::gm_8018841C() != 0 {
        spawn.kind = selectPokemonFromList(item, (&raw mut common_pokemon) as *mut ItemKind);
    } else if ext::gm_GetCurrentGameMode() as i32 == GM_OPENING_MV as i32 {
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
        ext::it_8026BB88(item_gobj, &raw mut spawn.pos);
        spawn.x3C_damage = 0;
        spawn.facing_dir = ext::it_8026B6A8(prev_pos, (*item2).owner);
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
    ext::efSync_Spawn(arg0, item_gobj, &[EfArg::Ptr(arg2 as *mut c_void), EfArg::F32P(&raw const sp1c)]);
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
    HSD_JObjSetRotation(item_jobj, &raw mut rotate);
    false
}

/// `it_8027AF50`: mirrors the velocity off the collision, re-derives the facing and rotates to face.
pub unsafe extern "C" fn it_8027AF50(item_gobj: *mut HSD_GObj) -> bool {
    let item = item_of(item_gobj);
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let mut rotate = it_803B8598;
    ext::lbVector_Mirror(&raw mut (*item).x40_vel, &raw mut (*item).xC58);
    ext::it_80272980(item_gobj);
    rotate.y = (IT_M_PI_2 * (*item).facing_dir as f64) as f32;
    HSD_JObjSetRotation(item_jobj, &raw mut rotate);
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
    if ext::ftLib_80086960((*spawn).x0_parent_gobj) != 0 {
        if ext::ftLib_80086960((*spawn).x4_parent_gobj2) != 0 {
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
        ext::ft_80089768(&raw mut (*spawn_item).xD94);
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
    if ext::ftLib_80086960((*item).owner) != 0 {
        let temp_r3 = ext::ft_800898A8((*item).owner);
        (*item).xD94 = (*temp_r3).x2074_vec;
        (*item).xD9C = (*temp_r3).x207C;
        (*item).xDA4_word = (*temp_r3).anon_10.x2084;
        return;
    }
    ext::ft_80089768(&raw mut (*item).xD94);
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
    if ext::ftLib_80086960((*item).owner) != 0 {
        let temp_r3 = ext::ft_800898A8((*item).owner);
        (*item).xD94 = (*temp_r3).x2074_vec;
        (*item).xD9C = (*temp_r3).x207C;
        (*item).xDA4_word = (*temp_r3).anon_10.x2084;
        return;
    }
    ext::ft_80089768(&raw mut (*item).xD94);
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
    if ext::it_8026B6C8(item_gobj) != 0 {
        let temp_r30 = (*ext::ft_80089884(fighter_gobj)).anon_0.x2073;
        let temp_r31 = ext::ftLib_800874BC(fighter_gobj);
        let temp_r3 = ext::ftLib_80086BE0(fighter_gobj) as u16;
        ext::pl_8003EB30(arg2, temp_r3 as i32, temp_r31, 6, 0, temp_r30 as i32);
    }
}

/// `it_8027B408`: as `it_8027B378` for the owner of `item_gobj1`.
pub unsafe extern "C" fn it_8027B408(item_gobj1: *mut HSD_GObj, item_gobj2: *mut HSD_GObj, arg8: f32) {
    if ext::it_8026B6C8(item_gobj2) != 0 {
        let item1 = item_of(item_gobj1);
        if ext::ftLib_80086960((*item1).owner) != 0 {
            let owner = (*item1).owner;
            let temp_r31 = ext::ftLib_800874BC(owner);
            let temp_r3 = ext::ftLib_80086BE0(owner) as u16;
            ext::pl_8003EB30(arg8, temp_r3 as i32, temp_r31, 6, 0, (*item1).xD90.anon_0.x2073 as i32);
        }
    }
}

/// `it_8027B4A4`: the fighter's player-state update for an item-class collision.
pub unsafe extern "C" fn it_8027B4A4(item_gobj1: *mut HSD_GObj, item_gobj2: *mut HSD_GObj) {
    if ext::it_8026B6C8(item_gobj2) != 0 {
        let temp_r31 = ext::ft_800898A8(item_gobj1) as *mut c_void;
        ext::pl_800384DC(item_gobj1, (*ext::ft_80089884(item_gobj1)).anon_0.x2073 as i32, temp_r31);
    }
}

/// `it_8027B508`: the owner's player-state update from the item's attack data.
pub unsafe extern "C" fn it_8027B508(item_gobj1: *mut HSD_GObj, item_gobj2: *mut HSD_GObj) {
    if ext::it_8026B6C8(item_gobj2) != 0 {
        let item1 = item_of(item_gobj1);
        if ext::ftLib_80086960((*item1).owner) != 0 {
            ext::pl_800384DC((*item1).owner, (*item1).xD90.anon_0.x2073 as i32, &raw mut (*item1).xD94 as *mut c_void);
        }
    }
}

/// `it_8027B564`: the owner's attack data pushed to the player state.
pub unsafe extern "C" fn it_8027B564(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    if ext::ftLib_80086960((*item).owner) != 0 {
        let mut sp10 = (*item).xD90;
        ext::pl_80037DF4((*item).owner, &raw mut sp10);
    }
}
