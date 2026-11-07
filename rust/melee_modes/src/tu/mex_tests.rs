//! pc/tests/mex_tests.c (with mex_onfloat_tests.inc): the game-side m-ex tests. Each test returns 0 on
//! success or 1 after calling `ext::TestFail(msg)`, exactly like the C. The PC layer (Mex_* dispatch,
//! script API, player table, PSMTXConcat, the test registry) is unported here: those callees are
//! TODO stubs in `mod ext`, named like the C functions.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs, unused_unsafe)]

use melee_core::fp::Fmadd as _;
use core::ffi::{c_char, c_void};
use melee_types::*;

pub mod ext {
    use super::*;
    macro_rules! todo_ext {
        ($(fn $n:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub unsafe extern "C" fn $n($($a: $t),*) $(-> $r)? {
                let _ = ($(&$a),*);
                todo!(concat!("TODO: ", stringify!($n), " is not ported"))
            }
        )*};
    }
    todo_ext! {
        fn TestRegister(name: *const c_char, f: Option<unsafe extern "C" fn() -> i32>);
        fn TestFail(msg: *const c_char);
        fn Mex_HookRegister(event: i32, kind: i32, f: Option<unsafe extern "C" fn(*mut c_void)>) -> i32;
        fn Mex_OnFrameDispatch(kind: i32, gobj: *mut c_void, vanilla: *mut c_void);
        fn Mex_PredicateRegister(event: i32, kind: i32, f: Option<unsafe extern "C" fn(*mut c_void) -> i32>) -> i32;
        fn Mex_GObjPredDispatch(event: i32, kind: i32, gobj: *mut c_void, vanilla: *mut c_void) -> i32;
        fn ScriptGame_FighterI(slot: i32, field: i32) -> i32;
        fn ScriptGame_FighterF(slot: i32, field: i32) -> f32;
        fn Player_ForgetEntities();
        fn PSMTXConcat(a: *mut [f32; 4], b: *mut [f32; 4], ab: *mut [f32; 4]);
        fn ftPe_8011BA54(gobj: *mut HSD_GObj) -> bool;
        fn ftPe_8011BAD8(gobj: *mut HSD_GObj) -> bool;
        fn lbHeap_803BA380_ptr() -> *mut MexHeapDesc;
        fn player_slots_ptr() -> *mut StaticPlayer;
        fn HSD_GObjPLinkHead_get() -> *mut *mut HSD_GObj;
        fn HSD_GObjPLinkHead_set(v: *mut *mut HSD_GObj);
        fn p_ftCommonData_get() -> *mut ftCommonData;
        fn p_ftCommonData_set(v: *mut ftCommonData);
    }
}

/// `struct mex_heap_desc` (mirror of lbHeap_HeapDesc in lbheap.c, which has no header).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MexHeapDesc {
    pub idx: u32,
    pub type_: u32,
    pub prev_idx: u32,
    pub size: u32,
}

const MEX_SPECIAL_COUNT: i32 = 7;
const MEX_EVENT_ON_FRAME: i32 = 3;
const MEX_EVENT_ON_FLOAT: i32 = 36;
const T_SI_PRESENT: i32 = 0;
const T_SI_KIND: i32 = 1;
const T_SI_ACTION: i32 = 3;
const T_SF_X: i32 = 0;

unsafe fn fail(msg: &'static str) -> i32 {
    ext::TestFail(msg.as_ptr() as *const c_char);
    1
}

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0")
    };
}

pub unsafe extern "C" fn test_external_special_range() -> i32 {
    melee_core::ct!("test_external_special_range");
    if ChKind_Max as i32 != 0x21
        || ChKind_Mex0 as i32 != 0x22
        || Ft_Kind_Mex0 as i32 != 0x21
        || Ft_Kind_Max as i32 > 127
        || ChKind_Cap as i32 > 128
        || Ft_Kind_Max as i32 - Ft_Kind_Mex0 as i32 != ChKind_Cap as i32 - ChKind_Mex0 as i32
    {
        return fail(cstr!("kind layout: expected ChKind_Max 0x21, ChKind_Mex0 0x22, Ft_Kind_Mex0 0x21, Ft_Kind_Max <= 127, ChKind_Cap <= 128, as many m-ex FighterKinds as CharacterKinds"));
    }
    if ChKind_Max as i32 - CKind_Playable_Count as i32 != MEX_SPECIAL_COUNT {
        return fail(cstr!("external special range is not 7 wide"));
    }
    0
}

