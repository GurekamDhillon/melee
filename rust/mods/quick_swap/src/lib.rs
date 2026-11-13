//! quick_swap: the quick-swap cast prototype (docs/rust-port/notes/quick-swap.md). Off unless this mod is loaded
//! (`MELEE_WASM_MODS`) and a port has a partner (`MELEE_TAG_PARTNER`); best with `MELEE_TAG_INPLACE=1` (one fighter
//! object per port, the benched character spawned from preloaded files at the swap).
//!
//! * swap: D-pad down (not a retail in-match control; the same button as tag_fighter's classic tag) while the
//!   fighter is free to act: `gd_tag_quick(slot, 0, 0)` (melee_modes::tu::tag_swap::quick): no freeze, the swap is
//!   applied between logic frames after the post chain.
//! * cooldown: COOLDOWN logic frames per port after a swap the engine accepted.
//!
//! All state is this module's linear memory (rolled back with the game). Reads only the synced sim input, so both
//! netplay peers and a replay swap on the same frame.
#![no_std]
#![allow(static_mut_refs)]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd")]
extern "C" {
    fn gd_fighter_from_slot(slot: i32) -> u32;
    fn gd_fighter_get_input(h: u32, out: *mut u32);
    fn gd_tag_partner(slot: i32) -> i32;
    fn gd_tag_quick(slot: i32, any: i32, pause: i32) -> i32;
    fn gd_log(msg: *const u8, value: i32);
}

/// Frames after a swap before the same port can swap again (0.5 s).
const COOLDOWN: i32 = 30;
const DPAD_DOWN: u32 = 0x0004;

static mut CD: [i32; 4] = [0; 4];
static mut HELD: [u8; 4] = [0; 4];

extern "C" fn on_frame_end() {
    unsafe {
        for slot in 0..4i32 {
            let s = slot as usize;
            if CD[s] > 0 {
                CD[s] -= 1;
            }
            if gd_tag_partner(slot) < 0 {
                continue;
            }
            let h = gd_fighter_from_slot(slot);
            if h == 0 {
                continue;
            }
            let mut w = [0u32; 8];
            gd_fighter_get_input(h, w.as_mut_ptr());
            let down = (w[6] & DPAD_DOWN != 0) as u8;
            // a press, not a hold: a held D-pad never swaps twice
            let press = down != 0 && HELD[s] == 0;
            HELD[s] = down;
            if !press || CD[s] > 0 {
                continue;
            }
            let rc = gd_tag_quick(slot, 0, 0);
            if rc == 0 {
                CD[s] = COOLDOWN;
                gd_log(b"quick_swap: swap, slot\0".as_ptr(), slot);
            } else {
                gd_log(b"quick_swap: refused, code\0".as_ptr(), rc);
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
const POST: u32 = 2;
static HOOKS: [HookDecl; 1] = [
    HookDecl { address: 0x8039_0CFC, mode: POST, priority: 50, flags: 0, f: on_frame_end as *const () },
];

#[no_mangle]
pub static gdmod_abi_version: u32 = ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: 0, optional_features: 0, state_bytes: 32, hook_count: 1, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.quick_swap"), mod_name: text(b"Quick-swap cast"),
};
