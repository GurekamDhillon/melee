//! An m-ex fighter's `ftFunction` PPC code run in the match: the native form of the C build's
//! `pc/platform/gw_mex_ftfunction_runtime.c` interpreter wiring (gw_mex_interp_*, the MoveLogic trampolines).
//!
//! The blob runs through mex_import's interpreter (`ppc::Machine`, PowerPC-exact float mode: fused multiply-add,
//! Gekko's fctiw/fctiwz) over `remap::GuestSpace`, which gives the native little-endian `Fighter` / `HSD_GObj` guest
//! addresses and rewrites every field access through the GameCube offset tables, so the mod writes the very objects
//! the native match simulates. Guest memory:
//!
//! * the relocated blob (code and its own data) at `CODE_BASE` (+ per fighter), within rel24 reach of the game's text
//!   so a `bl` to a vanilla function decodes to that function;
//! * the fighter's PlXx.dat data section as HSD_ArchiveParse leaves it (big-endian, relocated) at `DAT_BASE` (+ per
//!   fighter); the guest's `fp->ft_data` reads that ftData (its attributes and article tables are the mod's own
//!   layouts, which only the guest reads);
//! * `fp->dat_attrs_backup`, the 0x424-byte buffer of the game heap, as raw guest bytes: the mod's onLoad copies its
//!   private attributes there and points `fp->dat_attrs` at it, as m-ex does;
//! * r13 / r2 small data: the DOL's .sdata / .sdata2 around the two bases (the float constants the code loads).
//!
//! Rollback: every byte the guest can write lives in the game heap, in a native game object, or in this module's
//! statics and the arena-allocated regions they own, all of which the snapshot covers. The machine itself is made per
//! call. Nothing here reads the clock or iterates a hash map.
//!
//! Out-of-blob calls go through [`call_out`]: memset/memcpy, MEX_IndexFighterItem, and the vanilla functions of
//! `mex_bridge` (generated: arguments from r3.. / f1.., pointers translated between guest handles and native objects,
//! small records the guest keeps on its stack marshalled in and out). Any other target ends that callback and is
//! counted by name (`MELEE_MEX_REPORT=1` prints the counters at exit of each fighter's first faults).
#![allow(static_mut_refs)]

use core::ffi::c_void;
use mex_import::mexfn::{Blob, Owner};
use mex_import::ppc::{Bridge, Cpu, Fault, Machine, Mem, Step};
use mex_import::remap::GuestSpace;
use melee_types::{Fighter, HSD_GObj, ItemLogicTable, ItemStateTable, MotionState};
use std::collections::BTreeMap;

type G = *mut HSD_GObj;

pub const SLOT_NAMES: [&str; 46] = [
    "onLoad", "OnRespawn", "onDestroy", "MoveLogic", "SpecialN", "SpecialNAir", "SpecialS", "SpecialSAir", "SpecialHi",
    "SpecialHiAir", "SpecialLw", "SpecialLwAir", "onAbsorb", "OnItemPickup", "onMakeItemInvisible", "onMakeItemVisible",
    "OnItemRelease", "OnItemPickup2", "onUnknownItemRelated", "onApplyHeadItem", "onRemoveHeadItem", "EyeTextureDamaged",
    "EyeTextureNormal", "onFrame", "onActionStateChange", "onReapplyAttr", "onModelRender", "onShadowRender",
    "onUnknownMultijump", "onActionStateChangeWhileEyeTextureIsChanged", "onTwoEntryTable", "onFloat", "onDoubleJump",
    "onZair", "onLanding", "onFSmash", "onUSmash", "onDSmash", "onGetExtResultAnim", "onIndexExtResultAnim",
    "MoveLogicDemo", "onIntroL", "onIntroR", "onTaunt", "onCatch", "GetTrailData",
];
const SLOT_MOVE_LOGIC: u32 = 3;

/// (m-ex ftFunction slot, the engine's one-argument dispatch event): gw_mex_ftfunction_runtime.c's gw_mex_hook rows.
const SLOT_EVENTS: [(u32, usize); 22] = {
    use melee_fighter::platform as p;
    [
        (0, p::MEX_EVENT_ON_LOAD),
        (1, p::MEX_EVENT_ON_DEATH),
        (2, p::MEX_EVENT_ON_DESTROY),
        (23, p::MEX_EVENT_ON_FRAME),
        (12, p::MEX_EVENT_ON_ABSORB),
        (24, p::MEX_EVENT_ON_ACTION_STATE_CHANGE),
        (25, p::MEX_EVENT_ON_REAPPLY_ATTR),
        (4, p::MEX_EVENT_SPECIAL_N),
        (5, p::MEX_EVENT_SPECIAL_N_AIR),
        (6, p::MEX_EVENT_SPECIAL_S),
        (7, p::MEX_EVENT_SPECIAL_S_AIR),
        (8, p::MEX_EVENT_SPECIAL_HI),
        (9, p::MEX_EVENT_SPECIAL_HI_AIR),
        (10, p::MEX_EVENT_SPECIAL_LW),
        (11, p::MEX_EVENT_SPECIAL_LW_AIR),
        (32, p::MEX_EVENT_ON_DOUBLE_JUMP),
        (34, p::MEX_EVENT_ON_LANDING),
        (35, p::MEX_EVENT_ON_FSMASH),
        (36, p::MEX_EVENT_ON_USMASH),
        (37, p::MEX_EVENT_ON_DSMASH),
        (43, p::MEX_EVENT_ON_TAUNT),
        (44, p::MEX_EVENT_ON_CATCH),
    ]
};

const CODE_BASE: u32 = 0x8180_0000;
const CODE_END: u32 = 0x8200_0000;
const DAT_BASE: u32 = 0x8800_0000;
const STACK_BASE: u32 = 0x8170_0000;
const STACK_SIZE: u32 = 0x2_0000;
/// fighter.c: HSD_ObjAllocInit(&fighter_dat_attrs_alloc_data, 0x424, 4).
const ATTRS_SIZE: u32 = 0x424;
const ATTRS_BASE: u32 = 0x8600_0000; // + player-free index * 0x1000
const SCRATCH_BASE: u32 = 0x8700_0000; // marshalled records of one call
/// NTSC 1.02 `_SDA_BASE_` (r13) and `_SDA2_BASE_` (r2).
const R13: u32 = 0x804D_B6A0;
const R2: u32 = 0x804D_F9E0;
const BUDGET: u64 = 4_000_000;
/// Fighter GC offsets the runtime itself reads (fighter.h): motion_id, the MotionState table base index (x18),
/// ft_data, dat_attrs_backup.
const FT_MOTION_ID: u32 = 0x10;
const FT_X18: u32 = 0x18;
const FT_FT_DATA: u32 = 0x10C;
const FT_DAT_ATTRS_BACKUP: u32 = 0x2D8;

const MEMSET: u32 = 0x8000_3100;
const MEMCPY: u32 = 0x8000_31F4;
/// gw_mex_ftfunction_runtime.c GW_MEX_GUEST_INDEX_ITEM (MEX_IndexFighterItem).
const MEX_INDEX_FIGHTER_ITEM: u32 = 0x803D_7058;
/// GW_MEX_GUEST_GET_FT_ITEM_ID (MEX_GetFtItemID).
const MEX_GET_FT_ITEM_ID: u32 = 0x803D_7088;
const EF_SYNC_SPAWN: u32 = 0x8005_FDDC;
const HSD_MEM_ALLOC: u32 = 0x8037_F1E4;
/// GW_MEX_GUEST_GET_DATA (MEX_GetData).
const MEX_GET_DATA: u32 = 0x803D_7094;
const HSD_FREE: u32 = 0x8037_F1B0;
/// GW_MEX_GUEST_CALLOC (m-ex calloc(size)).
const MEX_CALLOC: u32 = 0x803D_706C;
/// Guest addresses of the game-heap blocks the guest allocates (HSD_MemAlloc).
const HEAP_GUEST: u32 = 0x8A00_0000;

#[derive(Default, Debug, Clone)]
pub struct SlotStats {
    pub calls: u64,
    pub returned: u64,
    pub insns: u64,
    pub faults: BTreeMap<String, u64>,
}

struct Fighter_ {
    file: String,
    blob: Blob,
    code_base: u32,
    ft_data: u32,
    /// slot -> stats; MoveLogic callbacks count under 1000 + which.
    stats: BTreeMap<u32, SlotStats>,
    /// The native MotionState table built from the blob's MoveLogic table (kept alive here).
    move_table: Vec<MotionState>,
    /// Guest callbacks of each MoveLogic row: [anim, input, phys, coll].
    move_guest: Vec<[u32; 4]>,
}

struct State {
    space: GuestSpace,
    kinds: BTreeMap<i32, Fighter_>,
    code_next: u32,
    dat_next: u32,
    /// Out-of-blob calls by target: (bound, unresolved).
    bound: BTreeMap<u32, u64>,
    unresolved: BTreeMap<u32, u64>,
    /// MEX_IndexFighterItem: (fighter kind, item id) -> ItemDesc guest address.
    articles: BTreeMap<(u32, u32), u32>,
    /// Pool trampolines: slot -> (kind, guest address).
    pool: Vec<(i32, u32)>,
    /// dat_attrs_backup buffers given a guest address: native pointer -> guest base.
    attrs: Vec<(usize, u32)>,
    small_data: bool,
    depth: u32,
    heap_next: u32,
    /// Native retail callback -> its vanilla address (fn_guest).
    retail: BTreeMap<usize, u32>,
    /// The fighters' item articles (itFunction): relocated code of each, owner fighter kind.
    arts: Vec<Art>,
    /// m-ex custom item kinds (>= 237): the native descriptor and logic table Item_80267978 takes.
    items: BTreeMap<u32, CustomItem>,
    /// Each fighter's PlXx.dat as the native decoder reads it, and its guest base (the ItemDescs
    /// MEX_IndexFighterItem names).
    natdat: BTreeMap<i32, (Box<melee_runtime::dat::DatArchive>, u32)>,
    plinks: bool,
}

struct Art {
    kind: i32,
    base: u32,
    blob: Blob,
}

/// One custom item kind: item.RuntimeIndex[kind - 237] (the ItemDesc the owner's onLoad hands MEX_IndexFighterItem,
/// decoded natively) and item.Custom[kind - 237] (the logic table the article's function relocs fill).
struct CustomItem {
    /// Owner fighter kind and the guest address of its ItemDesc (in the fighter's dat).
    desc_guest: u32,
    desc: *mut melee_types::Article,
    logic: Box<ItemLogicTable>,
    states: Vec<ItemStateTable>,
}

static mut ST: Option<State> = None;

/// The debug switches, read once (they are looked up on every call out of the blob).
static REPORT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static CALLTRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
fn flag(c: &std::sync::OnceLock<bool>, name: &str) -> bool {
    *c.get_or_init(|| std::env::var_os(name).is_some())
}

fn st() -> &'static mut State {
    unsafe {
        ST.get_or_insert_with(|| {
            let mut space = GuestSpace::new();
            space.add_bytes(STACK_BASE, vec![0; STACK_SIZE as usize]);
            space.fn_native = Some(fn_native);
            space.fn_guest = Some(fn_guest);
            space.void_pointee = Some(void_pointee);
            space.ptr_native = Some(ps_userfunc);
            if std::env::var_os("MELEE_MEX_TRACE").is_some() {
                space.trace = Some(Vec::new());
            }
            State {
                space,
                kinds: BTreeMap::new(),
                code_next: CODE_BASE,
                dat_next: DAT_BASE,
                bound: BTreeMap::new(),
                unresolved: BTreeMap::new(),
                articles: BTreeMap::new(),
                pool: Vec::new(),
                attrs: Vec::new(),
                small_data: false,
                depth: 0,
                heap_next: HEAP_GUEST,
                retail: BTreeMap::new(),
                arts: Vec::new(),
                items: BTreeMap::new(),
                natdat: BTreeMap::new(),
                plinks: false,
            }
        })
    }
}

