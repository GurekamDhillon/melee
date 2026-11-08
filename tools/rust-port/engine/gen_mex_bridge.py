#!/usr/bin/env python3
"""Generate rust/melee_engine/src/mex_bridge.rs: the vanilla functions m-ex guest code calls (mex_guest.rs).

A guest `bl 0x8xxxxxxx` leaves the blob; mex_guest names the target (mex_import::layout::vanilla_name) and calls
`call(name, c)`, whose arm converts the PowerPC arguments to the canonical port's parameter types: integers from
r3.. and floats from f1.. (the two files count apart, as the SysV PPC ABI does), pointers through the guest space
(`Call::ptr`: a registered native object, or a record the guest keeps in its own memory marshalled in and out), and
the result back into r3 / f1. `gobj_fn(name)` is every one-argument `HSD_GObj` callback as a native function pointer
(MoveLogic rows and fn-pointer fields that keep a retail function).

The names bridged are mex_bridge_names.txt (the targets the Akaneia fighters call; MELEE_MEX_REPORT=1 lists the
unresolved ones). No disc data: names and signatures only.

  gen_mex_bridge.py [-v]
"""
import os, re, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'link'))
import gen_link as gl

OUT = os.path.join(gl.RUST, 'melee_engine', 'src', 'mex_bridge.rs')
NAMES = os.path.join(os.path.dirname(__file__), 'mex_bridge_names.txt')
INTS = {'i8': 'i8', 'i16': 'i16', 'i32': 'i32', 'i64': 'i64', 'u8': 'u8', 'u16': 'u16', 'u32': 'u32', 'u64': 'u64',
        'isize': 'isize', 'usize': 'usize', 'c_int': 'i32', 'c_uint': 'u32', 'c_char': 'i8', 's32': 'i32', 's16': 'i16',
        's8': 'i8', 'BOOL': 'i32', 'bool': 'bool', 'enum_t': 'u32', 'c_long': 'i32', 'c_ulong': 'u32', 'c_uchar': 'u8'}
SKIP_TYPES = re.compile(r'^&|^\[|^Option<|^fn\(|^impl|dyn ')


def records():
    """GC record names the guest space can lay out (melee_types' GC_RECORDS rows)."""
    src = open(os.path.join(gl.RUST, 'melee_types', 'src', 'gen.rs'), encoding='utf8', errors='replace').read()
    i = src.find('pub static GC_RECORDS')
    return set(re.findall(r'\(<(\w+) as GcLayout>::GC_NAME', src[i:]))

def ret_type(crate, rel, name):
    src = open(os.path.join(gl.RUST, crate, 'src', rel), encoding='utf8', errors='replace').read()
    m = re.search(r'(?m)^[ \t]*pub\s+(?:unsafe\s+)?(?:extern\s+"C"\s+)?fn\s+' + re.escape(name) + r'\s*\(', src)
    if not m: return None
    cl = gl.match_close(gl.blank(src), m.end() - 1, '(', ')')
    after = src[cl + 1:src.find('{', cl)].split('where')[0]
    rm = re.match(r'\s*->\s*(.+?)\s*$', after, re.S)
    return rm.group(1).strip() if rm else ''


def strip_ty(ty):
    ty = ty.strip()
    ty = re.sub(r'^(melee_types|crate|super)::(\w+::)*', '', ty)
    return ty


def pointee(ty):
    m = re.match(r'^\*(?:mut|const)\s+(.+)$', ty)
    if not m: return None
    p = strip_ty(m.group(1)).split('::')[-1]
    return p


CRATES = ['melee_core', 'melee_runtime', 'melee_world', 'melee_fighter', 'melee_chara', 'melee_items']


def index_all(want):
    """name -> [(crate, modpath, nparams, hasret, rel)] over every non-raw source file of the game crates (the ports
    that live outside tu/: ftcommon.rs, ftlib.rs, gekko.rs ...), for the names in `want`."""
    out = {}
    for crate in CRATES:
        for f in sorted(gl.glob.glob(os.path.join(gl.RUST, crate, 'src', '**', '*.rs'), recursive=True)):
            rel = os.path.relpath(f, os.path.join(gl.RUST, crate, 'src')).replace('\\', '/')
            if rel.startswith('raw/') or 'test' in rel: continue
            raw = open(f, encoding='utf8', errors='replace').read()
            if not any(n in raw for n in want): continue
            t = gl.blank(raw)
            for m, chain in gl.nested_defs(t):
                if m.group(1) not in want: continue
                op = m.end() - 1
                cl = gl.match_close(t, op, '(', ')')
                if cl < 0: continue
                plist = t[op + 1:cl]
                c, mp = gl.modpath_of(f)
                mp = '::'.join([x for x in [mp] + chain if x])
                after = t[cl + 1:cl + 80].lstrip()
                out.setdefault(m.group(1), []).append((c, mp, gl.count_params(plist), after.startswith('->'), rel))
                gl.PLIST[(c, mp, m.group(1))] = plist
    return out


