"""Adapt c2rust reference output to the one generated type set (developer utility).

Requires the two reference outputs in rust/target/texp-transpile, produced with
tools/rust-port/gen_cc.py and c2rust. No C data or duplicate records are emitted.
"""
from pathlib import Path
import re,sys
ROOT=Path(__file__).resolve().parents[3]
def adapt(s):
    s=s.replace('#[no_mangle]\n','').replace('unsafe extern "C" fn','unsafe fn')
    for a,b in {'bool_0':'i32','u8_0':'u8','u16_0':'u16','u32_0':'u32','f32_0':'f32','f64_0':'f64','gc_size_t':'usize','type_0':'r#type','ref_0':'r#ref','C2RustUnnamed':'HSD_TExpRes_reg','true_0':'1','GX_ENABLE':'1','GX_DISABLE':'0','GX_TRUE':'1','GX_FALSE':'0'}.items():
        s=re.sub(r'\b'+a+r'\b',b,s)
    # Fields at offset zero are addressed without dereferencing null list tails.
    s=re.sub(r'&mut \(\*(\w+)\)\.cnst as \*mut HSD_TECnst',r'\1.cast::<HSD_TECnst>()',s)
    return s
raw=(ROOT/'rust/target/texp-transpile/raw/src/texp.rs').read_text()
body=adapt(raw[raw.index('#[inline]\nunsafe extern "C" fn IsThroughColor'):])
body=re.sub(r'(?m)^unsafe fn', 'pub unsafe fn',body)
# Signed reference counts follow the shipping C's wrapping integer arithmetic.
body=re.sub(r'(\(\*[^\n;]+\.tev\.[ca]_ref)\s*([+-])= 1 as ::core::ffi::c_int;',
            lambda m:f'{m[1]} = {m[1]}.wrapping_{"add" if m[2]=="+" else "sub"}(1);',body)
for name in ['IsThroughColor','IsThroughAlpha']:
    a=body.index('pub unsafe fn '+name); b=body.index('\n}',a)+2
    body=body[:a]+f'pub unsafe fn {name}(texp:*mut HSD_TExp)->i32 {{ melee_types::inline::{name}(&*texp) as i32 }}'+body[b:]
body=body.replace('&raw mut (*texp).cnst as *mut HSD_TECnst','texp.cast::<HSD_TECnst>()')
body=body.replace('&raw mut (*(*clist).next).cnst as *mut HSD_TECnst','(*clist).next.cast::<HSD_TECnst>()')
body=body.replace('&raw mut (**tevdesc).desc','(*tevdesc).cast::<HSD_TevDesc>()')

ext='''pub mod ext {
    use super::*;
    // TODO allocator / DAG bindings. Renderer hooks bind the explicit HsdRender
    // context used by the existing state/tev ports; their logic is not duplicated.
    macro_rules! hooks {
        (@default) => { None };
        (@default $e:expr) => { Some($e) };
        ($(fn $n:ident($($a:ident:$t:ty),*) $(->$r:ty)? $(= $default:expr)?;)*) => {
            pub mod hook { use super::*; $(pub static mut $n: Option<unsafe fn($($t),*) $(->$r)?>=hooks!(@default $($default)?);)* }
            $(pub unsafe fn $n($($a:$t),*) $(->$r)? {
                match hook::$n { Some(f)=>f($($a),*), None=>panic!(concat!("TODO binding: ",stringify!($n))) }
            })*
        }
    }
    hooks! {
        fn hsdAllocMemPiece(size:i32)->*mut core::ffi::c_void;
        fn hsdFreeMemPiece(mem:*mut core::ffi::c_void,size:i32);
        fn GXPixModeSync();
        fn GXSetTevColor(id:GXTevRegID,color:GXColor);
        fn GXSetTevKColor(id:GXTevKColorID,color:GXColor);
        fn HSD_StateInvalidate(mask:i32);
        fn HSD_StateAssignTev()->i32;
        fn HSD_SetupTevStage(desc:*mut HSD_TevDesc);
        fn HSD_Index2TevStage(idx:i32)->i32 = crate::tu::tev::HSD_Index2TevStage;
        fn HSD_TExpMakeDag(root:*mut HSD_TExp,list:*mut HSD_TExpDag)->i32;
        fn HSD_TExpSchedule(num:i32,list:*mut HSD_TExpDag,result:*mut *mut HSD_TExp,res:*mut HSD_TExpRes);
        fn HSD_TExpSimplify(texp:*mut HSD_TExp)->i32;
        fn HSD_TExpSimplify2(texp:*mut HSD_TExp)->i32;
    }
}
'''
head='''//! Native texture-expression construction, resource assignment and TEV compilation.
//! All 26 definitions from texp.c plus its two header predicates. Arithmetic is
//! integer except SetReg's separately rounded scalar multiplies (zero fmuladd
//! sites in the shipping-flags oracle IR). All records come from melee_types.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments,
         unused_parens, static_mut_refs, dead_code)]
use melee_types::*;
use ext::*;
pub const HSD_TEXP_TEX: *mut HSD_TExp = -1isize as *mut HSD_TExp;
pub const HSD_TEXP_RAS: *mut HSD_TExp = -2isize as *mut HSD_TExp;
unsafe fn memset(p:*mut core::ffi::c_void,v:i32,n:usize)->*mut core::ffi::c_void {
    core::ptr::write_bytes(p.cast::<u8>(),v as u8,n); p
}
unsafe fn __assert(_: *const i8,line:u32,condition:*const i8)->! {
    panic!("texp.c:{line}: {}",std::ffi::CStr::from_ptr(condition).to_string_lossy())
}
unsafe fn HSD_Panic(_: *const i8,line:u32,condition:*const i8)->! { __assert(core::ptr::null(),line,condition) }
'''
port=ROOT/'rust/melee_runtime/src/tu/texp.rs'
if '--red' in sys.argv:
    # All signatures exist so the behavioral oracle fails by executing a missing
    # implementation, not by failing to compile/import the module.
    body=re.sub(r'(?m)^pub unsafe fn (\w+)',r'pub unsafe fn \1',body)
    start=body.index('pub unsafe fn HSD_TExpGetType'); a=body.index('{',start); depth=1;b=a+1
    while depth:
        depth += (body[b]=='{')-(body[b]=='}'); b+=1
    body=body[:a]+'{ panic!("texp implementation pending") }'+body[b:]
