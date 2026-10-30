//! The sysdolphin GObj system: process links, GX (render) links, per-gobj procs and the frame's
//! proc walk. Native port of `src/sysdolphin/baselib/gobj.c`, `gobjplink.c`, `gobjgxlink.c`,
//! `gobjproc.c`, `gobjobject.c`, `gobjuserdata.c` and `gobjinit.c`.
//!
//! Ownership: the C keeps intrusive doubly linked lists of `HSD_GObj*` / `HSD_GObjProc*` in
//! `HSD_ObjAlloc` pools plus file-scope globals. Here one [`GObjSystem`] owns both pools (slot
//! arenas addressed by [`GObjId`] / [`ProcId`]) and every global; the list links are ids. The
//! link order, every list head/tail, the proc-queue order, the 2-bit `flags_3` phase marks and
//! the delayed-request block evolve exactly as in the C (oracle: `tests/oracle_gobj.rs`).
//!
//! Function pointers become type parameters: `F` is the callback token stored in
//! `on_invoke`, `render_cb`, `user_data_remove_func` and the obj-kind table, `O` the opaque
//! object handle stored in `hsd_obj` / `user_data`. Calling a callback goes through
//! [`GObjHost`], which receives `&mut GObjSystem` so callbacks may re-enter the system as the C
//! callbacks do.
//!
//! PC-only hooks of the C (`ScriptGame_StageSlot*`, `Snap_*`, `HSD_RandRender*`) are host
//! methods with the no-script defaults. The profiler zones and `gobj_trace_render` logging are
//! observability only and are not ported.
//!
//! Not ported (need other HSD objects): `HSD_GObj_LObjCallback`, `HSD_GObj_JObjCallback`,
//! `HSD_GObj_FogCallback`, `HSD_GObj_803910D8` (CObj), `HSD_GObj_80391120` / `HSD_GObj_803911C0`
//! (HSD_Obj refcount release); the obj-kind table names them as [`KindFunc`] built-ins, which
//! the host executes. TODO(group1-hsd-objects).
use melee_core::gc::{GcField, GcLayout};

pub const HSD_GOBJ_GXLINK_NONE: u8 = 0xFF;
pub const HSD_GOBJ_OBJ_NONE: u8 = 0xFF;
pub const HSD_GOBJ_USER_DATA_NONE: u8 = 0xFF;
pub const HSD_GOBJ_PLINK_MAX: u8 = 63;
pub const HSD_GOBJ_GX_LINK_MAX: u8 = 63;
pub const HSD_GOBJPROC_PRI_MAX: u8 = 2;

/// An `HSD_GObj*` (slot index in [`GObjSystem`]'s gobj pool).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct GObjId(pub u32);
/// An `HSD_GObjProc*` (slot index in [`GObjSystem`]'s proc pool).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ProcId(pub u32);

macro_rules! delegate_layout {
    ($ty:ty, $gc:ty) => {
        impl<F, O> GcLayout for $ty {
            const GC_NAME: &'static str = <$gc as GcLayout>::GC_NAME;
            const GC_SIZE: u32 = <$gc as GcLayout>::GC_SIZE;
            const GC_ALIGN: u32 = <$gc as GcLayout>::GC_ALIGN;
            const GC_FIELDS: &'static [GcField] = <$gc as GcLayout>::GC_FIELDS;
        }
    };
}

/// Native `HSD_GObj` (GameCube record `melee_types::HSD_GObj`, 0x38 bytes; see `GcLayout`).
#[derive(Clone, Debug, PartialEq)]
pub struct HsdGObj<F, O> {
    /* +0 */ pub classifier: u16,
    /* +2 */ pub p_link: u8,
    /* +3 */ pub gx_link: u8,
    /* +4 */ pub p_priority: u8,
    /* +5 */ pub render_priority: u8,
    /* +6 */ pub obj_kind: u8,
    /* +7 */ pub user_data_kind: u8,
    /* +8 */ pub next: Option<GObjId>,
    /* +C */ pub prev: Option<GObjId>,
    /* +10 */ pub next_gx: Option<GObjId>,
    /* +14 */ pub prev_gx: Option<GObjId>,
    /* +18 */ pub proc_: Option<ProcId>,
    /* +1C */ pub render_cb: Option<F>,
    /* +20 */ pub gxlink_prios: u64,
    /* +28 */ pub hsd_obj: Option<O>,
    /* +2C */ pub user_data: Option<O>,
    /* +30 */ pub user_data_remove_func: Option<F>,
    /* +34 */ pub x34_unk: u32,
}
delegate_layout!(HsdGObj<F, O>, melee_types::HSD_GObj);

/// Native `HSD_GObjProc` (GameCube record `melee_types::HSD_GObjProc`, 0x18 bytes). The four
/// bitfields at +0xD are separate fields (1, 1, 2, 2 bits; values kept masked).
#[derive(Clone, Debug, PartialEq)]
pub struct HsdGObjProc<F> {
    /* 0x00 */ pub child: Option<ProcId>,
    /* 0x04 */ pub next: Option<ProcId>,
    /* 0x08 */ pub prev: Option<ProcId>,
    /* 0x0C */ pub s_link: u8,
    /* 0x0D */ pub flags_1: u8,
    pub flags_2: u8,
    pub flags_3: u8,
    pub flags_4: u8,
    /* 0x10 */ pub gobj: GObjId,
    /* 0x14 */ pub on_invoke: F,
}
impl<F> GcLayout for HsdGObjProc<F> {
    const GC_NAME: &'static str = <melee_types::HSD_GObjProc as GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_GObjProc as GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_GObjProc as GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [GcField] = <melee_types::HSD_GObjProc as GcLayout>::GC_FIELDS;
}

/// One entry of the obj-kind removal table (`HSD_GObj_804D7810`, `GObjFunc`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KindFunc<F> {
    /// A function from the caller's `GObjFuncs` chain.
    User(F),
    /// `HSD_GObj_80391120` (camera kind: HSD_Obj refcount release).
    ObjRelease,
    /// `HSD_LObjRemoveAll` (light kind).
    LObjRemoveAll,
    /// `HSD_JObjRemoveAll` (jobj kind).
    JObjRemoveAll,
    /// `HSD_GObj_803911C0` (fog kind: same release as the camera kind).
    ObjRelease2,
}