include!("mex_guest_pool.rs");

/// What an `HSD_GObj`'s untyped fields point to: user_data by the gobj's class (a fighter, an item), hsd_obj its
/// root joint (fighters, items and the effects the guest reaches through EF_Effect.x4).
fn void_pointee(rec: &'static str, off: u32, obj: *mut u8, _p: usize) -> Option<&'static str> {
    if rec == "HSD_JObj" && off == 0x18 {
        // jobj->u: the member HSD_JObj's flags select (jobj.h: JOBJ_PTCL -> ptcl, JOBJ_SPLINE -> spline, else dobj);
        // the layout's first member is ptcl, so Sonic's Color_Shoes walking jobj->u.dobj->next read SList.data
        let f = unsafe { (*(obj as *mut melee_types::HSD_JObj)).flags };
        return Some(if f & (1 << 5) != 0 {
            "_HSD_SList"
        } else if f & (1 << 14) != 0 {
            "HSD_Spline"
        } else {
            "HSD_DObj"
        });
    }
    if rec != "HSD_GObj" {
        return None;
    }
    let g = obj as G;
    match off {
        0x28 => Some("HSD_JObj"),
        0x2C => match unsafe { (*g).classifier } as i32 {
            melee_types::HSD_GOBJ_CLASS_FIGHTER => Some("Fighter"),
            melee_types::HSD_GOBJ_CLASS_ITEM => Some("Item"),
            _ => None,
        },
        _ => None,
    }
}

/// A pool slot's guest code with r3 = the gobj; its r3 (an item state's / logic table's predicates return it, an
/// event's caller ignores it).
unsafe extern "C" fn pool_tramp<const I: usize>(g: G) -> i32 {
    let Some(&(kind, addr)) = st().pool.get(I) else { return 0 };
    run_at(kind, addr, 2000, g) as i32
}

fn in_code(a: u32) -> bool {
    unsafe { ST.as_ref() }.is_some_and(|s| {
        s.kinds.values().any(|k| a >= k.code_base && a < k.code_base + k.blob.code.len() as u32)
            || s.arts.iter().any(|r| a >= r.base && a < r.base + r.blob.code.len() as u32)
    })
}

fn kind_of_code(a: u32) -> Option<i32> {
    let s = st();
    s.kinds
        .iter()
        .find(|(_, k)| a >= k.code_base && a < k.code_base + k.blob.code.len() as u32)
        .map(|(&n, _)| n)
        .or_else(|| s.arts.iter().find(|r| a >= r.base && a < r.base + r.blob.code.len() as u32).map(|r| r.kind))
}

/// Guest code behind the two-gobj trampolines (`POOL2`), by slot.
static mut POOL2_SLOTS: Vec<(i32, u32)> = Vec::new();

unsafe extern "C" fn pool2_tramp<const I: usize>(g: G, h: G) {
    let Some(&(kind, addr)) = (&*core::ptr::addr_of!(POOL2_SLOTS)).get(I) else { return };
    let s = st();
    let Some(gh) = bind_fighter(s, g) else { return };
    let hh = if h.is_null() { 0 } else { bind_fighter(s, h).unwrap_or(0) };
    sync_globals(s);
    let mut cpu = Cpu::default();
    cpu.gpr[1] = STACK_BASE + STACK_SIZE - 0x100 - s.depth * 0x2000;
    cpu.gpr[2] = R2;
    cpu.gpr[13] = R13;
    cpu.gpr[3] = gh;
    cpu.gpr[4] = hh;
    cpu.lr = 0;
    cpu.pc = addr;
    let _ = run_cpu(kind, 2001, cpu);
}

const fn pool2() -> [unsafe extern "C" fn(G, G); 16] {
    let mut t: [unsafe extern "C" fn(G, G); 16] = [pool2_tramp::<0>; 16];
    macro_rules! set { ($($e:literal)*) => { $( t[$e] = pool2_tramp::<$e>; )* } }
    set!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15);
    t
}
static POOL2: [unsafe extern "C" fn(G, G); 16] = pool2();

/// Native `_PSUserFunc`s standing in for guest hook tables (guest address of the three words, the native record).
static mut PSUF: Vec<(u32, *mut melee_types::_PSUserFunc)> = Vec::new();

/// `ptr_native`: Tails' SpecialLw_SpawnParticle stores `&fp->fv + 0x64` (three guest code words: hookCreate,
/// hookDelete, setUserData) as `gen->userfunc`; the native particle code calls the hooks through a native
/// `_PSUserFunc`, whose entries are trampolines into the guest words (kept in step with them by `ps_refresh`).
fn ps_userfunc(sp: &GuestSpace, v: u32, rec: &'static str) -> Option<usize> {
    if rec != "_PSUserFunc" || sp.describe(v)?.0 != "Fighter" {
        return None;
    }
    let t = unsafe { &mut *core::ptr::addr_of_mut!(PSUF) };
    if let Some(&(_, p)) = t.iter().find(|e| e.0 == v) {
        return Some(p as usize);
    }
    if t.len() >= PS_CREATE.len() {
        eprintln!("mexguest: _PSUserFunc pool full: guest {v:08X} left unwired");
        return None;
    }
    let p = Box::leak(Box::new(melee_types::_PSUserFunc::zeroed())) as *mut _;
    t.push((v, p));
    if flag(&REPORT, "MELEE_MEX_REPORT") {
        eprintln!("mexguest: _PSUserFunc for guest {v:08X} ({:?})", sp.describe(v));
    }
    Some(p as usize)
}

/// Every native `_PSUserFunc` entry as its guest word says: the trampoline when the word is set, None when it is 0
/// (the native code tests the entry before calling it, as the console tests the word).
unsafe fn ps_refresh() {
    let t = &*core::ptr::addr_of!(PSUF);
    for (i, &(v, p)) in t.iter().enumerate() {
        let w = |k: u32| st().space.ld32(v + 4 * k).unwrap_or(0) != 0;
        (*p).hookCreate = if w(0) { Some(PS_CREATE[i]) } else { None };
        (*p).hookDelete = if w(1) { Some(PS_DELETE[i]) } else { None };
        (*p).setUserData = if w(2) { Some(PS_SETUD[i]) } else { None };
    }
}

unsafe fn ps_call(i: usize, k: u32, p: *mut melee_types::HSD_Particle, r4: u32, f1: f32) -> i32 {
    let Some(&(v, _)) = (&*core::ptr::addr_of!(PSUF)).get(i) else { return 0 };
    let s = st();
    let a = s.space.ld32(v + 4 * k).unwrap_or(0);
    let Some(kind) = (a != 0).then(|| kind_of_code(a)).flatten() else { return 0 };
    sync_globals(s);
    let Ok(ph) = s.space.handle(p as usize, Some("HSD_Particle")) else { return 0 };
    let mut cpu = Cpu::default();
    cpu.gpr[1] = STACK_BASE + STACK_SIZE - 0x100 - s.depth * 0x2000;
    cpu.gpr[2] = R2;
    cpu.gpr[13] = R13;
    cpu.gpr[3] = ph;
    cpu.gpr[4] = r4;
    cpu.fpr[1] = (f1 as f64).to_bits();
    cpu.lr = 0;
    cpu.pc = a;
    run_cpu(kind, 2002, cpu).0.gpr[3] as i32
}
unsafe extern "C" fn ps_create<const I: usize>(p: *mut melee_types::HSD_Particle) -> i32 {
    ps_call(I, 0, p, 0, 0.0)
}
unsafe extern "C" fn ps_delete<const I: usize>(p: *mut melee_types::HSD_Particle) -> i32 {
    ps_call(I, 1, p, 0, 0.0)
}
unsafe extern "C" fn ps_setud<const I: usize>(p: *mut melee_types::HSD_Particle, idx: u8, f: f32) -> i32 {
    ps_call(I, 2, p, idx as u32, f)
}
type PsFn = unsafe extern "C" fn(*mut melee_types::HSD_Particle) -> i32;
type PsUd = unsafe extern "C" fn(*mut melee_types::HSD_Particle, u8, f32) -> i32;
static PS_CREATE: [PsFn; 8] = [ps_create::<0>, ps_create::<1>, ps_create::<2>, ps_create::<3>, ps_create::<4>, ps_create::<5>, ps_create::<6>, ps_create::<7>];
static PS_DELETE: [PsFn; 8] = [ps_delete::<0>, ps_delete::<1>, ps_delete::<2>, ps_delete::<3>, ps_delete::<4>, ps_delete::<5>, ps_delete::<6>, ps_delete::<7>];
static PS_SETUD: [PsUd; 8] = [ps_setud::<0>, ps_setud::<1>, ps_setud::<2>, ps_setud::<3>, ps_setud::<4>, ps_setud::<5>, ps_setud::<6>, ps_setud::<7>];

/// A guest code address as a two-gobj native callback (Dedede's inhale grabbed_cb).
fn fn_native2(a: u32) -> Option<unsafe extern "C" fn(G, G)> {
    let kind = kind_of_code(a)?;
    let slots = unsafe { &mut *core::ptr::addr_of_mut!(POOL2_SLOTS) };
    let i = match slots.iter().position(|&(k, x)| k == kind && x == a) {
        Some(i) => i,
        None if slots.len() < POOL2.len() => {
            slots.push((kind, a));
            slots.len() - 1
        }
        None => {
            eprintln!("mexguest: two-gobj trampoline pool full: guest {a:08X} left unwired");
            return None;
        }
    };
    Some(POOL2[i])
}

/// A guest code address as a native callback: its pool trampoline (made on first use).
fn fn_native(a: u32) -> Option<usize> {
    let Some(kind) = kind_of_code(a) else {
        // a retail callback the guest installs (fp->cb = ftCo_..., the vanilla address in its code): the port's
        return vanilla_gobj_fn(a).map(|f| f as usize).or_else(|| ef_update_fn(a));
    };
    let s = st();
    let i = match s.pool.iter().position(|&(k, x)| k == kind && x == a) {
        Some(i) => i,
        None => {
            if s.pool.len() == POOL.len() {
                eprintln!("mexguest: trampoline pool full ({}): guest {a:08X} left unwired", POOL.len());
                return None;
            }
            s.pool.push((kind, a));
            s.pool.len() - 1
        }
    };
    Some(POOL[i] as usize)
}

