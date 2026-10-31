//! `extern "C" fn` hook slots filled from non-capturing closures: `cN(|a0, ..| body)` is an `unsafe extern "C" fn`
//! of the closure's signature (the closure is zero sized, so the trampoline conjures it).
#![allow(clippy::missing_safety_doc)]

unsafe extern "C" fn t0<R, F: Fn() -> R>() -> R { let f: F = unsafe { core::mem::zeroed() }; f() }
pub fn c0<R, F: Fn() -> R + Copy>(_f: F) -> unsafe extern "C" fn() -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t0::<R, F> }

unsafe extern "C" fn t1<A0, R, F: Fn(A0) -> R>(a0: A0) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0) }
pub fn c1<A0, R, F: Fn(A0) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t1::<A0, R, F> }

unsafe extern "C" fn t2<A0, A1, R, F: Fn(A0, A1) -> R>(a0: A0, a1: A1) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1) }
pub fn c2<A0, A1, R, F: Fn(A0, A1) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t2::<A0, A1, R, F> }

unsafe extern "C" fn t3<A0, A1, A2, R, F: Fn(A0, A1, A2) -> R>(a0: A0, a1: A1, a2: A2) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2) }
pub fn c3<A0, A1, A2, R, F: Fn(A0, A1, A2) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t3::<A0, A1, A2, R, F> }

unsafe extern "C" fn t4<A0, A1, A2, A3, R, F: Fn(A0, A1, A2, A3) -> R>(a0: A0, a1: A1, a2: A2, a3: A3) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2, a3) }
pub fn c4<A0, A1, A2, A3, R, F: Fn(A0, A1, A2, A3) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2, A3) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t4::<A0, A1, A2, A3, R, F> }

unsafe extern "C" fn t5<A0, A1, A2, A3, A4, R, F: Fn(A0, A1, A2, A3, A4) -> R>(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2, a3, a4) }
pub fn c5<A0, A1, A2, A3, A4, R, F: Fn(A0, A1, A2, A3, A4) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2, A3, A4) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t5::<A0, A1, A2, A3, A4, R, F> }

unsafe extern "C" fn t6<A0, A1, A2, A3, A4, A5, R, F: Fn(A0, A1, A2, A3, A4, A5) -> R>(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4, a5: A5) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2, a3, a4, a5) }
pub fn c6<A0, A1, A2, A3, A4, A5, R, F: Fn(A0, A1, A2, A3, A4, A5) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2, A3, A4, A5) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t6::<A0, A1, A2, A3, A4, A5, R, F> }

unsafe extern "C" fn t7<A0, A1, A2, A3, A4, A5, A6, R, F: Fn(A0, A1, A2, A3, A4, A5, A6) -> R>(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2, a3, a4, a5, a6) }
pub fn c7<A0, A1, A2, A3, A4, A5, A6, R, F: Fn(A0, A1, A2, A3, A4, A5, A6) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2, A3, A4, A5, A6) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t7::<A0, A1, A2, A3, A4, A5, A6, R, F> }

unsafe extern "C" fn t8<A0, A1, A2, A3, A4, A5, A6, A7, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7) -> R>(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6, a7: A7) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2, a3, a4, a5, a6, a7) }
pub fn c8<A0, A1, A2, A3, A4, A5, A6, A7, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2, A3, A4, A5, A6, A7) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t8::<A0, A1, A2, A3, A4, A5, A6, A7, R, F> }

unsafe extern "C" fn t9<A0, A1, A2, A3, A4, A5, A6, A7, A8, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7, A8) -> R>(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6, a7: A7, a8: A8) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2, a3, a4, a5, a6, a7, a8) }
pub fn c9<A0, A1, A2, A3, A4, A5, A6, A7, A8, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7, A8) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2, A3, A4, A5, A6, A7, A8) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t9::<A0, A1, A2, A3, A4, A5, A6, A7, A8, R, F> }

unsafe extern "C" fn t10<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) -> R>(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6, a7: A7, a8: A8, a9: A9) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) }
pub fn c10<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t10::<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, R, F> }

unsafe extern "C" fn t11<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) -> R>(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6, a7: A7, a8: A8, a9: A9, a10: A10) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10) }
pub fn c11<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t11::<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, R, F> }

unsafe extern "C" fn t12<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) -> R>(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6, a7: A7, a8: A8, a9: A9, a10: A10, a11: A11) -> R { let f: F = unsafe { core::mem::zeroed() }; f(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11) }
pub fn c12<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, R, F: Fn(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) -> R + Copy>(_f: F) -> unsafe extern "C" fn(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) -> R { assert!(core::mem::size_of::<F>() == 0, "cabi: capturing closure"); t12::<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, R, F> }