/// The default table gobj.c appends (`HSD_GObj_80408600` / `HSD_GObj_80408610`).
pub const HSD_GOBJ_BUILTIN_FUNCS: [KindFunc<()>; 4] =
    [KindFunc::ObjRelease, KindFunc::LObjRemoveAll, KindFunc::JObjRemoveAll, KindFunc::ObjRelease2];

/// `HSD_GObjLibInitDataType` (GameCube record `_HSD_GObjLibInitDataType`, 0x10 bytes). The
/// `GObjFuncs` chain is a vector of lists (each list's `size` is a `u8` in the C). `unk_2` (a
/// pointer to the pause mask, read every RunProcs) is [`GObjSystem::pause_mask`].
#[derive(Clone, Debug, PartialEq)]
pub struct HsdGObjLibInitData<F> {
    pub p_link_max: u8,
    pub gx_link_max: u8,
    pub gproc_pri_max: u8,
    pub funcs: Vec<Vec<KindFunc<F>>>,
}
impl<F> GcLayout for HsdGObjLibInitData<F> {
    const GC_NAME: &'static str = <melee_types::_HSD_GObjLibInitDataType as GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::_HSD_GObjLibInitDataType as GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::_HSD_GObjLibInitDataType as GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [GcField] = <melee_types::_HSD_GObjLibInitDataType as GcLayout>::GC_FIELDS;
}

/// `HSD_GObjSetInitDefaults`.
pub fn hsd_gobj_set_init_defaults<F>() -> HsdGObjLibInitData<F> {
    HsdGObjLibInitData {
        p_link_max: HSD_GOBJ_PLINK_MAX,
        gx_link_max: HSD_GOBJ_GX_LINK_MAX,
        gproc_pri_max: HSD_GOBJPROC_PRI_MAX,
        funcs: Vec::new(),
    }
}

/// `HSD_GObj_DelayedProcInfo` (`struct _unk_gobj_struct`, 0x10 bytes on the GC). `flags` is the
/// union of the four bits; clearing it (`flags = 0`) leaves `type_`..`gobj` as they were.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DelayedProcInfo {
    pub in_delayed_proc: bool,
    pub delay_remove_gobj: bool,
    pub delay_remove_proc: bool,
    pub delay_change_gobj_pri: bool,
    pub type_: u32,
    pub p_link: u8,
    pub p_prio: u8,
    pub gobj: Option<GObjId>,
}
impl GcLayout for DelayedProcInfo {
    const GC_NAME: &'static str = <melee_types::_unk_gobj_struct as GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::_unk_gobj_struct as GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::_unk_gobj_struct as GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [GcField] = <melee_types::_unk_gobj_struct as GcLayout>::GC_FIELDS;
}
impl DelayedProcInfo {
    /// The union word as the GameCube stores it (bit 31 = `in_delayed_proc`, MSB-first bitfields).
    pub fn flags(&self) -> u32 {
        (self.in_delayed_proc as u32) << 31
            | (self.delay_remove_gobj as u32) << 30
            | (self.delay_remove_proc as u32) << 29
            | (self.delay_change_gobj_pri as u32) << 28
    }
    fn any(&self) -> bool {
        self.in_delayed_proc || self.delay_remove_gobj || self.delay_remove_proc || self.delay_change_gobj_pri
    }
    fn clear_flags(&mut self) {
        self.in_delayed_proc = false;
        self.delay_remove_gobj = false;
        self.delay_remove_proc = false;
        self.delay_change_gobj_pri = false;
    }
}

/// What the C reaches through function pointers. Every method gets the system back so a
/// callback can create, free or relink gobjs as the game's callbacks do.
pub trait GObjHost<F: Copy, O: Copy> {
    /// `proc->on_invoke(proc->gobj)` in `HSD_GObj_RunProcs` (PC: the host also owns
    /// `ScriptGame_StageSlotProcBegin/End` and the `ScriptStageHostHidden` skip).
    fn invoke(&mut self, sys: &mut GObjSystem<F, O>, gobj: GObjId, func: F);
    /// `cur->render_cb(cur, pass)`.
    fn render(&mut self, sys: &mut GObjSystem<F, O>, gobj: GObjId, func: F, pass: i32);
    /// `gobj->user_data_remove_func(gobj->user_data)`.
    fn remove_user_data(&mut self, sys: &mut GObjSystem<F, O>, func: F, data: Option<O>);
    /// `HSD_GObj_804D7810[kind](gobj->hsd_obj)`.
    fn remove_obj(&mut self, sys: &mut GObjSystem<F, O>, func: KindFunc<F>, obj: Option<O>);
    /// PC `ScriptGame_StageSlotCanCreate`.
    fn stage_slot_can_create(&mut self, _sys: &mut GObjSystem<F, O>) -> bool {
        true
    }
    /// PC `ScriptGame_StageSlotCreated`.
    fn stage_slot_created(&mut self, _sys: &mut GObjSystem<F, O>, _gobj: GObjId) {}
    /// PC `ScriptGame_StageSlotDestroyed`.
    fn stage_slot_destroyed(&mut self, _sys: &mut GObjSystem<F, O>, _gobj: GObjId) {}
    /// PC `Snap_SkipRenderCb` (max-link walk only).
    fn snap_skip_render_cb(&mut self, _func: F) -> bool {
        false
    }
    /// `HSD_RandRenderBegin` (PC render random stream).
    fn rand_render_begin(&mut self) {}
    /// `HSD_RandRenderEnd`.
    fn rand_render_end(&mut self) {}
}

