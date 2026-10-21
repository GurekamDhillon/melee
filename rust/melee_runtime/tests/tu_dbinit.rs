#![allow(non_snake_case)]
use core::ffi::c_void;
use melee_runtime::tu::dbeffect::pad;
use melee_runtime::tu::dbinit::{db_filter_dpad, dblkind, DbInit, DbInitHost, DbPadIn};
use melee_types::{HSD_GObj, PADStatus};

/// db_filter_dpad against the C block cut from db_RunEveryFrame (oracle/tu_db_ref.sh).
#[test]
fn filter_dpad_matches_c() {
    let txt = include_str!("data/tu_db_ref.txt");
    let mut n = 0;
    for l in txt.lines().filter(|l| l.starts_with("dpad ")) {
        let v: Vec<i32> = l[5..].split(' ').map(|x| x.parse().unwrap()).collect();
        assert_eq!(db_filter_dpad(v[0]), v[1], "input {}", v[0]);
        n += 1;
    }
    assert_eq!(n, 264);
}

#[derive(Default)]
struct H {
    pads: [DbPadIn; 4],
    master: bool,
    log: Vec<String>,
    pad_reads: u32,
    probes: u32,
    plink: Vec<*mut HSD_GObj>,
    names: [*mut *mut i8; 3],
}
impl DbInitHost for H {
    fn VIWaitForRetrace(&mut self) { self.log.push("vi".into()) }
    fn PADRead(&mut self, s: &mut [PADStatus; 4]) {
        self.pad_reads += 1;
        // first read: pad 0 still being probed (-2); then pad 0 unplugged (-1), pad 1 OK.
        for (i, p) in s.iter_mut().enumerate() {
            p.err = if self.pad_reads == 1 && i == 0 { -2 } else if i == 1 { 0 } else { -1 };
            p.button = 0x100 + i as u16;
        }
    }
    fn CARDProbeEx(&mut self, _: i32) -> i32 { self.probes += 1; if self.probes < 3 { -1 } else { 0 } }
    fn lbArchive_LoadSymbols(&mut self, f: &str, s: &str) -> *mut c_void {
        assert_eq!((f, s), ("DbCo.dat", "dbLoadCommonData"));
        self.names.as_mut_ptr() as *mut c_void
    }
    fn OSReport(&mut self, s: &str) { self.log.push(s.into()) }
    fn HSD_GObjPLinkHead(&mut self, i: usize) -> *mut HSD_GObj { if i == 3 { self.plink[0] } else { core::ptr::null_mut() } }
    fn HSD_PadMasterStatus(&mut self, i: usize) -> DbPadIn { self.pads[i] }
    fn ftLib_IsMasterHandPresent(&mut self) -> bool { self.master }
    fn ftLib_IsCrazyHandPresent(&mut self) -> bool { false }
    fn stack_addr(&mut self) -> usize { 0x1000 }
    fn stack_end(&mut self) -> usize { 0x800 }
    fn read_u8(&mut self, a: usize) -> u8 { if a < 0x810 { 0xAA } else { 0 } }
    fn fn_SetupCpuHandicapInfo(&mut self) { self.log.push("setup".into()) }
    fn fn_SetupAnimationInfo(&mut self) { self.log.push("setup".into()) }
    fn fn_SetupItemAndPokemonMenu(&mut self) { self.log.push("setup".into()) }
    fn fn_SetupSoundInfo(&mut self) { self.log.push("setup".into()) }
    fn fn_SetupMiscStageVisuals(&mut self) { self.log.push("setup".into()) }
    fn fn_SetupCameraInfo(&mut self) { self.log.push("setup".into()) }
    fn fn_SetupBonusInfo(&mut self) { self.log.push("setup".into()) }
    fn fn_SetupObjAllocLimiter(&mut self) { self.log.push("setup".into()) }
    fn fn_Setup5xSpeed(&mut self) { self.log.push("setup".into()) }
    fn fn_CheckMiscVisualEffects(&mut self, _: i32) {}
    fn fn_CheckCameraInfo(&mut self, p: i32, d: i32, pr: i32, x: f32, _: f32) { self.log.push(format!("cam {p} {d} {pr} {x}")) }
    fn fn_CheckSoundInfo(&mut self, _: i32) {}
    fn fn_UpdateObjAllocLimiter(&mut self, _: i32) {}
    fn fn_CheckItemAndPokemonMenu(&mut self, _: i32) {}
    fn fn_CheckCpuHandicapInfo(&mut self, _: i32) {}
    fn fn_UpdateCpuHandicapInfo(&mut self) {}
    fn fn_CheckAnimationInfo(&mut self, _: i32) {}
    fn fn_UpdateAnimationInfo(&mut self) {}
    fn fn_CheckMiscStageEffects(&mut self, _: i32) {}
    fn fn_CheckBonusInfo(&mut self, _: i32) {}
    fn fn_Check5xSpeed(&mut self, p: i32) { self.log.push(format!("5x {p}")) }
}

