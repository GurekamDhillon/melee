//! stage_switch: seamless in-match stage switching, the Rust port of the C build's stage-switch demo
//! (pc/scripts/examples/stage_switch_demo: a loop BF -> Yoshi's Story -> Dream Land -> Fountain -> FD, ten
//! logic seconds per stage). The switch itself is native (`gd_stage_switch`, melee_modes::tu::stage_hop):
//! applied between logic frames, inside the simulation, so it rolls back and resimulates with the match.
//!
//! * `stage.on_start` post: (re)starts the ten-second clock; it fires at match start and inside every switch.
//! * `frame.run_procs` post: when the clock reaches PERIOD, request the next stage of the rotation after the
//!   live one. A refusal (fighter grabbed / thrown / respawning, -9) keeps the entry due and retries next frame;
//!   any other refusal (host stage outside the five supported) stops the rotation for this match.
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd")]
extern "C" {
    fn gd_stage_grkind() -> i32;
    fn gd_stage_switch(grkind: i32) -> i32;
    fn gd_log(msg: *const u8, value: i32);
}

/// Logic frames on each stage (the C demo's `after=10` seconds).
const PERIOD: i32 = 600;
/// Gr_Kind_Battle, Story, OldPupupu, Izumi, Last: the C demo's order after its initial FD.
const ROTATION: [i32; 5] = [36, 10, 28, 12, 37];
const BUSY: i32 = -9;

static mut CLOCK: i32 = -1; // -1: no match / rotation stopped
static mut WARNED: bool = false;

extern "C" fn stage_start(_p: u32) {
    unsafe {
        CLOCK = 0;
        WARNED = false;
    }
}

extern "C" fn on_frame_end() {
    unsafe {
        if CLOCK < 0 {
            return;
        }
        CLOCK += 1;
        if CLOCK < PERIOD {
            return;
        }
        let cur = gd_stage_grkind();
        let next = match ROTATION.iter().position(|&k| k == cur) {
            Some(i) => ROTATION[(i + 1) % ROTATION.len()],
            None => {
                if !WARNED { gd_log(b"stage_switch: host stage not supported, rotation off; grkind\0".as_ptr(), cur); WARNED = true; }
                CLOCK = -1;
                return;
            }
        };
        let rc = gd_stage_switch(next);
        if rc == 0 {
            gd_log(b"stage_switch: switching to grkind\0".as_ptr(), next);
            CLOCK = -1; // stage.on_start restarts it inside the switch
        } else if rc != BUSY {
            gd_log(b"stage_switch: refused, rotation off; rc\0".as_ptr(), rc);
            CLOCK = -1;
        }
    }
}

#[repr(C)]
pub struct HookDecl { address: u32, mode: u32, priority: i32, flags: u32, f: *const () }
#[repr(C)]
pub struct Manifest {
    magic: u32, struct_size: u32, abi_min: u32, abi_max: u32, hook_table_version: u32, mod_version: u32,
    required_features: u32, optional_features: u32, state_bytes: u32, hook_count: u32, hooks: *const HookDecl,
    mod_id: [u8; 32], mod_name: [u8; 48],
}
unsafe impl Sync for Manifest {}
unsafe impl Sync for HookDecl {}

const fn text<const N: usize>(s: &[u8]) -> [u8; N] {
    let mut o = [0u8; N];
    let mut i = 0;
    while i < s.len() { o[i] = s[i]; i += 1; }
    o
}

const ABI: u32 = 1 << 16;
const POST: u32 = 2;
static HOOKS: [HookDecl; 2] = [
    HookDecl { address: 0x801C_0FB8, mode: POST, priority: 0, flags: 0, f: stage_start as *const () },
    HookDecl { address: 0x8039_0CFC, mode: POST, priority: 100, flags: 0, f: on_frame_end as *const () },
];

#[no_mangle]
pub static gdmod_abi_version: u32 = ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: 0, optional_features: 1 << 3, state_bytes: 16, hook_count: 2, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.stage_switch"), mod_name: text(b"Stage switch"),
};
