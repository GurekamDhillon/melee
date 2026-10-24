//! State snapshots: resume the native sim from the C build's state (`MELEE_SNAP_DUMP`, pc/platform/
//! gw_snap.c; tools/rust-port/snap/). [`VsMatch::from_snapshot`] builds the match the normal way
//! (`VsMatch::new`: the disc files, stage, banks, ECB tables: everything the frame does not change),
//! then overlays the dynamic state from the dump, component by component, reading the C build's
//! objects through the GameCube offset tables (`crate::snap`).
//!
//! [`VsMatch::state_dump`] writes the same components as canonical text, so a from-0 run and a resumed
//! run can be diffed at the same frame (`melee golden --dump-state FRAME:FILE`), which is how every
//! loader below was checked: a field the loader gets wrong shows up as the first differing line.

use super::*;
use crate::snap::{Snap, MEM1_BASE};
use std::collections::HashMap;
use std::fmt::Write as _;
use melee_runtime::aobj::HsdAObj;

fn hexbytes(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for x in b {
        let _ = write!(s, "{x:02x}");
    }
    s
}

fn bits3(v: &Vec3) -> String {
    format!("{:08x} {:08x} {:08x}", v.x.to_bits(), v.y.to_bits(), v.z.to_bits())
}

/// The bytes of a plain-data value.
fn raw<T>(v: &T) -> &[u8] {
    // SAFETY: reading the bytes of a live value; padding is zero (values are zero-initialised then written).
    unsafe { std::slice::from_raw_parts(v as *const T as *const u8, std::mem::size_of::<T>()) }
}


fn mtx_bits(m: &Mtx) -> String {
    m.iter().flatten().map(|f| format!("{:08x}", f.to_bits())).collect::<Vec<_>>().join(" ")
}

/// Debug text without the hex addresses (function pointers differ per process).
fn strip_addrs(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("0x") {
        out += &rest[..i];
        rest = rest[i + 2..].trim_start_matches(|c: char| c.is_ascii_hexdigit());
    }
    out + rest
}

fn fnv(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in s.bytes() {
        h = (h ^ b as u64).wrapping_mul(0x100000001b3);
    }
    h
}

fn fobj_line(f: &melee_runtime::fobj::HsdFObj) -> String {
    format!("fobj ad {} len {} flags {:02x} op {} intrp {} type {:02x} fv {:02x} fs {:02x} nb {} start {} fterm {} t {:08x} p0 {:08x} p1 {:08x} d0 {:08x} d1 {:08x} head {}", f.ad,
        f.length, f.flags, f.op, f.op_intrp, f.obj_type, f.frac_value, f.frac_slope, f.nb_pack, f.startframe, f.fterm, f.time.to_bits(), f.p0.to_bits(), f.p1.to_bits(), f.d0.to_bits(),
        f.d1.to_bits(), f.ad_head.len())
}

fn aobj_line<O>(a: &HsdAObj<O>) -> String {
    let mut s = format!("aobj flags {:08x} cur {:08x} rew {:08x} end {:08x} rate {:08x}", a.flags, a.curr_frame.to_bits(), a.rewind_frame.to_bits(), a.end_frame.to_bits(), a.framerate.to_bits());
    for f in &a.fobj {
        s += &format!(" [{}]", fobj_line(f));
    }
    s
}

fn tree_dump(s: &mut String, tag: &str, tree: &crate::fighter_data::skeleton::Tree) {
    for (i, o) in tree.nodes.iter().enumerate() {
        let _ = writeln!(s, "{tag} j{i} next {:?} parent {:?} child {:?} flags {:08x} rot {:08x} {:08x} {:08x} {:08x} scale {} trans {} scl {:?} mtx {} aobj {}", o.next, o.parent, o.child, o.flags,
            o.rotate.x.to_bits(), o.rotate.y.to_bits(), o.rotate.z.to_bits(), o.rotate.w.to_bits(), bits3(&o.scale), bits3(&o.translate), o.scl.map(|v| bits3(&v)), mtx_bits(&o.mtx),
            o.aobj.as_ref().map_or("none".to_string(), aobj_line));
    }
}

