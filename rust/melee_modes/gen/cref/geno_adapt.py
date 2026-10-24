"""One-off native adaptation aid for the Geno TUs; raw output is reference input only.

The final modules use melee_types layouts, not c2rust's repeated C records.
"""
import os, re, sys, json
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'rust/melee_types/gen'))
os.chdir(ROOT)
os.environ.setdefault('GALE_INCLUDE', str(ROOT/'rust/target/empty-include'))
import gen_types as g

def end(s, p, op='{', cl='}'):
    n=0; quote=None; esc=False; i=p
    while i<len(s):
        c=s[i]
        if quote:
            if esc: esc=False
            elif c=='\\': esc=True
            elif c==quote: quote=None
        elif c=='"' or (c=="'" and re.match(r"'(?:[^'\\]|\\.)'",s[i:])): quote=c
        elif s.startswith('//', i):
            i=s.find('\n',i)
            if i<0:return len(s)
        elif c==op:n+=1
        elif c==cl:
            n-=1
            if not n:return i+1
        i+=1
    raise ValueError(s[p:p+90])

def decls(s):
    out={}
    for m in re.finditer(r'^pub (struct|union) (\w+) \{',s,re.M):
        z=end(s,m.end()-1)
        fields=re.findall(r'^    pub ([\w#]+): ([^\n]+),$',s[m.end():z-1],re.M)
        out[m[2]]=(m[1], fields, m.start(),z)
    return out

def norm(n):
    if n.startswith('c2rust_unnamed') or n.startswith('anon_'):return '@'
    if n.startswith('r#'):return n[2:]
    return re.sub(r'_0$','',n)

