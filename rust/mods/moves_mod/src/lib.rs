//! moves_mod: exercises the fighter rows of the generic host (notes/modhost.md).
//!
//! * `fighter.proc00` pre (input): reads slot 0's controller snapshot (`gd_fighter_get_input`) and counts frames with
//!   a button newly pressed.
//! * `action.Jump.enter` post (action-state change): every jump of any player gets 10% more vertical speed.
//! * `action.Squat.enter` replace (`GD_FEATURE_REPLACE`): runs the original through `gd_orig_call_void`, then counts.
//! * `action.Landing.enter` pre: counts landings, continues.
//! * `frame.run_procs` post: every 600 frames logs the counters; on frame 900 asks slot 0 for Squat if grounded and
//!   waiting (`gd_fighter_request_action_state`, `GD_FEATURE_ACTION_REQUEST`).
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[repr(C)]
#[derive(Default)]
struct Input { lx: f32, ly: f32, cx: f32, cy: f32, trigger: f32, held: u32, pressed: u32, released: u32 }

#[link(wasm_import_module = "gd")]
extern "C" {
    fn gd_fighter_slot(f: u32) -> i32;
    fn gd_fighter_from_slot(slot: i32) -> u32;
    fn gd_fighter_get_input(f: u32, out: *mut Input);
    fn gd_fighter_get_vel_x(f: u32) -> f32;
    fn gd_fighter_get_vel_y(f: u32) -> f32;
    fn gd_fighter_get_vel_z(f: u32) -> f32;
    fn gd_fighter_set_vel(f: u32, x: f32, y: f32, z: f32);
    fn gd_fighter_get_action_state(f: u32) -> i32;
    fn gd_fighter_request_action_state(f: u32, state: i32) -> i32;
    fn gd_orig_call_void();
    fn gd_match_frame() -> i32;
    fn gd_log(msg: *const u8, value: i32);
}

static mut PRESSES: i32 = 0;
static mut JUMPS: i32 = 0;
static mut SQUATS: i32 = 0;
static mut LANDINGS: i32 = 0;

extern "C" fn on_input(f: u32) -> i32 {
    unsafe {
        if gd_fighter_slot(f) == 0 {
            let mut i = Input::default();
            gd_fighter_get_input(f, &mut i);
            if i.pressed != 0 {
                PRESSES += 1;
            }
        }
    }
    0 // GD_CONTINUE
}

extern "C" fn on_jump(f: u32) {
    unsafe {
        JUMPS += 1;
        let vy = gd_fighter_get_vel_y(f);
        gd_fighter_set_vel(f, gd_fighter_get_vel_x(f), vy * 1.1, gd_fighter_get_vel_z(f));
    }
}

extern "C" fn squat_replace(_f: u32) {
    unsafe {
        gd_orig_call_void();
        SQUATS += 1;
    }
}

extern "C" fn on_landing(_f: u32, _msid: i32, _interrupt: i32, _flags: i32, _a: f32, _b: f32) -> i32 {
    unsafe { LANDINGS += 1 };
    0
}

extern "C" fn on_frame_end() {
    unsafe {
        let fr = gd_match_frame();
        if fr % 600 == 0 {
            gd_log(b"moves: slot 0 button presses\0".as_ptr(), PRESSES);
            gd_log(b"moves: jumps\0".as_ptr(), JUMPS);
            gd_log(b"moves: squats (replaced)\0".as_ptr(), SQUATS);
            gd_log(b"moves: landings\0".as_ptr(), LANDINGS);
        }
        if fr == 900 {
            let f = gd_fighter_from_slot(0);
            if f != 0 && gd_fighter_get_action_state(f) == 14 {
                gd_log(b"moves: request Squat for slot 0, accepted\0".as_ptr(), gd_fighter_request_action_state(f, 39));
            }
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
const PRE: u32 = 1;
const POST: u32 = 2;
const REPLACE: u32 = 3;
static HOOKS: [HookDecl; 5] = [
    HookDecl { address: 0x8006_A1BC, mode: PRE, priority: 0, flags: 0, f: on_input as *const () },
    HookDecl { address: 0x800C_B250, mode: POST, priority: 0, flags: 0, f: on_jump as *const () },
    HookDecl { address: 0x800D_600C, mode: REPLACE, priority: 0, flags: 0, f: squat_replace as *const () },
    HookDecl { address: 0x800D_5AEC, mode: PRE, priority: 0, flags: 0, f: on_landing as *const () },
    HookDecl { address: 0x8039_0CFC, mode: POST, priority: 0, flags: 0, f: on_frame_end as *const () },
];

#[no_mangle]
pub static gdmod_abi_version: u32 = ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0001_0000,
    // GD_FEATURE_REPLACE | GD_FEATURE_ACTION_REQUEST required, GD_FEATURE_LOG optional
    required_features: (1 << 2) | (1 << 1), optional_features: 1 << 3, state_bytes: 16, hook_count: 5, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.moves"), mod_name: text(b"Moves (Rust example)"),
};
