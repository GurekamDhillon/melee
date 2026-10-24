//! `libs/dolphin/src/dolphin/mtx/mtx44.c`: reuse the verified projection implementations.
pub use crate::mtx::{frustum as MTXFrustum, ortho as MTXOrtho, perspective as MTXPerspective};

/// The C-ABI shapes (`Mtx` pointer argument; the functions write four rows, so the caller passes a
/// `Mtx44`), checked bit for bit against the C by `tests/pure_mtx44.rs`
/// (`python rust/melee_runtime/gen/pure/run_pure.py mtx44`).
pub mod raw {
    #![allow(non_snake_case, clippy::missing_safety_doc)]
    use crate::mtx::{self, Mtx44};

    /// `MTXFrustum(m, t, b, l, r, n, f)`.
    pub unsafe fn MTXFrustum(m: *mut [f32; 4], t: f32, b: f32, l: f32, r: f32, n: f32, f: f32) {
        m.cast::<Mtx44>().write(mtx::frustum(t, b, l, r, n, f));
    }
    /// `MTXPerspective(m, fovY, aspect, n, f)`.
    pub unsafe fn MTXPerspective(m: *mut [f32; 4], fov_y: f32, aspect: f32, n: f32, f: f32) {
        m.cast::<Mtx44>().write(mtx::perspective(fov_y, aspect, n, f));
    }
    /// `MTXOrtho(m, t, b, l, r, n, f)`.
    pub unsafe fn MTXOrtho(m: *mut [f32; 4], t: f32, b: f32, l: f32, r: f32, n: f32, f: f32) {
        m.cast::<Mtx44>().write(mtx::ortho(t, b, l, r, n, f));
    }
}