/// All of gobj.c's globals plus the gobj and proc pools.
#[derive(Clone, Debug)]
pub struct GObjSystem<F, O> {
    gobjs: Vec<Option<HsdGObj<F, O>>>,
    free_gobjs: Vec<u32>,
    procs: Vec<Option<HsdGObjProc<F>>>,
    free_procs: Vec<u32>,
    /// `HSD_GObjLibInitData` (minus the funcs chain, which `init` flattens into `kind_funcs`).
    pub p_link_max: u8,
    pub gx_link_max: u8,
    pub gproc_pri_max: u8,
    /// `*HSD_GObjLibInitData.unk_2` (0 when the pointer is NULL): bit n pauses p_link n.
    pub pause_mask: u64,
    /// `HSD_GObjPLinkHead`.
    pub plink_head: Vec<Option<GObjId>>,
    /// `plinklow_gobjs` (tail of each p_link list).
    pub plink_low: Vec<Option<GObjId>>,
    /// `HSD_GObjGXLinkHead`.
    pub gxlink_head: Vec<Option<GObjId>>,
    /// `HSD_GObj_804D7820` (tail of each gx_link list).
    pub gxlink_low: Vec<Option<GObjId>>,
    /// `HSD_GObj_GObjProcHead` (one queue per s_link).
    pub proc_head: Vec<Option<ProcId>>,
    /// `HSD_GObj_ProcList[p_link + s_link * (p_link_max + 1)]`: last proc of that p_link in the
    /// s_link queue.
    pub proc_list: Vec<Option<ProcId>>,
    /// `HSD_GObj_804D783C`: the RunProcs phase, 0..=2.
    pub phase: i32,
    pub current_invoked_proc: Option<ProcId>,
    pub current_invoked_s_link: i32,
    pub next_invoked_proc: Option<ProcId>,
    pub current_invoked_proc_gobj: Option<GObjId>,
    /// `HSD_GObj_804D7818` (gobj being rendered by the max-link walk).
    pub gobj_804d7818: Option<GObjId>,
    /// `HSD_GObj_804D7814` (gobj being rendered by the camera walk).
    pub gobj_804d7814: Option<GObjId>,
    /// `HSD_GObj_804D7810` (NULL when empty).
    pub kind_funcs: Vec<KindFunc<F>>,
    pub camera_kind: u8,
    pub light_kind: i8,
    pub jobj_kind: u8,
    pub fog_kind: i8,
    pub delayed: DelayedProcInfo,
}

impl<F: Copy, O: Copy> Default for GObjSystem<F, O> {
    fn default() -> Self {
        Self::new()
    }
}

/// `HSD_GObj_804085F0` / `HSD_GObj_80390EB8` (render-pass flag table).
pub const HSD_GOBJ_804085F0: [i32; 4] = [1, 4, 2, 0];
pub use crate::tu::gobj::HSD_GObj_80390EB8 as hsd_gobj_80390eb8;

pub use crate::tu::gobj::HSD_GObj_803912A8 as hsd_gobj_803912a8;

impl<F: Copy, O: Copy> GObjSystem<F, O> {
    /// The state before `HSD_GObjInit` (all globals zero).
    pub fn new() -> Self {
        GObjSystem {
            gobjs: Vec::new(),
            free_gobjs: Vec::new(),
            procs: Vec::new(),
            free_procs: Vec::new(),
            p_link_max: 0,
            gx_link_max: 0,
            gproc_pri_max: 0,
            pause_mask: 0,
            plink_head: Vec::new(),
            plink_low: Vec::new(),
            gxlink_head: Vec::new(),
            gxlink_low: Vec::new(),
            proc_head: Vec::new(),
            proc_list: Vec::new(),
            phase: 0,
            current_invoked_proc: None,
            current_invoked_s_link: 0,
            next_invoked_proc: None,
            current_invoked_proc_gobj: None,
            gobj_804d7818: None,
            gobj_804d7814: None,
            kind_funcs: Vec::new(),
            camera_kind: 0,
            light_kind: 0,
            jobj_kind: 0,
            fog_kind: 0,
            delayed: DelayedProcInfo::default(),
        }
    }

    // ---- pools ----

    pub fn gobj(&self, id: GObjId) -> &HsdGObj<F, O> {
        self.gobjs[id.0 as usize].as_ref().expect("dangling HSD_GObj")
    }
    pub fn gobj_mut(&mut self, id: GObjId) -> &mut HsdGObj<F, O> {
        self.gobjs[id.0 as usize].as_mut().expect("dangling HSD_GObj")
    }
    pub fn proc_(&self, id: ProcId) -> &HsdGObjProc<F> {
        self.procs[id.0 as usize].as_ref().expect("dangling HSD_GObjProc")
    }
    pub fn proc_mut(&mut self, id: ProcId) -> &mut HsdGObjProc<F> {
        self.procs[id.0 as usize].as_mut().expect("dangling HSD_GObjProc")
    }
    pub fn is_live(&self, id: GObjId) -> bool {
        matches!(self.gobjs.get(id.0 as usize), Some(Some(_)))
    }
    fn alloc_gobj(&mut self, g: HsdGObj<F, O>) -> GObjId {
        if let Some(i) = self.free_gobjs.pop() {
            self.gobjs[i as usize] = Some(g);
            GObjId(i)
        } else {
            self.gobjs.push(Some(g));
            GObjId(self.gobjs.len() as u32 - 1)
        }
    }
    fn alloc_proc(&mut self, p: HsdGObjProc<F>) -> ProcId {
        if let Some(i) = self.free_procs.pop() {
            self.procs[i as usize] = Some(p);
            ProcId(i)
        } else {
            self.procs.push(Some(p));
            ProcId(self.procs.len() as u32 - 1)
        }
    }

    // ---- gobjinit.c / gobj.c init ----

