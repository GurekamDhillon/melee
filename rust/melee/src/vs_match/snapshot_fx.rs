//! Snapshot loader: the live effects (`efLib_Create` objects, ef/eflib.c) as the sim models them.
//!
//! The sim keeps an effect as its lifetime plus the particle tracks (FObj 0x28) of its joint
//! animation; the C keeps a GObj with a JObj tree. The effect's identity (which bank/descriptor it
//! was made from) is not stored in the C object, so it is recovered from the tree: the descriptor
//! whose joint tree has the live tree's shape and whose particle tracks carry the same keyframe bytes.

use super::snapshot::{be32s, c_aobj, gc_off, pair_trees, tree_shapes_match};
use super::*;
use crate::snap::Snap;
use std::collections::HashMap;

impl VsMatch {
    /// Replace `self.effects` with the C build's live effects (p_link 11 GObjs whose user data is an
    /// `EF_Effect`), and note every JObj of their trees in `refs`.
    pub(super) fn load_effects(&mut self, snap: &Snap, refs: &mut HashMap<u32, JObjRef>) -> Result<(), String> {
        self.effects.clear();
        self.next_effect_instance = 1;
        let fighters = snap.plink(8)?;
        let items = snap.plink(9)?;
        let fp_player_id = gc_off::<Fighter>("player_id");
        let player_of_fighter = |g: u32| snap.u8(snap.u32(g + 0x2C) + fp_player_id) as usize;
        for g in snap.plink(11)? {
            let ud = snap.u32(g + 0x2C);
            // classifier 8 = an effect gobj; its user data points back at the gobj.
            if snap.u16(g) != 8 || ud == 0 || snap.u32(ud + 4) != g {
                continue;
            }
            let root = snap.u32(g + 0x28);
            if root == 0 {
                return Err(format!("snapshot: effect {g:#x} has no JObj tree"));
            }
            let parent = snap.u32(ud + 8);
            let owner = if fighters.contains(&parent) {
                player_of_fighter(parent)
            } else if items.contains(&parent) {
                let o = snap.u32(snap.u32(parent + 0x2C) + 0x518);
                if !fighters.contains(&o) {
                    return Err(format!("snapshot: effect {g:#x}'s item has no fighter owner"));
                }
                player_of_fighter(o)
            } else {
                return Err(format!("snapshot: effect {g:#x} has parent {parent:#x}, neither a fighter nor an item"));
            };
            let e = self.identify_effect(snap, g, ud, root, owner, self.next_effect_instance, refs)?;
            self.next_effect_instance += 1;
            self.effects.push(e);
        }
        for k in 0..self.effects.len() {
            self.effect_matrices(k);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn identify_effect(&self, snap: &Snap, g: u32, ud: u32, root: u32, owner: usize, inst: u32, refs: &mut HashMap<u32, JObjRef>) -> Result<Effect, String> {
        let live_tracks = |pairs: &HashMap<u32, u32>, tracks: &[crate::stage::DptclTrack]| -> Option<Vec<u32>> {
            // Each descriptor track's live joint must carry an AObj whose 0x28 FObjs have the same bytes.
            let mut out = Vec::new();
            for t in tracks {
                let c = *pairs.get(&t.joint)?;
                let a = snap.u32(c + 0x7C);
                if a == 0 {
                    return None;
                }
                let mut f = snap.u32(a + 0x14);
                let mut want = t.aobj.fobjdesc.iter();
                while f != 0 {
                    if snap.u8(f + 0x13) == 0x28 {
                        let d = want.next()?;
                        let (head, len) = (snap.u32(f + 8), snap.u32(f + 0xC) as usize);
                        if len != d.length as usize || snap.bytes(head, len)? != &d.ad[..len.min(d.ad.len())] {
                            return None;
                        }
                    }
                    f = snap.u32(f);
                }
                if want.next().is_some() {
                    return None;
                }
                out.push(c);
            }
            Some(out)
        };
        let mut best: Option<(usize, usize, Vec<u32>, u32, HashMap<u32, u32>, usize)> = None; // (score, bank, cs, idx, pairs, top-ish)
        let mut best_top = 0u32;
        for (bi, bank) in self.effect_banks.iter().enumerate() {
            let Some(b) = bank else { continue };
            let d = &b.archive.data;
            for idx in 0..1000u32 {
                let at = b.table + 8 + idx * 0x14;
                if at as usize + 0x14 > d.len() {
                    break;
                }
                let desc = b.desc(idx);
                if desc.joint == 0 || desc.joint as usize + 0x40 > d.len() || desc.joint % 4 != 0 {
                    continue;
                }
                // The root may be wrapped like the stage's map JObjs.
                let Some(top) = [root, snap.u32(root + 0x10)].into_iter().find(|&c| c != 0 && tree_shapes_match(snap, d, desc.joint, c)) else { continue };
                let pairs = pair_trees(snap, d, desc.joint, top);
                // Past the table the "descriptors" are other data: reading their trees may fault; skip those.
                let prev = std::panic::take_hook();
                std::panic::set_hook(Box::new(|_| {}));
                let tracks = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::stage::dptcl_tracks_of(&b.archive, desc.joint, desc.animjoint)));
                std::panic::set_hook(prev);
                let Ok(tracks) = tracks else { continue };
                let Some(cs) = live_tracks(&pairs, &tracks) else { continue };
                // How well do the joints' own constants agree (the spawner only changes the root)?
                let score: usize = pairs
                    .iter()
                    .map(|(&j, &c)| {
                        let srt = [(0x14, 0x1C), (0x18, 0x20), (0x1C, 0x24), (0x20, 0x2C), (0x24, 0x30), (0x28, 0x34), (0x2C, 0x38), (0x30, 0x3C), (0x34, 0x40)]
                            .iter()
                            .filter(|&&(dj, cj)| be32s(d, j + dj) == snap.u32(c + cj))
                            .count();
                        srt + usize::from((be32s(d, j + 4) ^ snap.u32(c + 0x14)) & 0x00FF_FFBF == 0)
                    })
                    .sum();
                if best.as_ref().map_or(true, |b| score > b.0) {
                    best = Some((score, bi, cs, idx, pairs, 0));
                    best_top = top;
                }
            }
        }
        if let Some((_, bi, cs, idx, pairs, _)) = best {
            let top = best_top;
            let b = self.effect_banks[bi].as_ref().unwrap();
            let d = &b.archive.data;
            let desc = b.desc(idx);
            let tracks = crate::stage::dptcl_tracks_of(&b.archive, desc.joint, desc.animjoint);
            {
                // Found: build the sim's effect.
                let life = snap.u16(ud + 0x24);
                let rs = |o: u32| snap.f32(top + o);
                let scale = rs(0x2C);
                let attached = snap.u32(ud + 0xC) != 0;
                let fixed_pos = (!attached).then(|| [rs(0x38), rs(0x3C), rs(0x40)]);
                let own_y = f32::from_bits(be32s(d, desc.joint + 0x18));
                let own_z = f32::from_bits(be32s(d, desc.joint + 0x1C));
                let (cy, cz) = (rs(0x20), rs(0x24));
                let rot_y = (cy.to_bits() != own_y.to_bits()).then_some(cy);
                let rot_z = (cz.to_bits() != own_z.to_bits()).then_some(cz);
                let mut etracks = Vec::new();
                for (t, c) in tracks.iter().zip(cs) {
                    let a = snap.u32(c + 0x7C);
                    etracks.push(EffectTrack { joint: (inst << 24) | t.joint, chain: t.chain.clone(), aobj: c_aobj(snap, a, Some(0x28)) });
                }
                for (j, c) in &pairs {
                    refs.entry(*c).or_insert((inst << 24) | *j);
                }
                let _ = g;
                return Ok(Effect { inst, tev: None, id: idx, lifetime: life, owner, scale, bank: bi, tracks: etracks, fixed_pos, rot_y, rot_z, detached: false });
            }
        }
        Err(format!("snapshot: effect {g:#x} matches no descriptor of the loaded effect banks (lifetime {})", snap.u16(ud + 0x24)))
    }
}
