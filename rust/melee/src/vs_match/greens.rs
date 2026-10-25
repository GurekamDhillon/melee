//! Green Greens per-frame stage procs (`src/melee/gr/grgreens.c`): the wind state machine of
//! Whispy Woods (`grGreens_80213C10`, map gobj 5) and the block gobj procs (`grGreens_8021479C`,
//! map gobj 6: `grGreens_802166C4` / `grGreens_80216C20`).
//!
//! The stage procs start with the Ready/GO callback (`Ground_801C0FB8` runs the deferred
//! `fn_80214658`, which clears the blocks' "starting" flag and `grGr_804D6AAD`): before that the
//! map animations step but the state machines do not.
//!
//! Not ported (each returns `Err("... not ported")` where reached): the wind force
//! (`lb_80011A50`) and the fighters' wind reaction, the apples of Whispy (`it_802EE200`), spawning
//! and breaking blocks. Audio and the camera quake are not simulated.

use super::*;
use melee_runtime::aobj::{HsdAObjDesc, AOBJ_LOOP, AOBJ_REWINDED};

/// What `greens_init` drew and tabulated.
pub struct InitDraws {
    /// `u.greens.xC` of the block gobj (`randrange(blockTimerMin, blockTimerMax)`).
    pub block_timer: i32,
    /// `u.greens2.xC` of the wind gobj.
    pub wind_xc: i32,
    /// `u.greens2.x14`.
    pub wind_x14: i32,
    /// `grGr_8049F9E0`: block type per cell (0 none, 1 normal, 2 bomb).
    pub table: [u8; 30],
    /// `gp->u.greens.x4`: the block cells' positions (row, column), `grGreens_8021483C`.
    pub grid: [[(f32, f32); 6]; 5],
}

/// `struct grGreens_YakumonoParam` (words).
pub(super) struct Yak(pub(super) Vec<u32>);

impl Yak {
    pub(super) fn i(&self, off: usize) -> i32 {
        self.0[off / 4] as i32
    }

    pub(super) fn f(&self, off: usize) -> f32 {
        f32::from_bits(self.0[off / 4])
    }
}

