//! `src/sysdolphin/baselib/gobjinit.c`: library init defaults and `HSD_GObjInit`.
//!
//! Native ports: [`crate::gobj::hsd_gobj_set_init_defaults`] (the `init_defaults` static) and
//! `GObjSystem::init` (allocates the link heads / proc tables as `Vec`s, flattens the `funcs`
//! chain into the obj-kind table and resets the walk globals).
#![allow(non_snake_case)]
use crate::gobj::{hsd_gobj_set_init_defaults, GObjSystem, HsdGObjLibInitData};

/// `HSD_GObjSetInitDefaults`: `*arg0 = init_defaults`.
pub fn HSD_GObjSetInitDefaults<F>() -> HsdGObjLibInitData<F> {
    hsd_gobj_set_init_defaults()
}

/// `HSD_GObjInit(arg0)`.
pub fn HSD_GObjInit<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, arg0: &mut HsdGObjLibInitData<F>) {
    sys.init(arg0)
}
