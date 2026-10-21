//! Port of src/melee/db/dballoc.c (debug: freeze object-allocator limits at their peak).
#![allow(non_snake_case)]

use super::dbeffect::pad;

/// `DbLKind_Develop` (db.h).
pub const DB_LKIND_DEVELOP: i32 = 4;

/// The HSD_ObjAllocData pools this TU touches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocPool {
    /// `efLib_AllocData`
    EfLib,
    /// `hsd_804D0F90.alloc_data`
    Hsd804D0F90,
    /// `hsd_804D0F60.alloc_data`
    Hsd804D0F60,
    /// `HSD_PSAppSrt_804D10B0`
    PsAppSrt,
}

#[allow(non_snake_case)]
pub trait DbAllocHost {
    fn DbLevel(&mut self) -> i32;
    fn db_ButtonsDown(&mut self, player: i32) -> u32;
    fn db_ButtonsPressed(&mut self, player: i32) -> u32;
    fn HSD_ObjAllocGetPeak(&mut self, pool: AllocPool) -> u32;
    fn HSD_ObjAllocSetNumLimit(&mut self, pool: AllocPool, n: u32);
    fn HSD_ObjAllocEnableNumLimit(&mut self, pool: AllocPool);
    fn HSD_ObjAllocDisableNumLimit(&mut self, pool: AllocPool);
}

#[allow(non_snake_case)]
pub mod ext {
    use super::AllocPool;
    /// Unported callees (TODO: rewire to db.c / baselib objalloc ports).
    pub struct Ext;
    impl super::DbAllocHost for Ext {
        fn DbLevel(&mut self) -> i32 { todo!("DbLevel (db.c)") }
        fn db_ButtonsDown(&mut self, _: i32) -> u32 { todo!("db_ButtonsDown (db.c)") }
        fn db_ButtonsPressed(&mut self, _: i32) -> u32 { todo!("db_ButtonsPressed (db.c)") }
        fn HSD_ObjAllocGetPeak(&mut self, _: AllocPool) -> u32 { todo!("HSD_ObjAllocGetPeak") }
        fn HSD_ObjAllocSetNumLimit(&mut self, _: AllocPool, _: u32) { todo!("HSD_ObjAllocSetNumLimit") }
        fn HSD_ObjAllocEnableNumLimit(&mut self, _: AllocPool) { todo!("HSD_ObjAllocEnableNumLimit") }
        fn HSD_ObjAllocDisableNumLimit(&mut self, _: AllocPool) { todo!("HSD_ObjAllocDisableNumLimit") }
    }
}

/// `db_804D6BA0` (UnkFlagStruct; only b0/b1 used).
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbAlloc {
    pub b0: bool,
    pub b1: bool,
}

fn freeze<H: DbAllocHost + ?Sized>(h: &mut H, p: AllocPool) {
    let peak = h.HSD_ObjAllocGetPeak(p);
    h.HSD_ObjAllocSetNumLimit(p, peak);
    h.HSD_ObjAllocEnableNumLimit(p);
}

impl DbAlloc {
    /// `fn_SetupObjAllocLimiter` (0x80229220).
    pub fn fn_setup_obj_alloc_limiter(&mut self) {
        self.b0 = false;
        self.b1 = false;
    }

    /// `fn_UpdateObjAllocLimiter` (0x80229240).
    pub fn fn_update_obj_alloc_limiter<H: DbAllocHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if h.DbLevel() != DB_LKIND_DEVELOP {
            return;
        }
        if (h.db_ButtonsDown(player) & pad::B) != 0 && (h.db_ButtonsPressed(player) & pad::DPADUP) != 0 {
            if !self.b0 {
                freeze(h, AllocPool::EfLib);
                self.b0 = true;
            } else {
                h.HSD_ObjAllocDisableNumLimit(AllocPool::EfLib);
                self.b0 = false;
            }
        }
        if (h.db_ButtonsDown(player) & pad::A) != 0 && (h.db_ButtonsPressed(player) & pad::DPADUP) != 0 {
            if !self.b1 {
                freeze(h, AllocPool::Hsd804D0F90);
                freeze(h, AllocPool::Hsd804D0F60);
                freeze(h, AllocPool::PsAppSrt);
                self.b1 = true;
            } else {
                h.HSD_ObjAllocDisableNumLimit(AllocPool::Hsd804D0F90);
                h.HSD_ObjAllocDisableNumLimit(AllocPool::Hsd804D0F60);
                h.HSD_ObjAllocDisableNumLimit(AllocPool::PsAppSrt);
                self.b1 = false;
            }
        }
    }
}