/// A vanilla EF_Effect update callback (`EF_Effect.update`, `void(EF_Effect*)`) the guest stores into an effect it
/// spawned (Wolf's SpecialHi_GFX: `e->update = efLib_Cb_SetRotYZ_FromFighter`): the port's function.
fn ef_update_fn(a: u32) -> Option<usize> {
    Some(match mex_import::layout::vanilla_name(a)? {
        "efLib_Cb_SetOffsetY_FromParamY" => melee_world::tu::efn::eflib::efLib_Cb_SetOffsetY_FromParamY as usize,
        "efLib_Cb_SetScale_FromParamX" => melee_world::tu::efn::eflib::efLib_Cb_SetScale_FromParamX as usize,
        "efLib_Cb_SetRotYAndTransition" => melee_world::tu::efn::eflib::efLib_Cb_SetRotYAndTransition as usize,
        "efLib_Cb_SetJObjOffsetZ" => melee_world::tu::efn::eflib::efLib_Cb_SetJObjOffsetZ as usize,
        "efLib_Cb_SetRotY_FromFighterDir" => melee_world::tu::efn::eflib::efLib_Cb_SetRotY_FromFighterDir as usize,
        "efLib_Cb_SetRotYZ_FromFighter" => melee_world::tu::efn::eflib::efLib_Cb_SetRotYZ_FromFighter as usize,
        "efLib_Cb_Fall_FromParamY" => melee_world::tu::efn::eflib::efLib_Cb_Fall_FromParamY as usize,
        "efLib_Cb_SetOffset_FromParams" => melee_world::tu::efn::eflib::efLib_Cb_SetOffset_FromParams as usize,
        "efLib_Cb_LifetimeEndSpawn" => melee_world::tu::efn::eflib::efLib_Cb_LifetimeEndSpawn as usize,
        "efLib_Cb_SetScaleRotY_FromFighter" => melee_world::tu::efn::eflib::efLib_Cb_SetScaleRotY_FromFighter as usize,
        "efLib_Cb_SetRotYZ_FromParamZ_FighterDir" => melee_world::tu::efn::eflib::efLib_Cb_SetRotYZ_FromParamZ_FighterDir as usize,
        "efLib_Cb_ftMr_SpecialLw" => melee_world::tu::efn::eflib::efLib_Cb_ftMr_SpecialLw as usize,
        "efLib_Cb_ftLg_SpecialLw" => melee_world::tu::efn::eflib::efLib_Cb_ftLg_SpecialLw as usize,
        "efLib_Cb_ftKp_SpecialHi" => melee_world::tu::efn::eflib::efLib_Cb_ftKp_SpecialHi as usize,
        "efLib_Cb_ftCo_Bury" => melee_world::tu::efn::eflib::efLib_Cb_ftCo_Bury as usize,
        "efLib_Cb_ApplyStoredAlpha" => melee_world::tu::efn::eflib::efLib_Cb_ApplyStoredAlpha as usize,
        "efLib_Cb_AccumOffset_FromParams" => melee_world::tu::efn::eflib::efLib_Cb_AccumOffset_FromParams as usize,
        _ => return None,
    })
}

fn fn_guest(p: usize) -> Option<u32> {
    let s = unsafe { ST.as_mut()? };
    if let Some(&(v, _)) = unsafe { &*core::ptr::addr_of!(PSUF) }.iter().find(|e| e.1 as usize == p) {
        return Some(v);
    }
    if let Some(a) = s.pool.iter().enumerate().find(|(i, _)| POOL[*i] as usize == p).map(|(_, &(_, a))| a) {
        return Some(a);
    }
    // a native retail callback in a field the guest reads (to compare or to call): its vanilla address
    if s.retail.is_empty() {
        for n in crate::mex_bridge::GOBJ_FN_NAMES {
            if let (Some(f), Some(a)) = (crate::mex_bridge::gobj_fn(n), mex_import::layout::vanilla_addr(n)) {
                s.retail.insert(f as usize, a);
            }
        }
        for (f, n) in melee_fighter::tu::fighter::fighter_proc_fns() {
            if let Some(a) = mex_import::layout::vanilla_addr(n) {
                s.retail.insert(f, a);
            }
        }
    }
    s.retail.get(&p).copied()
}

/// The native function a vanilla guest address names, for a one-argument gobj callback (MoveLogic rows that keep a
/// retail function, fn-pointer fields the guest copies from vanilla code).
fn vanilla_gobj_fn(a: u32) -> Option<unsafe extern "C" fn(G)> {
    let name = mex_import::layout::vanilla_name(a)?;
    crate::mex_bridge::gobj_fn(name)
}

/// `gw_Mex_FtFunctionInstall(kind, archive data, size)`: relocate the fighter file's `ftFunction` and register its
/// overrides with the engine's m-ex dispatch tables. `k` is the m-ex internal id, `file` / `symbol` its PlXx.dat and
/// ftData public.
pub fn install(kind: i32, file: &str, symbol: &str) {
    if std::env::var_os("MELEE_MEX_GUEST_OFF").is_some() {
        eprintln!("mexguest: MELEE_MEX_GUEST_OFF: kind {kind} runs its clone base");
        return;
    }
    let s = st();
    if s.kinds.contains_key(&kind) {
        return;
    }
    let disc = &mut crate::host::Host::get().disc;
    let dat = match disc.read(file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("mexguest: kind {kind}: {file}: {e}");
            return;
        }
    };
    if !s.small_data {
        s.small_data = true;
        // .sdata / .sbss around r13, .sdata2 / .sbss2 around r2 (zero where the DOL has no bytes: the bss)
        for (lo, hi) in [(R13 - 0x8000, R13 + 0x8000), (R2 - 0x8000, R2 + 0x8000)] {
            let mut v = vec![0u8; (hi - lo) as usize];
            let mut a = lo;
            while a < hi {
                if let Ok(b) = disc.dol_bytes(a, 0x100) {
                    v[(a - lo) as usize..(a - lo) as usize + 0x100].copy_from_slice(&b);
                }
                a += 0x100;
            }
            s.space.add_bytes(lo, v);
        }
    }
    let mut ar = match mex_import::hsd::Archive::parse(&dat) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("mexguest: kind {kind}: {file}: {e}");
            return;
        }
    };
    let Some(off) = ar.public("ftFunction") else {
        eprintln!("mexguest: kind {kind}: {file} has no ftFunction");
        return;
    };
    let code_base = s.code_next;
    let blob = match mex_import::mexfn::parse_at_base(&ar, off, Owner::Fighter, format!("{file}:ftFunction"), code_base) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("mexguest: kind {kind}: {file}: ftFunction: {e}");
            return;
        }
    };
    if code_base + blob.code.len() as u32 > CODE_END {
        eprintln!("mexguest: kind {kind}: no code space left for {file}");
        return;
    }
    s.code_next = (code_base + blob.code.len() as u32 + 0xFFF) & !0xFFF;
    s.space.add_bytes(code_base, blob.code.clone());
    if std::env::var_os("MELEE_MEX_DIS").is_some() {
        for (k, w) in blob.code.chunks(4).enumerate() {
            let a = code_base + 4 * k as u32;
            let w = u32::from_be_bytes([w[0], w[1], w.get(2).copied().unwrap_or(0), w.get(3).copied().unwrap_or(0)]);
            let off = 4 * k as u32;
            let sym = blob.symbols.iter().rev().find(|s| s.0 <= off).map(|s| format!("{}+{:#x}", s.2, off - s.0)).unwrap_or_default();
            eprintln!("mexdis: {kind} {a:08X} {w:08X} {sym} {:?}", mex_import::ppc::ir::decode(w, a));
        }
    }
    let dat_base = s.dat_next;
    let mut data = ar.data.clone();
    for &o in &ar.relocs {
        let o = o as usize;
        if o + 4 <= data.len() {
            let w = u32::from_be_bytes(data[o..o + 4].try_into().unwrap());
            data[o..o + 4].copy_from_slice(&w.wrapping_add(dat_base).to_be_bytes());
        }
    }
    s.dat_next = (dat_base + data.len() as u32 + 0xFFFF) & !0xFFFF;
    s.space.add_bytes(dat_base, data);
    let Some(fd) = ar.public(symbol) else {
        eprintln!("mexguest: kind {kind}: {file} has no {symbol}");
        return;
    };
    let mut f = Fighter_ { file: file.into(), blob, code_base, ft_data: dat_base + fd, stats: BTreeMap::new(), move_table: Vec::new(), move_guest: Vec::new() };
    if std::env::var_os("MELEE_MEX_CALLS").is_some() {
        // every vanilla function the blob links to (its relocations with an absolute 0x8xxxxxxx target)
        let mut t: Vec<u32> = f.blob.relocs.iter().filter(|r| r.flag == 0x0A && r.target & 0xF000_0000 == 0x8000_0000).map(|r| r.target).collect();
        t.sort();
        t.dedup();
        for a in t {
            let n = mex_import::layout::vanilla_name(a).unwrap_or("?");
            let b = mex_import::layout::vanilla_name(a).is_some_and(|n| unsafe { crate::mex_bridge::bridged(n) });
            eprintln!("mexcall: {kind} {a:08X} {n}{}", if b { "" } else { " UNBOUND" });
        }
    }
    let natdat = melee_runtime::dat::DatArchive::parse(&dat).ok().map(Box::new);
    if let Some(n) = natdat {
        st().natdat.insert(kind, (n, dat_base));
    }
    load_items(kind, file, &mut ar);
    let mut wired = Vec::new();
    for (slot, off) in f.blob.overrides.clone() {
        if let Some(&(_, ev)) = SLOT_EVENTS.iter().find(|e| e.0 == slot) {
            melee_fighter::platform::Mex_SetHook(ev, kind as usize, Some(EVENT_TRAMPS[ev]));
            wired.push(SLOT_NAMES[slot as usize]);
        } else if slot == SLOT_MOVE_LOGIC {
            build_move_table(&mut f, code_base + off);
            wired.push("MoveLogic");
        }
    }
    // an override of a slot the engine does not dispatch runs the clone base's handler instead, silently (the C
    // runtime's gw_mex_slot_name diagnostic): name them
    let unwired: Vec<&str> = f.blob.overrides.iter().filter(|o| o.0 != SLOT_MOVE_LOGIC && !SLOT_EVENTS.iter().any(|e| e.0 == o.0))
        .map(|o| SLOT_NAMES.get(o.0 as usize).copied().unwrap_or("?")).collect();
    eprintln!(
        "mexguest: kind {kind}: {file} ftFunction {} bytes at {code_base:08X}, {} overrides; running {}{}",
        f.blob.code.len(),
        f.blob.overrides.len(),
        wired.join(" "),
        if unwired.is_empty() { String::new() } else { format!("; not dispatched: {}", unwired.join(" ")) }
    );
    s.kinds.insert(kind, f);
}