    /// `HSD_GObj_80391260`: append the four built-in kind functions and number the kinds.
    pub fn hsd_gobj_80391260(&mut self, data: &mut HsdGObjLibInitData<F>) {
        let builtins = HSD_GOBJ_BUILTIN_FUNCS
            .iter()
            .map(|k| match k {
                KindFunc::ObjRelease => KindFunc::ObjRelease,
                KindFunc::LObjRemoveAll => KindFunc::LObjRemoveAll,
                KindFunc::JObjRemoveAll => KindFunc::JObjRemoveAll,
                _ => KindFunc::ObjRelease2,
            })
            .collect();
        let mut count = hsd_gobj_803912a8(data, builtins);
        self.camera_kind = count;
        count = count.wrapping_add(1);
        self.light_kind = count as i8;
        count = count.wrapping_add(1);
        self.jobj_kind = count;
        count = count.wrapping_add(1);
        self.fog_kind = count as i8;
    }

    /// `HSD_GObjInit`. Like the C it appends the built-ins to `data.funcs`. Every gobj and proc
    /// of a previous init is dropped (the C leaks them in their old pools).
    pub fn init(&mut self, data: &mut HsdGObjLibInitData<F>) {
        self.hsd_gobj_80391260(data);
        self.p_link_max = data.p_link_max;
        self.gx_link_max = data.gx_link_max;
        self.gproc_pri_max = data.gproc_pri_max;
        let np = data.p_link_max as usize + 1;
        let ng = data.gx_link_max as usize + 2;
        let ns = data.gproc_pri_max as usize + 1;
        self.plink_head = vec![None; np];
        self.plink_low = vec![None; np];
        self.gxlink_head = vec![None; ng];
        self.gxlink_low = vec![None; ng];
        self.proc_head = vec![None; ns];
        self.proc_list = vec![None; ns * np];
        self.gobjs.clear();
        self.free_gobjs.clear();
        self.procs.clear();
        self.free_procs.clear();
        // The C sums the u8 `size` fields into an int and copies that many entries.
        self.kind_funcs = data.funcs.iter().flat_map(|l| l.iter().copied()).collect();
        self.phase = 0;
        self.current_invoked_proc_gobj = None;
        self.current_invoked_proc = None;
        self.delayed.clear_flags();
        self.gobj_804d7818 = None;
        self.gobj_804d7814 = None;
    }

    // ---- gobjplink.c ----

    /// `GObj_PReorder`: link `gobj` after `hiprio` (None: at the head) in its p_link list.
    pub fn p_reorder(&mut self, gobj: GObjId, hiprio: Option<GObjId>) {
        let link = self.gobj(gobj).p_link as usize;
        self.gobj_mut(gobj).prev = hiprio;
        let next = match hiprio {
            Some(h) => {
                let n = self.gobj(h).next;
                self.gobj_mut(h).next = Some(gobj);
                n
            }
            None => {
                let n = self.plink_head[link];
                self.plink_head[link] = Some(gobj);
                n
            }
        };
        self.gobj_mut(gobj).next = next;
        match next {
            Some(n) => self.gobj_mut(n).prev = Some(gobj),
            None => self.plink_low[link] = Some(gobj),
        }
    }

    /// gobjplink.c `gobj_first_lower_prio` (static inline).
    fn first_lower_prio(&mut self, gobj: GObjId) {
        let (pl, pr) = (self.gobj(gobj).p_link as usize, self.gobj(gobj).p_priority);
        let mut c = self.plink_low[pl];
        while let Some(x) = c {
            if self.gobj(x).p_priority > pr {
                c = self.gobj(x).prev;
            } else {
                break;
            }
        }
        self.p_reorder(gobj, c);
    }

    /// gobjplink.c `gobj_first_higher_prio` (static inline).
    fn first_higher_prio(&mut self, gobj: GObjId) {
        let (pl, pr) = (self.gobj(gobj).p_link as usize, self.gobj(gobj).p_priority);
        let mut c = self.plink_head[pl];
        while let Some(x) = c {
            if self.gobj(x).p_priority < pr {
                c = self.gobj(x).next;
            } else {
                break;
            }
        }
        let at = match c {
            Some(x) => self.gobj(x).prev,
            None => self.plink_low[pl],
        };
        self.p_reorder(gobj, at);
    }

    fn insert_where(&mut self, where_: u32, gobj: GObjId, position: Option<GObjId>) {
        match where_ {
            0 => self.first_lower_prio(gobj),
            1 => self.first_higher_prio(gobj),
            2 => self.p_reorder(gobj, position),
            3 => {
                let p = self.gobj(position.expect("CreateGObj where=3 needs a position")).prev;
                self.p_reorder(gobj, p)
            }
            _ => {}
        }
    }

    fn unlink_p(&mut self, gobj: GObjId) {
        let (prev, next, pl) = {
            let g = self.gobj(gobj);
            (g.prev, g.next, g.p_link as usize)
        };
        match prev {
            Some(p) => self.gobj_mut(p).next = next,
            None => self.plink_head[pl] = next,
        }
        match next {
            Some(n) => self.gobj_mut(n).prev = prev,
            None => self.plink_low[pl] = prev,
        }
    }

    /// `CreateGObj`. `where_`: 0 after the last gobj of priority <= `priority`, 1 before the
    /// first of priority >= `priority`, 2 after `position`, 3 before `position`; other values
    /// leave the gobj unlinked (as the C). None when the PC stage-slot hook refuses.
    pub fn create_gobj(
        &mut self,
        host: &mut impl GObjHost<F, O>,
        where_: i32,
        classifier: u16,
        p_link: u8,
        priority: u8,
        position: Option<GObjId>,
    ) -> Option<GObjId> {
        if !host.stage_slot_can_create(self) {
            return None;
        }
        assert!(p_link <= self.p_link_max, "gobjplink.c:0xA8 p_link <= p_link_max");
        let gobj = self.alloc_gobj(HsdGObj {
            classifier,
            p_link,
            gx_link: HSD_GOBJ_GXLINK_NONE,
            p_priority: priority,
            render_priority: 0,
            obj_kind: HSD_GOBJ_OBJ_NONE,
            user_data_kind: 0xFF,
            next: None,
            prev: None,
            next_gx: None,
            prev_gx: None,
            proc_: None,
            render_cb: None,
            gxlink_prios: 0,
            hsd_obj: None,
            user_data: None,
            user_data_remove_func: None,
            x34_unk: 0,
        });
        self.insert_where(where_ as u32, gobj, position);
        host.stage_slot_created(self, gobj);
        Some(gobj)
    }

