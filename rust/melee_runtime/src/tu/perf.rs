//! Native perf.c counters and tick conversion (no fused expressions).
#![allow(non_snake_case, non_upper_case_globals)]
use melee_types::HSD_PerfStat;
/// `__OSBusClock` (the console bus clock the C reads from 0x800000F8).
pub static mut __OSBusClock: u32 = 0;
pub static mut start_time: i64 = 0;
pub static mut HSD_PerfLastStat: HSD_PerfStat = unsafe { core::mem::zeroed() };
pub static mut HSD_PerfCurrentStat: HSD_PerfStat = unsafe { core::mem::zeroed() };
// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn OSGetTime() -> i64;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT
pub unsafe fn HSD_PerfInitStat() {
    HSD_PerfLastStat = HSD_PerfCurrentStat;
    HSD_PerfCurrentStat = HSD_PerfStat::zeroed();
}
pub unsafe fn HSD_PerfSetStartTime() { start_time = ext::OSGetTime(); }
pub unsafe fn HSD_PerfSetCPUTime() { HSD_PerfCurrentStat.cpu_time = ext::OSGetTime().wrapping_sub(start_time) as f32 / ((__OSBusClock / 4) / 60) as f32; }
pub unsafe fn HSD_PerfSetDrawTime() { HSD_PerfCurrentStat.draw_time = ext::OSGetTime().wrapping_sub(start_time) as f32 / ((__OSBusClock / 4) / 60) as f32; }
pub unsafe fn HSD_PerfSetTotalTime() { HSD_PerfCurrentStat.total_time = ext::OSGetTime().wrapping_sub(start_time) as f32 / ((__OSBusClock / 4) / 60) as f32; }
pub unsafe fn HSD_PerfCountEnvelopeBlending(n: i32) {
    assert!(n < 32, "HSD_PerfCountEnvelopeBlending: n < 32");
    HSD_PerfCurrentStat.env_blend[n as usize] = HSD_PerfCurrentStat.env_blend[n as usize].wrapping_add(1);
}
