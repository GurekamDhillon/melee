//! Native perf.c counters and tick conversion (no fused expressions).
#![allow(non_snake_case, non_upper_case_globals)]
use melee_types::HSD_PerfStat;
pub static mut start_time: i64 = 0;
pub static mut HSD_PerfLastStat: HSD_PerfStat = unsafe { core::mem::zeroed() };
pub static mut HSD_PerfCurrentStat: HSD_PerfStat = unsafe { core::mem::zeroed() };
pub mod ext {
    pub static mut OSGetTime_hook: Option<unsafe fn() -> i64> = None;
    pub static mut __OSBusClock: u32 = 0;
    pub unsafe fn OSGetTime() -> i64 { OSGetTime_hook.expect("TODO OSGetTime")() }
}
pub unsafe fn HSD_PerfInitStat() {
    HSD_PerfLastStat = HSD_PerfCurrentStat;
    HSD_PerfCurrentStat = HSD_PerfStat::zeroed();
}
pub unsafe fn HSD_PerfSetStartTime() { start_time = ext::OSGetTime(); }
pub unsafe fn HSD_PerfSetCPUTime() { HSD_PerfCurrentStat.cpu_time = ext::OSGetTime().wrapping_sub(start_time) as f32 / ((ext::__OSBusClock / 4) / 60) as f32; }
pub unsafe fn HSD_PerfSetDrawTime() { HSD_PerfCurrentStat.draw_time = ext::OSGetTime().wrapping_sub(start_time) as f32 / ((ext::__OSBusClock / 4) / 60) as f32; }
pub unsafe fn HSD_PerfSetTotalTime() { HSD_PerfCurrentStat.total_time = ext::OSGetTime().wrapping_sub(start_time) as f32 / ((ext::__OSBusClock / 4) / 60) as f32; }
pub unsafe fn HSD_PerfCountEnvelopeBlending(n: i32) {
    assert!(n < 32, "HSD_PerfCountEnvelopeBlending: n < 32");
    HSD_PerfCurrentStat.env_blend[n as usize] = HSD_PerfCurrentStat.env_blend[n as usize].wrapping_add(1);
}