fn gen_line(g: &melee_runtime::particle::Generator) -> String {
    let mut c = g.clone();
    let cl = c.cmd_list.take();
    format!("{c:?} cmd {:?}", cl.map(|r| (r.off, r.data.len())))
}

fn part_line(p: &melee_runtime::particle::Particle) -> String {
    let mut c = p.clone();
    let cl = c.cmd_list.take();
    format!("{c:?} cmd {:?}", cl.map(|r| (r.off, r.data.len())))
}

impl VsMatch {
    /// One line per component: what is live (a diagnostic for choosing what the loader must cover).
    pub fn state_summary(&self) -> String {
        let mut s = format!("frame {} seed {:08X}\n", self.frame, self.rand.logic_seed());
        for p in &self.players {
            let f = &p.fp;
            s += &format!("  p{} kind {} motion {} ga {} anim {} frame {:?} effq {} efops {} hit {} script {:#x}\n", p.slot + 1, f.kind, f.motion_id,
                f.ground_or_air, p.animating, p.skel.curr_frame(), p.ef_queue.len(), p.ef_ops.len(), p.hitboxes.iter().filter(|h| h.is_some()).count(), p.script);
        }
        s += &format!("  effects {} items {} dptcl {} gens {} parts {} srts {} fighter_jobjs {}\n", self.effects.len(), self.items.len(),
            self.dptcl.len(), self.ps.num_gens, self.ps.num_parts, self.ps.num_srts, self.fighter_jobjs.len());
        s
    }

