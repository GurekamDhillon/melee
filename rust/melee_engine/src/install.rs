//! Points the leaf crates' host seams at this crate: OS heaps and arena, the DVD, the archive host.
//! Seams only; the callers are the ported `lbFile` / `lbArchive` / `HSD_OSInit` / `HSD_MemAlloc` TUs.

use crate::host::Host;
use crate::{archive, dvd};
use core::ffi::c_void;
use melee_runtime::tu::{archive as tu_archive, initialize, lbdvd, lbfile, lbheap, memory};

fn os() -> &'static mut crate::os::Os {
    &mut Host::get().os
}

fn arena_lo() -> *mut c_void { os().arena_lo() }
fn arena_hi() -> *mut c_void { os().arena_hi() }
fn phys_size() -> u32 { os().physical_mem_size() }
fn init_alloc(lo: *mut c_void, hi: *mut c_void, n: i32) -> *mut c_void { os().init_alloc(lo, hi, n) }
fn set_arena_lo(p: *mut c_void) { os().set_arena_lo(p) }
fn create_heap(a: *mut c_void, b: *mut c_void) -> i32 {
    let id = os().create_heap(a, b);
    eprintln!("engine: OSCreateHeap({a:p}, {b:p}) = {id} ({:#x} bytes)", b as usize - a as usize);
    id
}
fn destroy_heap(id: i32) { os().destroy_heap(id) }
fn set_current(id: i32) -> i32 { os().set_current_heap(id) }
fn alloc_from(id: i32, n: u32) -> *mut c_void {
    let p = os().alloc_from_heap(id, n);
    melee_core::heaptrace::alloc(id, p as usize, n);
    if p.is_null() {
        eprintln!("engine: OSAllocFromHeap(heap {id}, {n:#x}) failed; heap has {:#x} free", os().check_heap(id));
    }
    p
}
fn free_to(id: i32, p: *mut c_void) {
    melee_core::heaptrace::free(p as usize);
    os().free_to_heap(id, p)
}
fn check_heap(id: i32) -> i32 { os().check_heap(id) }

