//! Shared by-value vector types (C `Vec2`, `Vec`/`Vec3`). Moved here from melee_runtime::mtx
//! and melee_world::vec during integration; the generated type tables in melee_types,
//! melee_fighter, melee_chara and melee_items still carry their own copies (see RUST_PORT.md).

/// `Vec2` (`{ f32 x, y; }`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
crate::gc_layout!(Vec2, "Vec2", size 0x8, align 4, [
    x @ 0x0: 4 "f32",
    y @ 0x4: 4 "f32",
]);
impl crate::hash::StateHash for Vec2 {
    fn state_hash(&self, h: &mut crate::hash::StateHasher) {
        self.x.state_hash(h);
        self.y.state_hash(h);
    }
}

/// `Vec` / `Point3d` (`{ f32 x, y, z; }`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
crate::gc_layout!(Vec3, "Vec", size 0xC, align 4, [
    x @ 0x0: 4 "f32",
    y @ 0x4: 4 "f32",
    z @ 0x8: 4 "f32",
]);
impl crate::hash::StateHash for Vec3 {
    fn state_hash(&self, h: &mut crate::hash::StateHasher) {
        self.x.state_hash(h);
        self.y.state_hash(h);
        self.z.state_hash(h);
    }
}

