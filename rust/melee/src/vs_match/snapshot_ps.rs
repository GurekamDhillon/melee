//! Snapshot loader: the HSD particle system (particle.c / generator.c / psappsrt.c state) and the
//! maps from guest pointers to the native handles the sim uses for them.

use super::*;
use crate::snap::Snap;
use melee_runtime::mtx::Quaternion;
use melee_runtime::particle::{AppSrt, Generator, PId, Particle, Rgba, SrtId, NUM_LINKS};
use std::collections::HashMap;

/// A C global by map name (`hsd_804D78FC`...): the `_gw_` common or the plain variable.
pub fn global(snap: &Snap, name: &str) -> Result<u32, String> {
    snap.sym(&format!("_gw_{name}"))
        .or_else(|| snap.sym(&format!("_{name}")))
        .ok_or_else(|| format!("snapshot: global {name} is not in the dump"))
}

fn vec3(snap: &Snap, va: u32) -> Vec3 {
    Vec3 { x: snap.f32(va), y: snap.f32(va + 4), z: snap.f32(va + 8) }
}

fn rgba(snap: &Snap, va: u32) -> Rgba {
    Rgba { r: snap.u8(va), g: snap.u8(va + 1), b: snap.u8(va + 2), a: snap.u8(va + 3) }
}

impl VsMatch {
    /// The loaded banks' command lists by guest address (`ptclref_804D0E5C[bank][idx]->cmdList`).
    fn ps_cmd_map(&self, snap: &Snap) -> Result<HashMap<u32, CmdRef>, String> {
        let ptclref = global(snap, "ptclref_804D0E5C")?;
        let mut m = HashMap::new();
        for (b, bank) in self.ps.banks.iter().enumerate() {
            let Some(bank) = bank else { continue };
            let arr = snap.u32(ptclref + 4 * b as u32);
            if arr == 0 {
                return Err(format!("snapshot: particle bank {b} is loaded natively but not in the C build"));
            }
            for idx in bank.data.base..bank.data.end {
                let Some(c) = bank.data.cmd(idx) else { continue };
                let cl = snap.u32(arr + 4 * idx as u32);
                if cl != 0 {
                    // HSD_PSCmdList::cmdList is the flexible array after the 0x3C header.
                    m.insert(cl + 0x3C, c.cmd_list.clone());
                }
            }
        }
        Ok(m)
    }