/// Install every seam. Call after `Host::boot`.
pub fn install() {
    unsafe {
        use initialize::ext::hook as ih;
        ih::OSGetArenaLo = Some(arena_lo);
        ih::OSGetArenaHi = Some(arena_hi);
        ih::OSGetPhysicalMemSize = Some(phys_size);
        ih::OSInitAlloc = Some(init_alloc);
        ih::OSSetArenaLo = Some(set_arena_lo);
        ih::OSCreateHeap = Some(create_heap);
        ih::OSDestroyHeap = Some(destroy_heap);
        ih::OSSetCurrentHeap = Some(set_current);
        ih::OSAllocFromHeap = Some(alloc_from);

        memory::ext::hook::OSAllocFromHeap = Some(alloc_from);
        memory::ext::hook::OSFreeToHeap = Some(free_to);

        lbheap::ext::H_OSCreateHeap = Some(create_heap);
        lbheap::ext::H_OSDestroyHeap = Some(destroy_heap);
        lbheap::ext::H_OSAllocFromHeap = Some(alloc_from);
        lbheap::ext::H_OSCheckHeap = Some(check_heap);
        lbheap::ext::H_OSDisableInterrupts = Some(dvd::OSDisableInterrupts);
        lbheap::ext::H_OSRestoreInterrupts = Some(dvd::OSRestoreInterrupts);

        // ARAM (lbMemory_8001564C): the arena starts at the audio-free base and runs to 32 MB (the port's ARAM size)
        static mut AR_CUR: u32 = 0x4000;
        melee_runtime::tu::lbmemory::ext::H_ARAlloc = Some(|n| { let p = AR_CUR; AR_CUR += n; p });
        melee_runtime::tu::lbmemory::ext::H_ARFree = Some(|sz| { *sz = AR_CUR - 0x4000; AR_CUR = 0x4000; 0x4000 });
        melee_runtime::tu::lbmemory::ext::H_ARGetSize = Some(|| 0x0200_0000);

        melee_modes::tu::gmmain_lib::ext::DVDCONVERTPATHTOENTRYNUM = Some(|p: &str| {
            let c = std::ffi::CString::new(p).unwrap();
            dvd::DVDConvertPathToEntrynum(c.as_ptr())
        });

        // the save-data pointer lives in melee_world::gmmain, above melee_runtime
        melee_runtime::tu::lblanguage::ext::H_gmMainLib_804D3EE0 = Some(|| melee_world::gmmain::gmMainLib_804D3EE0);

        // lbArchive_LoadSymbols (variadic in the C): the hooks that spell its arguments out
        {
            use core::ffi::c_char;
            melee_runtime::tu::lbrefract::H_lbArchive_LoadSymbols = Some(|file: *const c_char, out: *mut c_void, sym: *const c_char| {
                melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(file, &[out, sym as *mut c_void, core::ptr::null_mut()]);
            });
            melee_fighter::tu::player::ext::hook::lbArchive_LoadSymbols = Some(
                |file: *const c_char, out: *mut c_void, sym: *const c_char, _end: i32| {
                    melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(file, &[out, sym as *mut c_void, core::ptr::null_mut()])
                },
            );
            melee_world::tu::efn::efasync::ext::hook::lbArchive_80017040 = Some(
                |dst: *mut *mut melee_types::HSD_Archive, file: *const i8, out: *mut c_void, rest: &[*const c_void]| {
                    let mut v: Vec<*mut c_void> = vec![out];
                    v.extend(rest.iter().map(|p| *p as *mut c_void));
                    melee_runtime::tu::lbarchive::lbArchive_80017040(dst, file, &v) as i32
                },
            );
            melee_fighter::tu::fighter::ext::hook::lbArchive_LoadSymbols_PlCo = Some(|| {
                static mut P: *mut *mut c_void = core::ptr::null_mut();
                melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(
                    c"PlCo.dat".as_ptr(),
                    &[&raw mut P as *mut c_void, c"ftLoadCommonData".as_ptr() as *mut c_void, core::ptr::null_mut()],
                );
                P
            });
        }
        // the common and demo MotionState tables (ftmotionstates.c)
        melee_fighter::tu::fighter::ext::hook::ftData_MotionStateList =
            Some(|| &raw mut melee_link::retail_ms::ftmotionstates::ftData_MotionStateList as *mut melee_types::MotionState);
        melee_fighter::tu::ftdemo::ext::FTDATA_803C52A0 =
            Some(|| &raw mut melee_link::retail_ms::ftmotionstates::ftData_803C52A0 as *mut melee_types::MotionState);

        lbfile::ext::H_lb_800195D0 = Some(dvd::host_pump_disc);
        lbfile::ext::H_HSD_DevComRequest = Some(dvd::host_dev_com_request);
        // the movie player's frame reads (lbmthp.c fn_8001E910 / fn_8001F13C, through lbmemory's DevCom hook): the same
        // host disc as lbFile's (the canonical devcom.c queue is never initialised on the engine path), completing a frame later
        melee_runtime::tu::lbmemory::ext::H_HSD_DevComRequest = Some(dvd::host_dev_com_deferred);
        lbfile::ext::H_DVDFastOpen = Some(dvd::DVDFastOpen);
        lbfile::ext::H_DVDClose = Some(dvd::DVDClose);
        lbfile::ext::H_OSDisableInterrupts = Some(dvd::OSDisableInterrupts);
        lbfile::ext::H_OSRestoreInterrupts = Some(dvd::OSRestoreInterrupts);
        lbfile::ext::H_DVDConvertPathToEntrynum = Some(dvd::DVDConvertPathToEntrynum);

        lbdvd::ext::hook::DVDConvertPathToEntrynum = Some(|p| dvd::DVDConvertPathToEntrynum(p as *const _));
        lbdvd::ext::hook::OSDisableInterrupts = Some(dvd::OSDisableInterrupts);
        lbdvd::ext::hook::OSRestoreInterrupts = Some(dvd::OSRestoreInterrupts);
        lbdvd::ext::hook::lb_800195D0 = Some(dvd::host_pump_disc);

        melee_runtime::tu::lbext::H_OSReport = Some(|fmt, args| eprint!("OSReport: {fmt} {args:?}
"));
        melee_runtime::tu::lbext::H___assert = Some(|f, l, c| eprintln!("__assert {f}:{l}: {c}"));

        tu_archive::host_parse = Some(archive::host_parse);
        melee_runtime::tu::lbdvd::ASSET_HINT = Some(crate::asset::hint_entry);
        melee_modes::tu::envoy::PLAN_HINT = Some(crate::asset::hint_envoy_plan);
        tu_archive::host_public = Some(|a, s| archive::host_public_address(a, s as *const _));
        tu_archive::host_locate_extern = Some(|a, s, p| archive::host_locate_extern(a, s as *const _, p));
    }
}