pub unsafe extern "C" fn test_special_id_remap_is_identity() -> i32 {
    melee_core::ct!("test_special_id_remap_is_identity");
    let start = ChKind_Max as i32 - MEX_SPECIAL_COUNT;
    let mut e = CKind_Playable_Count as i32;
    while e < ChKind_Max as i32 {
        let remapped = start + (e - CKind_Playable_Count as i32);
        if remapped != e {
            return fail(cstr!("special remap is not identity under vanilla counts"));
        }
        e += 1;
    }
    0
}

pub unsafe extern "C" fn test_heap_table_shape() -> i32 {
    melee_core::ct!("test_heap_table_shape");
    let t = ext::lbHeap_803BA380_ptr();
    if (*t.add(4)).idx != 6 {
        return fail(cstr!("heap table slot 4 should be the added heap id 6"));
    }
    if (*t.add(4)).size != 0x20 {
        return fail(cstr!("added heap 6 size should be 0x20"));
    }
    if (*t.add(5)).idx != 7 {
        return fail(cstr!("heap descriptor terminator should be 7 on TARGET_PC"));
    }
    0
}

pub unsafe extern "C" fn test_1p_mode_classification() -> i32 {
    melee_core::ct!("test_1p_mode_classification");
    use crate::gm::gm_Is1PMode;
    if !gm_Is1PMode(GM_CLASSIC as u8) {
        return fail(cstr!("gm_Is1PMode(CLASSIC) should be true"));
    }
    if !gm_Is1PMode(GM_ADVENTURE as u8) {
        return fail(cstr!("gm_Is1PMode(ADVENTURE) should be true"));
    }
    if !gm_Is1PMode(GM_TRAINING as u8) {
        return fail(cstr!("gm_Is1PMode(TRAINING) should be true"));
    }
    if !gm_Is1PMode(GM_TARGET_TEST as u8) {
        return fail(cstr!("gm_Is1PMode(TARGET_TEST) should be true"));
    }
    0
}

pub unsafe extern "C" fn test_vs_mode_is_not_1p() -> i32 {
    melee_core::ct!("test_vs_mode_is_not_1p");
    use crate::gm::gm_Is1PMode;
    if gm_Is1PMode(GM_VS as u8) {
        return fail(cstr!("gm_Is1PMode(VS) should be false"));
    }
    if gm_Is1PMode(GM_TOURNAMENT as u8) {
        return fail(cstr!("gm_Is1PMode(TOURNAMENT) should be false"));
    }
    0
}

static mut mex_onframe_hook_calls: i32 = 0;
static mut mex_onframe_vanilla_calls: i32 = 0;

unsafe extern "C" fn mex_onframe_hook(_gobj: *mut c_void) {
    mex_onframe_hook_calls += 1;
}
unsafe extern "C" fn mex_onframe_vanilla(_gobj: *mut c_void) {
    mex_onframe_vanilla_calls += 1;
}