def hook_wrappers(want):
    """name -> (wrapper path, param list text, ret text) from melee_link's HOOKS table: a TU's `ext_fns!` wrapper
    (`<crate>::tu::<mod>::ext::NAME`, which calls what melee_link installed) for the names whose canonical port has
    another Rust name (ftCommon_Fall -> ftcommon::fall)."""
    t = open(os.path.join(gl.RUST, 'melee_link', 'src', 'gen_table.rs'), encoding='utf8', errors='replace').read()
    out = {}
    for m in re.finditer(r'\("(\w+)", "([\w:]+)::(?:hook::)?(\w+)", true, (?:true|false)\)', t):
        name, ext, h = m.group(1), m.group(2), m.group(3)
        if name not in want or name in out or h.lower().replace('_', '') != name.lower().replace('_', ''): continue
        ext = ext[:-6] if ext.endswith('::hook') else ext
        parts = ext.split('::')
        crate, mods = parts[0], parts[1:-1]
        if not mods: continue
        f = os.path.join(gl.RUST, crate, 'src', *mods[:-1], mods[-1] + '.rs')
        if not os.path.exists(f):
            f = os.path.join(gl.RUST, crate, 'src', *mods, 'mod.rs')
        if not os.path.exists(f): continue
        src = gl.blank(open(f, encoding='utf8', errors='replace').read())
        d = re.search(r'(?m)^\s*(?:fn\s+|\w+\s+)?' + name + r'\s*\(', src)
        if not d: continue
        cl = gl.match_close(src, d.end() - 1, '(', ')')
        plist = src[d.end():cl]
        ends = [e for e in (src.find('=', cl), src.find(';', cl), src.find('{', cl)) if e > 0]
        tail = src[cl + 1:min(ends)] if ends else ''
        rm = re.match(r'\s*->\s*(.+?)\s*$', tail, re.S)
        out[name] = (ext + '::' + name, plist, rm.group(1).strip() if rm else '')
    return out


