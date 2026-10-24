//! `src/sysdolphin/baselib/gobjobject.c`: attach / detach / release a GObj's HSD object.
//!
//! All four functions are native ports on [`GObjSystem`] (`crate::gobj`, oracle
//! `tests/oracle_gobj.rs`); this TU exposes them under the C names.
#![allow(non_snake_case)]
use crate::gobj::{GObjHost, GObjId, GObjSystem};

/// `HSD_GObjObject_80390A3C(id, p_link)`: first GObj of `p_link` whose classifier is `id`.
pub fn HSD_GObjObject_80390A3C<F: Copy, O: Copy>(sys: &GObjSystem<F, O>, id: u16, p_link: u8) -> Option<GObjId> {
    sys.gobjobject_80390a3c(id, p_link)
}

/// `HSD_GObjObject_80390A70`: asserts the GObj has no object, then attaches `obj` as `kind`.
pub fn HSD_GObjObject_80390A70<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId, kind: u8, obj: Option<O>) {
    sys.gobjobject_80390a70(gobj, kind, obj)
}

/// `HSD_GObjObject_80390ADC`: detaches and returns the object (None when there was none).
pub fn HSD_GObjObject_80390ADC<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId) -> Option<O> {
    sys.gobjobject_80390adc(gobj)
}

/// `HSD_GObjObject_80390B0C`: runs the obj-kind release function and clears the object.
pub fn HSD_GObjObject_80390B0C<F: Copy, O: Copy>(
    sys: &mut GObjSystem<F, O>,
    host: &mut impl GObjHost<F, O>,
    gobj: GObjId,
) {
    sys.gobjobject_80390b0c(host, gobj)
}