/// gw_mex_load_items_for (fighter half): every article of the file's `itFunction` (`{count, article[count]}`, each an
/// ordinary MEXFunction) relocated into its own code region; its function relocs `{slot, code offset}` fill the words
/// of item.Custom[global - 237], a 0x3C-byte ItemLogicTable (slot 0 the state table, which lives in the code).
fn load_items(kind: i32, file: &str, ar: &mut mex_import::hsd::Archive) {
    let Some(top) = ar.public("itFunction") else { return };
    let count = ar.u32(top);
    if count > 64 {
        eprintln!("mexguest: kind {kind}: {file}: itFunction count {count} implausible");
        return;
    }
    for n in 0..count {
        let art = ar.u32(top + 4 + 4 * n);
        if art == 0 || !ar.reloc_set.contains(&(top + 4 + 4 * n)) {
            continue;
        }
        let r = |o: u32| ar.u32(art + o);
        let (code_off, irt_off, irt_n, frt_off, frt_n, code_size, dbg_n, dbg_off) =
            (r(0), r(4), r(8), r(0xC), r(0x10), r(0x14), r(0x18), r(0x1C));
        if code_size == 0 {
            // a MexTK older than the codeSize field (gw_ftfunction_code_bound): the code ends at the next structure
            // the archive names, and covers the last word its instruction relocs touch
            let mut best = ar.data.len() as u32;
            let mut cands = vec![art];
            if irt_n != 0 {
                cands.push(irt_off);
            }
            if frt_n != 0 {
                cands.push(frt_off);
            }
            if dbg_n != 0 {
                cands.push(dbg_off);
            }
            cands.extend(ar.publics.iter().chain(ar.externs.iter()).map(|x| x.1));
            for &o in &ar.relocs {
                cands.push(o);
                cands.push(ar.u32(o));
            }
            for v in cands {
                if v > code_off && v < best {
                    best = v;
                }
            }
            let need = (0..irt_n).map(|i| (ar.u32(irt_off + i * 8) & 0xFF_FFFF) + 4).max().unwrap_or(0);
            if best <= code_off || best - code_off < need || need == 0 {
                eprintln!("mexguest: kind {kind}: itFunction[{n}] codeSize 0 not recoverable");
                continue;
            }
            let at = (art + 0x14) as usize;
            ar.data[at..at + 4].copy_from_slice(&(best - code_off).to_be_bytes());
        }
        let s = st();
        let base = s.code_next;
        let blob = match mex_import::mexfn::parse_at_base(ar, art, Owner::Item, format!("{file}:itFunction[{n}]"), base) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("mexguest: kind {kind}: {file}: itFunction[{n}]: {e}");
                continue;
            }
        };
        if base + blob.code.len() as u32 > CODE_END {
            eprintln!("mexguest: kind {kind}: no code space left for itFunction[{n}]");
            return;
        }
        s.code_next = (base + blob.code.len() as u32 + 0xFFF) & !0xFFF;
        s.space.add_bytes(base, blob.code.clone());
        let len = blob.code.len() as u32;
        let mut words = [0u32; 15];
        for &(slot, off) in &blob.overrides {
            if (slot as usize) < 15 && off < len {
                words[slot as usize] = base + off;
            }
        }
        if std::env::var_os("MELEE_MEX_CALLS").is_some() {
            let mut t: Vec<u32> = blob.relocs.iter().filter(|r| r.flag == 0x0A && r.target & 0xF000_0000 == 0x8000_0000).map(|r| r.target).collect();
            t.sort();
            t.dedup();
            for a in t {
                let nm = mex_import::layout::vanilla_name(a).unwrap_or("?");
                let b = mex_import::layout::vanilla_name(a).is_some_and(|n| unsafe { crate::mex_bridge::bridged(n) });
                eprintln!("mexcall: {kind} itFunction[{n}] {a:08X} {nm}{}", if b { "" } else { " UNBOUND" });
            }
        }
        let symbols: Vec<u32> = blob.symbols.iter().map(|x| x.0).chain(blob.overrides.iter().map(|o| o.1)).collect();
        let code = blob.code.clone();
        // register the code range BEFORE making trampolines for it (fn_native names the owner by its range)
        s.arts.push(Art { kind, base, blob });
        let global = match crate::mex_host::ft_item_global(kind, n) {
            Some(g) if g >= 237 => g,
            other => {
                eprintln!("mexguest: kind {kind}: itFunction[{n}] is not a custom item kind ({other:?})");
                continue;
            }
        };
        let mut states = Vec::new();
        if words[0] != 0 {
            // the state table rows (anim id, animated, physics, collided): every row that reads as one (an anim id
            // or -1, callbacks 0 / this article's code / vanilla text), up to the next function symbol. The debug
            // symbols do not mark the table's end (the first function may follow at any offset), so a bound from
            // them alone cut Charizard's tables to one row and a state change past it read the next item's table.
            let rel = words[0] - base;
            let end = symbols.iter().copied().filter(|&o| o > rel).filter(|o| !blob_is_data(&code, *o)).min().unwrap_or(len);
            let rd = |o: u32| code.get(o as usize..o as usize + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap())).unwrap_or(0);
            let cb_ok = |a: u32| a == 0 || (a >= base && a < base + len) || (0x8000_3000..0x8050_0000).contains(&a);
            for i in 0..((end.max(rel) - rel) / 0x10) {
                let r = rel + i * 0x10;
                let anim = rd(r);
                if !(anim < 0x1000 || anim == u32::MAX) || !cb_ok(rd(r + 4)) || !cb_ok(rd(r + 8)) || !cb_ok(rd(r + 12)) {
                    break;
                }
                if std::env::var_os("MELEE_MEX_CALLS").is_some() {
                    eprintln!("mexitem: kind {kind} itFunction[{n}] state {i}: anim {anim:#x} cbs {:08X} {:08X} {:08X}", rd(r + 4), rd(r + 8), rd(r + 12));
                }
                let mut t = ItemStateTable::zeroed();
                t.anim_id = rd(r) as i32;
                unsafe {
                    t.animated = guest_fn(rd(r + 4)).map(|p| core::mem::transmute::<usize, unsafe extern "C" fn(G) -> i32>(p));
                    t.physics_updated = guest_fn(rd(r + 8)).map(|p| core::mem::transmute::<usize, unsafe extern "C" fn(G)>(p));
                    t.collided = guest_fn(rd(r + 12)).map(|p| core::mem::transmute::<usize, unsafe extern "C" fn(G) -> i32>(p));
                }
                states.push(t);
            }
        }
        let mut logic = Box::new(ItemLogicTable::zeroed());
        unsafe {
            // ItemLogicTable words 1..14: the callbacks by slot, predicates and events alike (a pool trampoline returns
            // the guest's r3, which an event's caller ignores)
            let w = &mut *(logic.as_mut() as *mut ItemLogicTable as *mut [usize; 15]);
            for k in 1..15 {
                w[k] = guest_fn(words[k]).unwrap_or(0);
            }
        }
        let s = st();
        let it = s.items.entry(global).or_insert_with(CustomItem::new);
        it.states = states;
        it.logic = logic;
        it.logic.states = if it.states.is_empty() { core::ptr::null_mut() } else { it.states.as_mut_ptr() };
        eprintln!(
            "mexguest: kind {kind}: {file} itFunction[{n}] -> item kind {global}: {len} bytes at {base:08X}, {} states, {} logic slots",
            it.states.len(),
            words.iter().filter(|&&w| w != 0).count()
        );
    }
}

impl CustomItem {
    fn new() -> Self {
        CustomItem { desc_guest: 0, desc: core::ptr::null_mut(), logic: Box::new(ItemLogicTable::zeroed()), states: Vec::new() }
    }
}

/// `MEX_IndexFighterItem(fighter kind, ItemDesc *desc, item id)`: item.RuntimeIndex[global - 237] = desc, the native
/// Article decoded from the fighter's file at the guest descriptor's offset.
fn index_item(kind: u32, desc: u32, n: u32) {
    let s = st();
    s.articles.insert((kind, n), desc);
    let Some(global) = crate::mex_host::ft_item_global(kind as i32, n).filter(|&g| g >= 237) else { return };
    let nat = s.natdat.get(&(kind as i32)).filter(|(_, b)| desc >= *b).map(|(ar, b)| {
        // as the engine's archive loader decodes (archive.rs host_parse): byte blobs (images, display lists) point
        // into a raw copy of the data block, kept for the process like the decoded graph
        let raw: &'static [u8] = Box::leak(ar.data.clone().into_boxed_slice());
        let mut d = ar.decoder();
        d.set_raw_base(raw.as_ptr() as usize);
        d.set_roots(ar.publics.iter().map(|q| q.offset).collect());
        let root = d.object::<melee_types::Article>(desc - *b);
        Box::leak(Box::new(d.finish()));
        unsafe { crate::dat_custom::fix_custom_article(root, raw.as_ptr() as usize, desc - *b) };
        root
    });
    let it = s.items.entry(global).or_insert_with(CustomItem::new);
    if it.desc_guest != desc {
        it.desc_guest = desc;
        it.desc = nat.unwrap_or(core::ptr::null_mut());
    }
    unsafe {
        melee_items::tu::item::ext::hook::Mex_ItemCustomDesc = Some(item_custom_desc);
        melee_items::tu::item::ext::hook::Mex_ItemCustomLogic = Some(item_custom_logic);
    }
}

/// Does code offset `o` start data rather than a function (a symbol on a table)? A function opens with
/// `stwu r1,-n(r1)` / `mflr r0`; anything else is not one.
fn blob_is_data(code: &[u8], o: u32) -> bool {
    let w = code.get(o as usize..o as usize + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap())).unwrap_or(0);
    !(w == 0x7C08_02A6 || w >> 16 == 0x9421 || w == 0x4E80_0020)
}

/// A guest code (or vanilla) address as a native callback pointer; 0 stays None.
fn guest_fn(a: u32) -> Option<usize> {
    if a == 0 {
        None
    } else {
        fn_native(a)
    }
}

/// `Mex_ItemCustomDesc(kind)`: item.RuntimeIndex[kind - 237].
fn item_custom_desc(kind: i32) -> *mut c_void {
    match unsafe { ST.as_ref() }.and_then(|s| s.items.get(&(kind as u32))) {
        Some(it) if !it.desc.is_null() => it.desc as *mut c_void,
        _ => {
            eprintln!("mexguest: item kind {kind} not initialized (no MEX_IndexFighterItem registered it)");
            core::ptr::null_mut()
        }
    }
}

/// `Mex_ItemCustomLogic(kind)`: &item.Custom[kind - 237].
fn item_custom_logic(kind: i32) -> *mut c_void {
    match unsafe { ST.as_mut() }.and_then(|s| s.items.get_mut(&(kind as u32))) {
        Some(it) => it.logic.as_mut() as *mut ItemLogicTable as *mut c_void,
        None => core::ptr::null_mut(),
    }
}

/// Can custom item kind `k` be made (a descriptor and a state table)?
fn item_ready(k: u32) -> bool {
    unsafe { ST.as_ref() }.and_then(|s| s.items.get(&k)).is_some_and(|it| !it.desc.is_null() && !it.logic.states.is_null())
}

/// The MoveLogic table (a MotionState[] in the blob, gw_mex_movelogic_wire): a native copy whose anim/input/phys/coll
/// callbacks are trampolines into the row's guest code (or the retail function a row keeps), cam_cb the retail one.
fn build_move_table(f: &mut Fighter_, table: u32) {
    let rel = table - f.code_base;
    let end = f.blob.symbols.iter().find(|s| s.0 == rel && s.1 > rel).map(|s| s.1).unwrap_or_else(|| {
        f.blob.overrides.iter().filter(|o| o.0 & 0x8000_0000 == 0 && o.1 > rel).map(|o| o.1).min().unwrap_or(f.blob.code.len() as u32)
    });
    let n = ((end - rel) / 0x20) as usize;
    let rd = |o: u32| u32::from_be_bytes(f.blob.code[o as usize..o as usize + 4].try_into().unwrap());
    let s = st();
    for i in 0..n {
        let r = rel + i as u32 * 0x20;
        let mut m = MotionState::zeroed();
        m.anim_id = rd(r) as _;
        m.x4_flags = rd(r + 4) as _;
        // the anon_8 word (move_id << 24 and the flag bits) as the retail tables spell it (retail_ms.rs: `_x0 = (id << 24 | ...).to_be()`, GC byte order)
        m.anon_8._x0 = rd(r + 8).to_be();
        let mut g = [0u32; 4];
        let mut cbs: [melee_types::HSD_GObjEvent; 5] = [None; 5];
        for (w, cb) in cbs.iter_mut().enumerate() {
            let a = rd(r + 0xC + 4 * w as u32);
            if w < 4 {
                g[w] = a;
            }
            *cb = if a == 0 {
                None
            } else if a >= f.code_base && a < f.code_base + f.blob.code.len() as u32 && w < 4 {
                Some(MOVE_TRAMPS[w])
            } else {
                let v = vanilla_gobj_fn(a);
                if v.is_none() {
                    // a retail callback the bridge does not know would leave the row's slot empty (Dedede's multi-jump
                    // rows keep ftCo_JumpAerialF1_*: without their Phys the jump had no gravity or drift): name it
                    eprintln!("mexguest: {}: MoveLogic row {i} slot {w} keeps retail {:08X} {}, not in mex_bridge (add it to mex_bridge_names.txt)",
                        f.file, a, mex_import::layout::vanilla_name(a).unwrap_or("?"));
                }
                v
            };
        }
        m.anim_cb = cbs[0];
        m.input_cb = cbs[1];
        m.phys_cb = cbs[2];
        m.coll_cb = cbs[3];
        m.cam_cb = cbs[4];
        if std::env::var_os("MELEE_MEX_CALLS").is_some() {
            eprintln!("mexmove: row {i} anim {:#x} flags {:#x} word8 {:#010x} cbs {:08X?}", rd(r), rd(r + 4), rd(r + 8), (0..5).map(|w| rd(r + 0xC + 4 * w)).collect::<Vec<_>>());
        }
        f.move_table.push(m);
        f.move_guest.push(g);
    }
}

