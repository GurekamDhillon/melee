"""Apply the audited layout/math adaptations to the Geno reference translation."""
import re
from pathlib import Path
from geno_adapt import ROOT, end

def replace_ws(s,a,b,count=1):
    pat=r'\s*'.join(re.escape(x) for x in re.split(r'\s+',a.strip()))
    s,n=re.subn(pat,lambda m:b,s,count=count)
    if n!=count:raise ValueError((a,n,count))
    return s

for stem in ('geno_game','geno_tests'):
    path=ROOT/f'rust/melee_modes/src/tu/{stem}.rs'
    s=path.read_text()
    s=s.replace('AObjArgType','AObj_Arg_Type')
    s=s.replace('fn Geno_DefineVoiceState(a0: u32,','fn Geno_DefineVoiceState(a0: i32,')
    s=s.replace('fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: i32,','fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32,')
    # Repeated block externs are an old c2rust name-resolution defect. They are
    # ordinary calls/data imports and must never accidentally link back to C.
    extadd=[];dataadd=[]
    for m in list(re.finditer(r'    extern "C" \{',s))[::-1]:
        z=end(s,m.end()-1);block=s[m.end():z-1]
        aliases=dict(re.findall(r'#\[link_name = "(\w+)"\]\s*(?:fn|static mut) (\w+)',block))
        for orig,alias in aliases.items():s=re.sub(r'\b'+alias+r'\b',orig,s)
        block=re.sub(r'#\[link_name = "\w+"\]\s*','',block)
        for orig,alias in aliases.items():block=re.sub(r'\b'+alias+r'\b',orig,block)
        for fm in re.finditer(r'\bfn (\w+)\(',block):
            fz=end(block,fm.end()-1,'(',')');fz=block.index(';',fz)+1
            if not re.search(r'(?m)^fn '+fm[1]+r'\(',s):extadd.append(block[fm.start():fz])
        for dm in re.finditer(r'static mut (\w+): ([\s\S]*?);(?=\n|$)',block):
            if not re.search(r'(?m)^pub static mut '+dm[1]+r':',s):
                dataadd.append(f'pub static mut {dm[1]}: {dm[2]} = unsafe {{ core::mem::zeroed() }};')
        # alias replacement changes lengths; find this declaration again.
        m2=list(re.finditer(r'    extern "C" \{',s))[-1];z2=end(s,m2.end()-1)
        s=s[:m2.start()]+s[z2:]
    if extadd:s=s.replace('    hooks! {','    hooks! {\n'+'\n'.join(extadd),1)
    if dataadd:s=s.replace('}\nuse ext::*;', '\n'+'\n'.join(dataadd)+'\n}\nuse ext::*;',1)
    # C static initializers contain only addresses, constants and native sizeof/offsets.
    # Rust can evaluate these statically; no platform constructor list is required.
    m=re.search(r'pub unsafe extern "C" fn run_static_initializers\(\) \{',s)
    if m:
        z=end(s,m.end()-1);init=s[m.end():z-1];s=s[:m.start()]+s[z:]
        for a in re.finditer(r'(?m)^    (\w+) = ',init):
            p=a.end();i=p;depth=[];quote=False;esc=False
            while i<len(init):
                c=init[i]
                if quote:
                    if esc:esc=False
                    elif c=='\\':esc=True
                    elif c=='"':quote=False
                elif c=='"':quote=True
                elif c in '[{(':depth.append(c)
                elif c in ']})':depth.pop()
                elif c==';' and not depth:break
                i+=1
            name=a[1];v=init[p:i]
            sm=re.search(r'pub static mut '+name+r':[^=]+ = ',s)
            if not sm:raise ValueError(name)
            p2=sm.end(); i2=p2;depth=[];quote=False;esc=False
            while i2<len(s):
                c=s[i2]
                if quote:
                    if esc:esc=False
                    elif c=='\\':esc=True
                    elif c=='"':quote=False
                elif c=='"':quote=True
                elif c in '[{(':depth.append(c)
                elif c in ']})':depth.pop()
                elif c==';' and not depth:break
                i2+=1
            s=s[:p2]+'unsafe { '+v+' }'+s[i2:]
    s=s.replace('use core::ffi::c_void;','use core::ffi::c_void;\nuse melee_core::fp::fmadds;')
    if stem=='geno_game':
        # Exact contraction sites from the shipping PPC clang IR (fma_sites.py).
        for a,b,n in [
            ('sx * sx + sy * sy','fmadds(sx, sx, sy * sy)',2),
            ('dx * dx + dy * dy','fmadds(dx, dx, dy * dy)',5),
            ('*vx * *vx + *vy * *vy','fmadds(*vx, *vx, *vy * *vy)',3),
            ('vx * vx + vy * vy','fmadds(vx, vx, vy * vy)',1),
            ('sx as f32 * sx as f32 + sy as f32 * sy as f32','fmadds(sx, sx, sy * sy)',1),
            ('vx as f32 * vx as f32 + vy as f32 * vy as f32','fmadds(vx, vx, vy * vy)',1),
            ('(*st).motion_vy -= ((*fp).co_attrs.gravity as f32 * g) as f32;',
             '(*st).motion_vy = fmadds(-(*fp).co_attrs.gravity, g, (*st).motion_vy);',1),
            ('let mut a: f32 = (*st).move_f[0 as i32 as usize] + sy * geno_param( st, GENO_P_DRILL_W0 as i32 + 3 as i32, );',
             'let mut a = fmadds(sy, geno_param(st, GENO_P_DRILL_W0 as i32 + 3), (*st).move_f[0]);',1),
            ('pos.x = (*base).x + facing * fwd;','pos.x = fmadds(facing, fwd, (*base).x);',1),
            ('(*v).vy = vf * sn + vu * c;','(*v).vy = fmadds(vf, sn, vu * c);',1),
            ('(*fp).facing_dir as f32 * (*box_0).x + (*fp).cur_pos.x','fmadds((*fp).facing_dir, (*box_0).x, (*fp).cur_pos.x)',1),
            ('return (ix - x0) * (ix - x0) + (iy - y0) * (iy - y0);','return fmadds(ix - x0, ix - x0, (iy - y0) * (iy - y0));',1),
            ('sdx += ((*pp).pos.x - (*ip).pos.x) * (*ip).facing_dir;','sdx = fmadds((*pp).pos.x - (*ip).pos.x, (*ip).facing_dir, sdx);',1),
            ('svx += (*pp).vel.x * (*ip).facing_dir;','svx = fmadds((*pp).vel.x, (*ip).facing_dir, svx);',1),
        ]:
            s=replace_ws(s,a,b,n)
    path.write_text(s)
