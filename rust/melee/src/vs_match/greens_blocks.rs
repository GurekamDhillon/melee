//! Green Greens' falling blocks (`grgreens.c`): `grGreens_802166C4` (the spawn roll and the fall/removal loops),
//! `grGreens_80215358` (a block's creation), `grGreens_80215ED8` (one block's fall), `Ground_UpdateMapColl` (the
//! map joints follow the block JObjs) and `grGreens_80216C20` (the stitches of resting blocks).
//!
//! Not ported: a fighter standing on a falling block (`x1_4`), a hit or exploding block (`x1_2`, `x1_3`), the removal
//! loop and the diagonal stitches (`grGreens_802150C4`).

use super::greens::{randrange, Block, Greens};
use super::*;

impl VsMatch {
    /// `grGreens_8021479C`: `grGreens_802166C4`, `Ground_UpdateMapColl`, `grGreens_80216C20`.
    pub(super) fn greens_blocks(&mut self) -> Result<(), String> {
        let t = self.greens.as_ref().unwrap().block_timer;
        self.greens.as_mut().unwrap().block_timer -= 1;
        if t < 0 {
            self.greens_spawn_roll()?;
        }
        // grGreens_80215ED8 for every cell.
        for row in 0..5usize {
            for col in 0..6usize {
                self.greens_block_step(row, col)?;
            }
        }
        // The removal loop of grGreens_802166C4.
        if self.greens.as_ref().unwrap().blocks.iter().flatten().any(|b| b.x1_3) {
            return Err("grGreens_802166C4: removing a block (x1_3) not ported".into());
        }
        // Ground_UpdateMapColl(gobj 6): mpLib_80055E9C of every map joint with the block JObjs' matrices.
        for num in 0..30usize {
            let used = self.greens.as_ref().unwrap().blocks.iter().flatten().any(|b| b.status != 0 && b.joint as usize == num);
            if used {
                let m = {
                    let g = self.greens.as_mut().unwrap();
                    let j = g.tree.dfs[5 + num];
                    g.tree.mtx_of(j)
                };
                if !(m[0][0] == m[1][1] && m[0][0] == m[2][2]) {
                    return Err("mpLib_80055E9C: non-uniform block matrix not ported".into());
                }
                self.ground.mp_lib_80055e9c(num as i32, Some((m[0][0], m[0][3], m[1][3])));
            } else {
                self.ground.mp_lib_80055e9c(num as i32, None);
            }
        }
        // grGreens_80216C20: stitch the joints of newly resting blocks to the resting neighbours.
        let g = self.greens.as_mut().unwrap();
        for i in 0..5usize {
            for j in 0..6usize {
                if g.blocks[i][j].x1_5 {
                    let me = g.blocks[i][j].joint;
                    if i > 0 && g.blocks[i - 1][j].status == 3 {
                        self.ground.mp_lib_800581dc(me, g.blocks[i - 1][j].joint);
                    }
                    if j > 0 && g.blocks[i][j - 1].status == 3 {
                        self.ground.mp_lib_800581dc(me, g.blocks[i][j - 1].joint);
                    }
                    if i < 4 && g.blocks[i + 1][j].status == 3 {
                        self.ground.mp_lib_800581dc(me, g.blocks[i + 1][j].joint);
                    }
                    if j < 5 && g.blocks[i][j + 1].status == 3 {
                        self.ground.mp_lib_800581dc(me, g.blocks[i][j + 1].joint);
                    }
                    g.blocks[i][j].x1_5 = false;
                } else if g.blocks[i][j].x1_6 {
                    return Err("grGreens_802150C4: diagonal stitches of a removed block not ported".into());
                }
            }
        }
        // The block items follow their JObjs (it_2E6A_UnkMotion19_Phys: lb_8000B1CC(jobj, NULL, &item->pos)).
        self.greens_sync_items();
        Ok(())
    }

    /// The items of the blocks take the world position of their JObjs.
    fn greens_sync_items(&mut self) {
        let g = self.greens.as_mut().unwrap();
        for row in 0..5usize {
            for col in 0..6usize {
                let b = g.blocks[row][col];
                if b.status == 0 {
                    continue;
                }
                let j = g.tree.dfs[5 + b.joint as usize];
                let m = g.tree.mtx_of(j);
                for it in &mut self.items {
                    if let fox::ItemClass::GreensBlock { id } = it.class {
                        if id == b.item {
                            it.pos = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
                        }
                    }
                }
            }
        }
    }