/// The installed MoveLogic table of `kind` (None: the retail / clone-base one).
pub fn move_logic_table(kind: i32) -> Option<*mut MotionState> {
    let f = unsafe { ST.as_mut()? }.kinds.get_mut(&kind)?;
    (!f.move_table.is_empty()).then(|| f.move_table.as_mut_ptr())
}

const fn event_tramps() -> [unsafe extern "C" fn(G); melee_fighter::platform::MEX_EVENT_COUNT] {
    let mut t: [unsafe extern "C" fn(G); melee_fighter::platform::MEX_EVENT_COUNT] = [ev_tramp::<0>; melee_fighter::platform::MEX_EVENT_COUNT];
    macro_rules! set { ($($e:literal)*) => { $( t[$e] = ev_tramp::<$e>; )* } }
    set!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39);
    t
}
static EVENT_TRAMPS: [unsafe extern "C" fn(G); melee_fighter::platform::MEX_EVENT_COUNT] = event_tramps();
static MOVE_TRAMPS: [unsafe extern "C" fn(G); 4] = [move_tramp::<0>, move_tramp::<1>, move_tramp::<2>, move_tramp::<3>];

unsafe fn kind_of(g: G) -> Option<i32> {
    if g.is_null() || (*g).user_data.is_null() {
        return None;
    }
    Some((*((*g).user_data as *mut Fighter)).kind as i32)
}

unsafe extern "C" fn ev_tramp<const E: usize>(g: G) {
    let Some(kind) = kind_of(g) else { return };
    let Some(&(slot, _)) = SLOT_EVENTS.iter().find(|e| e.1 == E) else { return };
    let Some(f) = st().kinds.get(&kind) else { return };
    let Some(&(_, off)) = f.blob.overrides.iter().find(|o| o.0 == slot) else { return };
    let a = f.code_base + off;
    let faulted = run_at_f(kind, a, slot, g).1;
    // an onLoad that could not finish leaves the fighter half set up: the clone base's onLoad runs after it (its
    // attributes and article tables), as the native match did before the guest code ran
    if faulted && E == melee_fighter::platform::MEX_EVENT_ON_LOAD {
        if let Some(v) = melee_fighter::tu::ftdata::ftData_OnLoad.get(kind as usize).copied().flatten() {
            eprintln!("mexguest: kind {kind}: onLoad faulted - the clone base's onLoad runs");
            v(g);
        }
    }
}

unsafe extern "C" fn move_tramp<const W: usize>(g: G) {
    let Some(kind) = kind_of(g) else { return };
    let fp = (*g).user_data as *mut u8;
    let s = st();
    let (Some(_), Some(_)) = (bind_fighter(s, g), Some(())) else { return };
    let h = s.space.handle_of(fp).unwrap_or(0);
    let mid = s.space.ld32(h + FT_MOTION_ID).unwrap_or(0);
    let x18 = s.space.ld32(h + FT_X18).unwrap_or(0);
    let Some(f) = s.kinds.get(&kind) else { return };
    let idx = mid.wrapping_sub(x18) as usize;
    let Some(row) = f.move_guest.get(idx) else { return };
    let a = row[W];
    if a != 0 {
        run_at(kind, a, 1000 + W as u32, g);
    }
}

/// Register the gobj and its fighter (and the fighter's dat_attrs_backup buffer) in the guest space; the gobj's
/// guest handle.
unsafe fn bind_fighter(s: &mut State, g: G) -> Option<u32> {
    let fp = (*g).user_data as *mut Fighter;
    if fp.is_null() {
        return None;
    }
    if (*g).classifier as i32 == melee_types::HSD_GOBJ_CLASS_ITEM {
        return bind_item(s, g);
    }
    if (*g).classifier as i32 != melee_types::HSD_GOBJ_CLASS_FIGHTER {
        return match s.space.handle_of(g as *mut u8) {
            Some(h) => Some(h),
            None => s.space.add_native(None, "HSD_GObj", g as *mut u8).ok(),
        };
    }
    let gh = match s.space.handle_of(g as *mut u8) {
        Some(h) => h,
        None => s.space.add_native(None, "HSD_GObj", g as *mut u8).ok()?,
    };
    if s.space.handle_of(fp as *mut u8).is_none() {
        let fh = s.space.add_native(None, "Fighter", fp as *mut u8).ok()?;
        if let Some(f) = s.kinds.get(&((*fp).kind as i32)) {
            s.space.ptr_override.insert((fh, FT_FT_DATA), f.ft_data);
        }
        let ab = (*fp).dat_attrs_backup as usize;
        if ab != 0 && !s.attrs.iter().any(|a| a.0 == ab) {
            let base = ATTRS_BASE + s.attrs.len() as u32 * 0x1000;
            s.space.add_raw(base, ab as *mut u8, ATTRS_SIZE);
            s.attrs.push((ab, base));
        }
        let _ = FT_DAT_ATTRS_BACKUP;
    }
    Some(gh)
}

/// An item gobj and its Item; a custom item's ip->xC4_article_data reads as the guest's own ItemDesc (its attributes
/// are the mod's layouts, which only the guest reads).
unsafe fn bind_item(s: &mut State, g: G) -> Option<u32> {
    let ip = (*g).user_data as *mut melee_types::Item;
    let gh = match s.space.handle_of(g as *mut u8) {
        Some(h) => h,
        None => s.space.add_native(None, "HSD_GObj", g as *mut u8).ok()?,
    };
    // gobj->hsd_obj (a void*): the item's root joint, which the guest reads straight after a spawn
    let j = (*g).hsd_obj as *mut u8;
    if !j.is_null() && s.space.handle_of(j).is_none() {
        let _ = s.space.add_native(None, "HSD_JObj", j);
    }
    // the Item's handle may predate this item: HSD_ObjAlloc hands a freed Item's memory to the next spawn, whose
    // handle (and its article override) was registered for the old item, possibly another kind or a vanilla one
    // (Charizard's SpawnItem_Rock read the previous item's ItemDesc through ip->xC4)
    let ih = match s.space.handle_of(ip as *mut u8) {
        Some(h) => h,
        None => s.space.add_native(None, "Item", ip as *mut u8).ok()?,
    };
    match s.items.get(&((*ip).kind as u32)).map(|it| it.desc_guest).filter(|&d| d != 0) {
        Some(d) => {
            s.space.ptr_override.insert((ih, ITEM_ARTICLE_DATA), d);
        }
        None => {
            s.space.ptr_override.remove(&(ih, ITEM_ARTICLE_DATA));
        }
    }
    Some(gh)
}
/// Item GC offset of xC4_article_data (item.h).
const ITEM_ARTICLE_DATA: u32 = 0xC4;