port.write_text(head+ext+body)
# The test scenario driver is translated mechanically from the C fixture. The
# tested implementation is imported as m; fixture hook calls are separate.
d=(ROOT/'rust/target/texp-transpile/driver/src/texp_driver.rs').read_text()
d=adapt(d[d.index('static mut pool:'):])
# All native static records are zero-initialized using generated types.
for n,t in [('pool','[HSD_TExp;64]'),('nodes','[HSD_TExp;12]'),('descs','[HSD_TExpTevDesc;4]'),('textures','[HSD_TObj;2]')]:
    a=d.index('static mut '+n+':'); b=d.index('\nstatic mut ',a+1)
    d=d[:a]+f'static mut {n}: {t} = unsafe {{ core::mem::zeroed() }};'+d[b:]
names=re.findall(r'        fn (\w+)\(',ext)
for n in names:
    d=re.sub(r'(unsafe fn )'+n+r'\b',r'\1stub_'+n,d)
# Panic handling in the C fixture is unreachable for all generated valid cases.
for n in ['__assert','HSD_Panic']:
    a=d.index('pub unsafe fn '+n); b=d.index('\n}',a)+2
    d=d[:a]+d[b:]
d=re.sub(r'(?s)    if 0 as ::core::ffi::c_int != 0 \{.*?\n    \}', '',d)
test='''//! Field-by-field differential oracle, 14 scenario families x 256 seeds.
#![allow(non_snake_case,non_upper_case_globals,unused_mut,unused_variables,
         unused_assignments,unused_parens,static_mut_refs,dead_code)]
#[path="../src/tu/texp.rs"] mod m;
use m::*;
use melee_types::*;
pub mod tu { pub use melee_runtime::tu::tev; }
static mut WORDS: std::vec::Vec<u32> = std::vec::Vec::new();
unsafe fn emit(v:u32) { WORDS.push(v); }
unsafe fn memset(p:*mut core::ffi::c_void,v:i32,n:usize)->*mut core::ffi::c_void {
    core::ptr::write_bytes(p.cast::<u8>(),v as u8,n); p
}
'''+d+'''
#[test]
fn every_texp_function_matches_native_c_fields_and_transcript() {
    unsafe {
'''+''.join(f'        m::ext::hook::{n}=Some(stub_{n});\n' for n in names)+'''
        let oracle=include_str!("data/oracle_texp.txt");
        let mut lines=oracle.lines().filter(|s| !s.trim().is_empty());
        for op in 0..14 { for seed in 1..=256 {
            WORDS.clear(); run_case(op,seed);
            let expected:std::vec::Vec<u32>=lines.next().unwrap().split_whitespace()
                .map(|s|u32::from_str_radix(s,16).unwrap()).collect();
            assert_eq!(WORDS.len(),expected.len(),"op={op}, seed={seed}: word count");
            for (field,(got,want)) in WORDS.iter().zip(expected.iter()).enumerate() {
                assert_eq!(got,want,"op={op}, seed={seed}, field/call word={field}");
            }
        }}
        assert!(lines.next().is_none());
    }
}
'''
(ROOT/'rust/melee_runtime/tests/tu_texp.rs').write_text(test)