    /// `GObj_Create`.
    pub fn gobj_create(
        &mut self,
        host: &mut impl GObjHost<F, O>,
        classifier: u16,
        p_link: u8,
        priority: u8,
    ) -> Option<GObjId> {
        self.create_gobj(host, 0, classifier, p_link, priority, None)
    }

    /// `HSD_GObjFree`. Inside the gobj's own proc this only requests the free (done after the
    /// proc returns).
    pub fn free(&mut self, host: &mut impl GObjHost<F, O>, gobj: GObjId) {
        if !self.delayed.in_delayed_proc && Some(gobj) == self.current_invoked_proc_gobj {
            self.delayed.delay_remove_gobj = true;
            return;
        }
        host.stage_slot_destroyed(self, gobj);
        self.remove_user_data(host, gobj);
        self.gobjobject_80390b0c(host, gobj);
        self.remove_all_procs(gobj);
        if self.gobj(gobj).gx_link != HSD_GOBJ_GXLINK_NONE {
            self.gxlink_8039084c(gobj);
        }
        self.unlink_p(gobj);
        self.gobjs[gobj.0 as usize] = None;
        self.free_gobjs.push(gobj.0);
    }

    /// `HSD_GObjPLink_ChangeGObjPri_Unk`: move `gobj` to `p_link`/`priority` (placement as
    /// `create_gobj`'s `where_`), requeueing its procs. Delayed when `gobj` is running.
    pub fn change_gobj_pri(&mut self, type_: u32, gobj: GObjId, p_link: u8, priority: u8, position: Option<GObjId>) {
        assert!(p_link <= self.p_link_max, "gobjplink.c:0x1A3 p_link <= p_link_max");
        if !self.delayed.in_delayed_proc && Some(gobj) == self.current_invoked_proc_gobj {
            self.delayed.delay_change_gobj_pri = true;
            self.delayed.type_ = type_;
            self.delayed.p_link = p_link;
            self.delayed.p_prio = priority;
            self.delayed.gobj = position;
            return;
        }
        let mut cur = self.gobj(gobj).proc_;
        let mut proc_cur: Option<ProcId> = None;
        while let Some(c) = cur {
            self.unqueue_proc(c);
            let child = self.proc_(c).child;
            self.proc_mut(c).child = proc_cur;
            proc_cur = Some(c);
            cur = child;
        }
        self.gobj_mut(gobj).proc_ = None;
        self.unlink_p(gobj);
        self.gobj_mut(gobj).p_link = p_link;
        self.gobj_mut(gobj).p_priority = priority;
        self.insert_where(type_, gobj, position);
        let flags_new = if self.phase == 0 { 2 } else { self.phase - 1 };
        let flags_cur = if flags_new == 0 { 2 } else { flags_new - 1 };
        let mut cur = proc_cur;
        while let Some(c) = cur {
            let child = self.proc_(c).child;
            self.queue_proc(c);
            if self.proc_(c).flags_3 as i32 == flags_cur {
                self.proc_mut(c).flags_3 = (flags_new & 3) as u8;
            }
            cur = child;
        }
    }

    // ---- gobjgxlink.c ----

    /// `GObj_GXReorder`: link `gobj` after `hiprio` (None: at the head) in its gx_link list.
    pub fn gx_reorder(&mut self, gobj: GObjId, hiprio: Option<GObjId>) {
        let link = self.gobj(gobj).gx_link as usize;
        self.gobj_mut(gobj).prev_gx = hiprio;
        let next = match hiprio {
            Some(h) => {
                let n = self.gobj(h).next_gx;
                self.gobj_mut(h).next_gx = Some(gobj);
                n
            }
            None => {
                let n = self.gxlink_head[link];
                self.gxlink_head[link] = Some(gobj);
                n
            }
        };
        self.gobj_mut(gobj).next_gx = next;
        match next {
            Some(n) => self.gobj_mut(n).prev_gx = Some(gobj),
            None => self.gxlink_low[link] = Some(gobj),
        }
    }

    fn gx_last_not_above(&self, gobj: GObjId) -> Option<GObjId> {
        let g = self.gobj(gobj);
        let mut i = self.gxlink_low[g.gx_link as usize];
        while let Some(x) = i {
            if self.gobj(x).render_priority > g.render_priority {
                i = self.gobj(x).prev_gx;
            } else {
                break;
            }
        }
        i
    }

    /// gobjgxlink.c `GObj_GXFindPrioPosition` / `get_by_prio` (static inlines), then the
    /// "before it, or at the tail" choice their callers make.
    fn gx_before_first_not_below(&self, gobj: GObjId) -> Option<GObjId> {
        let g = self.gobj(gobj);
        let mut i = self.gxlink_head[g.gx_link as usize];
        while let Some(x) = i {
            if self.gobj(x).render_priority < g.render_priority {
                i = self.gobj(x).next_gx;
            } else {
                break;
            }
        }
        match i {
            Some(x) => self.gobj(x).prev_gx,
            None => self.gxlink_low[g.gx_link as usize],
        }
    }

    /// `GObj_SetupGXLink`.
    pub fn setup_gx_link(&mut self, gobj: GObjId, render_cb: Option<F>, gx_link: u8, priority: u32) {
        assert!(gx_link <= self.gx_link_max, "gobjgxlink.c:167 gx_link <= gx_link_max");
        let g = self.gobj_mut(gobj);
        g.render_cb = render_cb;
        g.gx_link = gx_link;
        g.render_priority = priority as u8;
        let i = self.gx_last_not_above(gobj);
        self.gx_reorder(gobj, i);
    }

    /// `GObj_SetupGXLinkMax` (the extra link `gx_link_max + 1`, drawn by `hsd_gobj_80390fc0`).
    pub fn setup_gx_link_max(&mut self, gobj: GObjId, render_cb: Option<F>, priority: u32) {
        let max_link = self.gx_link_max;
        let g = self.gobj_mut(gobj);
        g.render_cb = render_cb;
        g.gx_link = max_link.wrapping_add(1);
        g.render_priority = priority as u8;
        let i = self.gx_last_not_above(gobj);
        self.gx_reorder(gobj, i);
    }

