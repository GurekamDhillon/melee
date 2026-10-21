//! Port of `src/sysdolphin/baselib/bytecode.c`: `HSD_ByteCodeEval`, the stack machine behind
//! robj.c's bytecode expressions (`HSD_ByteCodeExpDesc`).
//!
//! Native shape: the C's `HSD_SList` stack of `void*` words is a `Vec<u32>` with the top at the
//! end. Only the low 32 bits of a word are ever read (the `ByteCodeVal` union's `i` / `f`), so a
//! word is a `u32` holding either an `int` or an `f32` bit pattern. The bytecode is a slice and
//! the program counter an index into it; a jump or read past its end panics (the C reads past
//! the buffer). `HSD_ASSERT` / `HSD_Panic` are panics.
//!
//! Float rule: `fma_sites.py` finds no fused site of bytecode.c's own; the only sites are the
//! inlined MSL `fmodf` (`aobj::fmodf`) and math_ppc `sqrtf` (`gekko::sqrtf`). The degree /
//! radian scalings are `(f32)(K * (f64)x)` with a cast in between, so nothing fuses there.
//! Integer ops follow gwtool's safe arithmetic (`cabi::sdiv` / `srem`, wrapping add/sub/mul/neg),
//! `(int)f` saturates (`cabi::f2i32`).
//!
//! `logf` (opcode 0x13) is not the game's own: the build has no `logf` definition (MSL math.c
//! is not compiled), so it is the host CRT's; the port uses Rust's `f32::ln` (the same CRT
//! `logf` on Windows). `expf` / `powf` are lb_00CE.c's (`lb_00ce`).
use crate::aobj::fmodf;
use crate::cabi;
use crate::gekko::{cosf, sinf, sqrtf, tanf};
use crate::hsd_mtx::fabsf_bitwise;
use crate::lb_00ce::{expf, powf};
use crate::lbtrigf::{acosf, asinf, atan2f, atanf};
use crate::random::HsdRand;

/// util.h `DEG_TO_RAD`.
pub const DEG_TO_RAD: f64 = 0.017453292519943295;
/// util.h `RAD_TO_DEG`.
pub const RAD_TO_DEG: f64 = 57.29577951308232;

#[inline]
fn f(w: u32) -> f32 {
    f32::from_bits(w)
}