    /// Canonical text of the sim's dynamic state (see the module docs).
    pub fn state_dump(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "frame {}", self.frame);
        let _ = writeln!(s, "rand seed {:08x} render {:08x} depth {}", self.rand.logic_seed(), self.rand.render_seed, self.rand.render_depth);
        for (i, p) in self.pads.iter().enumerate() {
            let _ = writeln!(s, "pad{i} {p:?}");
        }
        for p in &self.players {
            let n = p.slot + 1;
            let b = raw(&*p.fp);
            let mut f: Vec<(&str, usize)> = crate::fighter_fields::FIGHTER_FIELDS.to_vec();
            f.sort_by_key(|x| x.1);
            for (k, (name, off)) in f.iter().enumerate() {
                let end = f.get(k + 1).map_or(b.len(), |x| x.1);
                let _ = writeln!(s, "p{n} fp.{name} {}", hexbytes(&b[*off..end]));
            }
            let _ = writeln!(s, "p{n} animating {}", p.animating);
            let e = &p.entry;
            let _ = writeln!(s, "p{n} entry timer {} x4 {:08x} x8 {} x14 {} x20 {:08x} x24 {:08x} x28 {:08x} x2c {:08x} {:08x} {:08x},{:08x} {:08x},{:08x}", e.timer, e.x4.to_bits(),
                bits3(&e.x8), bits3(&e.x14), e.x20.to_bits(), e.x24.to_bits(), e.x28.to_bits(), e.x2c.top.to_bits(), e.x2c.bottom.to_bits(), e.x2c.left.x.to_bits(), e.x2c.left.y.to_bits(), e.x2c.right.x.to_bits(), e.x2c.right.y.to_bits());
            let _ = writeln!(s, "p{n} walk {:?} turn {:?} landing_allow {}", p.walk, p.turn, p.landing_allow_interrupt);
            let _ = writeln!(s, "p{n} fall {} {:08x} lw {:08x} {:08x}", p.fall_smid, p.fall_x4.to_bits(), p.ca_lw_x0.to_bits(), p.ca_lw_friction.to_bits());
            let _ = writeln!(s, "p{n} script {:#x} sp {} ret {:?}", p.script, p.script_sp, p.script_ret);
            let _ = writeln!(s, "p{n} x221d_b3 {} locked {} hud_randomize {} dead_timer {}", p.x221d_b3, p.input_locked, p.hud_randomize, p.dead_timer);
            let _ = writeln!(s, "p{n} guard {:?} jump {:?}", p.guard, p.jump);
            let _ = writeln!(s, "p{n} dash {:?} run {:?} fx {:?} attack1_x0 {}", p.dash, p.run, p.fx, p.attack1_x0);
            let _ = writeln!(s, "p{n} rb {:?}", p.rb);
            let _ = writeln!(s, "p{n} hitboxes {:?}", p.hitboxes);
            let _ = writeln!(s, "p{n} item_cmds {:?} ef_queue {:?} ef_ops {:?} keep_gfx {} blend_zero {}", p.item_cmds, p.ef_queue, p.ef_ops, p.keep_gfx, p.blend_zero);
            let _ = writeln!(s, "p{n} stocks {}", stale::stocks(p.slot));
            let k = &p.skel;
            let _ = writeln!(s, "p{n} skel root {:?} blend {:08x} {:08x}", k.root, k.blend_frames.to_bits(), k.blend_frame.to_bits());
            for (j, x) in k.x8b0.iter().enumerate() {
                let _ = writeln!(s, "p{n} x8b0[{j}] {:08x} {:08x} {:08x} {} {}", x.x4.to_bits(), x.x8.to_bits(), x.xc.to_bits(), x.x10, x.x11);
            }
            tree_dump(&mut s, &format!("p{n} tree"), &k.tree);
            tree_dump(&mut s, &format!("p{n} tree2"), &k.tree2);
        }
        let _ = writeln!(s, "ps gens {} peak {} parts {} peak {} srts {} peak {} idnum {:#x} head {:?} cursor {:?} queue {:?} slots {:?}", self.ps.num_gens, self.ps.peak_gens, self.ps.num_parts,
            self.ps.peak_parts, self.ps.num_srts, self.ps.peak_srts, self.ps.idnum_counter, self.ps.gen_head, self.ps.gen_cursor, self.ps.gen_queue, self.ps.jobj_slots);
        for g in self.ps.generators() {
            let _ = writeln!(s, "ps gen {g:?} {}", gen_line(self.ps.gen(g)));
        }
        for l in 0..melee_runtime::particle::NUM_LINKS {
            for q in self.ps.particles(l) {
                let _ = writeln!(s, "ps link{l} part {q:?} {}", part_line(self.ps.part(q)));
            }
        }
        for (i, d) in self.dptcl.iter().enumerate() {
            let _ = writeln!(s, "dptcl{i} joint {:#x} {}", d.joint, aobj_line(&d.aobj));
        }
        let _ = writeln!(s, "last_bg {:?}", self.last_bg);
        let _ = writeln!(s, "aobj_globals {} {}", self.aobj_globals.n_stopped, self.aobj_globals.n_running);
        for (i, e) in self.effects.iter().enumerate() {
            let _ = writeln!(s, "effect{i} life {} owner {} scale {:08x} bank {} fixed {:?} rot_y {:?} rot_z {:?}", e.lifetime, e.owner, e.scale.to_bits(), e.bank, e.fixed_pos, e.rot_y, e.rot_z);
            for t in &e.tracks {
                let _ = writeln!(s, "effect{i} track joint {:#x} chain {:?} {}", t.joint, t.chain, aobj_line(&t.aobj));
            }
        }
        let _ = writeln!(s, "next_effect_instance {}", self.next_effect_instance);
        let mut jm: Vec<_> = self.joint_mtx.iter().collect();
        jm.sort_by_key(|x| *x.0);
        for (k, m) in jm {
            let _ = writeln!(s, "joint_mtx {k:#x} {}", mtx_bits(m));
        }
        let _ = writeln!(s, "fighter_jobjs {:?}", self.fighter_jobjs);
        for (i, it) in self.items.iter().enumerate() {
            let _ = writeln!(s, "item{i} {it:?}");
        }
        let _ = writeln!(s, "respawn_busy {:?}", self.respawn_busy);
        let _ = writeln!(s, "ground fnv {:016x} cg fnv {:016x}", fnv(&strip_addrs(&format!("{:?}", self.ground))), fnv(&format!("{:?}", self.cg)));
        for slot in 0..6 {
            let _ = writeln!(s, "static_player{slot} fnv {:016x}", fnv(&format!("{:?}", raw(stale::player(slot)))));
        }
        let _ = writeln!(s, "stale counters {} {}", unsafe { stale::STALE_ATTACK_INSTANCE }, unsafe { stale::ATTACK_COUNTER });
        s
    }

    /// Resume from the C build's state dump: build the match normally, then overlay `snap`.
    pub fn from_snapshot(disc: &mut crate::disc::Disc, snap: &Snap, stage_file: &str, fighters: &[&str], seed: u32, stocks: i32) -> Result<Self, String> {
        let mut m = Self::new(disc, stage_file, fighters, seed, stocks)?;
        // Debug (MELEE_SNAP_KEEP, frame-0 snapshots): the from-0 sim's own frame-0 Fighters, for the
        // fields kept native.
        if std::env::var_os("MELEE_SNAP_KEEP").is_some() && snap.frame == 0 {
            use crate::Sim;
            m.step();
            m.keep_ref = m.players.iter().map(|p| raw(&*p.fp).to_vec()).collect();
            m.frame = -1;
        }
        m.load_snapshot(snap)?;
        Ok(m)
    }

    fn load_snapshot(&mut self, snap: &Snap) -> Result<(), String> {
        self.frame = snap.frame;
        // random.c: *HSD_RandSeedPtr (the default `seed`), no render draw in flight at a frame boundary.
        let seed_va = snap.ptr_cell("_gw_HSD_RandSeedPtr")?;
        if seed_va != snap.must_sym("_seed")? {
            return Err("snapshot: HSD_RandSeedPtr is redirected (a stage owns the logic seed): not wired".into());
        }
        self.rand.set_logic_seed(snap.u32(seed_va));
        self.rand.render_seed = snap.u32(snap.must_sym("_render_seed")?);
        // HSD_PadGameStatus[4].
        let pads = snap.must_sym("_gw_HSD_PadGameStatus")?;
        for port in 0..4 {
            let c: melee_types::HSD_PadStatus = snap.decode_flat(pads + 0x44 * port as u32);
            let q = &mut self.pads[port];
            q.button = c.button;
            q.last_button = c.last_button;
            q.trigger = c.trigger;
            q.release = c.release;
            q.stick_x = c.stickX;
            q.stick_y = c.stickY;
            q.sub_x = c.subStickX;
            q.sub_y = c.subStickY;
            q.analog_l = c.analogL;
            q.analog_r = c.analogR;
            q.nml_stick_x = c.nml_stickX;
            q.nml_stick_y = c.nml_stickY;
            q.nml_sub_x = c.nml_subStickX;
            q.nml_sub_y = c.nml_subStickY;
            q.nml_analog_l = c.nml_analogL;
            q.nml_analog_r = c.nml_analogR;
        }
        self.load_stage(snap)?;
        // The fighters (p_link 8): Fighter_Create's GObj list.
        let gobjs = snap.plink(8)?;
        let fp_player_id = gc_off::<Fighter>("player_id");
        for i in 0..self.players.len() {
            let slot = self.players[i].slot;
            let fp_va = gobjs.iter().map(|&g| snap.u32(g + 0x2c)).find(|&u| u != 0 && snap.u8(u + fp_player_id) as i32 == slot)
                .ok_or(format!("snapshot: no fighter for player {}", slot + 1))?;
            self.load_player(snap, i, fp_va)?;
        }
        Ok(())
    }

    /// One player: the `Fighter` record, then the sim's per-player state kept beside it.
    fn load_player(&mut self, snap: &Snap, i: usize, fp_va: u32) -> Result<(), String> {
        let p = &mut self.players[i];
        let before = if self.keep_ref.is_empty() { raw(&*p.fp).to_vec() } else { self.keep_ref[i].clone() };
        let p = &mut self.players[i];
        *p.fp = snap.decode_flat::<Fighter>(fp_va);
        let c_fp = p.fp.clone();
        // MODEL QUIRK. `Fighter_UnkInit` leaves smash_attrs.x2135 = -1 in the C build; `VsMatch::new`
        // leaves 0 until the first respawn writes -1 (rebirth_enter). With -1 the ported respawn
        // probe (ft_80082A68 in Fighter_UnkProcessDeath, rebirth.rs) runs and lands a Rebirth 1 ulp
        // off the golden, so a fighter that has not died yet (C: no falls) keeps the model's 0.
        // MELEE_SNAP_EXACT_X2135=1 loads the C value (a from-0 run then no longer matches either).
        let slot_pl = snap.must_sym("_player_slots")? + 0xE90 * p.slot as u32;
        let falls = snap.s32(slot_pl + 0x68) + snap.s32(slot_pl + 0x6C);
        if p.fp.smash_attrs.x2135 == -1 && falls == 0 && std::env::var_os("MELEE_SNAP_EXACT_X2135").is_none() {
            p.fp.smash_attrs.x2135 = 0;
        }
        stale::set_stocks(p.slot, snap.u8(slot_pl + gc_off::<melee_types::_StaticPlayer>("stocks")) as i8 as i32);
        // Debug: MELEE_SNAP_KEEP=field,field keeps those Fighter fields as `VsMatch::new` left them
        // (to find which unmodelled C values the native code is sensitive to).
        if let Ok(keep) = std::env::var("MELEE_SNAP_KEEP") {
            let mut f: Vec<(&str, usize)> = crate::fighter_fields::FIGHTER_FIELDS.to_vec();
            f.sort_by_key(|x| x.1);
            let size = std::mem::size_of::<Fighter>();
            let dst = &mut *p.fp as *mut Fighter as *mut u8;
            for (k, (name, o)) in f.iter().enumerate() {
                if keep.split(',').any(|x| x == *name || x == "ALL") {
                    let end = f.get(k + 1).map_or(size, |x| x.1);
                    // SAFETY: the span lies inside the Fighter.
                    unsafe { std::ptr::copy_nonoverlapping(before.as_ptr().add(*o), dst.add(*o), end - o) };
                }
            }
        }
        let n = p.slot + 1;
        p.animating = (p.fp.anim_id as i32) != -1;
        // The command script: `x3E4_fighterCmdScript.u` and its call/loop stack are guest pointers
        // into the fighter's data file; the sim keeps DAT offsets (the file's base is `ft_data`'s
        // address less its offset in the file).
        let dat_base = snap.u32(fp_va + gc_off::<Fighter>("ft_data")).wrapping_sub(p.files.ft_data);
        let cmd = fp_va + gc_off::<Fighter>("x3E4_fighterCmdScript");
        let off = |v: u32| if v >= MEM1_BASE { v.wrapping_sub(dat_base) } else { v };
        p.script = match snap.u32(cmd + 8) { 0 => 0, u => off(u) };
        p.script_sp = snap.u32(cmd + 0xC) as usize;
        p.script_ret = [0; 8];
        for k in 0..5 {
            p.script_ret[k] = off(snap.u32(cmd + 0x10 + 4 * k as u32));
        }
        if p.script_sp > 4 {
            return Err(format!("snapshot: player {n}: command stack depth {}", p.script_sp));
        }
        p.x221d_b3 = c_fp.x221D_b3() != 0;
        p.input_locked = c_fp.x221D_b4() != 0;
        p.guard.shield_on = c_fp.anon_221b.x221B_b0() != 0;
        if !snap.u32(fp_va + gc_off::<Fighter>("x60C")).eq(&0) {
            return Err(format!("snapshot: player {n}: efAsync queue (x60C) not empty at a frame boundary"));
        }
        for k in 0..4 {
            let hc = fp_va + gc_off::<Fighter>("x914") + k * 0x138;
            if snap.u32(hc) != 0 {
                return Err(format!("snapshot: player {n}: live hitbox {k}: HitCapsule -> Hitbox not wired"));
            }
        }
        // fp->mv, by the motion state that owns it (the union is only meaningful for that state).
        let mv = fp_va + gc_off::<Fighter>("mv");
        let (mf, mi) = (|o: u32| snap.f32(mv + o), |o: u32| snap.s32(mv + o));
        let mb = |o: u32| snap.u8(mv + o) != 0;
        let m = p.fp.motion_id;
        match m {
            FTCO_MS_ENTRY | FTCO_MS_ENTRY_START | FTCO_MS_ENTRY_END => {
                p.entry = EntryVars {
                    timer: mi(0),
                    x4: mf(4),
                    x8: Vec3 { x: mf(8), y: mf(0xC), z: mf(0x10) },
                    x14: Vec3 { x: mf(0x14), y: mf(0x18), z: mf(0x1C) },
                    x20: mf(0x20),
                    x24: mf(0x24),
                    x28: mf(0x28),
                    x2c: snap.decode_flat::<ftCollisionBox>(mv + 0x2C),
                };
            }
            FTCO_MS_WALK_SLOW..=FTCO_MS_WALK_FAST => {
                p.walk = WalkVars { x0: mf(0), msid: mi(4), slow_anim_frame: mf(8), middle_anim_frame: mf(0xC), fast_anim_frame: mf(0x10),
                    slow_anim_rate: mf(0x14), middle_anim_rate: mf(0x18), fast_anim_rate: mf(0x1C), accel_mul: mf(0x20) };
            }
            FTCO_MS_TURN => {
                p.turn = TurnVars { has_turned: mb(0), facing_after: mf(4), x8: mf(8), frames_to_turn: mf(0x10), just_turned: mb(0x18), x1c: snap.u32(mv + 0x1C) };
            }
            FTCO_MS_LANDING => p.landing_allow_interrupt = mb(0),
            FTCO_MS_FALL | FTCO_MS_FALL_AERIAL => {
                p.fall_smid = mi(0);
                p.fall_x4 = mf(4);
            }
            FTCO_MS_GUARD_ON..=FTCO_MS_GUARD_REFLECT => {
                p.guard = GuardVars { x0: mf(0), x4: mf(4), x8: mf(8), xc: mb(0xC), x10: mf(0x10), x14: mf(0x14), x18: mf(0x18),
                    x1c: mi(0x1C), x20: mi(0x20), x24: mi(0x24), x2c: mf(0x2C), shield_on: p.guard.shield_on };
            }
            FTCO_MS_KNEE_BEND => {
                p.jump.is_short_hop = mi(0) != 0;
                p.jump.jump_input = mi(4);
            }
            m if is_jump_motion(m) => {
                p.jump.x4 = mb(4);
                // kneebend's short-hop flag is read again by the jump's enter only.
            }
            dashrun::FTCO_MS_DASH => p.dash = dashrun::DashVars { x0: mf(0), x4: mi(4) },
            dashrun::FTCO_MS_RUN => p.run = dashrun::RunVars { x0: mf(0), x4: mf(4) },
            FTCO_MS_ATTACK11..=44 => p.attack1_x0 = mb(0),
            FTCO_MS_DEAD_DOWN => p.dead_timer = mi(0),
            rebirth::FTCO_MS_REBIRTH | rebirth::FTCO_MS_REBIRTH_WAIT => {
                p.rb.timer = mi(0);
                p.rb.target = Vec3 { x: mf(4), y: mf(8), z: mf(0xC) };
            }
            FTCA_MS_SPECIAL_LW | FTCA_MS_SPECIAL_LW_END | FTCA_MS_SPECIAL_LW_END_AIR => {
                p.ca_lw_x0 = snap.u16(mv) as f32;
                p.ca_lw_friction = mf(4);
            }
            m if fox::is_fox_special_n(m) => p.fx.is_loop = mb(0),
            _ => {}
        }
        if p.fp.kind == Ft_Kind_Fox {
            p.fx.blaster = snap.u32(fp_va + gc_off::<Fighter>("u") + 0) != 0;
            p.fx.accessory4 = snap.u32(fp_va + gc_off::<Fighter>("accessory4_cb")) != 0;
        }
        Ok(())
    }
}