pub unsafe extern "C" fn test_mex_onframe_hook_register_and_clear() -> i32 {
    melee_core::ct!("test_mex_onframe_hook_register_and_clear");
    let van = mex_onframe_vanilla as unsafe extern "C" fn(*mut c_void) as *mut c_void;
    let fox = Ft_Kind_Fox as i32;
    if ext::Mex_HookRegister(999, 0, Some(mex_onframe_hook)) != 0 {
        return fail(cstr!("Mex_HookRegister should reject an out-of-range event"));
    }
    mex_onframe_hook_calls = 0;
    mex_onframe_vanilla_calls = 0;
    ext::Mex_OnFrameDispatch(fox, core::ptr::null_mut(), van);
    if mex_onframe_hook_calls != 0 || mex_onframe_vanilla_calls != 1 {
        return fail(cstr!("OnFrame dispatch without an override should run the vanilla entry"));
    }
    if ext::Mex_HookRegister(MEX_EVENT_ON_FRAME, fox, Some(mex_onframe_hook)) != 1 {
        return fail(cstr!("Mex_HookRegister rejected a valid (event, kind)"));
    }
    mex_onframe_hook_calls = 0;
    mex_onframe_vanilla_calls = 0;
    ext::Mex_OnFrameDispatch(fox, core::ptr::null_mut(), van);
    if mex_onframe_hook_calls != 1 || mex_onframe_vanilla_calls != 0 {
        return fail(cstr!("registered OnFrame hook should run instead of the vanilla entry"));
    }
    if ext::Mex_HookRegister(MEX_EVENT_ON_FRAME, fox, None) != 1 {
        return fail(cstr!("Mex_HookRegister should accept a NULL fn to clear the slot"));
    }
    mex_onframe_hook_calls = 0;
    mex_onframe_vanilla_calls = 0;
    ext::Mex_OnFrameDispatch(fox, core::ptr::null_mut(), van);
    if mex_onframe_hook_calls != 0 || mex_onframe_vanilla_calls != 1 {
        return fail(cstr!("clearing the OnFrame slot should restore the vanilla entry"));
    }
    0
}

unsafe extern "C" fn mex_pred_hook(_gobj: *mut c_void) -> i32 {
    1
}
unsafe extern "C" fn mex_pred_vanilla(_gobj: *mut c_void) -> i32 {
    2
}

pub unsafe extern "C" fn test_mex_predicate_register_and_clear() -> i32 {
    melee_core::ct!("test_mex_predicate_register_and_clear");
    let van = mex_pred_vanilla as unsafe extern "C" fn(*mut c_void) -> i32 as *mut c_void;
    let nul = core::ptr::null_mut::<c_void>();
    let fox = Ft_Kind_Fox as i32;
    if ext::Mex_PredicateRegister(999, 0, Some(mex_pred_hook)) != 0 {
        return fail(cstr!("Mex_PredicateRegister should reject an out-of-range event"));
    }
    if ext::Mex_GObjPredDispatch(MEX_EVENT_ON_FRAME, fox, nul, nul) != 0 {
        return fail(cstr!("predicate dispatch with neither override nor vanilla should return 0"));
    }
    if ext::Mex_GObjPredDispatch(MEX_EVENT_ON_FRAME, fox, nul, van) != 2 {
        return fail(cstr!("predicate dispatch without an override should return the vanilla result"));
    }
    if ext::Mex_PredicateRegister(MEX_EVENT_ON_FRAME, fox, Some(mex_pred_hook)) != 1 {
        return fail(cstr!("Mex_PredicateRegister rejected a valid (event, kind)"));
    }
    if ext::Mex_GObjPredDispatch(MEX_EVENT_ON_FRAME, fox, nul, van) != 1 {
        return fail(cstr!("registered predicate should return its own result, not the vanilla one"));
    }
    if ext::Mex_PredicateRegister(MEX_EVENT_ON_FRAME, fox, None) != 1 {
        return fail(cstr!("Mex_PredicateRegister should accept NULL to clear the slot"));
    }
    if ext::Mex_GObjPredDispatch(MEX_EVENT_ON_FRAME, fox, nul, van) != 2 {
        return fail(cstr!("clearing the predicate slot should restore the vanilla result"));
    }
    0
}

static mut t_fighter_gobj: HSD_GObj = unsafe { core::mem::zeroed() };
static mut t_fighter: Fighter = unsafe { core::mem::zeroed() };
static mut t_plinks: [*mut HSD_GObj; HSD_GOBJ_PLINK_MAX as usize + 1] =
    [core::ptr::null_mut(); HSD_GOBJ_PLINK_MAX as usize + 1];