/// Vanilla pointer globals of the small-data areas the guest reads (`lwz r9, p_ftCommonData`): their guest handles
/// written into the r13 / r2 bytes before a run (the DOL's image holds 0 there, the bss).
unsafe fn sync_globals(s: &mut State) {
    // it_804D6D28: the ItemCommonData (Diddy's Nut_Phys reads its gravity, +0x68)
    let rows: [(u32, usize, &'static str); 2] = [
        (0x804D_6554, melee_runtime::shared::p_ftCommonData as usize, "ftCommonData"),
        (0x804D_6D28, melee_items::tu::it_3F14::it_804D6D28 as usize, "ItemCommonData"),
    ];
    for (a, p, rec) in rows {
        if let Ok(h) = s.space.handle(p, Some(rec)) {
            let _ = s.space.st32(a, h);
        }
    }
    // HSD_GObjPLinkHead (0x804D782C), the gobj list heads by p_link: a guest table of their handles, rewritten before
    // every run (Sonic's SpecialN_SearchTarget walks the item list, plink 9)
    {
        use melee_runtime::tu::gobj_native as gn;
        if !s.plinks {
            s.space.add_bytes(PLINK_GUEST, vec![0; PLINK_MAX as usize * 4]);
            s.plinks = true;
        }
        let n = (gn::HSD_GObjLibInitData.p_link_max as u32 + 1).min(PLINK_MAX);
        if !gn::HSD_GObjPLinkHead.is_null() {
            for i in 0..n {
                let g = *gn::HSD_GObjPLinkHead.add(i as usize);
                let h = s.space.handle(g as usize, Some("HSD_GObj")).unwrap_or(0);
                let _ = s.space.st32(PLINK_GUEST + 4 * i, h);
            }
            let _ = s.space.st32(0x804D_782C, PLINK_GUEST);
        }
    }
}
/// The guest copy of HSD_GObjPLinkHead's table (sync_globals).
const PLINK_GUEST: u32 = 0x8580_0000;
const PLINK_MAX: u32 = 64;

/// Run guest code at `a` with r3 = the gobj (gw_ppc_call). Re-entrant: a nested call (a bridged function that calls
/// back into guest code) runs on the stack below the outer one.
unsafe fn run_at(kind: i32, a: u32, slot: u32, g: G) -> u32 {
    run_at_f(kind, a, slot, g).0
}

/// `run_at`, also saying whether the run faulted.
unsafe fn run_at_f(kind: i32, a: u32, slot: u32, g: G) -> (u32, bool) {
    let s = st();
    let Some(gh) = bind_fighter(s, g) else { return (0, true) };
    sync_globals(s);
    let mut cpu = Cpu::default();
    cpu.gpr[1] = STACK_BASE + STACK_SIZE - 0x100 - s.depth * 0x2000;
    cpu.gpr[2] = R2;
    cpu.gpr[13] = R13;
    cpu.gpr[3] = gh;
    cpu.lr = 0;
    cpu.pc = a;
    let (r, faulted) = run_cpu(kind, slot, cpu);
    (r.gpr[3], faulted)
}

unsafe fn run_cpu(kind: i32, slot: u32, cpu: Cpu) -> (Cpu, bool) {
    let s = st();
    if s.depth >= 8 {
        return (cpu, true);
    }
    s.depth += 1;
    if flag(&CALLTRACE, "MELEE_MEX_CALLTRACE") {
        eprintln!("mexrun: kind {kind} slot {slot} pc {:08X} r3 {:08X}", cpu.pc, cpu.gpr[3]);
    }
    let mut m = Machine::new_gekko(cpu);
    let mut n = 0u64;
    let mut fault = None;
    let mut br = Br;
    loop {
        if n >= BUDGET {
            fault = Some(Fault::Budget);
            break;
        }
        n += 1;
        match m.step(&mut st().space, &mut br) {
            Ok(Step::Continue) => {}
            Ok(Step::Return) => break,
            Err(f) => {
                fault = Some(f);
                break;
            }
        }
    }
    let s = st();
    s.depth -= 1;
    ps_refresh();
    if let Some(t) = s.space.trace.as_mut().map(core::mem::take) {
        for r in t {
            if r.store {
                eprintln!("mextrace: slot {slot} store {}+{:#x} ({} bytes)", r.rec, r.gc_off, r.size);
            } else if std::env::var("MELEE_MEX_TRACE").is_ok_and(|v| v == "2") {
                eprintln!("mextrace: slot {slot} load {}+{:#x} ({} bytes) @{:08X} {:X?}", r.rec, r.gc_off, r.size, r.ea, s.space.regions_at(r.ea));
            }
        }
    }
    let fk = fault.map(|f| match f {
        Fault::Unresolved(t) => format!("unresolved {t:08X} {}", mex_import::layout::vanilla_name(t).unwrap_or("?")),
        Fault::Mem(a) => format!("mem {a:08X} at {}", where_(kind, m.cpu.pc.wrapping_sub(4))),
        other => format!("{other:?} at {}", where_(kind, m.cpu.pc.wrapping_sub(4))),
    });
    let mut dump = String::new();
    if fk.is_some() && flag(&REPORT, "MELEE_MEX_REPORT") {
        let pc = m.cpu.pc.wrapping_sub(4);
        let c = &m.cpu;
        dump += &format!("mexguest:   r3..r12 {:08X?} r28..r31 {:08X?} lr {:08X}\n", &c.gpr[3..13], &c.gpr[28..32], c.lr);
        if let Some(Fault::Mem(a)) = fault {
            if let Some((rec, off)) = s.space.describe(a) {
                dump += &format!("mexguest:   fault address is {rec}+{off:#x}\n");
            }
        }
        for a in (pc.wrapping_sub(0x30)..=pc).step_by(4) {
            if let Ok(w) = s.space.ld32(a) {
                dump += &format!("mexguest:   {a:08X} {w:08X} {:?}\n", mex_import::ppc::ir::decode(w, a));
            }
        }
    }
    let faulted = fk.is_some();
    if let Some(f) = s.kinds.get_mut(&kind) {
        let stt = f.stats.entry(slot).or_default();
        stt.calls += 1;
        stt.insns += n;
        match fk {
            None => stt.returned += 1,
            Some(k) => {
                if *stt.faults.entry(k.clone()).or_default() == 0 && flag(&REPORT, "MELEE_MEX_REPORT") {
                    eprintln!("mexguest: kind {kind} slot {slot}: {k}");
                    eprint!("{dump}");
                }
                *stt.faults.get_mut(&k).unwrap() += 1;
            }
        }
    }
    (m.cpu, faulted)
}

fn where_(kind: i32, pc: u32) -> String {
    if let Some(a) = st().arts.iter().find(|r| pc >= r.base && pc < r.base + r.blob.code.len() as u32) {
        let off = pc - a.base;
        return match a.blob.symbols.iter().rev().find(|s| s.0 <= off) {
            Some(s) => format!("{}:{}+{:#x}", a.blob.label, s.2, off - s.0),
            None => format!("{}+{off:#x}", a.blob.label),
        };
    }
    let Some(f) = st().kinds.get(&kind) else { return format!("{pc:08X}") };
    let off = pc.wrapping_sub(f.code_base);
    match f.blob.symbols.iter().rev().find(|s| s.0 <= off) {
        Some(s) if off < f.blob.code.len() as u32 => format!("{}+{:#x}", s.2, off - s.0),
        _ => format!("{pc:08X}"),
    }
}

struct Br;

impl Bridge<GuestSpace> for Br {
    fn in_blob(&self, a: u32) -> bool {
        in_code(a)
    }
    fn call(&mut self, cpu: &mut Cpu, mem: &mut GuestSpace, target: u32) -> Result<(), Fault> {
        call_out(cpu, mem, target)
    }
}

fn call_out(cpu: &mut Cpu, mem: &mut GuestSpace, target: u32) -> Result<(), Fault> {
    if flag(&CALLTRACE, "MELEE_MEX_CALLTRACE") {
        let f = |i: usize| f64::from_bits(cpu.fpr[i]);
        eprintln!(
            "mexcall: {target:08X} {} r3..r6 {:08X?} f1..f3 {:?} {:?} {:?} lr {:08X}",
            mex_import::layout::vanilla_name(target).unwrap_or("?"),
            &cpu.gpr[3..7],
            f(1),
            f(2),
            f(3),
            cpu.lr
        );
    }
    let (d, a, n) = (cpu.gpr[3], cpu.gpr[4], cpu.gpr[5]);
    let ok = match target {
        MEMCPY => {
            if d <= a {
                for k in 0..n {
                    let b = mem.ld8(a.wrapping_add(k))?;
                    mem.st8(d.wrapping_add(k), b)?;
                }
            } else {
                for k in (0..n).rev() {
                    let b = mem.ld8(a.wrapping_add(k))?;
                    mem.st8(d.wrapping_add(k), b)?;
                }
            }
            true
        }
        MEMSET => {
            for k in 0..n {
                mem.st8(d.wrapping_add(k), a as u8)?;
            }
            true
        }
        MEX_INDEX_FIGHTER_ITEM => {
            // (fighter kind, ItemDesc, item id): gw_mex_shim_index_item, item.RuntimeIndex[global - 237] = desc. The
            // descriptor is the guest's (in its dat); the native item system gets the same ItemDesc decoded from the
            // file, and an item's guest code reading ip->xC4 is pointed back at the guest's bytes (bind_item).
            index_item(d, a, n);
            cpu.gpr[3] = 0;
            true
        }
        HSD_MEM_ALLOC | MEX_CALLOC => {
            // a block of the game heap (so a rollback restores it) as plain guest bytes at a fresh guest address
            let p = unsafe { melee_runtime::tu::memory::HSD_MemAlloc(d as i32) } as *mut u8;
            if !p.is_null() && target == MEX_CALLOC {
                // gw_mex_shim_calloc: m-ex's calloc(size), zeroed
                unsafe { core::ptr::write_bytes(p, 0, d as usize) };
            }
            if p.is_null() {
                cpu.gpr[3] = 0;
            } else {
                let s = st();
                let base = s.heap_next;
                s.heap_next = (base + d.max(4) + 0x1F) & !0x1F;
                unsafe { mem.add_raw(base, p, d.max(4)) };
                cpu.gpr[3] = base;
            }
            true
        }
        HSD_FREE => {
            if let Some(p) = mem.native(d) {
                unsafe { melee_runtime::tu::memory::HSD_Free(p as *mut c_void) };
                mem.regions.retain(|r| !(r.base == d && matches!(r.backing, mex_import::remap::Backing::Raw { .. })));
            }
            true
        }
        MEX_GET_DATA => {
            // gw_mex_shim_get_data: MxDt.dat tables (the image is mapped as guest bytes at its own address) and the
            // game's costume table (the native rows, by handle)
            let Some((base, size, root)) = crate::mex_host::mexdata_image() else { return Err(Fault::Unresolved(target)) };
            if !mem.regions.iter().any(|r| r.base == base) {
                unsafe { mem.add_raw(base, base as usize as *mut u8, size) };
            }
            let field = |mem: &mut GuestSpace, arch: u32, f: u32| -> u32 {
                let a = mem.ld32(root + arch).unwrap_or(0);
                if a == 0 { 0 } else { mem.ld32(a + f).unwrap_or(0) }
            };
            cpu.gpr[3] = match d {
                8 => {
                    let p = unsafe { core::ptr::addr_of_mut!(melee_fighter::tu::ftdata::CostumeListsForeachCharacter) } as usize;
                    mem.handle(p, Some("UnkCostumeList")).unwrap_or(0)
                }
                10 => field(mem, 0x28, 0x00),
                11 => field(mem, 0x28, 0x10),
                12 => field(mem, 0x08, 0x00),
                13 => field(mem, 0x08, 0x04),
                14 => field(mem, 0x08, 0x08),
                15 => field(mem, 0x08, 0x0C),
                16 => root,
                _ => return Err(Fault::Unresolved(0xD000_0000 | d)),
            };
            true
        }
        MEX_GET_FT_ITEM_ID => {
            // gw_mex_shim_get_ft_item_id: the global item kind of the fighter's article `a` (MxDt.dat item_lookup)
            let kind = mem.native(d).filter(|&p| p != 0).map(|g| unsafe { kind_of(g as G) }).flatten();
            match kind.and_then(|k| crate::mex_host::ft_item_global(k, a)) {
                Some(v) => {
                    cpu.gpr[3] = v;
                    true
                }
                None => false,
            }
        }
        EF_SYNC_SPAWN => {
            // efSync_Spawn(id, gobj, ...): the variadic words as the native call takes them (pointers to objects the
            // guest knows by handle, records it keeps on its stack copied word-swapped in and out, plain values)
            let g = mem.native(a).unwrap_or(0);
            let mut words = [0usize; 6];
            let mut tmp: Vec<(u32, Box<[u32; 16]>)> = Vec::new();
            let mut bad_joint = false;
            for (k, w) in words.iter_mut().enumerate() {
                let h = cpu.gpr[5 + k];
                *w = if h == 0 {
                    0
                } else if let Some(p) = mem.interior(h).or_else(|| mem.regions.iter().find(|r| r.base == h && matches!(r.backing, mex_import::remap::Backing::Native { .. })).and_then(|_| mem.native(h))) {
                    p
                } else if mem.native(h).is_some() {
                    let mut b = Box::new([0u32; 16]);
                    for (j, x) in b.iter_mut().enumerate() {
                        *x = mem.ld32(h + 4 * j as u32).unwrap_or(0);
                    }
                    let p = b.as_mut_ptr() as usize;
                    tmp.push((h, b));
                    p
                } else if h < 0x1_0000 {
                    h as usize
                } else if k == 0 {
                    // the first tail word (the joint an attached effect hangs on) is neither an object the guest knows
                    // nor a small value: Tails' guest passes 1.0f there, which the native effect dereferenced (and a
                    // NULL joint asserts in HSD_RObjSetConstraintObj). The effect is not made; r3 = NULL, as when the
                    // effect cannot be made
                    bad_joint = true;
                    0
                } else {
                    // a register past the effect's own arguments (the tail is variadic; Wolf's Laser_Spawn leaves a
                    // float's bits in r7)
                    0
                };
            }
            if bad_joint {
                *st().unresolved.entry(0xE000_0000 | d).or_default() += 1;
                cpu.gpr[3] = 0;
                return Ok(());
            }
            let e = unsafe { melee_chara::adapt::efSync_Spawn_usize(d as i32, g as G, &words) };
            for (h, b) in tmp {
                for (j, x) in b.iter().enumerate() {
                    let _ = mem.st32(h + 4 * j as u32, *x);
                }
            }
            // the effect it made (the guest sets its callback, `e->x10 = fn`): by handle; it returned the id in r3
            // before, and the guest's store through it faulted (Wolf's SpecialHi_GFX)
            // a particle effect returns its HSD_Generator, not an EF_Effect (efSync_Spawn's hsd_8039F05C /
            // efLib_CreateGenerator paths; Tails' SpecialLw_SpawnParticle stores the generator's fields): by the live
            // generator list
            let is_gen = unsafe {
                let mut g = melee_runtime::tu::generator::hsd_804D78FC;
                while !g.is_null() && g as *mut c_void != e {
                    g = (*g).next;
                }
                !g.is_null()
            };
            cpu.gpr[3] = if e.is_null() { 0 } else { mem.handle(e as usize, Some(if is_gen { "HSD_Generator" } else { "EF_Effect" })).unwrap_or(0) };
            true
        }
        _ if hand_bridge(cpu, mem, target)? => true,
        _ if mex_import::layout::vanilla_name(target) == Some("HSD_ArchiveGetPublicAddress") && untyped_public(mem, d, a).is_some() => {
            // a public the engine's type table does not know, of another file the mod loaded (PlyTailsColor in the
            // costume file): that archive's raw data block mapped as guest bytes
            cpu.gpr[3] = untyped_public(mem, d, a).unwrap_or(0);
            true
        }
        _ if mex_import::layout::vanilla_name(target) == Some("HSD_ArchiveGetPublicAddress") && guest_public(mem, a).is_some() => {
            // a symbol of an m-ex fighter's own file (PlyTailsColor, the mod's costume data, which the engine's type
            // table does not know): the guest's copy of that file, read as the console reads it
            cpu.gpr[3] = guest_public(mem, a).unwrap_or(0);
            true
        }
        _ => match mex_import::layout::vanilla_name(target) {
            // an m-ex custom article (global item kind >= CustomItemStart 237, SpawnItem.kind at GC +0x8): the native
            // item system has no m-ex item table (its ItemDesc and itFunction states are the guest's), so the spawn
            // spawns nothing: the call returns NULL (what Item_80268B18 returns when the item cannot be made) and the
            // guest goes on; counted under 0xC0000000 | kind
            Some(n) if matches!(n, "Item_80268B18" | "Item_80268B5C" | "Item_80268B9C" | "Item_8026862C") && mem.ld32(d.wrapping_add(8)).is_ok_and(|k| k >= 237 && !item_ready(k)) => {
                *st().unresolved.entry(0xC000_0000 | mem.ld32(d + 8).unwrap_or(0)).or_default() += 1;
                cpu.gpr[3] = 0;
                return Ok(());
            }
            // an item function on the NULL a refused custom spawn returned: nothing to do (the native one would fault)
            Some(n) if d == 0 && (n.starts_with("Item_") || n.starts_with("it_") || n.starts_with("it")) => {
                cpu.gpr[3] = 0;
                return Ok(());
            }
            Some(name) => {
                let mut c = Call { cpu, mem, fix: Vec::new(), bad: false, scratch: SCRATCH_BASE + 0x10000 * st().depth };
                let hit = unsafe { crate::mex_bridge::call(name, &mut c) };
                c.finish();
                if c.bad {
                    let s = st();
                    *s.unresolved.entry(target).or_default() += 1;
                    return Err(Fault::Mem(0));
                }
                // a spawned item's gobj comes back by handle; its user_data (a void*) reads as the Item only once the
                // Item is registered, as bind_fighter does for a fighter (Charizard's SpawnItem_Fire read gobj->x2C)
                if hit && matches!(name, "Item_80268B18" | "Item_80268B5C" | "Item_80268B9C" | "Item_8026862C") && cpu.gpr[3] != 0 {
                    if let Some(g) = mem.native(cpu.gpr[3]) {
                        unsafe {
                            let g = g as G;
                            if (*g).classifier as i32 == melee_types::HSD_GOBJ_CLASS_ITEM && !(*g).user_data.is_null() {
                                let _ = bind_item(st(), g);
                            }
                        }
                    }
                }
                hit
            }
            None => false,
        },
    };
    let s = st();
    if ok {
        *s.bound.entry(target).or_default() += 1;
        Ok(())
    } else {
        *s.unresolved.entry(target).or_default() += 1;
        Err(Fault::Unresolved(target))
    }
}

/// The vanilla targets gen_mex_bridge.py leaves out (variadic tails, a stack argument, no unique port), bridged by
/// hand. Guest records are read and written word by word through the guest space (a native object's field or the
/// guest's own big-endian bytes alike). Ok(false): not one of them.
fn hand_bridge(cpu: &mut Cpu, mem: &mut GuestSpace, target: u32) -> Result<bool, Fault> {
    use melee_types::{Vec3, HSD_JObj};
    let Some(name) = mex_import::layout::vanilla_name(target) else { return Ok(false) };
    fn rd_vec(mem: &mut GuestSpace, h: u32) -> Result<Vec3, Fault> {
        Ok(Vec3 { x: f32::from_bits(mem.ld32(h)?), y: f32::from_bits(mem.ld32(h + 4)?), z: f32::from_bits(mem.ld32(h + 8)?) })
    }
    fn wr_vec(mem: &mut GuestSpace, h: u32, v: &Vec3) -> Result<(), Fault> {
        if h != 0 {
            mem.st32(h, v.x.to_bits())?;
            mem.st32(h + 4, v.y.to_bits())?;
            mem.st32(h + 8, v.z.to_bits())?;
        }
        Ok(())
    }
    let gobj = |mem: &GuestSpace, h: u32| -> G { if h == 0 { core::ptr::null_mut() } else { mem.native(h).unwrap_or(0) as G } };
    let r = |i: usize| cpu.gpr[3 + i];
    match name {
        // lbAnim_8001E8F8(FigaTree*): the animation's frame count (0 for NULL); the tree is the native one the guest
        // read out of the fighter (by handle) or a tree in guest bytes (frames at +8)
        "lbAnim_8001E8F8" => {
            let h = r(0);
            let v = if h == 0 {
                0.0
            } else if let Some(p) = mem.native(h).filter(|&p| p != 0) {
                unsafe { melee_runtime::tu::lbanim::lbAnim_8001E8F8((p as *const melee_types::FigaTree).as_ref()) }
            } else {
                f32::from_bits(mem.ld32(h + 8)?)
            };
            cpu.fpr[1] = (v as f64).to_bits();
        }
        // HSD_ForeachAnim(obj, type, mask, func, arg_type, ...): the variadic argument by arg_type (*F: a double in f1,
        // *U / *V: r8) and the AObj callback as the port's (Sonic's guest restarts its joints with HSD_AObjReqAnim)
        "HSD_ForeachAnim" => {
            use melee_runtime::tu::aobj::{self as ao, CallbackArg};
            let obj = mem.native(r(0)).filter(|&p| p != 0).ok_or(Fault::Mem(r(0)))? as *mut c_void;
            let func: usize = match mex_import::layout::vanilla_name(r(3)) {
                Some("HSD_AObjReqAnim") => ao::HSD_AObjReqAnim as usize,
                Some("HSD_AObjSetRate") => ao::HSD_AObjSetRate as usize,
                Some("HSD_AObjSetCurrentFrame") => ao::HSD_AObjSetCurrentFrame as usize,
                Some("HSD_AObjSetEndFrame") => ao::HSD_AObjSetEndFrame as usize,
                Some("HSD_AObjSetRewindFrame") => ao::HSD_AObjSetRewindFrame as usize,
                _ => return Err(Fault::Unresolved(r(3))),
            };
            // every callback above is (aobj, f32): on the console it reads f1 whatever the form, so with a *F form the
            // variadic double (f1) and with another (Sonic passes AOBJ_ARG_AOV to HSD_AObjSetRate) the guest's f1 as the
            // call left it - HSD_ForeachAnim's walk does no float work. Both are the AF form with f1
            let arg = CallbackArg { f: f64::from_bits(cpu.fpr[1]) as f32 };
            unsafe { ao::HSD_ForeachAnim(obj, r(1) as _, r(2) as _, func as *mut c_void, melee_types::AOBJ_ARG_AF as _, arg) };
        }
        // ftLib_800866DC (ftLib_GetECBCenter): the fighter's ECB centre into a Vec3
        "ftLib_GetECBCenter" | "ftLib_GetCameraBonePos" | "ftLib_800866DC" => {
            let g = gobj(mem, r(0));
            if g.is_null() {
                return Err(Fault::Mem(0));
            }
            let mut v = rd_vec(mem, r(1))?;
            unsafe {
                let _ = melee_fighter::ftlib::ftlib_getcamerabonepos(g as _, &mut *(&mut v as *mut Vec3 as *mut _));
            }
            wr_vec(mem, r(1), &v)?;
        }
        // MTXRotRad(Mtx m, char axis, f32 rad): m is the guest's 3x4 float matrix
        "MTXRotRad" => {
            let h = r(0);
            let mut m: melee_runtime::mtx::Mtx = [[0.0; 4]; 3];
            for (i, row) in m.iter_mut().enumerate() {
                for (j, x) in row.iter_mut().enumerate() {
                    *x = f32::from_bits(mem.ld32(h + 4 * (4 * i + j) as u32)?);
                }
            }
            melee_runtime::mtx::rot_rad(&mut m, r(1) as u8, f64::from_bits(cpu.fpr[1]) as f32);
            for (i, row) in m.iter().enumerate() {
                for (j, x) in row.iter().enumerate() {
                    mem.st32(h + 4 * (4 * i + j) as u32, x.to_bits())?;
                }
            }
        }
        // lb_80011E24(root, &out, index..., -1): the joint at a path of child indices (r5.., -1 terminated)
        "lb_80011E24" => {
            let root = mem.native(r(0)).unwrap_or(0) as *mut HSD_JObj;
            let mut path = Vec::new();
            for i in 2..8 {
                let v = r(i) as i32;
                path.push(v);
                if v == -1 {
                    break;
                }
            }
            let mut out: *mut HSD_JObj = core::ptr::null_mut();
            let ret = unsafe { melee_modes::mn_inl::lb_80011E24(root, &mut out, &path) };
            if r(1) != 0 {
                let h = if out.is_null() { 0 } else { mem.handle(out as usize, Some("HSD_JObj"))? };
                mem.st32(r(1), h)?;
            }
            cpu.gpr[3] = ret as u32;
        }
        // mpCheckFloor(ax, ay, bx, by, y_offset, vec_out, line_id_out, flags_out, normal_out, line_id_skip,
        // joint_id_skip, joint_id_only, cb, gobj): f1..f5, r3..r10, the gobj on the stack (sp + 8)
        "mpCheckFloor" => {
            let f = |i: usize| f64::from_bits(cpu.fpr[1 + i]) as f32;
            let gh = mem.ld32(cpu.gpr[1] + 8)?;
            let g = gobj(mem, gh);
            let cb = match r(7) {
                0 => None,
                a => match fn_native(a) {
                    Some(p) => Some(unsafe { core::mem::transmute::<usize, unsafe extern "C" fn(G, i32) -> i32>(p) }),
                    None => return Err(Fault::Unresolved(a)),
                },
            };
            let (mut v, mut n) = (melee_types::Vec::zeroed(), melee_types::Vec::zeroed());
            let (mut line, mut flags) = (0i32, 0u32);
            let ret = unsafe {
                melee_world::tu::mplib::mpCheckFloor(
                    f(0), f(1), f(2), f(3), f(4),
                    if r(0) != 0 { &mut v } else { core::ptr::null_mut() },
                    if r(1) != 0 { &mut line } else { core::ptr::null_mut() },
                    if r(2) != 0 { &mut flags } else { core::ptr::null_mut() },
                    if r(3) != 0 { &mut n } else { core::ptr::null_mut() },
                    r(4) as i32, r(5) as i32, r(6) as i32, cb, g,
                )
            };
            wr_vec(mem, r(0), &Vec3 { x: v.x, y: v.y, z: v.z })?;
            if r(1) != 0 {
                mem.st32(r(1), line as u32)?;
            }
            if r(2) != 0 {
                mem.st32(r(2), flags)?;
            }
            wr_vec(mem, r(3), &Vec3 { x: n.x, y: n.y, z: n.z })?;
            cpu.gpr[3] = ret as u32;
        }
        // efAsync_Spawn(gobj, queue head, kind, gfx, jobj, ...): the tail words as efSync_Spawn takes them (records the
        // guest keeps on its stack copied in and out)
        "efAsync_Spawn" => {
            let g = gobj(mem, r(0));
            let q = mem.interior(r(1)).or_else(|| mem.native(r(1))).unwrap_or(0) as *mut c_void;
            let jobj = mem.native(r(4)).unwrap_or(0) as *mut HSD_JObj;
            let mut tmp: Vec<(u32, Box<[u32; 16]>)> = Vec::new();
            let mut tail: Vec<*const c_void> = Vec::new();
            for k in 5..8 {
                let h = r(k);
                tail.push(if h == 0 {
                    core::ptr::null()
                } else if let Some(p) = mem.interior(h) {
                    p as *const c_void
                } else if mem.native(h).is_some() {
                    let mut b = Box::new([0u32; 16]);
                    for (j, x) in b.iter_mut().enumerate() {
                        *x = mem.ld32(h + 4 * j as u32).unwrap_or(0);
                    }
                    let p = b.as_ptr() as *const c_void;
                    tmp.push((h, b));
                    p
                } else if h < 0x1_0000 {
                    h as usize as *const c_void
                } else {
                    core::ptr::null()
                });
            }
            unsafe { melee_world::tu::efasync::efAsync_Spawn(g, q, r(2), r(3), jobj, &tail) };
            for (h, b) in tmp {
                for (j, x) in b.iter().enumerate() {
                    let _ = mem.st32(h + 4 * j as u32, *x);
                }
            }
        }
        // GX state of a guest render callback (onModelRender / an article's draw): nothing the simulation reads, and
        // the port's renderer sets its own state per draw, so these are no-ops here
        n if n.starts_with("GX") => {}
        // the out-of-line MSL fmodf (0x80364340, math.h's inline body): `a - b * (s64)(a / b)` as a product and a
        // subtraction. The bridge took aobj::fmodf, the contracted fnmsubs form, which gives -0 on an exact multiple:
        // Tails' guest passed fmodf(15, 15) = -0 to ChangeMotionState where the console has +0 (state_frame -0.0)
        "fmodf" => {
            let (a, b) = (f64::from_bits(cpu.fpr[1]) as f32, f64::from_bits(cpu.fpr[2]) as f32);
            let r = if b.abs() > a.abs() { a } else { a - b * (melee_core::ppc::fcvt_i64(a / b) as f32) };
            cpu.fpr[1] = (r as f64).to_bits();
        }
        // sqrtf(f32): the library routine (frsqrte + Newton steps) as the port has it
        "sqrtf__Ff" => cpu.fpr[1] = (melee_runtime::gekko::sqrtf(f64::from_bits(cpu.fpr[1]) as f32) as f64).to_bits(),
        "ftCommon_UnlockECB" => {
            let fp = mem.native(r(0)).unwrap_or(0) as *mut Fighter;
            if fp.is_null() {
                return Err(Fault::Mem(0));
            }
            unsafe { melee_fighter::ftcommon2::unlock_ecb(&mut *fp) };
        }
        "ftCo_Jump_GetInput" => {
            let g = gobj(mem, r(0));
            cpu.gpr[3] = melee_fighter::tu::ftCo_Jump::ftCo_Jump_GetInput(g as _) as u32;
        }
        "Player_GetEntity" => {
            let g = unsafe { melee_fighter::tu::player::Player_GetEntity(r(0) as i32) };
            cpu.gpr[3] = if g.is_null() { 0 } else { mem.handle(g as usize, Some("HSD_GObj"))? };
        }
        "Player_GetPlayerCharacter" => cpu.gpr[3] = unsafe { melee_fighter::tu::player::Player_GetPlayerCharacter(r(0) as i32) } as u32,
        "Player_GetPlayerSlotType" => cpu.gpr[3] = unsafe { melee_fighter::tu::player::Player_GetPlayerSlotType(r(0) as i32) } as u32,
        "Player_GetTeam" => cpu.gpr[3] = unsafe { melee_fighter::tu::player::Player_GetTeam(r(0) as i32) } as u32,
        _ => return Ok(false),
    }
    Ok(true)
}

/// The guest address of an untyped public of the archive at guest `arch` (name: C string at guest `p`): the
/// archive's data block mapped (once) as raw guest bytes.
fn untyped_public(mem: &mut GuestSpace, arch: u32, p: u32) -> Option<u32> {
    let archive = mem.native(arch)? as *mut melee_types::HSD_Archive;
    let mut name = Vec::new();
    for k in 0..128 {
        match mem.ld8(p.wrapping_add(k)).ok()? {
            0 => break,
            c => name.push(c),
        }
    }
    let (data, len, off) = crate::archive::untyped_public(archive, std::str::from_utf8(&name).ok()?)?;
    let base = match mem.regions.iter().find(|r| matches!(r.backing, mex_import::remap::Backing::Raw { ptr } if ptr as usize == data)) {
        Some(r) => r.base,
        None => {
            let s = st();
            let base = s.heap_next;
            s.heap_next = (base + len + 0x1F) & !0x1F;
            unsafe { mem.add_raw(base, data as *mut u8, len) };
            base
        }
    };
    Some(base + off)
}

/// The guest address of public `name` (a C string at guest `p`) in an installed m-ex fighter's file, the bytes the guest reads
/// (big-endian, relocated to the guest's copy), whatever native type the engine gives it.
fn guest_public(mem: &mut GuestSpace, p: u32) -> Option<u32> {
    let mut name = Vec::new();
    for k in 0..128 {
        match mem.ld8(p.wrapping_add(k)).ok()? {
            0 => break,
            c => name.push(c),
        }
    }
    let name = String::from_utf8(name).ok()?;
    let s = st();
    s.natdat.values().find_map(|(ar, base)| ar.public_offset(&name).map(|o| base + o))
}

/// One bridged call's arguments and result (mex_bridge's generated arms use it).
pub struct Call<'a> {
    cpu: &'a mut Cpu,
    mem: &'a mut GuestSpace,
    /// Marshalled records: (guest address, native buffer, record).
    fix: Vec<(u32, Box<[u64]>, &'static str, u32)>,
    pub bad: bool,
    scratch: u32,
}

impl Call<'_> {
    pub fn int(&self, i: usize) -> u32 {
        self.cpu.gpr[3 + i]
    }
    pub fn flt(&self, i: usize) -> f64 {
        f64::from_bits(self.cpu.fpr[1 + i])
    }
    /// Pointer argument `i` (an r3.. word) to a `rec` (None: untyped): a registered object's native pointer, a record
    /// the guest keeps in plain memory marshalled into a native copy (written back after the call), or the host byte
    /// of plain memory for an untyped pointer.
    pub fn ptr(&mut self, i: usize, rec: Option<&'static str>) -> *mut c_void {
        let h = self.cpu.gpr[3 + i];
        if h == 0 {
            return core::ptr::null_mut();
        }
        // inside a registered native object: the field / element that starts there (fp->coll_data, &fp->cur_pos)
        if let Some(nat) = self.mem.interior(h) {
            return nat as *mut c_void;
        }
        if let Some(nat) = self.mem.native(h) {
            let registered = self.mem.regions.iter().any(|r| r.base == h && matches!(r.backing, mex_import::remap::Backing::Native { .. }));
            if registered || rec.is_none() {
                return nat as *mut c_void;
            }
        }
        if let Some(rec) = rec {
            if let (Some(ns), Some(gs)) = (self.mem.layouts.native_size(rec), self.mem.layouts.gc_size(rec)) {
                let mut buf = vec![0u64; ns.div_ceil(8)].into_boxed_slice();
                let base = self.scratch;
                self.scratch += (gs + 0x1F) & !0xF;
                let p = buf.as_mut_ptr() as *mut u8;
                unsafe {
                    if self.mem.add_native(Some(base), rec, p).is_ok() {
                        copy_record(self.mem, h, base, gs);
                        self.fix.push((h, buf, rec, base));
                        return p as *mut c_void;
                    }
                }
            }
        }
        self.bad = true;
        core::ptr::null_mut()
    }
    /// A two-gobj callback argument (`HSD_GObjInteraction`, fp->grabbed_cb): guest code as a two-argument trampoline,
    /// a vanilla address as the port's function.
    pub fn func2(&mut self, i: usize) -> Option<unsafe extern "C" fn(G, G)> {
        let a = self.cpu.gpr[3 + i];
        if a == 0 {
            return None;
        }
        let f = fn_native2(a).or_else(|| mex_import::layout::vanilla_name(a).and_then(crate::mex_bridge::gobj2_fn));
        if f.is_none() {
            self.bad = true;
        }
        f
    }
    /// A gobj callback argument: guest code as its pool trampoline, a vanilla address as the port's function.
    pub fn func(&mut self, i: usize) -> Option<unsafe extern "C" fn(G)> {
        let a = self.cpu.gpr[3 + i];
        if a == 0 {
            return None;
        }
        if let Some(p) = fn_native(a) {
            return Some(unsafe { core::mem::transmute::<usize, unsafe extern "C" fn(G)>(p) });
        }
        let f = vanilla_gobj_fn(a);
        if f.is_none() {
            self.bad = true;
        }
        f
    }
    pub fn ret_int(&mut self, v: u32) {
        self.cpu.gpr[3] = v;
    }
    pub fn ret_flt(&mut self, v: f64) {
        self.cpu.fpr[1] = v.to_bits();
    }
    pub fn ret_ptr(&mut self, p: *mut c_void, rec: Option<&'static str>) {
        match self.mem.handle(p as usize, rec) {
            Ok(h) => self.cpu.gpr[3] = h,
            Err(_) => {
                self.cpu.gpr[3] = 0;
                self.bad = true;
            }
        }
    }
    fn finish(&mut self) {
        for (h, mut buf, rec, base) in std::mem::take(&mut self.fix) {
            let gs = self.mem.layouts.gc_size(rec).unwrap_or(0);
            copy_record(self.mem, base, h, gs);
            self.mem.forget_native(buf.as_mut_ptr() as *mut u8);
        }
    }
}

