//! Whole-TU native ports of the item engine's `.c` files (shards 641-648). Each file ports every function of
//! one `src/melee/it/<stem>.c` (statics and inlines included) with the C's name and signature; callees of other
//! TUs go through the file's generated `ext` hooks (unset: panics "TODO"). Checked by tests/tu_<stem>.rs against
//! the C (gen/mtu).
pub mod jobj;
pub mod ithitbox;
pub mod itdrop;
pub mod iteffect;
pub mod itdraw;
pub mod itcoll;
pub mod itgroundcoll;
pub mod itmaplib;
pub mod item;
pub mod itanimlist;

// Shard 657-664: whole-TU native ports of the item kinds' `.c` files (shard 657-664). Each file ports every function of one
// `src/melee/it/kinds/<stem>.c` (statics and header inlines included) with the C's name and signature;
// callees of other TUs go through the file's generated `ext` hooks (unset: panics "TODO"). Checked by
// tests/kinds_<stem>.rs against the C (gen/kinds).
pub mod kinds_inl;
pub mod itarwinglaser;
pub mod itbat;
pub mod itcapsule;
pub mod itcerebi;
pub mod itchicorita;
pub mod itclimbersblizzard;
pub mod itbombhei;
pub mod itbox;

// Shard 769-776: it/kinds itpeach*, itpikachu*, itpippi (same oracle as above: gen/kinds, tests/kinds_<stem>.rs).
pub mod itpeachparasol;
pub mod itpeachtoad;
pub mod itpeachtoadspore;
pub mod itpeachturnip;
pub mod itpikachuthunder;
pub mod itpikachutjoltair;
pub mod itpikachutjoltground;
pub mod itpippi;

pub mod itegg;
pub mod itentei;
pub mod itevyoshiegg;
pub mod itfflower;
pub mod itfflowerflame;
pub mod itfoods;
pub mod itflipper;
pub mod itfire;
pub mod itgamewatchfire;
pub mod itgamewatchgreenhouse;
pub mod itgamewatchjudge;
pub mod itgamewatchmanhole;
pub mod itgamewatchpanic;
pub mod itgamewatchparachute;
pub mod itgamewatchrescue;
pub mod itgamewatchturtle;

// Shard 649-656 (generator copy under gen/mtu649).
// Whole-TU native ports of the item engine .c files (shard 649-656).

