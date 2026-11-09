//! tag_fighter: two-character tag teams (an idea after Joyastick's melee-pc-TagFighter; clean-room, no code or
//! assets from it). The sim half: it reads the tag button, checks the swap is allowed and drives the transition.
//! The engine does the swap natively (melee_modes::tu::tag_swap: the retail Zelda / Sheik transform path), the
//! freeze is the retail pause mask, and the cinematic is the presentation mod rust/mods/tag_fighter_ui.
//!
//! * team: `MELEE_TAG_PARTNER` (engine env) gives each port a partner, spawned asleep beside the main fighter.
//! * tag: press D-pad down while grounded and actionable (Wait / walk / dash / run / turn / squat), no item held,
//!   not grabbed; 2 s cooldown per port after a tag.
//! * transition: FREEZE for `LEN` logic frames (everything frozen; the cinematic plays), then SWAP + RELEASE in
//!   one step between frames. D-pad down again during the freeze (after the first 6 frames) skips to the swap:
//!   the skip is a pad read of the synced sim input, so both peers skip on the same frame.
//!
//! All state is this module's linear memory (rolled back with the game) or the engine's transition record.
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
    fn gd_tag_check(slot: i32) -> i32;
    fn gd_tag_begin(slot: i32, len: i32) -> i32;
    fn gd_tag_swap(skip: i32) -> i32;
    fn gd_transition_phase() -> i32;
    fn gd_transition_frame() -> i32;
    fn gd_transition_slot() -> i32;
    fn gd_pad_pressed(port: i32) -> u32;
    fn gd_log(msg: *const u8, value: i32);
}

/// Freeze length in logic frames (1.1 s): the cinematic's timeline (tag_fighter_ui) is built on it.
const LEN: i32 = 66;
/// Frames after a tag before the same port can tag again.
const COOLDOWN: i32 = 120;
/// The skip is not taken in the first frames (a held D-pad press must not skip its own tag).
const SKIP_AFTER: i32 = 6;
const DPAD_DOWN: u32 = 0x0004;

static mut CD: [i32; 4] = [0; 4];

extern "C" fn on_frame_end() {
    unsafe {
        if gd_transition_phase() == 1 {
            let slot = gd_transition_slot();
            let f = gd_transition_frame();
            if f >= SKIP_AFTER && slot >= 0 && gd_pad_pressed(slot) & DPAD_DOWN != 0 {
                gd_tag_swap(1);
            } else if f + 1 >= LEN {
                gd_tag_swap(0);
            }
            return;
        }
        for c in CD.iter_mut() {
            if *c > 0 {
                *c -= 1;
            }
        }
        for slot in 0..4 {
            if CD[slot as usize] > 0 || gd_tag_partner(slot) < 0 {
                continue;
            }
            let h = gd_fighter_from_slot(slot);
            if h == 0 {
                continue;
            }
            let mut w = [0u32; 8];
            gd_fighter_get_input(h, w.as_mut_ptr());
            if w[6] & DPAD_DOWN == 0 {
                continue;
            }
            let rc = gd_tag_check(slot);
            if rc != 0 {
                continue;
            }
            if gd_tag_begin(slot, LEN) == 0 {
                CD[slot as usize] = COOLDOWN;
                gd_log(b"tag_fighter: tag, slot\0".as_ptr(), slot);
                break; // one transition at a time
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
    required_features: 0, optional_features: 0, state_bytes: 16, hook_count: 1, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.tag_fighter"), mod_name: text(b"Tag fighter"),
};