    /// The first half of `grGreens_802166C4`: pick a column by the stack heights and drop a block there.
    fn greens_spawn_roll(&mut self) -> Result<(), String> {
        let (weights, p) = {
            let g = self.greens.as_ref().unwrap();
            let mut weights = [0u8; 6];
            for col in 0..6usize {
                weights[col] = g.yak.i(0x1C) as u8;
                for row in (0..5usize).rev() {
                    if g.blocks[row][col].status != 0 {
                        weights[col] = match row {
                            0 => g.yak.i(0x18),
                            1 => g.yak.i(0x14),
                            2 => g.yak.i(0x10),
                            3 => g.yak.i(0x0C),
                            _ => g.yak.i(0x08),
                        } as u8;
                        break;
                    }
                }
            }
            (weights, (g.yak.i(0x20), g.yak.i(0x00), g.yak.i(0x04)))
        };
        let mut left_zero = false;
        let mut right_zero = false;
        for k in 0..3usize {
            if weights[k] == 0 {
                left_zero = true;
            }
            if weights[k + 3] == 0 {
                right_zero = true;
            }
        }
        let w = |i: usize| weights[i] as i32;
        let mut ix = -1i32;
        if !left_zero {
            if !right_zero {
                let total = w(0) + w(1) + w(2) + w(3) + w(4) + w(5);
                let mut pick = if total != 0 { self.rand.randi(total) } else { 0 };
                ix = 0;
                for k in 0..6usize {
                    pick -= w(k);
                    if pick >= 0 {
                        ix = k as i32 + 1;
                    } else {
                        break;
                    }
                }
                if ix >= 6 {
                    return Err("grGreens_802166C4: HSD_ASSERT(ix < 6 * 2) (all columns)".into());
                }
            } else {
                let total = w(0) + w(1) + w(2);
                let mut pick = if total != 0 { self.rand.randi(total) } else { 0 };
                ix = 0;
                for k in 0..3usize {
                    pick -= w(k);
                    if pick >= 0 {
                        ix = k as i32 + 1;
                    } else {
                        break;
                    }
                }
                if ix >= 3 {
                    return Err("grGreens_802166C4: HSD_ASSERT(ix < 6) (left)".into());
                }
            }
        } else if !right_zero {
            let total = w(3) + w(4) + w(5);
            let mut pick = if total != 0 { self.rand.randi(total) } else { 0 };
            ix = 3;
            pick -= w(3);
            if pick >= 0 {
                ix = 4;
                pick -= w(4);
                if pick >= 0 {
                    ix = 5;
                    pick -= w(5);
                    if pick >= 0 {
                        ix = 6;
                    }
                }
            }
            if ix >= 6 {
                return Err("grGreens_802166C4: HSD_ASSERT(ix < 6 * 2) (right)".into());
            }
        }
        if ix != -1 {
            let ix = ix as usize;
            let mut spawn_row = 4usize;
            while spawn_row > 0 {
                if self.greens.as_ref().unwrap().blocks[spawn_row - 1][ix].status != 0 {
                    break;
                }
                spawn_row -= 1;
            }
            let roll = if p.0 != 0 { self.rand.randi(p.0) } else { 0 };
            let ty = if roll != 0 { 1 } else { 2 };
            self.greens_create_block(ix, spawn_row, ty, 1)?;
        }
        let t = randrange(&mut self.rand, p.1, p.2);
        self.greens.as_mut().unwrap().block_timer = t;
        Ok(())
    }