/// `gp->u.greens2`.
#[derive(Default)]
struct Wind {
    x0: i32,
    x4: i32,
    x8: i32,
    xc: i32,
    x10: i32,
    x14: i32,
    x18: i32,
    x1c: i32,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Block {
    /// `status` (0 none, 1 falling, 2 stacked on a falling one, 3 resting).
    pub(super) status: i32,
    /// `x18`: the map joint of the block (= `index`).
    pub(super) joint: i32,
    /// `x1_1`: a bomb block.
    pub(super) bomb: bool,
    /// `x1_2`: the bomb is exploding.
    pub(super) x1_2: bool,
    /// `x1_3`: to be removed.
    pub(super) x1_3: bool,
    /// `x1_4`: a fighter stands on it (the joint's collision callback).
    pub(super) x1_4: bool,
    /// `x1_5`: a resting block whose neighbours' joints need stitching.
    pub(super) x1_5: bool,
    /// `x1_6`: removed; its neighbours' diagonal stitches are redone.
    pub(super) x1_6: bool,
    /// `x1_7`: hit.
    pub(super) x1_7: bool,
    /// `x4`: the fall speed; `x8`: the height.
    pub(super) x4: f32,
    pub(super) x8: f32,
    /// The block's item (`SimItem` `GreensBlock { id }`).
    pub(super) item: u32,
}

pub struct Greens {
    stage: std::sync::Arc<DatArchive>,
    pub(super) yak: Yak,
    w: Wind,
    /// The AObj `grAnime_801C8318(gobj 5)` finds: the timing of Whispy's animation.
    timing: Option<HsdAObj<()>>,
    pub(super) blocks: [[Block; 6]; 5],
    pub(super) grid: [[(f32, f32); 6]; 5],
    /// The map gobj 6 JObj tree (block JObjs 5..34 under the `Ground_801C0498()` scale root).
    pub(super) tree: crate::fighter_data::skeleton::MapTree,
    /// `Ground_801C0498()`.
    pub(super) scale: f32,
    pub(super) next_item: u32,
    /// `u.greens.xC`.
    pub(super) block_timer: i32,
    /// The stage procs run (`fn_80214658` ran at GO).
    started: bool,
}

const GR_803E7734: [i32; 10] = [0, 1, 0, 1, 0, 2, 0, 1, 0, 2];
const GR_803E775C: [i32; 8] = [0, 0, 0, 14, 0, 0, 0, 14];
const GR_803E777C: [[i32; 2]; 7] = [[1, 8], [15, 16], [2, 9], [3, 10], [4, 11], [5, 12], [6, 13]];

pub(super) fn randrange(rand: &mut HsdRand, min: i32, max: i32) -> i32 {
    if min > max {
        let d = min - max;
        max + if d != 0 { rand.randi(d) } else { 0 }
    } else if min < max {
        let d = max - min;
        min + if d != 0 { rand.randi(d) } else { 0 }
    } else {
        min
    }
}

impl Greens {
    pub fn new(stage: std::sync::Arc<DatArchive>, d: InitDraws) -> Greens {
        let yak = Yak(crate::stage::yakumono_words(&stage, 0x1E).expect("yakumono_param"));
        // grGreens_80214FA8 -> grGreens_80215358: the lowest free block index per created cell.
        let mut blocks = [[Block::default(); 6]; 5];
        let mut num = 0;
        for row in 0..5 {
            for col in 0..6 {
                if matches!(d.table[row * 6 + col], 1 | 2) {
                    blocks[row][col] = Block { status: 3, joint: num, x1_5: true, bomb: d.table[row * 6 + col] == 2, item: num as u32, x8: d.grid[row][col].1, ..Default::default() };
                    num += 1;
                }
            }
        }
        let scale = f32::from_bits(u32::from_be_bytes(stage.data[crate::stage::ground_param(&stage).expect("grGroundParam") as usize..][..4].try_into().unwrap()));
        let tree = {
            let dd = &stage.data;
            let head = stage.public_offset("map_head").expect("map_head");
            let gobjs = u32::from_be_bytes(dd[head as usize + 8..][..4].try_into().unwrap());
            let root = u32::from_be_bytes(dd[(gobjs + 6 * 0x34) as usize..][..4].try_into().unwrap());
            crate::fighter_data::skeleton::MapTree::load(dd, root, scale).expect("map gobj 6")
        };
        let mut tree = tree;
        {
            // grGreens_80215358: each initial block's JObj sits at its cell (translate = cell / Ground_801C0498()).
            let f = (1.0f64 / scale as f64) as f32;
            for row in 0..5 {
                for col in 0..6 {
                    let b = blocks[row][col];
                    if b.status != 0 {
                        let j = tree.dfs[5 + b.joint as usize];
                        tree.set_translate(j, Vec3 { x: d.grid[row][col].0 * f, y: d.grid[row][col].1 * f, z: 0.0 });
                    }
                }
            }
        }
        Greens { grid: d.grid, tree, scale, next_item: 100, stage, yak, w: Wind { xc: d.wind_xc, x10: 1, x14: d.wind_x14, ..Default::default() }, timing: None, blocks, block_timer: d.block_timer, started: false }
    }
}

impl VsMatch {
    /// `Ground_801C0FB8`'s deferred callbacks (at GO): the stage procs start.
    pub(super) fn greens_start(&mut self) {
        if let Some(g) = &mut self.greens {
            g.started = true;
        }
    }

    /// The map animations step (`Ground_801C1CD0`, s_link 1) of the AObj the state machine reads.
    pub(super) fn greens_step_anim(&mut self) {
        if let Some(g) = &mut self.greens {
            if let Some(a) = &mut g.timing {
                let mut none: melee_runtime::fobj::ObjUpdate<'_> = None;
                a.interpret_anim(&mut self.aobj_globals, &mut none);
            }
        }
    }

    /// `grAnime_801C8138(gobj, map_id, idx)` for map gobj `g`: replace the animation of the gobj,
    /// run the frame-0 hold (`HSD_JObjAnimAll`) and spawn its frame-0 particle keys.
    fn greens_set_anim(&mut self, g: u32, idx: i32) -> Result<(), String> {
        let stage = self.greens.as_ref().unwrap().stage.clone();
        self.dptcl.retain(|t| t.gobj != g);
        let mut spawns = Vec::new();
        for t in crate::stage::dptcl_tracks(&stage, g, idx as u32) {
            let mut a: HsdAObj<()> = HsdAObj::load_desc(&t.aobj, |_| None);
            a.req_anim(0.0);
            a.set_rate(1.0);
            self.joint_mtx.insert(t.joint, crate::stage::bind_pose_matrix(&stage.data, &t.chain));
            let joint = t.joint;
            let mut cb = |ty: u8, fv: f32| {
                if ty == 0x28 {
                    let iv = fv.to_bits() as i32;
                    spawns.push((iv & 0x3F, (iv >> 6) & 0xFF_FFFF, joint));
                }
            };
            let mut upd: melee_runtime::fobj::ObjUpdate<'_> = Some(&mut cb);
            a.interpret_anim(&mut self.aobj_globals, &mut upd);
            self.dptcl.push(DptclAnim { joint: t.joint, aobj: a, started: true, gobj: g, skip: true });
        }
        if g == 5 {
            let timing = crate::stage::anim_timing(&stage, g, idx as u32).map(|(flags, end)| {
                let mut a: HsdAObj<()> = HsdAObj::load_desc(&HsdAObjDesc { flags, end_frame: end, fobjdesc: Vec::new(), obj_id: 0 }, |_| None);
                a.req_anim(0.0);
                if crate::stage::anim_loops(&stage, g, idx as u32) {
                    a.set_flags(AOBJ_LOOP);
                }
                let mut none: melee_runtime::fobj::ObjUpdate<'_> = None;
                a.interpret_anim(&mut self.aobj_globals, &mut none);
                a
            });
            self.greens.as_mut().unwrap().timing = timing;
        }
        self.dptcl_spawns(spawns, None)
    }

