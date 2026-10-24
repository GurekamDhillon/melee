//! The boss-intro camera script entry points of `src/melee/cm/camera.c` (`Camera_8002E6FC` ..
//! `Camera_8002F274`, with `Camera_8002DFE4` / `Camera_8002E234`), over [`CameraWorld`].
//!
//! `Player_GetEntity(slot)` + `ftLib_80086B74(gobj)` (the subject's bone position) live above this
//! crate; the owner of the world installs them in [`CameraWorld::subject_bone_pos`]. Unset, no
//! fighter exists, which is the C's `gobj == NULL` branch.
//!
//! FMA: `fma_sites.py` reports three sites, all in `Camera_8002F0E4`
//! (`x*x + z*z` -> `fma(x, x, z*z)`; `z*z + (x*x + y*y)` -> `fma(z, z, fma(x, x, y*y))`).
//!
//! The callback slots `x344.cb` / `x35C.cb` are called through a one-byte return: the callbacks are
//! `bool` (MSL `int`) functions and only the truth value is read.
use super::camera::CameraWorld;
use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;
use melee_runtime::lbtrigf::atan2f;
use melee_runtime::tu::lbvector::{lbVector_Add, lbVector_Diff, lbVector_Normalize, lbVector_Rotate};
use melee_types::{Vec as Vec3, CAMERA_BOSS_INTRO};

type RawCb = unsafe extern "C" fn(*mut Vec3) -> u8;

#[inline]
unsafe fn call_cb(cb: *const (), out: *mut Vec3) -> bool {
    let f: RawCb = core::mem::transmute(cb);
    f(out) != 0
}

impl CameraWorld {
    fn boss_enter(&mut self) {
        if self.game_camera.mode != CAMERA_BOSS_INTRO {
            self.camera_8002fe38();
        }
    }