pub unsafe extern "C" fn test_script_stale_fighter_reads_absent() -> i32 {
    melee_core::ct!("test_script_stale_fighter_reads_absent");
    let s5 = ext::player_slots_ptr().add(5);
    let saved = *s5;
    let mut rc = 0;
    t_fighter_gobj.next = core::ptr::null_mut();
    t_fighter_gobj.user_data = 0x8B8B8B8Bu32 as usize as *mut c_void;
    (*s5).transformed[0] = 0;
    (*s5).player_entity[0] = &raw mut t_fighter_gobj;
    if ext::ScriptGame_FighterI(5, T_SI_PRESENT) != 0
        || ext::ScriptGame_FighterI(5, T_SI_ACTION) != -1
        || ext::ScriptGame_FighterF(5, T_SF_X) != 0.0
    {
        fail(cstr!("a slot whose fighter is not in the live fighter list must read as absent"));
        rc = 1;
    }
    *s5 = saved;
    rc
}

pub unsafe extern "C" fn test_script_live_fighter_reads() -> i32 {
    melee_core::ct!("test_script_live_fighter_reads");
    let s5 = ext::player_slots_ptr().add(5);
    let saved = *s5;
    let saved_heads = ext::HSD_GObjPLinkHead_get();
    let mut rc = 0;
    if saved_heads.is_null() {
        ext::HSD_GObjPLinkHead_set(t_plinks.as_mut_ptr());
    }
    let heads = ext::HSD_GObjPLinkHead_get();
    let saved_first = *heads.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    t_fighter.kind = Ft_Kind_Falco as _;
    t_fighter.cur_pos.x = 12.5;
    t_fighter_gobj.next = core::ptr::null_mut();
    t_fighter_gobj.user_data = (&raw mut t_fighter) as *mut c_void;
    *heads.add(HSD_GOBJ_PLINK_FIGHTER as usize) = &raw mut t_fighter_gobj;
    (*s5).transformed[0] = 0;
    (*s5).player_entity[0] = &raw mut t_fighter_gobj;
    if ext::ScriptGame_FighterI(5, T_SI_PRESENT) != 1
        || ext::ScriptGame_FighterI(5, T_SI_KIND) != Ft_Kind_Falco as i32
        || ext::ScriptGame_FighterF(5, T_SF_X) != 12.5
    {
        fail(cstr!("a live fighter must still read through the script API"));
        rc = 1;
    }
    *heads.add(HSD_GOBJ_PLINK_FIGHTER as usize) = saved_first;
    ext::HSD_GObjPLinkHead_set(saved_heads);
    *s5 = saved;
    rc
}

pub unsafe extern "C" fn test_scene_start_forgets_player_entities() -> i32 {
    melee_core::ct!("test_scene_start_forgets_player_entities");
    let slots = ext::player_slots_ptr();
    let mut saved: [StaticPlayer; 6] = core::mem::zeroed();
    let mut rc = 0;
    for slot in 0..6 {
        saved[slot] = *slots.add(slot);
        for i in 0..PL_MAX_SUB_FIGHTERS as usize {
            (*slots.add(slot)).player_entity[i] = &raw mut t_fighter_gobj;
        }
    }
    ext::Player_ForgetEntities();
    for slot in 0..6 {
        for i in 0..PL_MAX_SUB_FIGHTERS as usize {
            if !(*slots.add(slot)).player_entity[i].is_null() {
                rc = 1;
            }
        }
        *slots.add(slot) = saved[slot];
    }
    if rc != 0 {
        fail(cstr!("Player_ForgetEntities left a fighter pointer in the player table"));
    }
    rc
}

static mut fma_rng: u32 = 0x9E37_79B9;

fn fma_rand_float() -> f32 {
    unsafe {
        let mut v;
        loop {
            fma_rng = fma_rng.wrapping_mul(1664525).wrapping_add(1013904223);
            v = fma_rng ^ (fma_rng >> 7).wrapping_mul(2654435761);
            if v & 0x7F80_0000 != 0x7F80_0000 {
                break;
            }
        }
        if (fma_rng >> 3) & 1 != 0 {
            v = (v & 0x807F_FFFF) | ((((v >> 23) & 0x1F) + 112) << 23);
        }
        f32::from_bits(v)
    }
}

/// `(float)((double)a * (double)c + (double)b)` contracted to one fma.
fn fma_ref_madd(a: f32, c: f32, b: f32) -> f32 {
    (a as f64).fmadd(c as f64, b as f64) as f32
}