    /// `grAnime_801C83D0` (finished: `AOBJ_NO_ANIM`) / `grAnime_801C84A4` (`AOBJ_REWINDED`) on map
    /// gobj 5.
    fn greens_anim_flag(&self, flag: u32) -> bool {
        self.greens.as_ref().and_then(|g| g.timing.as_ref()).map_or(false, |a| a.flags & flag != 0)
    }

    /// The stage gobjs' procs at s_link 4: the block gobj (6), then the wind gobj (5).
    pub(super) fn greens_procs(&mut self) -> Result<(), String> {
        if !self.greens.as_ref().map_or(false, |g| g.started) {
            return Ok(());
        }
        self.greens_blocks()?;
        self.greens_wind()
    }

    /// `ftLib_800864A8(&pos, NULL)`: +1 when more fighters are right of `x` than left of it,
    /// else -1 (a tie draws `HSD_Randi(2)`).
    fn ftlib_800864a8(&mut self, x: f32) -> f32 {
        let mut result = 0i32;
        for p in &mut self.players {
            if p.fp.x221F_b3() != 0 {
                continue;
            }
            let bone = p.fp.co_attrs.camera_zoom_target_bone as usize;
            let o = p.fp.co_attrs.x170;
            let j = p.skel.part(bone);
            let v = p.skel.world_point(j, Vec3 { x: o.x, y: o.y, z: o.z });
            result += if v.x - x < 0.0 { -1 } else { 1 };
        }
        if result == 0 {
            result = if self.rand.randi(2) != 0 { 1 } else { -1 };
        }
        if result < 0 {
            -1.0
        } else {
            1.0
        }
    }

    /// `grGreens_80213C10`.
    fn greens_wind(&mut self) -> Result<(), String> {
        self.greens.as_mut().unwrap().w.x18 = 0;
        let (x4, x10) = {
            let w = &self.greens.as_ref().unwrap().w;
            (w.x4, w.x10)
        };
        self.greens.as_mut().unwrap().w.x1c += 1;
        match x4 {
            0 => {
                if x10 != 0 {
                    let (y38, y34) = {
                        let y = &self.greens.as_ref().unwrap().yak;
                        (y.i(0x38), y.i(0x34))
                    };
                    let x8 = self.rand.randi(8);
                    let xc = randrange(&mut self.rand, y38, y34);
                    {
                        let w = &mut self.greens.as_mut().unwrap().w;
                        w.x10 = 0;
                        w.x8 = x8;
                        w.xc = xc;
                    }
                    self.greens_set_anim(5, GR_803E775C[x8 as usize])?;
                    self.greens_set_anim(4, GR_803E775C[x8 as usize])?;
                    return Ok(());
                }
                self.greens.as_mut().unwrap().w.xc -= 1;
                if self.greens_anim_flag(AOBJ_REWINDED) {
                    if self.greens.as_ref().unwrap().w.xc > 0 {
                        let x8 = self.rand.randi(8);
                        self.greens.as_mut().unwrap().w.x8 = x8;
                        self.greens_set_anim(5, GR_803E775C[x8 as usize])?;
                        self.greens_set_anim(4, GR_803E775C[x8 as usize])?;
                        return Ok(());
                    }
                    let w = &mut self.greens.as_mut().unwrap().w;
                    w.x0 = (w.x0 + 1) % 10;
                    w.x4 = GR_803E7734[w.x0 as usize];
                    w.x10 = 1;
                }
                Ok(())
            }
            1 => {
                if x10 != 0 {
                    let side = self.ftlib_800864a8(0.0);
                    let dir = if side == 1.0 { 1 } else { 0 };
                    {
                        let w = &mut self.greens.as_mut().unwrap().w;
                        w.x10 = 0;
                        w.x8 = 0;
                        w.x14 = dir;
                        w.xc = 0;
                    }
                    self.greens_set_anim(5, GR_803E777C[0][dir as usize])?;
                    self.greens_set_anim(4, GR_803E777C[0][dir as usize])?;
                    return Ok(());
                }
                Err("grGreens_80213C10: wind blowing (state 1) not ported".into())
            }
            2 => Err("grGreens_80213C10: apples of Whispy (state 2) not ported".into()),
            s => Err(format!("grGreens_80213C10: state {s}")),
        }
    }
}