    /// The `x341_b1_b2` switch that follows `target_interest` (shared by 8002E6FC / E948 / EF14).
    fn boss_interest_switch(&mut self) {
        unsafe {
            match self.game_camera.x341_b1_b2() {
                1 => {
                    let slot = self.game_camera.x344.slot;
                    if let Some(f) = self.subject_bone_pos {
                        if let Some(p) = f(slot) {
                            self.game_camera.transform.target_interest = p;
                        }
                    }
                }
                2 => self.game_camera.transform.target_interest = self.game_camera.x344.vec,
                3 => {
                    if let Some(cb) = self.game_camera.x344.cb {
                        let mut v: Vec3 = core::mem::zeroed();
                        if call_cb(cb as *const (), &mut v) {
                            self.game_camera.transform.target_interest = v;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// The `x341_b3_b4` switch that follows `target_position`.
    fn boss_position_switch(&mut self) {
        unsafe {
            match self.game_camera.x341_b3_b4() {
                1 => self.game_camera.transform.target_position = self.game_camera.x35C.vec,
                3 => {
                    if let Some(cb) = self.game_camera.x35C.cb {
                        let mut v: Vec3 = core::mem::zeroed();
                        if call_cb(cb as *const (), &mut v) {
                            self.game_camera.transform.target_position = v;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Camera_8002E6FC
    pub fn camera_8002e6fc(&mut self, slot: i32) {
        self.boss_enter();
        self.game_camera.set_x341_b1_b2(1);
        self.game_camera.x344.slot = slot;
        self.boss_interest_switch();
    }

    /// Camera_8002E948
    pub fn camera_8002e948(&mut self, cb: unsafe extern "C" fn(*mut Vec3) -> bool) {
        self.boss_enter();
        self.game_camera.set_x341_b1_b2(3);
        // SAFETY: the slot is read back only through `call_cb`, which reads one byte of the return.
        self.game_camera.x344.cb = Some(unsafe { core::mem::transmute::<_, unsafe extern "C" fn(*mut Vec3) -> i32>(cb) });
        self.boss_interest_switch();
    }

    fn boss_pos_axis_prologue(&mut self, clear_a: u8, clear_b: u8) {
        if self.game_camera.x341_b3_b4() != 2 {
            self.game_camera.set_x341_b3_b4(2);
            unsafe {
                for which in [clear_a, clear_b] {
                    match which {
                        0 => self.game_camera.x35C.bits.set_b0(0),
                        1 => self.game_camera.x35C.bits.set_b1(0),
                        _ => self.game_camera.x35C.bits.set_b2(0),
                    }
                }
            }
        }
    }

    /// Camera_8002EB5C
    pub fn camera_8002eb5c(&mut self, v: f32) {
        self.boss_enter();
        self.boss_pos_axis_prologue(0, 2);
        unsafe {
            self.game_camera.x35C.vec.y = v;
            self.game_camera.x35C.bits.set_b1(1);
        }
        self.boss_position_switch();
    }

    /// Camera_8002EC7C
    pub fn camera_8002ec7c(&mut self, v: f32) {
        self.boss_enter();
        self.boss_pos_axis_prologue(0, 1);
        unsafe {
            self.game_camera.x35C.vec.z = v;
            self.game_camera.x35C.bits.set_b2(1);
        }
        self.boss_position_switch();
    }

    /// Camera_8002ED9C
    pub fn camera_8002ed9c(&mut self, v: f32) {
        self.boss_enter();
        self.boss_pos_axis_prologue(1, 2);
        unsafe {
            self.game_camera.x35C.bits.x2 = v as i32 as i16;
            self.game_camera.x35C.bits.set_b0(1);
        }
        self.boss_position_switch();
    }

    /// Camera_8002F274
    pub fn camera_8002f274(&mut self) {
        self.boss_enter();
        self.game_camera.set_x341_b3_b4(1);
        self.game_camera.x35C.vec = self.game_camera.transform.position;
        self.boss_position_switch();
    }

    /// Camera_8002DFE4(arg0, interest, transform): the eased interest from `arg0` toward
    /// `interest`; true when finished.
    fn camera_8002dfe4(&mut self, arg0: Vec3, interest: Vec3) -> (bool, Vec3) {
        let gc = &self.game_camera;
        let mut done = false;
        let mut f = 0.0f32;
        unsafe {
            match gc.x341_b5_b6() {
                0 => done = true,
                1 => {
                    if gc.x378.s32_v == gc.x37C.s32_v {
                        done = true;
                    } else {
                        f = gc.x378.s32_v as f32 / gc.x37C.s32_v as f32;
                    }
                }
                2 => {
                    if gc.x378.f32_v >= 1.0 {
                        done = true;
                    } else {
                        f = gc.x378.f32_v;
                    }
                }
                _ => {}
            }
        }
        if done {
            (true, interest)
        } else {
            let (mut a, mut b, mut d) = (interest, arg0, Vec3 { x: 0.0, y: 0.0, z: 0.0 });
            unsafe { lbVector_Diff(&mut a, &mut b, &mut d) };
            d.x *= f;
            d.y *= f;
            d.z *= f;
            unsafe { lbVector_Add(&mut d, &mut b) };
            (false, d)
        }
    }

    /// Camera_8002E234: ease `position` to the target (modes 0/1/3) or rebuild both positions from
    /// the spherical (mode 2) script values; true when done.
    fn camera_8002e234(&mut self) -> bool {
        let mut ret = false;
        match self.game_camera.x341_b3_b4() {
            0 | 1 | 3 => {
                let (r, v) = self.camera_8002dfe4(self.game_camera.x368, self.game_camera.transform.target_position);
                self.game_camera.transform.position = v;
                ret = r;
            }
            2 => unsafe {
                ret = true;
                let (mut sp10, mut spc, mut sp8) = (0.0f32, 0.0f32, 0.0f32);
                let x368_yaw = *(&self.game_camera.x368 as *const Vec3 as *const i16) as f32;
                if self.game_camera.x35C.bits.b1() != 0 {
                    let (d, v) = self.camera_8002e158(self.game_camera.x368.y, self.game_camera.x35C.vec.y);
                    ret &= d;
                    sp10 = v;
                }
                if self.game_camera.x35C.bits.b2() != 0 {
                    let (d, v) = self.camera_8002e158(self.game_camera.x368.z, self.game_camera.x35C.vec.z);
                    ret &= d;
                    spc = v;
                }
                if self.game_camera.x35C.bits.b0() != 0 {
                    let (d, v) = self.camera_8002e158(x368_yaw, self.game_camera.x35C.bits.x2 as f32);
                    ret &= d;
                    sp8 = v;
                }
                let interest = self.game_camera.transform.interest;
                let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };
                lbVector_Rotate(&mut sp14, 1, -self.game_camera.x35C.vec.y);
                lbVector_Rotate(&mut sp14, 2, self.game_camera.x35C.vec.z);
                let t = self.game_camera.x35C.bits.x2 as f32;
                lbVector_Normalize(&mut sp14);
                sp14.x *= t;
                sp14.y *= t;
                sp14.z *= t;
                let mut i = interest;
                lbVector_Add(&mut sp14, &mut i);
                self.game_camera.transform.target_position = sp14;
                let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };
                lbVector_Rotate(&mut sp14, 1, -sp10);
                lbVector_Rotate(&mut sp14, 2, spc);
                lbVector_Normalize(&mut sp14);
                sp14.x *= sp8;
                sp14.y *= sp8;
                sp14.z *= sp8;
                let mut i = interest;
                lbVector_Add(&mut sp14, &mut i);
                self.game_camera.transform.position = sp14;
            },
            _ => {}
        }
        ret
    }

    /// Camera_8002EF14
    pub fn camera_8002ef14(&mut self) {
        self.boss_enter();
        self.game_camera.set_x341_b5_b6(0);
        self.boss_interest_switch();
        if self.game_camera.x341_b1_b2() <= 3 {
            let (_, v) = self.camera_8002dfe4(self.game_camera.x350, self.game_camera.transform.target_interest);
            self.game_camera.transform.interest = v;
        }
        self.boss_position_switch();
        self.camera_8002e234();
        self.game_camera.transform.fov = self.game_camera.transform.target_fov;
    }

    /// Camera_8002F0E4
    pub fn camera_8002f0e4(&mut self, frames: i32) {
        self.boss_enter();
        self.game_camera.set_x341_b5_b6(1);
        self.game_camera.x378.s32_v = 0;
        self.game_camera.x37C.s32_v = frames;
        if self.game_camera.x341_b1_b2() <= 3 {
            self.game_camera.x350 = self.game_camera.transform.interest;
        }
        match self.game_camera.x341_b3_b4() {
            0 | 1 | 3 => self.game_camera.x368 = self.game_camera.transform.position,
            2 => unsafe {
                let mut a = self.game_camera.transform.position;
                let mut b = self.game_camera.transform.interest;
                let mut d: Vec3 = core::mem::zeroed();
                lbVector_Diff(&mut a, &mut b, &mut d);
                let f30 = atan2f(d.y, sqrtf(fmadds(d.x, d.x, d.z * d.z)));
                let f31 = atan2f(d.x, d.z);
                let len = sqrtf(fmadds(d.z, d.z, fmadds(d.x, d.x, d.y * d.y)));
                *(&mut self.game_camera.x368 as *mut Vec3 as *mut i16) = len as i32 as i16;
                self.game_camera.x368.y = f30;
                self.game_camera.x368.z = f31;
            },
            _ => {}
        }
        self.game_camera.x374 = self.game_camera.transform.fov;
    }
}

/// The one `game_camera` of the running game (camera.c's global), created on first use.
static mut GLOBAL_WORLD: Option<CameraWorld> = None;

/// `game_camera` and the other camera.c globals.
pub fn global() -> &'static mut CameraWorld {
    // SAFETY: single-threaded game state, like the C globals.
    unsafe { (*(&raw mut GLOBAL_WORLD)).get_or_insert_with(CameraWorld::default) }
}