def main(stem):
    raw=(ROOT/f'rust/melee_modes/src/raw/pc/geno/{stem}.rs').read_text()
    gen=(ROOT/'rust/melee_types/src/gen.rs').read_text()
    rr, gg=decls(raw),decls(gen)
    aliases=dict(re.findall(r'^pub type (\w+) = ([^;]+);',raw,re.M))
    tm={n:n for n in rr if n in gg}
    fields={}
    def rec(rn,gn):
        tm[rn]=gn
        rfs,gfs=rr[rn][1],gg[gn][1]
        for rf,rt in rfs:
            choices=[(gf,gt) for gf,gt in gfs if norm(rf)==norm(gf)]
            if len(choices)==1:
                gf,gt=choices[0];fields[(rn,rf)]=gf
                rbase=next((x for x in re.findall(r'\b\w+\b',rt) if x in rr),None)
                gbase=next((x for x in re.findall(r'\b\w+\b',gt) if x in gg),None)
                if rbase and gbase and rbase not in tm:rec(rbase,gbase)
    for n in list(tm):rec(n,n)
    # Anonymous local records have stable generated provenance names; match their field set.
    for n,(kind,fs,_,_) in rr.items():
        if n in tm:continue
        sig=tuple(norm(f) for f,t in fs)
        candidates=[gn for gn,(gk,gfs,_,_) in gg.items() if gk==kind and tuple(norm(f) for f,t in gfs)==sig]
        if candidates:
            # Prefer a Geno-provenance record when shapes are identical.
            candidates.sort(key=lambda x: ('pc/geno/' not in gen[max(0,gg[x][2]-200):gg[x][2]], x))
            rec(n,candidates[0])
    prim={'u8_0':'u8','u16_0':'u16','u32_0':'u32','u64_0':'u64','f32_0':'f32','f64_0':'f64','s8':'i8','s16':'i16','s32':'i32','s64':'i64','bool_0':'i32','gc_size_t':'usize','size_t':'usize','true_0':'1','false_0':'0'}
    ffi={'c_int':'i32','c_uint':'u32','c_char':'i8','c_schar':'i8','c_uchar':'u8','c_short':'i16','c_ushort':'u16','c_long':'i32','c_ulong':'u32','c_longlong':'i64','c_ulonglong':'u64','c_float':'f32','c_double':'f64','c_void':'c_void'}
    def ty(t):
        t=re.sub(r'::core::ffi::(\w+)',lambda m:ffi.get(m[1],m[0]),t)
        return re.sub(r'\b\w+\b',lambda m:prim.get(m[0],tm.get(m[0],m[0])),t)
    # Resolve remaining aliases to the shared types (including primitive enum aliases).
    known=set(re.findall(r'^pub (?:struct|union|type) (\w+)',gen,re.M))
    for n,t in aliases.items():
        if n in prim or n in tm:continue
        if n in known:tm[n]=n
        else:prim[n]=ty(t)
    for _ in range(4):
        for n,t in list(prim.items()):prim[n]=ty(t)
    tu=g.ci.Index.create().parse(f'pc/geno/{stem}.c',args=g.NAT)
    noinit=set()
    for c in tu.cursor.walk_preorder():
        if c.kind==g.K.VAR_DECL and c.location.file and 'pc/geno/' in c.location.file.name.replace('\\','/'):
            toks=[x.spelling for x in c.get_tokens()]
            if '=' not in toks or all(x in ('{','}','0',',',';') for x in toks[toks.index('=')+1:]):noinit.add(c.spelling)
    functions=[];statics=[];constants=[]
    for m in re.finditer(r'^(?:pub )?unsafe extern "C" fn (\w+)\(',raw,re.M):
        p=raw.index('{',m.end());z=end(raw,p)
        functions.append((m[1],raw[m.start():z]))
    for m in re.finditer(r'^(?:pub )?static mut (\w+): ',raw,re.M):
        p=m.end(); depth=[];i=p;quote=False;esc=False
        while i<len(raw):
            if quote:
                if esc:esc=False
                elif raw[i]=='\\':esc=True
                elif raw[i]=='"':quote=False
                i+=1;continue
            if raw[i]=='"':quote=True
            elif raw[i] in '{[(':depth.append(raw[i])
            elif raw[i] in '}])':depth.pop()
            elif raw[i]==';' and not depth:break
            i+=1
        s=raw[m.start():i+1]
        if m[1] in noinit:s=s[:s.index('=')+1]+' unsafe { core::mem::zeroed() };'
        statics.append((m[1],s))
    # Reuse exported constants. Keep only constants absent from melee_types.
    knownconst=set(re.findall(r'^pub const (\w+):',gen,re.M))
    for m in re.finditer(r'^pub const (\w+):[\s\S]*?;(?=\n|$)',raw,re.M):
        if m[1] not in knownconst and m[1] not in ('true_0','false_0'):constants.append(m[0])
    # Build a native ext layer with individually installable oracle hooks.
    ex=raw[raw.index('extern "C" {')+len('extern "C" {'):]
    ex=ex[:ex.index('\n}')]
    ed=[];ef=[]
    for m in re.finditer(r'    static mut (\w+): ([\s\S]*?);(?=\n|$)',ex):
        ed.append(f'pub static mut {m[1]}: {ty(m[2])} = unsafe {{ core::mem::zeroed() }};')
    for m in re.finditer(r'    fn (\w+)\(',ex):
        n=m[1]; z=end(ex,m.end()-1,'(',')');args=ex[m.end():z-1];ret=ex[z:ex.index(';',z)]
        if '...' in args or n=='__assert':continue
        args=re.sub(r'\s+',' ',args).strip().rstrip(',')
        # Split only commas at parameter nesting depth zero.
        parts=[];depth=0;last=0
        for i,c in enumerate(args):
            if c in '(<[':depth+=1
            elif c in ')>]':
                if c!='>' or (i>0 and args[i-1]!='-'):depth-=1
            elif c==',' and depth==0:parts.append(args[last:i]);last=i+1
        if args:parts.append(args[last:])
        ps=[f'a{i}: {ty(p.split(":",1)[1].strip())}' for i,p in enumerate(parts)]
        ef.append(f'fn {n}({", ".join(ps)}){ty(ret)};')
    body='\n\n'.join(constants+[s for n,s in statics]+[s for n,s in functions if n!='sqrtf'])
    body=ty(body)
    body=body.replace('#[no_mangle]','').replace('#[linkage = "external"]','')
    body=re.sub(r'(?m)^unsafe extern "C" fn ', 'pub unsafe extern "C" fn ',body)
    body=re.sub(r'(?m)^static mut ', 'pub static mut ',body)
    body=re.sub(r'\bfn_0\b','r#fn',body)
    body=body.replace('.type_0','.r#type').replace('type_0:', 'r#type:')
    # Replace uninitialised local aggregate constructors using C's zero initialisation.
    for m in list(re.finditer(r'(?m)^\s*(?:static mut|let mut) (\w+): [^=]+ = ',body))[::-1]:
        if re.sub(r'_\d+$','',m[1]) not in noinit and m[1] not in noinit:continue
        p=m.end();i=p;depth=[];quote=False;esc=False
        while i<len(body):
            c=body[i]
            if quote:
                if esc:esc=False
                elif c=='\\':esc=True
                elif c=='"':quote=False
            elif c=='"':quote=True
            elif c in '{[(':depth.append(c)
            elif c in '}])':
                if not depth:break
                depth.pop()
            elif c==';' and not depth:break
            i+=1
        vtype=m[0].split(':',1)[1].split('=')[0].strip()
        if re.match(r'(?:\[)?[A-Z]\w*',vtype) and '{' in body[p:i] and i<len(body) and body[i]==';':
            body=body[:p]+'unsafe { core::mem::zeroed() }'+body[i:]
    # Member paths through anonymous C unions are explicit in melee_types.
    paths={}
    def walk(rn,pre=(),post=(),seen=()):
        if rn in seen:return
        for rf,rt in rr[rn][1]:
            gf=fields.get((rn,rf),rf)
            if rf.startswith('c2rust_unnamed'):
                ch=next((x for x in re.findall(r'\b\w+\b',rt) if x in rr),None)
                if ch:walk(ch,pre+(rf,),post+(gf,),seen+(rn,))
            elif pre:
                paths.setdefault(pre+(rf,),set()).add(post+(gf,))
    for rn in tm:
        if rn in rr:walk(rn)
    for old,new in sorted(paths.items(),key=lambda kv:-len(kv[0])):
        if len(new)!=1:continue
        pat=r'\s*\.\s*'.join(map(re.escape,old))
        body=re.sub(r'\.'+pat, '.'+'.'.join(next(iter(new))), body)
    # Flattening bitfields changes only accessor paths, never the bitfield values.
    body=re.sub(r'\.c2rust_unnamed\s*\.c2rust_unnamed(?=\s*\.(?:set_)?x594_b0)', '.anon_594.anon_0',body)
    body=re.sub(r'\.c2rust_unnamed\s*\.c2rust_unnamed(?=\s*\.(?:set_)?move_id)', '.anon_8.anon_0',body)
    body=body.replace('.x44_flag.c2rust_unnamed.', '.x44_flag.anon_0.')
    body=body.replace('(*ip).anon_0.xDAA_flag.c2rust_unnamed.', '(*ip).anon_daa.xDAA_flag.anon_0.')
    body=re.sub(r'\.c2rust_unnamed\s*\.c2rust_unnamed\s*;', '.anon_8.anon_0;',body)
    body=body.replace('.c2rust_unnamed_1.set_x221B_b0', '.anon_221b.set_x221B_b0')
    body=body.replace('HSD_PAD_L | HSD_PAD_R | HSD_PAD_LR','HSD_PAD_L | HSD_PAD_R | HSD_PAD_LR as i32')
    body=body.replace('OSReport(', 'OSReport!(')
    body=body.replace('HSD_ForeachAnim(', 'HSD_ForeachAnim!(')
    if stem=='geno_game':
        ef += ['fn Geno_DefineVoiceState(a0: u32, a1: u32, a2: i32) -> i32;',
               'fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: i32, a3: u32, a4: *mut HSD_JObj);',
               'fn HSD_ForeachAnim(a0: *mut c_void, a1: HSD_Type, a2: HSD_TypeMask, a3: *mut c_void, a4: AObjArgType, a5: u64);']
        body=body.replace('efAsync_Spawn(\n', 'efAsync_Spawn(\n')
    # C's generated native enum aliases are unsigned if all enumerators are positive;
    # casts in the reference remain explicit and are checked by rustc below.
    ext='''pub mod ext {
    use super::*;
    macro_rules! hooks {
        ($(fn $n:ident($($a:ident:$t:ty),*) $(->$r:ty)?;)*) => {
            pub mod hook { use super::*; $(pub static mut $n: Option<fn($($t),*) $(->$r)?> = None;)* }
            $(pub unsafe extern "C" fn $n($($a:$t),*) $(->$r)? {
                match hook::$n { Some(f)=>f($($a),*), None=>panic!(concat!("TODO: ",stringify!($n)," is not ported")) }
            })*
            pub mod cb { pub use super::*; }
        }
    }
    hooks! {\n'''+ '\n'.join(ef)+'\n}\n'+'\n'.join(ed)+'\n}\nuse ext::*;\n'
    hdr='''//! Native Geno translation. Shared records and bitfields come from melee_types.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_imports, unused_mut, unused_variables, unused_assignments, unused_parens, unused_unsafe, static_mut_refs, clippy::all)]
use core::ffi::c_void;
pub use melee_types::*;
pub use melee_runtime::gekko::sqrtf;
macro_rules! OSReport { ($f:expr $(,$a:expr)* $(,)?) => {{
    let text=unsafe { core::ffi::CStr::from_ptr($f) }.to_string_lossy();
    eprintln!("{} {:?}",text, &[$(crate::tu::ext_hook::Val::to_u64($a)),*] as &[u64]);
}}; }
macro_rules! HSD_ForeachAnim { ($a:expr,$b:expr,$c:expr,$d:expr,$e:expr,$f:expr $(,)?) => {
    ext::HSD_ForeachAnim($a,$b,$c,$d,$e,crate::tu::ext_hook::Val::to_u64($f))
}; }
unsafe fn __assert(file:*const i8,line:u32,condition:*const i8)->! {
    panic!("{}:{}: {}",core::ffi::CStr::from_ptr(file).to_string_lossy(),line,core::ffi::CStr::from_ptr(condition).to_string_lossy())
}
'''
    out=ROOT/f'rust/melee_modes/src/tu/{stem}.rs'
    out.write_text(hdr+ext+body+'\n')
    missing=sorted(set(re.findall(r'\bC2RustUnnamed(?:_\d+)?\b',ty(body))))
    (ROOT/f'rust/target/{stem}_type_map.json').write_text(json.dumps(tm,indent=2))
    print(stem,len(functions),len(statics),'missing',missing)

if __name__=='__main__':
    for stem in sys.argv[1:]:main(stem)