#[test]
fn launch_button_state_waits_for_pads_and_card() {
    let mut s = DbInit::default();
    let mut h = H::default();
    s.db_GetGameLaunchButtonState(&mut h);
    assert_eq!(h.pad_reads, 2);
    assert_eq!(s.game_launch_button_state, 0x101);
    assert_eq!(h.probes, 3);
}

#[test]
fn run_every_frame_latches_and_filters() {
    let mut s = DbInit::default();
    let mut h = H::default();
    s.db_RunEveryFrame(&mut h);
    assert!(h.log.is_empty(), "NoDebugRom does nothing");
    s.level = dblkind::DEBUG_ROM;
    h.pads[0].button = pad::A | pad::DPADUP | pad::DPADLEFT;
    h.pads[0].nml_sub_stick_x = 0.5;
    s.db_RunEveryFrame(&mut h);
    assert_eq!(s.db_ButtonsDown(0), pad::A);
    assert_eq!(s.db_ButtonsPressed(0), pad::A);
    h.pads[0].button = pad::B;
    s.db_RunEveryFrame(&mut h);
    assert_eq!(s.button_states[0].released, pad::A);
    assert_eq!(s.db_ButtonsPressed(0), pad::B);
    assert!(h.log.contains(&format!("cam 0 {} {} 0.5", pad::B, pad::B)));
    // Master Hand: only players 0-1 latch, but every check runs for 4.
    h.master = true;
    h.pads[3].button = pad::X;
    s.db_RunEveryFrame(&mut h);
    assert_eq!(s.db_ButtonsDown(3), 0);
    assert_eq!(h.log.iter().filter(|l| l.starts_with("5x")).count(), 12);
}

#[test]
fn setup_reads_common_data() {
    let mut s = DbInit { level: dblkind::DEVELOP, ..Default::default() };
    let mut h = H::default();
    let mut a = 0i8;
    let mut b = 0i8;
    h.names = [(&mut a as *mut i8).cast(), (&mut b as *mut i8).cast(), core::ptr::null_mut()];
    s.button_states[2].current = 7;
    s.db_Setup(&mut h);
    assert_eq!(s.bonus_names, h.names[0]);
    assert_eq!(s.motionstate_names, h.names[1]);
    assert_eq!(s.button_states[2].current, 0);
    assert_eq!(h.log.len(), 9);
}

#[test]
fn print_entity_counts_and_thread_info() {
    let s = DbInit::default();
    let mut g: [HSD_GObj; 2] = unsafe { core::mem::zeroed() };
    let p1: *mut HSD_GObj = &mut g[1];
    g[0].next = p1;
    let mut h = H { plink: vec![&mut g[0]], ..Default::default() };
    s.db_PrintEntityCounts(&mut h);
    assert_eq!(h.log[1], "    0 ");
    assert_eq!(h.log[14], "    0 ");
    assert_eq!(h.log[16], "    2 ");
    h.log.clear();
    s.db_PrintThreadInfo(&mut h);
    assert_eq!(h.log[1], "base:1000, end:800, size:2048 peak:2032 \n");
}
