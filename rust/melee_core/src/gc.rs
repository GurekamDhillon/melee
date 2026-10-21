//! GameCube (ppc32, big-endian) layout metadata for native structs.
//!
//! Native structs are ordinary little-endian Rust with 8-byte pointers, so their offsets differ
//! from the game's. Each ported struct also implements [`GcLayout`], recording where each field
//! lived on the GameCube. This one table is meant to drive:
//!   * the big-endian DAT/archive loader (decode GC bytes into the native struct),
//!   * netplay / Slippi serialisers that must speak the GC byte layout,
//!   * the future m-ex importer's "GameCube offset -> native field" map.
//!
//! The tables are generated, never hand-typed: `clang --target=ppc32-none-eabi -Xclang
//! -fdump-record-layouts-complete` over the C headers (docs/rust-port/inventory.md).
//! Bitfields keep PPC numbering: `bits.0` counts from the MSB of the storage unit.

/// One field of a GameCube struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GcField {
    /// C field name.
    pub name: &'static str,
    /// Byte offset in the GameCube struct.
    pub offset: u32,
    /// Size in bytes on the GameCube (storage unit size for a bitfield).
    pub size: u32,
    /// For bitfields: (first bit counted from the MSB of the storage unit, width).
    pub bits: Option<(u8, u8)>,
    /// C type spelling, for tools and diagnostics.
    pub c_type: &'static str,
}

impl GcField {
    pub const fn new(name: &'static str, offset: u32, size: u32, c_type: &'static str) -> Self {
        Self { name, offset, size, bits: None, c_type }
    }
    pub const fn bitfield(
        name: &'static str,
        offset: u32,
        size: u32,
        bit: u8,
        width: u8,
        c_type: &'static str,
    ) -> Self {
        Self { name, offset, size, bits: Some((bit, width)), c_type }
    }
}

/// GameCube layout of a native type.
pub trait GcLayout {
    /// C type name (`Fighter`, `HSD_JObj`...).
    const GC_NAME: &'static str;
    /// `sizeof` on the GameCube.
    const GC_SIZE: u32;
    /// `alignof` on the GameCube.
    const GC_ALIGN: u32;
    /// Fields in declaration order.
    const GC_FIELDS: &'static [GcField];

    /// Field by C name.
    fn gc_field(name: &str) -> Option<&'static GcField> {
        Self::GC_FIELDS.iter().find(|f| f.name == name)
    }
    /// The field covering GameCube byte offset `off` (top level only; nested structs are
    /// resolved by the caller through the field type's own `GcLayout`).
    fn gc_field_at(off: u32) -> Option<&'static GcField> {
        Self::GC_FIELDS.iter().find(|f| off >= f.offset && off < f.offset + f.size.max(1))
    }
}

/// Declare a [`GcLayout`] impl. Generated tables use this form:
/// ```
/// use melee_core::gc::GcLayout;
/// pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }
/// melee_core::gc_layout!(Vec3, "Vec3", size 0xC, align 4, [
///     x @ 0x0: 4 "f32",
///     y @ 0x4: 4 "f32",
///     z @ 0x8: 4 "f32",
/// ]);
/// assert_eq!(Vec3::gc_field_at(5).unwrap().name, "y");
/// assert_eq!(Vec3::GC_SIZE, 12);
/// ```
/// A bitfield is written `name @ off: size "type" bits FIRST_MSB + WIDTH`.
#[macro_export]
macro_rules! gc_layout {
    ($ty:ty, $cname:literal, size $size:literal, align $align:literal,
     [ $( $f:ident @ $off:literal : $sz:literal $cty:literal $( bits $bit:literal + $w:literal )? ),* $(,)? ]) => {
        impl $crate::gc::GcLayout for $ty {
            const GC_NAME: &'static str = $cname;
            const GC_SIZE: u32 = $size;
            const GC_ALIGN: u32 = $align;
            const GC_FIELDS: &'static [$crate::gc::GcField] = &[
                $( $crate::gc_layout!(@field $f, $off, $sz, $cty $(, $bit, $w)?) ),*
            ];
        }
    };
    (@field $f:ident, $off:literal, $sz:literal, $cty:literal) => {
        $crate::gc::GcField::new(stringify!($f), $off, $sz, $cty)
    };
    (@field $f:ident, $off:literal, $sz:literal, $cty:literal, $bit:literal, $w:literal) => {
        $crate::gc::GcField::bitfield(stringify!($f), $off, $sz, $bit, $w, $cty)
    };
}

#[cfg(test)]
mod tests {
    use super::GcLayout;

    struct Flags;
    crate::gc_layout!(Flags, "Flags", size 4, align 4, [
        a @ 0x0: 1 "u8" bits 0 + 1,
        b @ 0x0: 1 "u8" bits 1 + 7,
        c @ 0x2: 2 "u16",
    ]);

    #[test]
    fn bitfields_and_lookup() {
        assert_eq!(Flags::gc_field("b").unwrap().bits, Some((1, 7)));
        assert_eq!(Flags::gc_field_at(3).unwrap().name, "c");
        assert!(Flags::gc_field_at(1).is_none());
    }
}