def main():
    rows = [l.split('#')[0].strip() for l in open(NAMES, encoding='utf8').read().splitlines()]
    rows = [n for n in rows if n]
    # `name = crate::module` picks the port by hand where several definitions exist
    forced = {r.split('=')[0].strip(): r.split('=')[1].strip() for r in rows if '=' in r}
    names = [r.split('=')[0].strip() for r in rows]
    defs = gl.index_defs()
    extra = index_all(set(names))
    cmap = gl.cfile_map()
    recs = records()
    arms, gobj, skipped = [], [], []
    def pick_of(name):
        cands = defs.get(name, []) or extra.get(name, [])
        if name in forced:
            cands = [c for c in extra.get(name, []) + defs.get(name, []) if f'{c[0]}::{c[1]}' == forced[name]]
            return cands[0] if cands else None
        if len(set((c[0], c[1]) for c in cands)) == 1: return cands[0]
        return gl.choose(cands, name, cmap, None)
    wraps = hook_wrappers(set(names))
    for name in sorted(set(names)):
        pick = pick_of(name)
        if pick:
            crate, mp, _, _, rel = pick
            plist = gl.PLIST.get((crate, mp, name), '')
            path = f'{crate}::{mp}::{name}'
            rty = ret_type(crate, rel, name)
        elif name in wraps:
            path, plist, rty = wraps[name]
        else:
            skipped.append((name, 'no unique port')); continue
        params = [p for p in gl.split_top(plist, ',') if p.strip()]
        types = [strip_ty(p.split(':', 1)[1]) for p in params]
        # the TUs' local aliases: G (a gobj), F (a fighter: `type F = *mut Fighter`; read as a scalar alias it
        # passed the guest handle itself as the pointer)
        types = ['*mut HSD_GObj' if t in ('G', 'I') else '*mut Fighter' if t == 'F' else t for t in types]
        ni = nf = 0
        args, ok, pre = [], True, []
        for ty in types:
            base = ty.split('::')[-1]
            if base in INTS:
                if ni > 7: ok = False; break
                cv = INTS[base]
                args.append(f'(c.int({ni}) != 0)' if cv == 'bool' else f'(c.int({ni}) as _)'); ni += 1
            elif base in ('f32', 'f64'):
                if nf > 7: ok = False; break
                args.append(f'(c.flt({nf}) as {base})'); nf += 1
            elif ty.startswith('*'):
                if ni > 7: ok = False; break
                p = pointee(ty)
                rec = f'Some("{p}")' if p in recs else 'None'
                args.append(f'(c.ptr({ni}, {rec}) as _)'); ni += 1
            elif re.match(r'^&(mut\s+)?[\w:]+$', ty):
                # a reference: the same pointer, never null (a null one ends the callback instead of the call)
                if ni > 7: ok = False; break
                mut = 'mut ' if ty.startswith('&mut') else ''
                p = re.sub(r'^&(mut\s+)?', '', ty).split('::')[-1]
                rec = f'Some("{p}")' if p in recs else 'None'
                pre.append(f'let p{ni} = c.ptr({ni}, {rec}); if p{ni}.is_null() {{ c.bad = true; return true; }}')
                args.append(f'(&{mut}*(p{ni} as *{"mut" if mut else "const"} _))'); ni += 1
            elif base == 'HSD_GObjInteraction':
                # a two-gobj callback (fp->grabbed_cb): a guest address as its two-argument trampoline, a vanilla one as
                # the port's (Dedede's inhale passes its own to ftCommon_8007E2D0; read as a word it was None)
                if ni > 7: ok = False; break
                args.append(f'core::mem::transmute(c.func2({ni}))'); ni += 1
            elif base in ('HSD_GObjEvent', 'HSD_GObjFunc', 'GObj_RenderFunc', 'LineCb') or re.fullmatch(r'Option<\s*unsafe\s+extern\s+(?:"C"\s*)?fn\s*\(\s*(\*mut\s+(\w+::)*HSD_GObj|G)\s*\)\s*>', ty):
                # a gobj callback: a guest code address as its trampoline, a vanilla one as the port (a render or line callback
                # too: the trampoline passes the gobj, the guest sees 0 in r4)
                if ni > 7: ok = False; break
                args.append(f'core::mem::transmute(c.func({ni}))'); ni += 1
            elif re.fullmatch(r'[A-Z]\w*', base) and not SKIP_TYPES.search(ty):
                # a scalar alias (FighterKind, enum newtypes): its word
                if ni > 7: ok = False; break
                args.append(f'w2v(c.int({ni}))'); ni += 1
            else:
                ok = False; break
        if not ok:
            skipped.append((name, 'param ' + plist.replace('\n', ' '))); continue
        if rty is None:
            skipped.append((name, 'no source')); continue
        if rty == 'G': rty = '*mut HSD_GObj'
        rbase = strip_ty(rty).split('::')[-1]
        if rty in ('', '()'):
            ret = 'let _ = r;'
        elif rbase in INTS:
            ret = 'c.ret_int(r as i64 as u32);'
        elif rbase in ('f32', 'f64'):
            ret = 'c.ret_flt(r as f64);'
        elif strip_ty(rty).startswith('*'):
            p = pointee(strip_ty(rty))
            ret = f'c.ret_ptr(r as *mut core::ffi::c_void, {chr(34) + p + chr(34) if p in recs else None});'.replace('Some(None)', 'None') \
                if False else (f'c.ret_ptr(r as *mut core::ffi::c_void, Some("{p}"));' if p in recs else 'c.ret_ptr(r as *mut core::ffi::c_void, None);')
        elif re.fullmatch(r'[A-Z]\w*', rbase):
            ret = 'c.ret_int(v2w(r));'
        else:
            skipped.append((name, 'ret ' + rty)); continue
        arms.append(f'        "{name}" => {{ {" ".join(pre)} let r = {path}({", ".join(args)}); {ret} }}')
    # every void (HSD_GObj*) function of the ports, for MoveLogic rows / callbacks that keep a retail function
    gobj2 = []
    for name, cands in sorted(defs.items()):
        cands = [c for c in cands if c[2] == 2 and not c[3]]
        if not cands: continue
        pick = pick_of(name)
        if not pick or pick[2] != 2 or pick[3]: continue
        crate, mp, _, _, rel = pick
        plist = gl.PLIST.get((crate, mp, name), '')
        ps = [strip_ty(q.split(':', 1)[1]) for q in gl.split_top(plist, ',') if ':' in q]
        if len(ps) != 2 or not all(re.fullmatch(r'(\*mut\s+(\w+::)*HSD_GObj|G)', t) for t in ps): continue
        gobj2.append(f'        "{name}" => {{ unsafe extern "C" fn f(g: G, h: G) {{ {crate}::{mp}::{name}(g as _, h as _) }} f }}')
    seen = set()
    for name, cands in sorted(defs.items()):
        cands = [c for c in cands if c[2] == 1 and not c[3]]
        if not cands: continue
        pick = pick_of(name) if len(cands) != len(defs[name]) or True else None
        if not pick or pick[2] != 1 or pick[3]: continue
        crate, mp, _, _, rel = pick
        plist = gl.PLIST.get((crate, mp, name), '')
        ty = strip_ty(plist.split(':', 1)[1]) if ':' in plist else ''
        if not re.fullmatch(r'\*mut\s+(\w+::)*HSD_GObj', ty): continue
        if name in seen: continue
        seen.add(name)
        gobj.append(f'        "{name}" => {{ unsafe extern "C" fn f(g: G) {{ {crate}::{mp}::{name}(g as _) }} f }}')
    with open(OUT, 'w', newline='\n') as fh:
        fh.write('// @generated by tools/rust-port/engine/gen_mex_bridge.py. Do not edit.\n')
        fh.write('#![allow(non_snake_case, unused_unsafe, unused_variables, unused_parens, clippy::all, unreachable_code)]\n')
        fh.write('use crate::mex_guest::Call;\ntype G = *mut melee_types::HSD_GObj;\n\n')
        fh.write('#[inline(always)]\nfn w2v<T: Copy>(w: u32) -> T { if core::mem::size_of::<T>() > 4 { return unsafe { core::mem::zeroed() }; } unsafe { core::mem::transmute_copy(&w) } }\n')
        fh.write('#[inline(always)]\nfn v2w<T: Copy>(r: T) -> u32 { assert!(core::mem::size_of::<T>() <= 4); let mut w = 0u32; unsafe { core::ptr::copy_nonoverlapping(&r as *const T as *const u8, &mut w as *mut u32 as *mut u8, core::mem::size_of::<T>()) }; w }\n\n')
        fh.write('/// Call vanilla function `name` with the guest\'s arguments; false: not bridged.\n')
        fh.write('pub unsafe fn call(name: &str, c: &mut Call) -> bool {\n    match name {\n')
        fh.write('\n'.join(arms) + '\n')
        fh.write('        _ => return false,\n    }\n    true\n}\n\n')
        fh.write('/// Is `name` bridged?\n')
        fh.write('pub unsafe fn bridged(name: &str) -> bool {\n    matches!(name, ' + (' | '.join(chr(34) + a.split(chr(34))[1] + chr(34) for a in arms) or chr(34) + chr(34)) + ')\n}\n\n')
        fh.write('/// The native port of a one-argument gobj callback, by name.\n')
        fh.write('pub fn gobj_fn(name: &str) -> Option<unsafe extern "C" fn(G)> {\n    Some(match name {\n')
        fh.write('\n'.join(gobj) + '\n')
        fh.write('        _ => return None,\n    })\n}\n\n')
        fh.write('/// The native port of a two-gobj callback (`HSD_GObjInteraction`), by name.\n')
        fh.write('pub fn gobj2_fn(name: &str) -> Option<unsafe extern "C" fn(G, G)> {\n    Some(match name {\n')
        fh.write('\n'.join(gobj2) + '\n')
        fh.write('        _ => return None,\n    })\n}\n\n')
        # the same names as a list, for the reverse lookup (a native callback the guest reads back out of a field)
        fh.write('/// Every name `gobj_fn` answers.\n')
        fh.write('pub static GOBJ_FN_NAMES: &[&str] = &[\n')
        fh.write('\n'.join('    "' + g.split('"')[1] + '",' for g in gobj) + '\n];\n')
    if '-v' in sys.argv:
        for n, why in skipped: print(f'  left out {n}: {why}')
    print(f'{len(arms)} bridged, {len(gobj)} gobj callbacks, {len(skipped)} left out -> {OUT}')


if __name__ == '__main__':
    main()
