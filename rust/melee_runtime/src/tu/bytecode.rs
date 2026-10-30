//! `src/sysdolphin/baselib/bytecode.c` has one function, `HSD_ByteCodeEval`, already ported (and
//! checked against the C) in [`crate::bytecode`]; this TU re-exports it under the C's name.
#![allow(non_snake_case)]

pub use crate::bytecode::byte_code_eval;
use crate::random::HsdRand;

/// `HSD_ByteCodeEval(bytecode, args, nb_args)`; the RNG state the C reads globally is passed in.
pub fn HSD_ByteCodeEval(bytecode: Option<&[u8]>, args: &[f32], nb_args: i32, rand: &mut HsdRand) -> f32 {
    melee_core::ct!("HSD_ByteCodeEval");
    byte_code_eval(bytecode, args, nb_args, rand)
}