/// Copy a record's `gs` guest bytes from `src` to `dst`, a word at a time where a word moves (a pointer field only
/// moves whole: a byte store to one faults, and the byte loop this replaces left every pointer of a marshalled record
/// NULL - Wolf's Laser SpawnItem reached Item_8026862C with no parent, so the laser had no owner and its hits never
/// credited Wolf's last attack), a byte at a time otherwise.
fn copy_record(mem: &mut GuestSpace, src: u32, dst: u32, gs: u32) {
    let mut k = 0;
    while k < gs {
        if k % 4 == 0 && k + 4 <= gs {
            if let Ok(w) = mem.ld32(src + k) {
                if mem.st32(dst + k, w).is_ok() {
                    k += 4;
                    continue;
                }
            }
        }
        for j in k..(k + 4 - k % 4).min(gs) {
            if let Ok(b) = mem.ld8(src + j) {
                let _ = mem.st8(dst + j, b);
            }
        }
        k += 4 - k % 4;
    }
}

/// Is a guest code callback behind this native function pointer (a pool trampoline)?
pub fn is_guest_fn(p: usize) -> bool {
    fn_guest(p).is_some()
}

/// The per-fighter counters (`MELEE_MEX_REPORT`, the slpcheck summary).
pub fn report() -> String {
    let Some(s) = (unsafe { ST.as_ref() }) else { return String::new() };
    let mut o = String::new();
    for (kind, f) in &s.kinds {
        o += &format!("mexguest: kind {kind} {} ({} bytes)\n", f.file, f.blob.code.len());
        for (slot, st) in &f.stats {
            let name = if *slot >= 2000 {
                "fn-pointer".to_string()
            } else if *slot >= 1000 {
                format!("MoveLogic {}", ["anim", "input", "phys", "coll"][(*slot - 1000) as usize])
            } else {
                SLOT_NAMES.get(*slot as usize).copied().unwrap_or("?").to_string()
            };
            o += &format!("mexguest:   {name}: {} calls, {} returned, {} insns", st.calls, st.returned, st.insns);
            for (k, v) in &st.faults {
                o += &format!("; {k} x{v}");
            }
            o += "\n";
        }
    }
    for (t, n) in &s.unresolved {
        o += &format!("mexguest: unresolved {t:08X} {} x{n}\n", mex_import::layout::vanilla_name(*t).unwrap_or("?"));
    }
    for (t, n) in &s.bound {
        o += &format!("mexguest: bound {t:08X} {} x{n}\n", mex_import::layout::vanilla_name(*t).unwrap_or("?"));
    }
    o
}
