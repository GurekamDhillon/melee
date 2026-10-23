"""Variadic effect-spawn callees (efSync_Spawn / efAsync_Spawn) of the item TUs: the C driver rewrites each call into a
fixed-arity recording function (tagged arguments, see EF_PRE), the Rust port passes `&[EfArg]` slices.
Shared by the iteffect / itmaplib / item oracles."""


# kind tags: 0 = integer, 1 = pointer, 2 = float*, 3 = Vec3* (its three floats are traced)
_ARG_TRACE = (
    "for (i = 0; i < n; i++) { int kind = __builtin_va_arg(ap, int); unsigned long long v = __builtin_va_arg(ap, unsigned long long);"
    " switch (kind) { case 0: T((u32) v); break; case 1: T(PI((void*) v)); break;"
    " case 2: T(B(*(float*) v)); break; default: T(B(((float*) v)[0])); T(B(((float*) v)[1])); T(B(((float*) v)[2])); break; } }")

EFSYNC_C = (
    "void* oracle_efsync_n(int n, int a0, HSD_GObj* a1, ...) { __builtin_va_list ap; int i; __builtin_va_start(ap, a1);"
    " T(@ID@); T((u32) a0); T(PI(a1)); T((u32) n); " + _ARG_TRACE + " __builtin_va_end(ap); return 0; }")
EFSYNC_R = (
    "|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t(a2.len() as u32); for a in a2.iter() { match *a {"
    " EfArg::Int(v) => t(v as u32), EfArg::Ptr(p) => t(pi(p)), EfArg::F32P(p) => t((*p).to_bits()),"
    " EfArg::Vec3P(p) => { t((*p).x.to_bits()); t((*p).y.to_bits()); t((*p).z.to_bits()); } } } core::ptr::null_mut() }")
EFASYNC_C = (
    "void oracle_efasync_n(int n, HSD_GObj* a0, void* a1, u32 a2, u32 a3, HSD_JObj* a4, ...) { __builtin_va_list ap; int i;"
    " __builtin_va_start(ap, a4); T(@ID@); T(PI(a0)); T(PI(a1)); T(a2); T(a3); T(PI(a4)); T((u32) n); " + _ARG_TRACE +
    " __builtin_va_end(ap); }")
EFASYNC_R = (
    "|a0, a1, a2, a3, a4, a5| { t(@ID@); t(pi(a0)); t(pi(a1)); t(a2); t(a3); t(pi(a4)); t(a5.len() as u32); for a in a5.iter() { match *a {"
    " EfArg::Int(v) => t(v as u32), EfArg::Ptr(p) => t(pi(p)), EfArg::F32P(p) => t((*p).to_bits()),"
    " EfArg::Vec3P(p) => { t((*p).x.to_bits()); t((*p).y.to_bits()); t((*p).z.to_bits()); } } } }")

# placed in the driver before the TU (after the headers it includes have been seen once)
EF_PRE = ("#include <Runtime/platform.h>\n#include <melee/ef/efsync.h>\n#include <melee/ef/efasync.h>\n" + r"""#define NARGN_(_1,_2,_3,_4,_5,_6,_7,_8,_9,N,...) N
#define NARGN(...) NARGN_(__VA_ARGS__,9,8,7,6,5,4,3,2,1,0)
#define EFA(x) _Generic((x), Vec3*: 3, float*: 2, int: 0, default: 1), (unsigned long long) (x)
#define CAT_(a,b) a##b
#define CAT(a,b) CAT_(a,b)
#define EFS_2(id, g) oracle_efsync_n(0, id, g)
#define EFS_3(id, g, a) oracle_efsync_n(1, id, g, EFA(a))
#define EFS_4(id, g, a, b) oracle_efsync_n(2, id, g, EFA(a), EFA(b))
#define EFS_5(id, g, a, b, c) oracle_efsync_n(3, id, g, EFA(a), EFA(b), EFA(c))
#define EFS_6(id, g, a, b, c, d) oracle_efsync_n(4, id, g, EFA(a), EFA(b), EFA(c), EFA(d))
#define EFAS_5(g, q, k, id, j) oracle_efasync_n(0, g, q, k, id, j)
#define EFAS_6(g, q, k, id, j, a) oracle_efasync_n(1, g, q, k, id, j, EFA(a))
#define EFAS_7(g, q, k, id, j, a, b) oracle_efasync_n(2, g, q, k, id, j, EFA(a), EFA(b))
#define EFAS_8(g, q, k, id, j, a, b, c) oracle_efasync_n(3, g, q, k, id, j, EFA(a), EFA(b), EFA(c))
#define EFAS_9(g, q, k, id, j, a, b, c, d) oracle_efasync_n(4, g, q, k, id, j, EFA(a), EFA(b), EFA(c), EFA(d))
#define efSync_Spawn(...) CAT(EFS_, NARGN(__VA_ARGS__))(__VA_ARGS__)
#define efAsync_Spawn(...) CAT(EFAS_, NARGN(__VA_ARGS__))(__VA_ARGS__)
void* oracle_efsync_n(int, int, HSD_GObj*, ...);
void oracle_efasync_n(int, HSD_GObj*, void*, u32, u32, HSD_JObj*, ...);
static void* ARCP;
""")

EF_CUSTOM = {
    "efSync_Spawn": dict(
        c=EFSYNC_C, ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[EfArg]) -> *mut c_void;", hook=EFSYNC_R),
    "efAsync_Spawn": dict(
        c=EFASYNC_C,
        ext="fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: &[EfArg]);",
        hook=EFASYNC_R),
}

# the Rust-side argument type, defined once at the top of every port file that calls these
EFARG_RS = """/// One argument of a variadic effect spawn (`efSync_Spawn` / `efAsync_Spawn` are C varargs; the port keeps the
/// C type of each extra argument so the eventual implementation can read it back).
#[derive(Clone, Copy)]
pub enum EfArg {
    Int(i32),
    Ptr(*mut c_void),
    F32P(*const f32),
    Vec3P(*const Vec3),
}"""
