//! `src/sysdolphin/baselib/generator.c`: particle generators. Every function in the C file is a
//! native port in `crate::generator` (oracle `tests/oracle_generator.rs`): the state lives in
//! [`GeneratorSystem`], particle.c / JObj callees go through its `GeneratorHost`. This TU
//! exposes them under the C names.
#![allow(non_snake_case)]
pub use crate::generator::{set_userfunc as hsd_8039D1E4, GenId, GeneratorHost, GeneratorSystem, HsdGenerator};
use crate::generator::GeneratorHost as Host;

type Sys<J> = GeneratorSystem<J>;

/// `hsd_8039D1EC`.
pub fn hsd_8039D1EC<J: Copy + PartialEq>(s: &mut Sys<J>) -> u16 {
    s.next_idnum()
}
/// `hsd_8039D214`.
pub fn hsd_8039D214<J: Copy + PartialEq>(s: &mut Sys<J>, g: GenId, h: &mut impl Host<J>) {
    s.follow(g, h)
}
/// `hsd_8039D354`.
pub fn hsd_8039D354<J: Copy + PartialEq>(s: &mut Sys<J>, h: &mut impl Host<J>) {
    s.init(h)
}
/// `hsd_8039D3AC`.
pub fn hsd_8039D3AC<J: Copy + PartialEq>(s: &mut Sys<J>, g: GenId, prev: Option<GenId>, h: &mut impl Host<J>) -> Option<GenId> {
    s.remove(g, prev, h)
}
/// `hsd_8039D4DC`.
pub fn hsd_8039D4DC<J: Copy + PartialEq>(s: &mut Sys<J>, g: GenId, h: &mut impl Host<J>) {
    s.kill(g, h)
}
/// `hsd_8039D580`.
pub fn hsd_8039D580<J: Copy + PartialEq>(s: &mut Sys<J>, j: Option<J>, h: &mut impl Host<J>) {
    s.kill_by_jobj(j, h)
}
/// `hsd_8039D5DC`.
pub fn hsd_8039D5DC<J: Copy + PartialEq>(s: &mut Sys<J>, j: Option<J>, h: &mut impl Host<J>) {
    s.kill_by_jobj_tree(j, h)
}
/// `hsd_8039D688`.
pub fn hsd_8039D688<J: Copy + PartialEq>(s: &mut Sys<J>, j: Option<J>, h: &mut impl Host<J>) {
    s.remove_by_jobj(j, h)
}
/// `hsd_8039D71C`.
pub fn hsd_8039D71C<J: Copy + PartialEq>(s: &mut Sys<J>, g: GenId, h: &mut impl Host<J>) {
    s.attach_transform(g, h)
}
/// `hsd_8039D9C8`.
pub fn hsd_8039D9C8<J: Copy + PartialEq>(s: &mut Sys<J>) -> Option<GenId> {
    s.alloc()
}
/// `hsd_8039EE24`.
pub fn hsd_8039EE24<J: Copy + PartialEq>(s: &mut Sys<J>, mask: u32, h: &mut impl Host<J>) {
    s.update(mask, h)
}
/// `hsd_8039EFAC`.
pub fn hsd_8039EFAC<J: Copy + PartialEq>(s: &mut Sys<J>, link: i32, bank: i32, gfx: i32, j: Option<J>, h: &mut impl Host<J>) -> Option<GenId> {
    s.spawn_on_jobj(link, bank, gfx, j, h)
}
/// `hsd_8039F05C`.
pub fn hsd_8039F05C<J: Copy + PartialEq>(s: &mut Sys<J>, link: i32, bank: i32, idx: i32, h: &mut impl Host<J>) -> Option<GenId> {
    s.spawn(link, bank, idx, h)
}
/// `hsd_8039F6CC`.
pub fn hsd_8039F6CC<J: Copy + PartialEq>(s: &mut Sys<J>, link: i32, bank: i32, gfx: i32, j: Option<J>, h: &mut impl Host<J>) -> Option<GenId> {
    s.spawn_on_jobj_once(link, bank, gfx, j, h)
}
/// `HSD_StageSlotParticlesClear`.
pub fn HSD_StageSlotParticlesClear<J: Copy + PartialEq>(s: &mut Sys<J>, h: &mut impl Host<J>) {
    s.stage_slot_particles_clear(h)
}

#[allow(non_snake_case, unused_imports, clippy::missing_safety_doc)]
pub mod cnames {
    #[allow(unused_imports)]
    use crate::generator::*;
    use super::*;
    /// `hsd_8039DAD4`: the C entry point over `crate::generator::emit` (the owned object model).
    #[inline]
    pub fn hsd_8039DAD4<J: Copy + PartialEq, H: GeneratorHost<J>>(gen: &mut HsdGenerator<J>, id: GenId, host: &mut H) -> f32 {
        crate::generator::emit(gen, id, host)
    }
}
pub use cnames::*;