    /// Replace the particle system's live objects with the C build's (the hsd_804D78FC generator
    /// list, the hsd_804D0908 particle lists, the AppSRTs they reference). `jobj_ref` maps a guest
    /// JObj to the sim's synthetic handle.
    pub(super) fn load_particles(&mut self, snap: &Snap, jobj_ref: &mut dyn FnMut(u32) -> Result<JObjRef, String>) -> Result<(), String> {
        let cmds = self.ps_cmd_map(snap)?;
        let cmd = |va: u32| -> Result<Option<CmdRef>, String> {
            if va == 0 {
                return Ok(None);
            }
            cmds.get(&va).cloned().map(Some).ok_or_else(|| format!("snapshot: command list {va:#x} is not in a loaded particle bank"))
        };
        // Discover the objects: generators on the list, particles on the 16 lists.
        let mut gen_ids: HashMap<u32, GenId> = HashMap::new();
        let mut gen_vas = Vec::new();
        let mut g = snap.u32(global(snap, "hsd_804D78FC")?);
        while g != 0 {
            gen_ids.insert(g, GenId(gen_vas.len() as u32));
            gen_vas.push(g);
            g = snap.u32(g);
            if gen_vas.len() > 4096 {
                return Err("snapshot: generator list does not end".into());
            }
        }
        if snap.u32(global(snap, "hsd_804D78F4")?) != 0 {
            return Err("snapshot: hsd_804D78F4 (queued generator) is set: not wired".into());
        }
        let heads = global(snap, "hsd_804D0908")?;
        let mut part_ids: HashMap<u32, PId> = HashMap::new();
        let mut part_vas = Vec::new();
        let mut link_heads: [Option<PId>; NUM_LINKS] = [None; NUM_LINKS];
        for (l, head) in link_heads.iter_mut().enumerate() {
            let mut p = snap.u32(heads + 4 * l as u32);
            while p != 0 {
                let id = PId(part_vas.len() as u32);
                if head.is_none() {
                    *head = Some(id);
                }
                part_ids.insert(p, id);
                part_vas.push(p);
                p = snap.u32(p);
                if part_vas.len() > 65536 {
                    return Err("snapshot: particle list does not end".into());
                }
            }
        }
        let mut srt_ids: HashMap<u32, SrtId> = HashMap::new();
        let mut srt_vas: Vec<u32> = Vec::new();
        for va in gen_vas.iter().map(|&g| snap.u32(g + 0x54)).chain(part_vas.iter().map(|&p| snap.u32(p + 0x8C))) {
            if va != 0 && !srt_ids.contains_key(&va) {
                srt_ids.insert(va, SrtId(srt_vas.len() as u32));
                srt_vas.push(va);
            }
        }
        // Generators.
        let mut gens = Vec::new();
        for &g in &gen_vas {
            if snap.u32(g + 0x58) != 0 || snap.u32(g + 0x5C) != 0 {
                return Err(format!("snapshot: generator {g:#x} has a user func / callback: not wired"));
            }
            let jobj = snap.u32(g + 0x10);
            let mut aux = [0f32; 13];
            for (k, a) in aux.iter_mut().enumerate() {
                *a = snap.f32(g + 0x60 + 4 * k as u32);
            }
            let next = snap.u32(g);
            gens.push(Some(Generator {
                next: (next != 0).then(|| gen_ids[&next]),
                kind: snap.u32(g + 4),
                random: snap.f32(g + 8),
                count: snap.f32(g + 0xC),
                jobj: if jobj == 0 { None } else { Some(jobj_ref(jobj)?) },
                gen_life: snap.u16(g + 0x14),
                type_: snap.u16(g + 0x16),
                bank: snap.u8(g + 0x18),
                link_no: snap.u8(g + 0x19),
                tex_group: snap.u8(g + 0x1A),
                idnum: snap.u16(g + 0x1C),
                life: snap.u16(g + 0x1E),
                cmd_list: cmd(snap.u32(g + 0x20))?,
                pos: vec3(snap, g + 0x24),
                vel: vec3(snap, g + 0x30),
                grav: snap.f32(g + 0x3C),
                fric: snap.f32(g + 0x40),
                size: snap.f32(g + 0x44),
                radius: snap.f32(g + 0x48),
                angle: snap.f32(g + 0x4C),
                num_child: snap.u32(g + 0x50),
                appsrt: match snap.u32(g + 0x54) {
                    0 => None,
                    a => Some(srt_ids[&a]),
                },
                userfunc: None,
                callback: None,
                aux,
                aux_flag: snap.u16(g + 0x90),
            }));
        }
        // Particles.
        let mut parts = Vec::new();
        for &p in &part_vas {
            if snap.u32(p + 0x90) != 0 {
                return Err(format!("snapshot: particle {p:#x} has userdata: not wired"));
            }
            if snap.u32(p + 0x94) != 0 {
                return Err(format!("snapshot: particle {p:#x} has a psCallback: not wired"));
            }
            let next = snap.u32(p);
            let pgen = snap.u32(p + 0x88);
            let psrt = snap.u32(p + 0x8C);
            parts.push(Some(Particle {
                next: (next != 0).then(|| part_ids[&next]),
                kind: snap.u32(p + 4),
                bank: snap.u8(p + 8),
                tex_group: snap.u8(p + 9),
                pose_num: snap.u8(p + 0xA),
                pal_num: snap.u8(p + 0xB),
                size_count: snap.u16(p + 0xC),
                prim_col_count: snap.u16(p + 0xE),
                env_col_count: snap.u16(p + 0x10),
                prim_col: rgba(snap, p + 0x12),
                env_col: rgba(snap, p + 0x16),
                cmd_wait: snap.u16(p + 0x1A),
                loop_count: snap.u8(p + 0x1C),
                link_no: snap.u8(p + 0x1D),
                idnum: snap.u16(p + 0x1E),
                cmd_list: cmd(snap.u32(p + 0x20))?,
                cmd_ptr: snap.u16(p + 0x24),
                cmd_mark_ptr: snap.u16(p + 0x26),
                cmd_loop_ptr: snap.u16(p + 0x28),
                life: snap.u16(p + 0x2A),
                vel: vec3(snap, p + 0x2C),
                grav: snap.f32(p + 0x38),
                fric: snap.f32(p + 0x3C),
                pos: vec3(snap, p + 0x40),
                size: snap.f32(p + 0x4C),
                rotate: snap.f32(p + 0x50),
                a_cmp_count: snap.u16(p + 0x54),
                a_cmp_mode: snap.u8(p + 0x56),
                a_cmp_param1: snap.u8(p + 0x57),
                a_cmp_param2: snap.u8(p + 0x58),
                p_jobj_ofs: snap.u8(p + 0x59),
                mat_col_count: snap.u16(p + 0x5A),
                amb_col_count: snap.u16(p + 0x5C),
                rotate_count: snap.u16(p + 0x5E),
                size_target: snap.f32(p + 0x60),
                rotate_target: snap.f32(p + 0x64),
                prim_col_remain: snap.u16(p + 0x68),
                env_col_remain: snap.u16(p + 0x6A),
                prim_col_target: rgba(snap, p + 0x6C),
                env_col_target: rgba(snap, p + 0x70),
                mat_col_remain: snap.u16(p + 0x74),
                amb_col_remain: snap.u16(p + 0x76),
                a_cmp_remain: snap.u16(p + 0x78),
                a_cmp_param1_target: snap.u8(p + 0x7A),
                a_cmp_param2_target: snap.u8(p + 0x7B),
                mat_rgb: snap.u8(p + 0x7C),
                mat_a: snap.u8(p + 0x7D),
                amb_rgb: snap.u8(p + 0x7E),
                amb_a: snap.u8(p + 0x7F),
                mat_rgb_target: snap.u8(p + 0x80),
                mat_a_target: snap.u8(p + 0x81),
                amb_rgb_target: snap.u8(p + 0x82),
                amb_a_target: snap.u8(p + 0x83),
                trail: snap.f32(p + 0x84),
                gen: if pgen == 0 {
                    None
                } else {
                    Some(*gen_ids.get(&pgen).ok_or(format!("snapshot: particle {p:#x}'s generator {pgen:#x} is not on the generator list"))?)
                },
                appsrt: (psrt != 0).then(|| srt_ids[&psrt]),
                userdata: None,
                callback: None,
            }));
        }
        // AppSRTs.
        let mut srts = Vec::new();
        for &s in &srt_vas {
            if snap.u32(s + 0x9C) != 0 {
                return Err(format!("snapshot: AppSRT {s:#x} has a freefunc: not wired"));
            }
            let mut mmtx = [[0f32; 4]; 3];
            for (r, row) in mmtx.iter_mut().enumerate() {
                for (c, v) in row.iter_mut().enumerate() {
                    *v = snap.f32(s + 0x34 + 16 * r as u32 + 4 * c as u32);
                }
            }
            let mut x6c = [0f32; 12];
            for (k, v) in x6c.iter_mut().enumerate() {
                *v = snap.f32(s + 0x6C + 4 * k as u32);
            }
            let gp = snap.u32(s + 4);
            srts.push(Some(AppSrt {
                next: None,
                gp: if gp == 0 {
                    None
                } else {
                    Some(*gen_ids.get(&gp).ok_or(format!("snapshot: AppSRT {s:#x}'s generator {gp:#x} is not on the generator list"))?)
                },
                translate: vec3(snap, s + 8),
                rot: Quaternion { x: snap.f32(s + 0x14), y: snap.f32(s + 0x18), z: snap.f32(s + 0x1C), w: snap.f32(s + 0x20) },
                scale: vec3(snap, s + 0x24),
                status: snap.u8(s + 0x30),
                frame_num: snap.u8(s + 0x31),
                used_count: snap.u16(s + 0x32),
                mmtx,
                ssx: snap.f32(s + 0x64),
                ssy: snap.f32(s + 0x68),
                x6c,
                freefunc: None,
                idnum: snap.u16(s + 0xA0),
                x_a2: snap.u8(s + 0xA2),
            }));
        }
        let ps = &mut self.ps;
        ps.restore_live(gens, parts, srts);
        ps.gen_head = (!gen_vas.is_empty()).then_some(GenId(0));
        ps.gen_cursor = match snap.u32(global(snap, "hsd_804D78F8")?) {
            0 => None,
            g => Some(*gen_ids.get(&g).ok_or("snapshot: hsd_804D78F8 is not on the generator list")?),
        };
        ps.gen_queue = Vec::new();
        ps.heads = link_heads;
        ps.num_gens = snap.u16(global(snap, "hsd_804D78E0")?);
        ps.peak_gens = snap.u16(global(snap, "hsd_804D78DA")?);
        ps.num_parts = snap.u16(global(snap, "hsd_804D78E2")?);
        ps.peak_parts = snap.u16(global(snap, "numPeakParticles")?);
        ps.num_srts = snap.u16(global(snap, "hsd_804D78DE")?);
        ps.peak_srts = snap.u16(global(snap, "hsd_804D78D8")?);
        ps.idnum_counter = snap.u16(global(snap, "lbl_804D6368")?);
        let slots = global(snap, "hsd_804D08E8")?;
        for k in 0..8 {
            if snap.u32(slots + 4 * k) != 0 {
                return Err("snapshot: particle JObj slots (hsd_804D08E8) in use: not wired".into());
            }
        }
        if ps.num_gens as usize != gen_vas.len() || ps.num_parts as usize != part_vas.len() || ps.num_srts as usize != srt_vas.len() {
            return Err(format!(
                "snapshot: particle system counts {}/{}/{} but the lists hold {}/{}/{} objects",
                ps.num_gens,
                ps.num_parts,
                ps.num_srts,
                gen_vas.len(),
                part_vas.len(),
                srt_vas.len()
            ));
        }
        Ok(())
    }
}