    /// `GObj_SetupGXLinkMaxSorted`.
    pub fn setup_gx_link_max_sorted(&mut self, gobj: GObjId, render_cb: Option<F>, priority: u32) {
        let max_link = self.gx_link_max;
        let g = self.gobj_mut(gobj);
        g.render_cb = render_cb;
        g.gx_link = max_link.wrapping_add(1);
        g.render_priority = priority as u8;
        let i = self.gx_before_first_not_below(gobj);
        self.gx_reorder(gobj, i);
    }

    /// `HSD_GObjGXLink_8039084C`: unlink from the gx list (asserts it is linked).
    pub fn gxlink_8039084c(&mut self, gobj: GObjId) {
        assert!(self.gobj(gobj).gx_link != HSD_GOBJ_GXLINK_NONE, "gobjgxlink.c:415 gx_link != NONE");
        let (prev, next, link) = {
            let g = self.gobj(gobj);
            (g.prev_gx, g.next_gx, g.gx_link as usize)
        };
        match prev {
            Some(p) => self.gobj_mut(p).next_gx = next,
            None => self.gxlink_head[link] = next,
        }
        match next {
            Some(n) => self.gobj_mut(n).prev_gx = prev,
            None => self.gxlink_low[link] = prev,
        }
        let g = self.gobj_mut(gobj);
        g.gx_link = HSD_GOBJ_GXLINK_NONE;
        g.render_priority = 0;
        g.prev_gx = None;
        g.next_gx = None;
    }

    /// `HSD_GObjGXLink_80390908`: move to `gx_link` / `priority` (before the first gobj of
    /// priority >= `priority`).
    pub fn gxlink_80390908(&mut self, gobj: GObjId, gx_link: u8, priority: u8) {
        assert!(gx_link <= self.gx_link_max, "gobjgxlink.c:535 gx_link <= gx_link_max");
        self.gxlink_8039084c(gobj);
        let g = self.gobj_mut(gobj);
        g.gx_link = gx_link;
        g.render_priority = priority;
        let at = self.gx_before_first_not_below(gobj);
        self.gx_reorder(gobj, at);
    }

    /// `HSD_GObjGXLink_803909D8`: move just before `other` in its gx list.
    pub fn gxlink_803909d8(&mut self, gobj: GObjId, other: GObjId) {
        let prio = self.gobj(other).render_priority;
        let link = self.gobj(other).gx_link;
        self.gxlink_8039084c(gobj);
        let g = self.gobj_mut(gobj);
        g.gx_link = link;
        g.render_priority = prio;
        let at = self.gobj(other).prev_gx;
        self.gx_reorder(gobj, at);
    }

    // ---- gobjproc.c ----

    fn pl_index(&self, p_link: i32, s_link: i32) -> usize {
        (p_link + s_link * (self.p_link_max as i32 + 1)) as usize
    }

