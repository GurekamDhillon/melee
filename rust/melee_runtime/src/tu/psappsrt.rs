//! Complete psappsrt.c entry points over the existing native arena.
//! The explicit pool replaces C's allocator and particle-owned live/peak globals.
#![allow(non_snake_case)]
use crate::generator::GenId;
pub use crate::psappsrt::{AppSrtHost, AppSrtPool, HsdPsAppSrt, SrtId};

pub mod ext {
    #![allow(non_upper_case_globals)]
    use melee_types::HSD_ObjAllocData;
    pub static mut HSD_ObjAllocInit: unsafe fn(
        *mut HSD_ObjAllocData,
        u32,
        u32,
    ) = missing;
    unsafe fn missing(_: *mut HSD_ObjAllocData, _: u32, _: u32) {
        panic!("TODO HSD_ObjAllocInit")
    }
}

pub unsafe fn psInitAppSRT(
    pool: &mut AppSrtPool,
    data: &mut melee_types::HSD_ObjAllocData,
    num: i32,
    size: i32,
) -> i32 {
    melee_core::ct!("psInitAppSRT");
    let result = pool.init(num, size);
    unsafe {
        (ext::HSD_ObjAllocInit)(data, size as u32, 4);
    }
    result
}
pub fn psAddGeneratorAppSRT(
    pool: &mut AppSrtPool,
    status: i32,
    idnum: u16,
) -> Option<SrtId> {
    melee_core::ct!("psAddGeneratorAppSRT");
    pool.add(status, idnum)
}
pub fn psAddParticleAppSRT_begin(
    pool: &mut AppSrtPool,
    slot: &mut Option<SrtId>,
    status: i32,
    idnum: u16,
) -> Option<SrtId> {
    melee_core::ct!("psAddParticleAppSRT_begin");
    pool.add_begin(slot, status, idnum)
}
/// `psAddGeneratorAppSRT_begin`: same pool operation as the particle variant (the C bodies are identical).
pub fn psAddGeneratorAppSRT_begin(
    pool: &mut AppSrtPool,
    slot: &mut Option<SrtId>,
    status: i32,
    idnum: u16,
) -> Option<SrtId> {
    melee_core::ct!("psAddGeneratorAppSRT_begin");
    pool.add_begin(slot, status, idnum)
}
pub fn psAttachParticleAppSRT(
    pool: &mut AppSrtPool,
    slot: Option<&mut Option<SrtId>>,
    srt: Option<SrtId>,
) -> i32 {
    melee_core::ct!("psAttachParticleAppSRT");
    pool.attach(slot, srt)
}
/// `psAttachGeneratorAppSRT`: same pool operation as the particle variant (the C bodies are identical).
pub fn psAttachGeneratorAppSRT(
    pool: &mut AppSrtPool,
    slot: Option<&mut Option<SrtId>>,
    srt: Option<SrtId>,
) -> i32 {
    melee_core::ct!("psAttachGeneratorAppSRT");
    pool.attach(slot, srt)
}
pub fn psRemoveParticleAppSRT<H: AppSrtHost>(
    pool: &mut AppSrtPool,
    slot: &mut Option<SrtId>,
    host: &mut H,
) -> i32 {
    melee_core::ct!("psRemoveParticleAppSRT");
    pool.remove_particle(slot, host)
}
pub fn psRemoveGeneratorSRT<H: AppSrtHost>(
    pool: &mut AppSrtPool,
    gen: GenId,
    slot: &mut Option<SrtId>,
    host: &mut H,
) -> i32 {
    melee_core::ct!("psRemoveGeneratorSRT");
    pool.remove_generator(gen, slot, host)
}
