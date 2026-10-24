//! Native port of `src/sysdolphin/baselib/leak.c`: the debug memory-leak checker.
//!
//! The checker walks the GameCube MEM1 address space (32-bit tagged heap headers). It is dead
//! on PC in practice: nothing registers blocks, so `HSD_Leak_80407B58.table` stays NULL and the
//! only reachable path reports "Leak checker is not initialized". The scanning path is still
//! ported as written, with every address kept as the C's `u32` and dereferenced through
//! `usize`; callers that install a mapped image through `ext` get the C behaviour.
#![allow(non_snake_case, non_upper_case_globals)]
use melee_types::HSD_LeakChecker;

const HEAP_MAGIC: u32 = 0x01234567;

pub static mut HSD_Leak_80407B58: HSD_LeakChecker = HSD_LeakChecker {
    table: core::ptr::null_mut(), x04: 0, used: 0, capacity: 0,
    x10: 0, x14: 0, peak: 0, x1C: -1,
};

pub unsafe fn HSD_LeakGetCapacityPtr(lc: *mut HSD_LeakChecker) -> *mut u32 {
    &raw mut (*lc).capacity
}

pub unsafe fn HSD_LeakReportSpaces(count: u32) {
    for _ in 0..count {
        ext::OSReport(" ");
    }
}

#[inline]
unsafe fn rd(addr: u32) -> u32 {
    *(addr as usize as *const u32)
}

pub unsafe fn HSD_Leak_80387DF8(indent: i32) -> i32 {
    let lc = &raw mut HSD_Leak_80407B58;
    let mut scan: u32 = ext::lbCommand_803B9840() as usize as u32;
    let mut heap_start_phys: u32 = ext::OSGetConsoleSimulatedMemSize().wrapping_add(0x8000_0000);
    let heap_start_align: u32 = scan.wrapping_add(32) & 0x0FFF_FFE0;
    let mut val: u32 = ext::OSGetPhysicalMemSize();
    let mut leak_count: i32 = 0;

    if (*lc).table.is_null() {
        HSD_LeakReportSpaces(indent as u32);
        ext::OSReport("Leak checker is not initialized.\n");
        return 0;
    }

    HSD_LeakReportSpaces(indent as u32);
    ext::OSReport("Begin memory leak check ...\n");

    HSD_LeakReportSpaces(indent.wrapping_add(2) as u32);
    let cap_ptr = HSD_LeakGetCapacityPtr(lc);
    ext::OSReport(&format!(
        "number of registered ptr: {} / {} (peak {})\n",
        (*lc).used, *cap_ptr, (*lc).peak
    ));

    // Scan memory for heap references.
    while scan < heap_start_phys {
        let mut word = rd(scan);
        if word & 3 == 0 {
            let top = word >> 28;
            if top == 8 || top == 0xC {
                word &= 0x0FFF_FFFF;
                if word >= heap_start_align && val > word {
                    word |= 0x8000_0000;
                    let hdr = word.wrapping_sub(32);
                    if rd(hdr) == HEAP_MAGIC {
                        let reg_idx = rd(hdr.wrapping_add(4));
                        if reg_idx.wrapping_add(0x10000) != 0xFFFF && reg_idx < *cap_ptr {
                            let slot = (*lc).table.add(reg_idx as usize);
                            let entry = *slot;
                            if hdr == entry & !1u32 {
                                *slot = entry | 1;
                            }
                        }
                    }
                }
            }
        }
        scan = scan.wrapping_add(4);
    }

    let i = (indent as u32).wrapping_add(2);
    scan = (*lc).table as usize as u32;
    val = scan.wrapping_add((*cap_ptr).wrapping_mul(4));
    for ofs in 0..*cap_ptr {
        let ep = (*lc).table.add(ofs as usize);
        heap_start_phys = *ep;
        if heap_start_phys & 1 != 0 {
            *ep = heap_start_phys & !1u32;
        } else if scan <= heap_start_phys && heap_start_phys < val {
        } else if heap_start_phys != 0 {
            let block = heap_start_phys;
            let mut reported = false;
            if rd(block) == HEAP_MAGIC {
                let reg_idx = rd(block.wrapping_add(4));
                if reg_idx.wrapping_add(0x10000) != 0xFFFF && reg_idx < *cap_ptr {
                    let table = (*lc).table as usize as u32;
                    if heap_start_phys == rd(table.wrapping_add(reg_idx << 2)) {
                        HSD_LeakReportSpaces(i);
                        ext::OSReport(&format!(
                            "leak detected ({:x}) nb_reg ({}) mark ({:08x})\n",
                            heap_start_phys.wrapping_add(0x20),
                            rd(block.wrapping_add(8)),
                            rd(block.wrapping_add(12))
                        ));
                        reported = true;
                    }
                }
            }
            if !reported {
                for _ in 0..i {
                    ext::OSReport(" ");
                }
                ext::OSReport(&format!(
                    "leak detected ({:x}) [destroyed header]\n",
                    heap_start_phys.wrapping_add(0x20)
                ));
            }
            leak_count += 1;
        }
    }

    if leak_count > 0 {
        HSD_LeakReportSpaces(indent.wrapping_add(2) as u32);
        ext::OSReport(&format!("number of leaked memory: {}.\n", leak_count));
    } else {
        HSD_LeakReportSpaces(indent.wrapping_add(2) as u32);
        ext::OSReport("leak is not detected.\n");
    }

    HSD_LeakReportSpaces(indent as u32);
    ext::OSReport("done.\n");

    leak_count
}

/// Callees in unported TUs. `OSReport` takes an already formatted message here.
pub mod ext {
    pub static mut REPORT: Option<fn(&str)> = None;
    pub static mut SCAN: Option<fn() -> *mut u32> = None;
    pub static mut SIMULATED: Option<fn() -> u32> = None;
    pub static mut PHYSICAL: Option<fn() -> u32> = None;
    pub fn OSReport(s: &str) { unsafe { if let Some(f) = REPORT { f(s) } else { todo!("OSReport (dolphin/os)") } } }
    pub fn lbCommand_803B9840() -> *mut u32 { unsafe { if let Some(f) = SCAN { f() } else { todo!("lbCommand_803B9840 (lb data)") } } }
    pub fn OSGetConsoleSimulatedMemSize() -> u32 { unsafe { if let Some(f) = SIMULATED { f() } else { todo!("OSGetConsoleSimulatedMemSize") } } }
    pub fn OSGetPhysicalMemSize() -> u32 { unsafe { if let Some(f) = PHYSICAL { f() } else { todo!("OSGetPhysicalMemSize") } } }
}