    /// `grGreens_80215358(gobj, col, row, arg3, arg4)`: a block in cell (`row`, `col`) (`arg3` 1 block, 2 bomb;
    /// `arg4` 1 falling from the top, 3 resting).
    fn greens_create_block(&mut self, col: usize, row: usize, arg3: i32, arg4: i32) -> Result<(), String> {
        let top = self.blast.top;
        let g = self.greens.as_mut().unwrap();
        if g.blocks[row][col].status != 0 {
            return Err("grGreens_80215358: HSD_ASSERT(status == None)".into());
        }
        let mut used = [false; 30];
        for r in g.blocks.iter() {
            for b in r.iter() {
                if b.status != 0 {
                    used[b.joint as usize] = true;
                }
            }
        }
        let Some(num) = used.iter().position(|u| !u) else {
            return Err("grGreens_80215358: HSD_ASSERT(num < Gr_Greens_Block_Max)".into());
        };
        let x8 = if arg4 == 1 { top } else { g.grid[row][col].1 };
        let id = g.next_item;
        g.next_item += 1;
        g.blocks[row][col] = Block { status: arg4, joint: num as i32, bomb: arg3 == 2, x1_5: arg4 == 3, x4: 0.0, x8, item: id, ..Default::default() };
        // The JObj x14: translate = cell position / Ground_801C0498().
        let f = (1.0f64 / g.scale as f64) as f32;
        let v = Vec3 { x: g.grid[row][col].0, y: x8, z: 0.0 };
        let j = g.tree.dfs[5 + num];
        // The item is made at the JObj's current (hidden) position, before the translate.
        let m0 = g.tree.mtx_of(j);
        g.tree.set_translate(j, Vec3 { x: v.x * f, y: v.y * f, z: v.z * f });
        let m = g.tree.mtx_of(j);
        if !(m[0][0] == m[1][1] && m[0][0] == m[2][2]) {
            return Err("mpLib_80055E9C: non-uniform block matrix not ported".into());
        }
        self.items.push(fox::SimItem { kind: 160, msid: 0, pos: Vec3 { x: m0[0][3], y: m0[1][3], z: m0[2][3] }, vel: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, class: fox::ItemClass::GreensBlock { id } });
        self.ground.mp_lib_80055e9c(num as i32, Some((m[0][0], m[0][3], m[1][3])));
        self.ground.mp_lib_80057424(num as i32);
        Ok(())
    }

    /// `grGreens_80215ED8(gobj, col, row)`.
    fn greens_block_step(&mut self, row: usize, col: usize) -> Result<(), String> {
        let g: &mut Greens = self.greens.as_mut().unwrap();
        let b = g.blocks[row][col];
        if b.status == 0 || b.x1_3 {
            return Ok(());
        }
        if b.x1_2 {
            return Err("grGreens_80215ED8: an exploding bomb block not ported".into());
        }
        let f = (1.0f64 / g.scale as f64) as f32;
        let (x2c, x30) = (g.yak.f(0x2C), g.yak.f(0x30));
        match b.status {
            1 => {
                if b.x1_4 {
                    return Err("grGreens_80215ED8: a block stood on (grGreens_802159B8) not ported".into());
                }
                {
                    let bm = &mut g.blocks[row][col];
                    bm.x4 += x30;
                    if bm.x4 > x2c {
                        bm.x4 = x2c;
                    }
                    bm.x8 -= bm.x4;
                }
                if row > 0 && matches!(g.blocks[row - 1][col].status, 1 | 2) {
                    let spacing = g.grid[row][col].1 - g.grid[row - 1][col].1;
                    if g.blocks[row][col].x8 - g.blocks[row - 1][col].x8 < spacing {
                        g.blocks[row][col].status = 2;
                        g.blocks[row][col].x8 = spacing + g.blocks[row - 1][col].x8;
                    }
                }
                if g.blocks[row][col].x8 < g.grid[row][col].1 {
                    g.blocks[row][col].x8 = g.grid[row][col].1;
                    g.blocks[row][col].status = 3;
                    g.blocks[row][col].x1_5 = true;
                    for nr in row + 1..5 {
                        if g.blocks[nr][col].status != 2 {
                            break;
                        }
                        g.blocks[nr][col].x8 = g.grid[nr][col].1;
                        g.blocks[nr][col].status = 3;
                        g.blocks[nr][col].x1_5 = true;
                        let pos = Vec3 { x: g.grid[nr][col].0, y: g.blocks[nr][col].x8, z: 0.0 };
                        let j = g.tree.dfs[5 + g.blocks[nr][col].joint as usize];
                        g.tree.set_translate(j, Vec3 { x: pos.x * f, y: pos.y * f, z: pos.z * f });
                    }
                } else {
                    for nr in row + 1..5 {
                        if g.blocks[nr][col].status != 2 {
                            break;
                        }
                        let spacing = g.grid[nr][col].1 - g.grid[nr - 1][col].1;
                        let y = spacing + g.blocks[nr - 1][col].x8;
                        g.blocks[nr][col].x8 = y;
                        let pos = Vec3 { x: g.grid[nr][col].0, y, z: 0.0 };
                        let j = g.tree.dfs[5 + g.blocks[nr][col].joint as usize];
                        g.tree.set_translate(j, Vec3 { x: pos.x * f, y: pos.y * f, z: pos.z * f });
                    }
                }
            }
            _ => return Ok(()),
        }
        let pos = Vec3 { x: g.grid[row][col].0, y: g.blocks[row][col].x8, z: 0.0 };
        let j = g.tree.dfs[5 + g.blocks[row][col].joint as usize];
        g.tree.set_translate(j, Vec3 { x: pos.x * f, y: pos.y * f, z: pos.z * f });
        Ok(())
    }
}