/// GameCube offset of a top-level field.
fn gc_off<T: melee_core::gc::GcLayout>(field: &str) -> u32 {
    T::gc_field(field).unwrap_or_else(|| panic!("{}: no field {field}", T::GC_NAME)).offset
}

fn be32s(d: &[u8], at: u32) -> u32 {
    u32::from_be_bytes(d[at as usize..at as usize + 4].try_into().unwrap())
}

/// `HSD_FObj` at `va` -> the native track (its byte stream copied out of the image).
pub fn c_fobj(snap: &Snap, va: u32) -> melee_runtime::fobj::HsdFObj {
    let (ad, head, len) = (snap.u32(va + 4), snap.u32(va + 8), snap.u32(va + 0xC));
    melee_runtime::fobj::HsdFObj {
        ad: ad.wrapping_sub(head) as usize,
        ad_head: Arc::from(snap.bytes(head, len as usize).unwrap_or(&[])),
        length: len,
        flags: snap.u8(va + 0x10),
        op: snap.u8(va + 0x11),
        op_intrp: snap.u8(va + 0x12),
        obj_type: snap.u8(va + 0x13),
        frac_value: snap.u8(va + 0x14),
        frac_slope: snap.u8(va + 0x15),
        nb_pack: snap.u16(va + 0x16),
        startframe: snap.u16(va + 0x18) as i16,
        fterm: snap.u16(va + 0x1A),
        time: snap.f32(va + 0x1C),
        p0: snap.f32(va + 0x20),
        p1: snap.f32(va + 0x24),
        d0: snap.f32(va + 0x28),
        d1: snap.f32(va + 0x2C),
    }
}

