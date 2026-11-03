//! hello_mod: the Rust twin of docs/specs/examples/hello_mod.c. Every damage slot 0 takes costs it 1% extra, and
//! every 600 frames the mod logs how many hits it counted. State is plain statics in linear memory (rolled back by
//! the host with the rest of the game).
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd")]
extern "C" {
    fn gd_fighter_slot(f: u32) -> i32;
    fn gd_fighter_get_percent(f: u32) -> f32;
    fn gd_fighter_set_percent(f: u32, p: f32);
    fn gd_match_frame() -> i32;
    fn gd_log(msg: *const u8, value: i32);
}

static mut HITS: u32 = 0;

extern "C" fn on_damage(f: u32, d: f32) {
    unsafe {
        if gd_fighter_slot(f) == 0 && d > 0.0 {
            gd_fighter_set_percent(f, gd_fighter_get_percent(f) + 1.0);
            HITS = HITS.wrapping_add(1);
        }
    }
}

extern "C" fn on_frame_end() {
    unsafe {
        if gd_match_frame() % 600 == 0 {
            gd_log(b"hello: hits on slot 0 so far\0".as_ptr(), HITS as i32);
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
static HOOKS: [HookDecl; 2] = [
    HookDecl { address: 0x8006_CC7C, mode: 2, priority: 0, flags: 0, f: on_damage as *const () },
    HookDecl { address: 0x8039_0CFC, mode: 2, priority: 0, flags: 0, f: on_frame_end as *const () },
];

#[no_mangle]
pub static gdmod_abi_version: u32 = ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: 0, optional_features: 1 << 3, state_bytes: 4, hook_count: 2, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.hello"), mod_name: text(b"Hello (Rust)"),
};
