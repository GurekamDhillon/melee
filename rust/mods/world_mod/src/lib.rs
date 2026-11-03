//! world_mod: exercises the item and stage rows of the generic host (notes/modhost.md).
//!
//! * `item.spawn` pre: logs each spawn request's kind and height (`gd_spawn_get_kind/pos`); post counts the result.
//! * `item.change_state` post: counts item state changes.
//! * `stage.update` pre: counts stage updates; `stage.on_start` post: logs the stage kind.
//! * `frame.run_procs` post: every 600 frames logs the counters, the live item count (walking `gd_item_at` with
//!   `gd_item_read`) and a value from the host's trig (`gd_sinf`), so a host trig change shows in the log.
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[repr(C)]
#[derive(Default)]
struct Vec3 { x: f32, y: f32, z: f32 }
#[repr(C)]
#[derive(Default)]
struct ItemView { pos: Vec3, vel: Vec3, facing: f32, kind: i32, owner_slot: i32, reserved: [u32; 4] }

#[link(wasm_import_module = "gd")]
extern "C" {
    fn gd_spawn_get_kind(s: u32) -> i32;
    fn gd_spawn_get_pos(s: u32, out: *mut Vec3);
    fn gd_item_count() -> i32;
    fn gd_item_at(i: i32) -> u32;
    fn gd_item_read(it: u32, out: *mut ItemView);
    fn gd_stage_kind() -> i32;
    fn gd_sinf(x: f32) -> f32;
    fn gd_match_frame() -> i32;
    fn gd_log(msg: *const u8, value: i32);
}

static mut SPAWNS: i32 = 0;
static mut SPAWNED: i32 = 0;
static mut CHANGES: i32 = 0;
static mut STAGE_UPDATES: i32 = 0;

extern "C" fn spawn_pre(s: u32) -> i32 {
    unsafe {
        SPAWNS += 1;
        let mut p = Vec3::default();
        gd_spawn_get_pos(s, &mut p);
        gd_log(b"world: spawn kind\0".as_ptr(), gd_spawn_get_kind(s));
        gd_log(b"world: spawn y (x100)\0".as_ptr(), (p.y * 100.0) as i32);
    }
    0
}

extern "C" fn spawn_post(_s: u32, ret: u32) -> u32 {
    unsafe {
        if ret != 0 {
            SPAWNED += 1;
        }
    }
    ret
}

extern "C" fn item_state(_it: u32, _state: i32, _flags: i32) {
    unsafe { CHANGES += 1 };
}

extern "C" fn stage_update(_g: u32) -> i32 {
    unsafe { STAGE_UPDATES += 1 };
    0
}

extern "C" fn stage_start(_p: u32) {
    unsafe { gd_log(b"world: stage kind\0".as_ptr(), gd_stage_kind()) };
}

extern "C" fn on_frame_end() {
    unsafe {
        if gd_match_frame() % 600 == 0 {
            let n = gd_item_count();
            let mut owned = 0;
            for i in 0..n {
                let mut v = ItemView::default();
                gd_item_read(gd_item_at(i), &mut v);
                if v.owner_slot >= 0 {
                    owned += 1;
                }
            }
            gd_log(b"world: spawn requests / spawned\0".as_ptr(), SPAWNS * 1000 + SPAWNED);
            gd_log(b"world: item state changes\0".as_ptr(), CHANGES);
            gd_log(b"world: live items / owned\0".as_ptr(), n * 1000 + owned);
            gd_log(b"world: stage updates\0".as_ptr(), STAGE_UPDATES);
            gd_log(b"world: sin(1) x 1e6\0".as_ptr(), (gd_sinf(1.0) * 1_000_000.0) as i32);
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
static HOOKS: [HookDecl; 6] = [
    HookDecl { address: 0x8026_862C, mode: PRE, priority: 0, flags: 0, f: spawn_pre as *const () },
    HookDecl { address: 0x8026_862C, mode: POST, priority: 0, flags: 0, f: spawn_post as *const () },
    HookDecl { address: 0x8026_8E5C, mode: POST, priority: 0, flags: 0, f: item_state as *const () },
    HookDecl { address: 0x801C_0C2C, mode: PRE, priority: 0, flags: 0, f: stage_update as *const () },
    HookDecl { address: 0x801C_0FB8, mode: POST, priority: 0, flags: 0, f: stage_start as *const () },
    HookDecl { address: 0x8039_0CFC, mode: POST, priority: 10, flags: 0, f: on_frame_end as *const () },
];

#[no_mangle]
pub static gdmod_abi_version: u32 = ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: 0, optional_features: 1 << 3, state_bytes: 16, hook_count: 6, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.world"), mod_name: text(b"World (Rust example)"),
};