/// `HSD_AObj` at `va` -> the native AObj; `only_type` keeps just the FObjs of that `obj_type`
/// (the sim runs only a joint's particle track, FObj type 0x28).
pub fn c_aobj<O>(snap: &Snap, va: u32, only_type: Option<u8>) -> HsdAObj<O> {
    let mut fobj = Vec::new();
    let mut f = snap.u32(va + 0x14);
    while f != 0 {
        if only_type.map_or(true, |t| snap.u8(f + 0x13) == t) {
            fobj.push(c_fobj(snap, f));
        }
        f = snap.u32(f);
    }
    HsdAObj { flags: snap.u32(va), curr_frame: snap.f32(va + 4), rewind_frame: snap.f32(va + 8), end_frame: snap.f32(va + 0xC), framerate: snap.f32(va + 0x10), fobj, hsd_obj: None }
}

impl VsMatch {
    /// The stage's joint descriptor (DAT offset) -> the C JObj of map gobj `g`, by walking the
    /// descriptor tree and the live JObj tree together.
    fn stage_jobj_map(&self, snap: &Snap) -> Result<HashMap<u32, u32>, String> {
        let d = &self.stage_data;
        let gobjs = be32s(d, self.stage_map_head + 8);
        let mut out = HashMap::new();
        for g in snap.plink(5)? {
            let ud = snap.u32(g + 0x2c);
            if ud == 0 {
                continue;
            }
            let map_id = snap.u32(ud + 0x14);
            let root_c = snap.u32(g + 0x28);
            let root_j = be32s(d, gobjs + map_id * 0x34);
            // The gobj's JObj may carry one wrapper above the descriptor's root: take the candidate
            // whose whole tree has the descriptor tree's shape.
            let root_c = [root_c, snap.u32(root_c + 0x10)]
                .into_iter()
                .find(|&c| c != 0 && tree_shapes_match(snap, d, root_j, c))
                .ok_or(format!("snapshot: map gobj {map_id}'s JObj tree does not have its joint tree's shape"))?;
            let mut stack = vec![(root_j, root_c)];
            while let Some((j, c)) = stack.pop() {
                if j == 0 || c == 0 {
                    continue;
                }
                out.insert(j, c);
                stack.push((be32s(d, j + 0xC), snap.u32(c + 8)));
                if be32s(d, j + 4) & melee_runtime::jobj::JOBJ_INSTANCE == 0 {
                    stack.push((be32s(d, j + 8), snap.u32(c + 0x10)));
                }
            }
        }
        Ok(out)
    }