/// The split form: a double multiply (exact for float operands), then a double add.
fn fma_split_madd(a: f32, c: f32, b: f32) -> f32 {
    let p = a as f64 * c as f64;
    (p + b as f64) as f32
}

pub unsafe extern "C" fn test_mtx_madd_matches_fma() -> i32 {
    melee_core::ct!("test_mtx_madd_matches_fma");
    for _ in 0..200000 {
        let a = fma_rand_float();
        let c = fma_rand_float();
        let b = fma_rand_float();
        if fma_split_madd(a, c, b).to_bits() != fma_ref_madd(a, c, b).to_bits() {
            return fail(cstr!("double mul+add differs from fma() on float operands"));
        }
    }
    for _ in 0..5000 {
        let mut a: Mtx = [[0.0; 4]; 3];
        let mut b: Mtx = [[0.0; 4]; 3];
        let mut r: Mtx = [[0.0; 4]; 3];
        for i in 0..3 {
            for j in 0..4 {
                a[i][j] = fma_rand_float();
                b[i][j] = fma_rand_float();
            }
        }
        ext::PSMTXConcat(a.as_mut_ptr(), b.as_mut_ptr(), r.as_mut_ptr());
        for i in 0..3 {
            for j in 0..4 {
                let mut t = b[0][j] * a[i][0];
                t = fma_ref_madd(b[1][j], a[i][1], t);
                t = fma_ref_madd(b[2][j], a[i][2], t);
                if j == 3 {
                    t = fma_ref_madd(1.0, a[i][3], t);
                } else if j == 2 {
                    t = fma_ref_madd(0.0, a[i][3], t);
                }
                if r[i][j].to_bits() != t.to_bits() {
                    return fail(cstr!("PSMTXConcat differs from its fma reference"));
                }
            }
        }
    }
    0
}

// ---- mex_onfloat_tests.inc ----

static mut mex_float_calls: i32 = 0;
static mut mex_float_result: i32 = 0;
static mut mex_float_gobj: *mut c_void = core::ptr::null_mut();

unsafe extern "C" fn mex_float_predicate(gobj: *mut c_void) -> i32 {
    mex_float_calls += 1;
    mex_float_gobj = gobj;
    mex_float_result
}