/// `HSD_ByteCodeEval(bytecode, args, nb_args)`. `bytecode` None is NULL (returns 0).
pub fn byte_code_eval(bytecode: Option<&[u8]>, args: &[f32], nb_args: i32, rand: &mut HsdRand) -> f32 {
    let Some(code) = bytecode else { return 0.0 };
    let mut pc: usize = 0;
    let mut stack: Vec<u32> = Vec::new();
    let mut operand_count: i32 = 0;
    let mut operand: u32 = 0;
    let mut last_command: u8 = 0;

    macro_rules! top {
        () => {
            *stack.last_mut().expect("bytecode: empty stack")
        };
    }
    // Binary ops: pop the top (`rhs`), the new top is `lhs`.
    macro_rules! pop2 {
        () => {{
            assert!(stack.len() >= 2, "bytecode: stack underflow");
            stack.pop().unwrap()
        }};
    }

    loop {
        if operand_count > 0 {
            operand_count -= 1;
            operand = (operand << 8) | code[pc] as u32;
            pc += 1;
            if operand_count != 0 {
                continue;
            }
            match last_command {
                2 => {
                    assert!((operand as i64) < nb_args as i64, "bytecode: operand < nb_args");
                    stack.push(args[operand as usize].to_bits());
                }
                5 => {
                    let mut i: u32 = 0;
                    while i < operand {
                        stack.pop();
                        i += 1;
                    }
                }
                0x3C => {
                    let depth = operand as usize;
                    if depth >= stack.len() {
                        panic!("specified stack doesn't exist ({}).", operand as i32);
                    }
                    let v = stack[stack.len() - 1 - depth];
                    stack.push(v);
                }
                3 => {
                    let v = stack.pop().expect("bytecode: empty stack");
                    if v != 0 {
                        pc += operand as usize;
                    }
                }
                4 => pc += operand as usize,
                6 => stack.push(operand),
                0xFF => panic!("not yet implemented."),
                _ => panic!("unexpected byte code."),
            }
            continue;
        }

        last_command = code[pc];
        pc += 1;
        match last_command {
            0 => {}
            1 => {
                return f(*stack.last().expect("bytecode: empty stack"));
            }
            5 | 0x3C | 0xFF => {
                operand_count = 1;
                operand = 0;
            }
            2 | 3 | 4 => {
                operand_count = 2;
                operand = 0;
            }
            6 => {
                operand_count = 4;
                operand = 0;
            }
            7 => {
                let t = &mut top!();
                *t = cabi::f2i32(f(*t) as f64) as u32;
            }
            8 => {
                let t = &mut top!();
                *t = (*t as i32 as f32).to_bits();
            }
            9 => {
                let t = &mut top!();
                *t = (-f(*t)).to_bits();
            }
            0x0A => {
                let t = &mut top!();
                *t = (*t as i32).wrapping_neg() as u32;
            }
            0x0B => {
                let _ = top!();
                let v = rand.randi(2);
                top!() = v as u32;
            }
            0x0C => {
                let _ = top!();
                let v = rand.randf();
                top!() = v.to_bits();
            }
            0x0D => {
                let t = &mut top!();
                *t = sinf((DEG_TO_RAD * f(*t) as f64) as f32).to_bits();
            }
            0x0E => {
                let t = &mut top!();
                *t = cosf((DEG_TO_RAD * f(*t) as f64) as f32).to_bits();
            }
            0x0F => {
                let t = &mut top!();
                *t = tanf((DEG_TO_RAD * f(*t) as f64) as f32).to_bits();
            }
            0x10 => {
                let t = &mut top!();
                *t = ((RAD_TO_DEG * asinf(f(*t)) as f64) as f32).to_bits();
            }
            0x11 => {
                let t = &mut top!();
                *t = ((RAD_TO_DEG * acosf(f(*t)) as f64) as f32).to_bits();
            }
            0x12 => {
                let t = &mut top!();
                *t = ((RAD_TO_DEG * atanf(f(*t)) as f64) as f32).to_bits();
            }
            0x13 => {
                let t = &mut top!();
                *t = f(*t).ln().to_bits();
            }
            0x14 => {
                let t = &mut top!();
                *t = expf(f(*t)).to_bits();
            }
            0x15 => {
                let t = &mut top!();
                if f(*t) < 0.0 {
                    *t = (-f(*t)).to_bits();
                }
            }
            0x28 => {
                let t = &mut top!();
                let d0 = *t as i32;
                if d0 < 0 {
                    *t = d0.wrapping_neg() as u32;
                }
            }
            0x16 => {
                let t = &mut top!();
                *t = sqrtf(f(*t)).to_bits();
            }
            0x31 => {
                let t = &mut top!();
                *t = (*t as i32 == 0) as u32;
            }
            0x17..=0x1B | 0x21 | 0x22 | 0x23 | 0x26 | 0x33..=0x38 => {
                let f0 = f(pop2!());
                let t = &mut top!();
                let f1 = f(*t);
                *t = match last_command {
                    0x17 => (f1 + f0).to_bits(),
                    0x18 => (f1 - f0).to_bits(),
                    0x19 => (f1 * f0).to_bits(),
                    0x1A => (f1 / f0).to_bits(),
                    0x1B => fmodf(f1, f0).to_bits(),
                    0x21 => powf(f1, f0).to_bits(),
                    0x22 => {
                        if f1 > f0 {
                            f0.to_bits()
                        } else {
                            *t
                        }
                    }
                    0x23 => {
                        if f1 < f0 {
                            f0.to_bits()
                        } else {
                            *t
                        }
                    }
                    0x26 => {
                        let fv = if fabsf_bitwise(f0) == 0.0 {
                            if f1 >= 0.0 {
                                90.0f32
                            } else {
                                -90.0
                            }
                        } else {
                            (RAD_TO_DEG * atan2f(f1, f0) as f64) as f32
                        };
                        fv.to_bits()
                    }
                    0x33 => (f1 < f0) as u32,
                    0x34 => (f1 > f0) as u32,
                    0x35 => (f1 <= f0) as u32,
                    0x36 => (f1 >= f0) as u32,
                    0x37 => (f1 == f0) as u32,
                    _ => (f1 != f0) as u32,
                };
            }
            0x1C..=0x20 | 0x24 | 0x25 | 0x29..=0x30 | 0x32 | 0x39..=0x3B | 0x27 => {
                let d1 = pop2!() as i32;
                let d0 = top!() as i32;
                let r: i32 = match last_command {
                    0x1C => d0.wrapping_add(d1),
                    0x1D => d0.wrapping_sub(d1),
                    0x1E => d0.wrapping_mul(d1),
                    0x1F => cabi::sdiv(d0, d1),
                    0x20 => cabi::srem(d0, d1),
                    0x24 => {
                        if d0 > d1 {
                            d1
                        } else {
                            d0
                        }
                    }
                    0x25 => {
                        if d0 < d1 {
                            d1
                        } else {
                            d0
                        }
                    }
                    0x29 => (d0 < d1) as i32,
                    0x2A => (d0 > d1) as i32,
                    0x2B => (d0 <= d1) as i32,
                    0x2C => (d0 >= d1) as i32,
                    0x2D => (d0 == d1) as i32,
                    0x2E => (d0 != d1) as i32,
                    0x2F => (d0 != 0 && d1 != 0) as i32,
                    0x30 => !(d0 == 0 && d1 == 0) as i32,
                    0x32 => ((d0 == 0 && d1 != 0) || (d0 != 0 && d1 == 0)) as i32,
                    0x39 => d0 & d1,
                    0x3A => d0 | d1,
                    0x3B => d0 ^ d1,
                    _ => d0.wrapping_add(rand.randi(d1.wrapping_sub(d0).wrapping_add(1))),
                };
                top!() = r as u32;
            }
            op => panic!("unexpected opcode 0x{:x}.", op),
        }
    }
}