    /// The stage's dynamic state: the particle tracks' AObjs, the background sway, then the particles.
    fn load_stage(&mut self, snap: &Snap) -> Result<(), String> {
        let jm = self.stage_jobj_map(snap)?;
        for t in &mut self.dptcl {
            let c = *jm.get(&t.joint).ok_or(format!("snapshot: stage joint {:#x} has no live JObj", t.joint))?;
            let a = snap.u32(c + 0x7c);
            if a == 0 {
                return Err(format!("snapshot: stage joint {:#x} has no AObj", t.joint));
            }
            t.aobj = c_aobj(snap, a, Some(0x28));
        }
        // grLast: the sway state, Ground::u.last of map gobj 7 (+0xC4..).
        for g in snap.plink(5)? {
            let ud = snap.u32(g + 0x2c);
            if ud != 0 && snap.u32(ud + 0x14) == 7 {
                let b = ud + 0xC4;
                if snap.u32(b + 0x1C) != 0 {
                    return Err("snapshot: grLast phase-9 generator (xE0) set: not wired".into());
                }
                self.last_bg = melee_world::gr::grlast::LastBg {
                    x_c4: snap.f32(b), x_c8: snap.f32(b + 4), x_cc: snap.f32(b + 8), x_d0: snap.f32(b + 0xC),
                    x_d4: snap.f32(b + 0x10), x_d8: snap.f32(b + 0x14), x_dc: snap.f32(b + 0x18), x_e0: None,
                };
            }
        }
        self.aobj_globals.n_stopped = snap.s32(snapshot_ps::global(snap, "HSD_AObj_804D762C")?);
        self.aobj_globals.n_running = snap.s32(snapshot_ps::global(snap, "HSD_AObj_804D7630")?);
        let rev: HashMap<u32, u32> = jm.iter().map(|(j, c)| (*c, *j)).collect();
        let mut jobj_ref = |c: u32| -> Result<JObjRef, String> {
            rev.get(&c).copied().ok_or(format!("snapshot: a generator follows JObj {c:#x}, which is not a stage joint (effect/fighter joints: not wired yet)"))
        };
        self.load_particles(snap, &mut jobj_ref)
    }
}

/// Do the descriptor tree at `j` and the live JObj tree at `c` have the same child/next shape?
fn tree_shapes_match(snap: &Snap, d: &[u8], j: u32, c: u32) -> bool {
    let mut stack = vec![(j, c)];
    while let Some((j, c)) = stack.pop() {
        if (j == 0) != (c == 0) {
            return false;
        }
        if j == 0 {
            continue;
        }
        stack.push((be32s(d, j + 0xC), snap.u32(c + 8)));
        if be32s(d, j + 4) & melee_runtime::jobj::JOBJ_INSTANCE == 0 {
            stack.push((be32s(d, j + 8), snap.u32(c + 0x10)));
        } else if snap.u32(c + 0x10) != 0 {
            return false;
        }
    }
    true
}