    /// `HSD_GObjProc_QueueProc`: insert `gproc` into its s_link queue after the last proc of
    /// the same s_link belonging to this gobj or an earlier gobj of its p_link list, else after
    /// the last proc of a lower p_link, else at the queue head; then push it on the gobj's
    /// proc chain.
    pub fn queue_proc(&mut self, gproc: ProcId) {
        let proc_gobj = self.proc_(gproc).gobj;
        let s_link = self.proc_(gproc).s_link;
        let mut p_link = self.gobj(proc_gobj).p_link as i32;
        let idx = self.pl_index(p_link, s_link as i32);
        let mut dst: Option<ProcId> = None;
        'found: {
            if self.proc_list[idx].is_some() {
                let mut cur_gobj = Some(proc_gobj);
                while let Some(cg) = cur_gobj {
                    let mut d = self.gobj(cg).proc_;
                    while let Some(dp) = d {
                        if self.proc_(dp).s_link == s_link {
                            if self.proc_list[idx] == Some(dp) {
                                self.proc_list[idx] = Some(gproc);
                            }
                            dst = Some(dp);
                            break 'found;
                        }
                        d = self.proc_(dp).child;
                    }
                    cur_gobj = self.gobj(cg).prev;
                }
            } else {
                self.proc_list[idx] = Some(gproc);
            }
            while p_link != 0 {
                p_link -= 1;
                let i = self.pl_index(p_link, s_link as i32);
                if let Some(d) = self.proc_list[i] {
                    dst = Some(d);
                    break 'found;
                }
            }
        }
        match dst {
            None => {
                let n = self.proc_head[s_link as usize];
                let p = self.proc_mut(gproc);
                p.next = n;
                p.prev = None;
                self.proc_head[s_link as usize] = Some(gproc);
            }
            Some(d) => {
                let n = self.proc_(d).next;
                self.proc_mut(gproc).next = n;
                self.proc_mut(d).next = Some(gproc);
                self.proc_mut(gproc).prev = Some(d);
            }
        }
        if let Some(n) = self.proc_(gproc).next {
            self.proc_mut(n).prev = Some(gproc);
        }
        let head = self.gobj(proc_gobj).proc_;
        self.proc_mut(gproc).child = head;
        self.gobj_mut(proc_gobj).proc_ = Some(gproc);
        let p = self.proc_(gproc);
        if self.delayed.in_delayed_proc
            && p.prev == self.current_invoked_proc
            && p.next == self.next_invoked_proc
            && s_link as i32 == self.current_invoked_s_link
        {
            self.next_invoked_proc = Some(gproc);
        }
    }

    /// `HSD_GObjProc_UnqueueProc`: take `gproc` out of its s_link queue (not off its gobj).
    pub fn unqueue_proc(&mut self, gproc: ProcId) {
        let p_link = self.gobj(self.proc_(gproc).gobj).p_link as i32;
        let s_link = self.proc_(gproc).s_link as i32;
        if self.delayed.in_delayed_proc && Some(gproc) == self.next_invoked_proc {
            self.next_invoked_proc = self.proc_(gproc).next;
        }
        let idx = self.pl_index(p_link, s_link);
        let (prev, next) = (self.proc_(gproc).prev, self.proc_(gproc).next);
        if self.proc_list[idx] == Some(gproc) {
            self.proc_list[idx] = match prev {
                Some(pv) if self.gobj(self.proc_(pv).gobj).p_link as i32 == p_link => Some(pv),
                _ => None,
            };
        }
        match prev {
            Some(pv) => self.proc_mut(pv).next = next,
            None => self.proc_head[s_link as usize] = next,
        }
        if let Some(n) = next {
            self.proc_mut(n).prev = prev;
        }
    }

    /// `HSD_GObjProc_UnlinkProcFromGObj`.
    pub fn unlink_proc_from_gobj(&mut self, gproc: ProcId) {
        let gobj = self.proc_(gproc).gobj;
        self.unqueue_proc(gproc);
        let child = self.proc_(gproc).child;
        if self.gobj(gobj).proc_ == Some(gproc) {
            self.gobj_mut(gobj).proc_ = child;
        } else {
            let mut cur = self.gobj(gobj).proc_.expect("proc not on its gobj");
            while self.proc_(cur).child != Some(gproc) {
                cur = self.proc_(cur).child.expect("proc not on its gobj");
            }
            self.proc_mut(cur).child = child;
        }
    }

    /// `HSD_GObj_SetupProc`.
    pub fn setup_proc(&mut self, gobj: GObjId, func: F, pri: u8) -> ProcId {
        let gproc = self.alloc_proc(HsdGObjProc {
            child: None,
            next: None,
            prev: None,
            s_link: 0,
            flags_1: 0,
            flags_2: 0,
            flags_3: 0,
            flags_4: 0,
            gobj,
            on_invoke: func,
        });
        assert!(pri <= self.gproc_pri_max, "gobjproc.c:216 pri <= gproc_pri_max");
        let p = self.proc_mut(gproc);
        p.s_link = pri;
        p.flags_1 = 0;
        p.flags_2 = 0;
        p.flags_3 = 3;
        self.queue_proc(gproc);
        gproc
    }

    /// `HSD_GObjProc_RemoveProc` (delayed when it is the running proc).
    pub fn remove_proc(&mut self, gproc: ProcId) {
        if !self.delayed.in_delayed_proc && Some(gproc) == self.current_invoked_proc {
            self.delayed.delay_remove_proc = true;
        } else {
            self.unlink_proc_from_gobj(gproc);
            self.procs[gproc.0 as usize] = None;
            self.free_procs.push(gproc.0);
        }
    }

    /// `HSD_GObjProc_RemoveAllProcs`.
    pub fn remove_all_procs(&mut self, gobj: GObjId) {
        let mut cur = self.gobj(gobj).proc_;
        while let Some(c) = cur {
            let next = self.proc_(c).child;
            self.remove_proc(c);
            cur = next;
        }
    }

    // ---- gobjobject.c ----

    /// `HSD_GObjObject_80390A3C`: first gobj of `p_link` with this classifier.
    pub fn gobjobject_80390a3c(&self, id: u16, p_link: u8) -> Option<GObjId> {
        let mut cur = self.plink_head[p_link as usize];
        while let Some(c) = cur {
            if self.gobj(c).classifier == id {
                break;
            }
            cur = self.gobj(c).next;
        }
        cur
    }

    /// `HSD_GObjObject_80390A70`: attach `obj` of `kind` (asserts none attached).
    pub fn gobjobject_80390a70(&mut self, gobj: GObjId, kind: u8, obj: Option<O>) {
        let g = self.gobj_mut(gobj);
        assert!(g.obj_kind == HSD_GOBJ_OBJ_NONE, "gobjobject.c:42 obj_kind == NONE");
        g.obj_kind = kind;
        g.hsd_obj = obj;
    }

    /// `HSD_GObjObject_80390ADC`: detach and return the object (None when none attached).
    pub fn gobjobject_80390adc(&mut self, gobj: GObjId) -> Option<O> {
        let g = self.gobj_mut(gobj);
        if g.obj_kind != HSD_GOBJ_OBJ_NONE {
            g.obj_kind = HSD_GOBJ_OBJ_NONE;
            g.hsd_obj.take()
        } else {
            None
        }
    }

    /// `HSD_GObjObject_80390B0C`: run the kind's removal function on the object, detach it.
    pub fn gobjobject_80390b0c(&mut self, host: &mut impl GObjHost<F, O>, gobj: GObjId) {
        let (kind, obj) = (self.gobj(gobj).obj_kind, self.gobj(gobj).hsd_obj);
        if kind != HSD_GOBJ_OBJ_NONE {
            let f = self.kind_funcs[kind as usize];
            host.remove_obj(self, f, obj);
            let g = self.gobj_mut(gobj);
            g.obj_kind = HSD_GOBJ_OBJ_NONE;
            g.hsd_obj = None;
        }
    }

    // ---- gobjuserdata.c ----

    /// `GObj_InitUserData` (asserts none attached).
    pub fn init_user_data(&mut self, gobj: GObjId, kind: u8, remove_func: Option<F>, data: Option<O>) {
        let g = self.gobj_mut(gobj);
        assert!(g.user_data_kind == HSD_GOBJ_USER_DATA_NONE, "gobjuserdata.c:40 user_data_kind == NONE");
        g.user_data_kind = kind;
        g.user_data = data;
        g.user_data_remove_func = remove_func;
    }

    /// `GObj_RemoveUserData`. Like the C it leaves `user_data_remove_func` set.
    pub fn remove_user_data(&mut self, host: &mut impl GObjHost<F, O>, gobj: GObjId) {
        let g = self.gobj(gobj);
        if g.user_data_kind == HSD_GOBJ_USER_DATA_NONE {
            return;
        }
        let f = g.user_data_remove_func.expect("gobjuserdata.c:99 user_data_remove_func");
        let d = g.user_data;
        host.remove_user_data(self, f, d);
        let g = self.gobj_mut(gobj);
        g.user_data_kind = HSD_GOBJ_USER_DATA_NONE;
        g.user_data = None;
    }

    // ---- gobj.c ----

    fn set_proc_chain(&mut self, gobj: GObjId, mut f: impl FnMut(&mut HsdGObjProc<F>)) {
        let mut p = self.gobj(gobj).proc_;
        while let Some(c) = p {
            f(self.proc_mut(c));
            p = self.proc_(c).child;
        }
    }
    /// `HSD_GObj_80390C5C`: pause every proc of the gobj (`flags_1 = 1`).
    pub fn hsd_gobj_80390c5c(&mut self, gobj: GObjId) {
        self.set_proc_chain(gobj, |p| p.flags_1 = 1);
    }
    /// `HSD_GObj_80390C84`: `flags_1 = 0`.
    pub fn hsd_gobj_80390c84(&mut self, gobj: GObjId) {
        self.set_proc_chain(gobj, |p| p.flags_1 = 0);
    }
    /// `HSD_GObj_80390CAC`: `flags_2 = 0`.
    pub fn hsd_gobj_80390cac(&mut self, gobj: GObjId) {
        self.set_proc_chain(gobj, |p| p.flags_2 = 0);
    }
    /// `HSD_GObj_80390CD4`: mark every proc as already run this phase.
    pub fn hsd_gobj_80390cd4(&mut self, gobj: GObjId) {
        let ph = (self.phase & 3) as u8;
        self.set_proc_chain(gobj, |p| p.flags_3 = ph);
    }

    /// `HSD_GObj_RunProcs`: advance the phase, then run every proc queue in s_link order,
    /// skipping procs already marked this phase, paused p_links and flagged procs, and apply
    /// the free / reprioritise / remove-proc requests a proc made on its own gobj.
    pub fn run_procs(&mut self, host: &mut impl GObjHost<F, O>) {
        let mask = self.pause_mask;
        self.phase += 1;
        if self.phase > 2 {
            self.phase = 0;
        }
        let mut i = 0i32;
        while i <= self.gproc_pri_max as i32 {
            self.current_invoked_s_link = i;
            let mut proc_ = self.proc_head[i as usize];
            while let Some(p) = proc_ {
                self.next_invoked_proc = self.proc_(p).next;
                if self.proc_(p).flags_3 as i32 != self.phase {
                    let ph = (self.phase & 3) as u8;
                    self.proc_mut(p).flags_3 = ph;
                    let gobj = self.proc_(p).gobj;
                    // `1LL << p_link` (gwtool: shift >= 64 gives 0)
                    let bit = 1u64.checked_shl(self.gobj(gobj).p_link as u32).unwrap_or(0);
                    let (f1, f2, f) = {
                        let pr = self.proc_(p);
                        (pr.flags_1, pr.flags_2, pr.on_invoke)
                    };
                    if mask & bit == 0 && f1 == 0 && f2 == 0 {
                        self.current_invoked_proc_gobj = Some(gobj);
                        self.current_invoked_proc = Some(p);
                        host.invoke(self, gobj, f);
                        self.next_invoked_proc = self.proc_(p).next;
                        if self.delayed.any() {
                            self.delayed.in_delayed_proc = true;
                            let pg = self.proc_(p).gobj;
                            if self.delayed.delay_remove_gobj {
                                self.free(host, pg);
                            } else {
                                if self.delayed.delay_change_gobj_pri {
                                    let d = self.delayed;
                                    self.change_gobj_pri(d.type_, pg, d.p_link, d.p_prio, d.gobj);
                                }
                                if self.delayed.delay_remove_proc {
                                    self.remove_proc(p);
                                }
                            }
                            self.delayed.clear_flags();
                        }
                        self.current_invoked_proc_gobj = None;
                        self.current_invoked_proc = None;
                    }
                }
                proc_ = self.next_invoked_proc;
            }
            i += 1;
        }
    }

    /// `HSD_GObj_80390ED0` (GObj_SetTextureCamera): for each set bit `i` of `mask`, draw every
    /// gx link whose bit is set in `gobj.gxlink_prios`, in list order, with pass `i`.
    pub fn hsd_gobj_80390ed0(&mut self, host: &mut impl GObjHost<F, O>, gobj: GObjId, mut mask: u32) {
        let mut i = 0i32;
        host.rand_render_begin();
        while mask != 0 {
            if mask & 1 != 0 {
                let mut prios = self.gobj(gobj).gxlink_prios;
                let mut j = 0usize;
                while prios != 0 {
                    if prios & 1 != 0 {
                        let mut cur = self.gxlink_head[j];
                        while let Some(c) = cur {
                            if let Some(cb) = self.gobj(c).render_cb {
                                let saved = self.gobj_804d7814;
                                self.gobj_804d7814 = Some(c);
                                host.render(self, c, cb, i);
                                self.gobj_804d7814 = saved;
                            }
                            cur = self.gobj(c).next_gx;
                        }
                    }
                    j += 1;
                    prios >>= 1;
                }
            }
            i += 1;
            mask >>= 1;
        }
        host.rand_render_end();
    }

    /// `HSD_GObj_80390FC0` (GObj_RunGXLinkMaxCallbacks): draw the `gx_link_max + 1` list, pass 0.
    /// PC: the callback tested at the start of the iteration is the one called.
    pub fn hsd_gobj_80390fc0(&mut self, host: &mut impl GObjHost<F, O>) {
        let mut cur = self.gxlink_head[self.gx_link_max as usize + 1];
        host.rand_render_begin();
        while let Some(c) = cur {
            if let Some(cb) = self.gobj(c).render_cb {
                let saved = self.gobj_804d7818;
                self.gobj_804d7818 = Some(c);
                if !host.snap_skip_render_cb(cb) {
                    host.render(self, c, cb, 0);
                }
                self.gobj_804d7818 = saved;
            }
            cur = self.gobj(c).next_gx;
        }
        host.rand_render_end();
    }
}
