//! Port of src/sysdolphin/baselib/crypt.c. All five functions (`HSD_Checksum`, `encryptByte`,
//! `decryptByte`, `HSD_Encrypt`, `HSD_Decrypt`) are already ported in [`crate::hsd_util`]
//! (oracle-checked by tests/oracle_anim.rs); this module re-exports them under the C names.
#![allow(non_snake_case)]

pub use crate::hsd_util::{checksum as HSD_Checksum, decrypt as HSD_Decrypt, encrypt as HSD_Encrypt};