pub unsafe extern "C" fn test_mex_onfloat_inputs() -> i32 {
    melee_core::ct!("test_mex_onfloat_inputs");
    static mut fp: Fighter = unsafe { core::mem::zeroed() };
    static mut gobj: HSD_GObj = unsafe { core::mem::zeroed() };
    static mut common: ftCommonData = unsafe { core::mem::zeroed() };
    let saved_common = ext::p_ftCommonData_get();
    let mut failed = 0;

    fp = core::mem::zeroed();
    gobj = core::mem::zeroed();
    common = core::mem::zeroed();
    common.x88 = 0.7;
    common.tap_jump_threshold = 0.7;
    ext::p_ftCommonData_set(&raw mut common);
    gobj.user_data = (&raw mut fp) as *mut c_void;
    fp.kind = Ft_Kind_Mex0 as _;
    mex_float_calls = 0;
    mex_float_result = 1;
    mex_float_gobj = core::ptr::null_mut();
    let g = &raw mut gobj;

    macro_rules! check {
        ($c:expr, $m:expr) => {
            if !($c) {
                fail(cstr!($m));
                failed = 1;
                break;
            }
        };
    }
    loop {
        check!(
            ext::Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, fp.kind as i32, Some(mex_float_predicate)) == 1,
            "onFloat predicate registration rejected"
        );
        // Down+XY: both inputs are required; an ascending fighter may float.
        fp.input.lstick[0].y = -common.x88;
        check!(!ext::ftPe_8011BA54(g) && mex_float_calls == 0, "onFloat ran without X/Y held");
        fp.input.held_buttons[0] = HSD_PAD_X as _;
        fp.input.lstick[0].y = 0.0;
        check!(!ext::ftPe_8011BA54(g) && mex_float_calls == 0, "onFloat ran without stick down");
        fp.input.lstick[0].y = -common.x88;
        fp.self_vel.y = 1.0;
        check!(
            ext::ftPe_8011BA54(g) && mex_float_calls == 1 && mex_float_gobj == g as *mut c_void,
            "Down+XY did not return the onFloat result for the same GObj"
        );
        mex_float_result = 0;
        check!(!ext::ftPe_8011BA54(g) && mex_float_calls == 2, "Down+XY treated callback presence as success");

        // JumpPeak: descending/zero velocity and stick up OR X/Y, not AND.
        mex_float_result = 1;
        fp.input.lstick[0].y = 0.0;
        check!(!ext::ftPe_8011BAD8(g) && mex_float_calls == 2, "JumpPeak called onFloat while ascending");
        fp.self_vel.y = 0.0;
        fp.input.held_buttons[0] = 0;
        check!(!ext::ftPe_8011BAD8(g) && mex_float_calls == 2, "JumpPeak called onFloat without jump input");
        fp.input.lstick[0].y = common.tap_jump_threshold;
        check!(ext::ftPe_8011BAD8(g) && mex_float_calls == 3, "JumpPeak did not float at zero velocity with stick up");
        fp.self_vel.y = -1.0;
        fp.input.lstick[0].y = 0.0;
        fp.input.held_buttons[0] = HSD_PAD_Y as _;
        check!(ext::ftPe_8011BAD8(g) && mex_float_calls == 4, "JumpPeak did not float while falling with Y held");
        mex_float_result = 0;
        check!(!ext::ftPe_8011BAD8(g) && mex_float_calls == 5, "JumpPeak discarded the callback false result");

        // Another kind has no hook; Peach keeps her retail has_float gate.
        fp.kind = (Ft_Kind_Mex0 + 1) as _;
        fp.input.lstick[0].y = -common.x88;
        check!(
            !ext::ftPe_8011BA54(g) && !ext::ftPe_8011BAD8(g) && mex_float_calls == 5,
            "onFloat leaked into a kind with no override"
        );
        ext::Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, Ft_Kind_Peach as i32, Some(mex_float_predicate));
        fp.kind = Ft_Kind_Peach as _;
        fp.u.pe.has_float = 0;
        check!(
            !ext::ftPe_8011BA54(g) && !ext::ftPe_8011BAD8(g) && mex_float_calls == 5,
            "Peach bypassed the retail has_float gate"
        );

        fp.kind = Ft_Kind_Mex0 as _;
        ext::Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, fp.kind as i32, None);
        check!(
            !ext::ftPe_8011BA54(g) && !ext::ftPe_8011BAD8(g) && mex_float_calls == 5,
            "cleared onFloat hook still fired"
        );
        break;
    }
    ext::Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, Ft_Kind_Mex0 as i32, None);
    ext::Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, Ft_Kind_Peach as i32, None);
    ext::p_ftCommonData_set(saved_common);
    failed
}

/// `MexTestRegisterAll`.
pub unsafe extern "C" fn MexTestRegisterAll() {
    melee_core::ct!("MexTestRegisterAll");
    macro_rules! reg {
        ($n:literal, $f:ident) => {
            ext::TestRegister(cstr!($n).as_ptr() as *const c_char, Some($f));
        };
    }
    reg!("mex_onfloat_inputs", test_mex_onfloat_inputs);
    reg!("gm_Is1PMode_1p_modes", test_1p_mode_classification);
    reg!("gm_Is1PMode_vs_modes", test_vs_mode_is_not_1p);
    reg!("heap_table_shape", test_heap_table_shape);
    reg!("charid_special_range_is_7", test_external_special_range);
    reg!("charid_remap_identity_vanilla", test_special_id_remap_is_identity);
    reg!("mex_onframe_hook_register_and_clear", test_mex_onframe_hook_register_and_clear);
    reg!("mex_predicate_register_and_clear", test_mex_predicate_register_and_clear);
    reg!("script_stale_fighter_reads_absent", test_script_stale_fighter_reads_absent);
    reg!("script_live_fighter_reads", test_script_live_fighter_reads);
    reg!("scene_start_forgets_player_entities", test_scene_start_forgets_player_entities);
    reg!("mtx_madd_matches_fma", test_mtx_madd_matches_fma);
}