/// The `static inline` HSD_JObj setters of sysdolphin/baselib/jobj.h and the item-kind spawn helpers of
/// it/kinds/inlines.h, as every including TU compiles them (each TU's own `ext` holds the out-of-line
/// callee `HSD_JObjSetMtxDirtySub`). Invoked inside a TU module that has `ext::HSD_JObjSetMtxDirtySub`.
macro_rules! jobj_inl {
    () => {
        #[allow(dead_code)]
        unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
            assert!(!jobj.is_null(), "HSD_ASSERT(564, jobj)");
            !((*jobj).flags & JOBJ_USER_DEF_MTX as u32 != 0) && ((*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0)
        }
        /// `HSD_JObjSetMtxDirty` (macro): `if (jobj != NULL && !HSD_JObjMtxIsDirty(jobj)) HSD_JObjSetMtxDirtySub(jobj)`.
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
            if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
            if jobj.is_null() { core::ptr::null_mut() } else { (*jobj).child }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
            assert!(!jobj.is_null(), "HSD_ASSERT(618, jobj)");
            assert!(!rotate.is_null(), "HSD_ASSERT(619, rotate)");
            (*jobj).rotate = *rotate;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
            assert!(!jobj.is_null(), "HSD_ASSERT(699, jobj)");
            assert!(!rotate.is_null(), "HSD_ASSERT(700, rotate)");
            *rotate = (*jobj).rotate;
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, x: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(639, jobj)");
            assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(640, !(jobj->flags & JOBJ_USE_QUATERNION))");
            (*jobj).rotate.x = x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(660, jobj)");
            assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(661, !(jobj->flags & JOBJ_USE_QUATERNION))");
            (*jobj).rotate.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(681, jobj)");
            assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(682, !(jobj->flags & JOBJ_USE_QUATERNION))");
            (*jobj).rotate.z = z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *mut Vec3) {
            assert!(!jobj.is_null(), "HSD_ASSERT(760, jobj)");
            assert!(!scale.is_null(), "HSD_ASSERT(761, scale)");
            (*jobj).scale = *scale;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetScaleY(jobj: *mut HSD_JObj) -> f32 {
            assert!(!jobj.is_null(), "HSD_ASSERT(888, jobj)");
            (*jobj).scale.y
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            assert!(!jobj.is_null(), "HSD_ASSERT(916, jobj)");
            assert!(!translate.is_null(), "HSD_ASSERT(917, translate)");
            (*jobj).translate = *translate;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            assert!(!jobj.is_null(), "HSD_ASSERT(979, jobj)");
            assert!(!translate.is_null(), "HSD_ASSERT(980, translate)");
            *translate = (*jobj).translate;
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScaleX(jobj: *mut HSD_JObj, x: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(776, jobj)");
            (*jobj).scale.x = x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScaleY(jobj: *mut HSD_JObj, y: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(791, jobj)");
            (*jobj).scale.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScaleZ(jobj: *mut HSD_JObj, z: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(806, jobj)");
            (*jobj).scale.z = z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
    };
}

/// it/kinds/inlines.h spawn helpers (`Item_InitSpawnPosition`, `Item_InitSpawnCommonFields`).
macro_rules! spawn_inl {
    () => {
        #[allow(dead_code)]
        unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
            (*spawn).prev_pos = *pos;
            if on_plane {
                (*spawn).prev_pos.z = 0.0;
            }
            (*spawn).pos = (*spawn).prev_pos;
        }
        #[allow(dead_code)]
        unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
            (*spawn).facing_dir = facing_dir;
            (*spawn).x3C_damage = 0;
            (*spawn).vel.x = 0.0;
            (*spawn).vel.y = 0.0;
            (*spawn).vel.z = 0.0;
            (*spawn).x0_parent_gobj = parent;
            (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
            (*spawn).x44_flag.anon_0.set_b0(initial_collision as _);
            (*spawn).x40 = 0;
        }
    };
}

/// `Item_InitSpawnPositionFromParent` (it/kinds/inlines.h): needs `ext::it_8026BB68`.
macro_rules! spawn_parent_inl {
    () => {
        #[allow(dead_code)]
        unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
            (*spawn).prev_pos = *pos;
            (*spawn).prev_pos.z = 0.0;
            ext::it_8026BB68(parent, &mut (*spawn).pos);
        }
    };
}

pub mod itclimbersice;
pub mod itclinkmilk;
pub mod itcrazyhandbomb;
pub mod itdkinoko;
pub mod itcoin;
pub mod itclimbersstring;
pub mod itdrmariopill;
pub mod itdosei;
pub mod itmaterial;
pub mod itspawn;
pub mod itzako;
pub mod it_26B1;
pub mod it_2725;
pub mod it_279C;
pub mod it_3F14;
pub mod it_3F2F;
pub mod itfoxblaster;
pub mod itfoxillusion;
pub mod itfoxlaser;
pub mod itfreeze;
pub mod itfreezer;
pub mod itfushigibana;
pub mod itgamewatchbreath;
pub mod itgamewatchchef;

// Shard 697-704.

pub mod inl;
pub mod itheart;
pub mod itgreatfoxlaser;
pub mod ithammerhead;
pub mod ithammer;
pub mod itharisen;
pub mod itgshell;
pub mod ithassam;
pub mod itheiho;

pub mod ithinoarashi;
pub mod ithitodeman;
pub mod ithouou;
pub mod itkabigon;
pub mod itkamex;
pub mod itkinoko;
pub mod itkirbycutterbeam;
pub mod itkirbygamewatchchefpan;
pub mod itkirbyhammer;
pub mod itkirbyyoshispecialn;
pub mod itkirby_2F23;
pub mod itkireihana;
pub mod itklap;
pub mod itkoopaflame;
pub mod itkyasarin;
pub mod itkusudama;
// Shard 721-728
pub mod itkyasarinegg;
pub mod itlgun;
pub mod itlgunray;
pub mod itlgunbeam;
pub mod itleadead;
pub mod itlikelike;
pub mod itlinkarrow;
pub mod itlinkbomb;
pub mod inl_it;
pub mod itlinkbow;
pub mod itlipstick;
pub mod itlipstickspore;
pub mod itlizardon;
pub mod itlucky;
pub mod itlugia;
pub mod itlinkboomerang;
pub mod itlinkhookshot;
pub mod itluigifireball;
pub mod itmaril;
pub mod itmariocape;
pub mod itmariofireball;
pub mod itmarumine;
pub mod itmasterhandbullet;
pub mod itmasterhandlaser;
pub mod itmatadogas;
pub mod itnessbat;
pub mod itnesspkfire;
pub mod itnesspkfirepillar;
pub mod itnesspkflash;
pub mod itnesspkflashexplode;
pub mod itnesspkthunderball;
pub mod itnesspkthundertrail;
pub mod itnessyoyo;
pub mod itpeachexplode;
pub mod itparasol;
pub mod itoctarockstone;
pub mod itoctarock;
pub mod itoldkuri;
pub mod itoldottosea;
pub mod itpatapata;
pub mod itnokonoko;
// Shard 745-752:
pub mod itmato;
pub mod itmetalb;
pub mod itmetamon;
pub mod itmew;
pub mod itmewtwodisable;
pub mod itmball;
pub mod itmsbomb;
pub mod itmewtwoshadowball;
pub mod itporygon2;
pub mod itrabbitc;
pub mod itraikou;
pub mod itsamusmissile;
pub mod itsamusbomb;
pub mod itsamuschargeshot;
pub mod itrshell;
pub mod itsamusgrapple;
pub mod itscball;
pub mod itseakchain;
pub mod itseakneedleheld;
pub mod itseakneedlethrown;
pub mod itseakvanish;
pub mod itsonans;
pub mod itspycloak;
pub mod itsscope;
pub mod itsscopebeam;
pub mod itstar;
pub mod itstarrod;
pub mod itstarrodstar;
pub mod itsuikun;
pub mod itsword;
pub mod ittaru;
pub mod ittarucann;
// Shard 801-808.
pub mod itthunder;
pub mod ittogepy;
pub mod ittomato;
pub mod ittosakinto;
pub mod ittools;
pub mod itunknown;
pub mod itwhispyapple;
pub mod ittincle;
pub mod ityoshitongue;
pub mod itwhitebea;
pub mod itwstar;
pub mod ityaku;
pub mod ityoshiegglay;
pub mod ityoshieggthrow;
pub mod ityoshistar;
pub mod itzeldadinfire;
// Shard 817-824: whole-TU ports of itzeldadinfireexplode, itzgshell, itzrshell, it_2ADA, it_2E5A, it_2F28.
pub mod itzeldadinfireexplode;
pub mod itzgshell;
pub mod itzrshell;
pub mod it_2ADA;
pub mod it_2E5A;
pub mod it_2F28;
pub mod toy;
pub mod tydisplay;
pub mod tyfigupon;
// Shard 897-904: whole-TU native ports of the vi (visual scenes) and ty/tylist TUs.
pub mod vi;
pub mod vi0102;
pub mod vi0401;
pub mod vi0402;
pub mod vi0501;
pub mod vi0502;
pub mod vi0601;
pub mod tylist;
pub mod vi0801;
pub mod vi1101;
pub mod vi1201v1;
pub mod vi1201v2;
pub mod vi1202;
