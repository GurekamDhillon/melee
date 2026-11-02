#!/usr/bin/env python3
"""Generate rust/melee_engine/src/ext_dispatch.rs: the by-name dispatcher behind the string-hook ext stubs.

Two stub forms in melee_modes call their callee by NAME with the arguments as 64-bit words (ints sign-extended, f32 as
its bits, pointers and fn pointers as addresses): `ext_fns!` (tu/ext_hook.rs: EXT_HOOK, then EXT_FALLBACK) and
`gmtou_ext!` (tu/gmtoulib.rs: hook::HOOK). gen_link.py lists them as `other` and cannot point them at a port; the menu
and result scenes (mnCharSel, mnStageSel, gmResult, the title) are mostly written that way. This table resolves each
such NAME to its canonical tu/ definition with gen_link's own index and `choose` rule (same arity and return-ness), and
converts the words to the definition's parameter types. Names with a by-value struct parameter are left out.

  gen_ext_dispatch.py          write the table
"""
import os, re, sys, collections
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'link'))
import gen_link as gl

OUT = os.path.join(gl.RUST, 'melee_engine', 'src', 'ext_dispatch.rs')
INTS = {'i8', 'i16', 'i32', 'i64', 'isize', 'u8', 'u16', 'u32', 'u64', 'usize', 'c_int', 'c_uint', 'c_char', 'c_long', 'c_ulong', 'c_uchar', 's32', 'u32', 's16', 's8', 'BOOL'}


ALIAS = {'s32': 'i32', 's16': 'i16', 's8': 'i8', 'BOOL': 'i32'}


def decls():
    """name -> (nparams, hasret, file) of string-hook ext declarations with no `= port`."""
    out = {}
    for crate, f, rel in gl.tu_files():
        if crate != 'melee_modes' or not rel.startswith('tu/') or 'test' in rel: continue
        raw = open(f, encoding='utf8', errors='replace').read()
        t = gl.blank(raw)
        for im in re.finditer(r'(?<![\w$])(ext_fns|oext_fns|gmtou_ext)!\s*\{', t):
            if t[max(0, im.start() - 10):im.start()].endswith('rules! '): continue
            pre = t[max(0, im.start() - 40):im.start()]
            if 'macro_rules' in pre: continue
            cl = gl.match_close(t, im.end() - 1)
            if cl < 0: continue
            body = t[im.end():cl]
            # local `macro_rules! ext_fns` (pub mod hook form) is linked by gen_link: only the crate macros route by name
            if im.group(1) == 'ext_fns' and re.search(r'macro_rules!\s+ext_fns', t): continue
            for d in re.finditer(r'(?:extern\s+fn|fn)\s+(\w+)\s*\(', body):
                op = im.end() + d.end() - 1
                pc = gl.match_close(t, op, '(', ')')
                semi = t.find(';', pc)
                tail = t[pc + 1:semi]
                if '=' in tail and not '->' in tail.split('=')[0] + '': pass
                if re.search(r'=\s*[\w:]', tail): continue
                has = bool(re.match(r'\s*->\s*(?!\(\s*\))', tail)) and not re.match(r'\s*->\s*!', tail)
                if '&[' in t[op + 1:pc]: continue
                out.setdefault(d.group(1), (gl.count_params(t[op + 1:pc]), has, rel))
        # the gen_tuo.py `hook!(H_NAME, NAME, (params) -> R)` stubs (gmopening.rs ...): their unset arm calls ext_hook::fallback
        if 'ext_hook::fallback' in raw:
            for d in re.finditer(r'(?<![\w$])hook!\(\s*H_\w+\s*,\s*(\w+)\s*,\s*\(', t):
                op = d.end() - 1
                pc = gl.match_close(t, op, '(', ')')
                tail = t[pc + 1:t.find(';', pc)]
                if re.search(r',\s*[\w:]+\s*\)\s*$', tail): continue   # has a port path
                has = bool(re.match(r'\s*->', tail))
                if '&[' in t[op + 1:pc]: continue   # a C `...` as a slice: seams.rs spells those by hand
                out.setdefault(d.group(1), (gl.count_params(t[op + 1:pc]), has, rel))
    for name, (npar, has, _ty) in tu_ext_hooks().items():
        out.setdefault(name, (npar, has, 'tu_ext.rs'))
    return out


TU_EXT = os.path.join(gl.RUST, 'melee_modes', 'src', 'tu_ext.rs')


def tu_ext_hooks():
    """name -> (nparams, hasret, (param types, ret type)) of melee_modes/src/tu_ext.rs `HOOK_<name>` statics whose unset
    arm stops on "not ported" (the shard 529-536 gm TUs: the title demo and the VS scene's setup reach them)."""
    out = {}
    t = open(TU_EXT, encoding='utf8').read()
    for m in re.finditer(r'pub static mut HOOK_(\w+): Option<fn\((.*?)\)(?:\s*->\s*(.+?))?> = None;\n(.*?\n.*?\n.*?\n)', t):
        name, plist, rty, body = m.group(1), m.group(2), (m.group(3) or '').strip(), m.group(4)
        if 'unported!' not in body: continue
        types = [x.strip() for x in gl.split_top(plist, ',') if x.strip()]
        out[name] = (len(types), bool(rty), (types, rty))
    return out


def to_word(ty, v):
    """A tu_ext hook argument as its 64-bit word (the inverse of conv_arg)."""
    base = ty.split('::')[-1]
    if base in INTS or ty in INTS: return f'{v} as i64 as u64'
    if ty == 'bool': return f'{v} as u64'
    if ty == 'f32': return f'{v}.to_bits() as u64'
    if ty == 'f64': return f'{v}.to_bits()'
    return f'p2w({v})'


def tu_ext_installer():
    """`install_tu_ext`: every unported tu_ext hook routed by name through the engine's ext path (seams::ext_real:
    the hand-written callees, this table's canonical ports, the host seams). A name none of them resolves still stops."""
    rows = []
    for name, (_, has, (types, rty)) in sorted(tu_ext_hooks().items()):
        ps = ', '.join(f'a{i}: {ty}' for i, ty in enumerate(types))
        ws = ', '.join(to_word(ty, f'a{i}') for i, ty in enumerate(types))
        call = f'crate::seams::ext_real("{name}", &[{ws}]).unwrap_or_else(|| melee_core::unported!("TODO: {name} is not ported"))'
        if not has:
            body = f'{{ {call}; }}'
        else:
            body = '{ let w = ' + call + '; ' + conv_arg(rty, 'w') + ' }'
        rows.append(f'        m::HOOK_{name} = Some(|{ps}| unsafe {body});')
    return rows


def conv_arg(ty, w):
    ty = ty.strip()
    base = ty.split('::')[-1]
    if base in INTS or ty in INTS: return f'{w} as {ALIAS.get(base, ty)}'
    if ty == 'bool': return f'{w} != 0'
    if ty == 'f32': return f'f32::from_bits({w} as u32)'
    if ty == 'f64': return f'f64::from_bits({w})'
    if ty.startswith('*') or ty.startswith('&') or ty.startswith('Option<') or ty.startswith('fn(') or ty.startswith('unsafe extern') \
            or re.match(r'^[A-Z]\w*(Func|Event|Callback|Fn|Cb|Proc)$', base):
        return f'w2p({w})'
    if re.fullmatch(r'[A-Za-z_][\w:]*', ty) and len(ty) > 1:
        return f'w2v({w})'   # a scalar alias (CharacterKind, Gm_PKind, bool_0...): its low bytes; by-value records fail the size check
    return None


def conv_ret(ty):
    ty = ty.strip()
    base = ty.split('::')[-1]
    if ty in ('', '()'): return '{ r; 0 }'
    if base in INTS or ty in INTS: return 'r as i64 as u64'
    if ty == 'bool': return 'r as u64'
    if ty == 'f32': return 'r.to_bits() as u64'
    if ty == 'f64': return 'r.to_bits()'
    if ty.startswith('*') or ty.startswith('&') or ty.startswith('Option<') or re.match(r'^[A-Z]\w*(Func|Event|Callback|Fn|Cb|Proc)$', base):
        return 'p2w(r)'
    if re.fullmatch(r'[A-Za-z_][\w:]*', ty):
        return 'v2w(r)'
    return None


def seam_rows():
    """seams.txt: (regex, off value or None). A name that is a host seam keeps its seam (the "feature off" body, or what
    the audio / render drivers install); the dispatcher never calls the port for it."""
    rows = []
    for ln in open(os.path.join(os.path.dirname(__file__), 'seams.txt'), encoding='utf8').read().splitlines():
        if not ln.strip() or ln.startswith('#'): continue
        f = ln.split('	')
        rows.append((re.compile(f[0].lstrip('!')), f[1].strip() if len(f) > 1 else None))
    return rows


def main():
    seams = seam_rows()
    defs = gl.index_defs()
    cmap = gl.cfile_map()
    arms, skipped, seam_arms = [], [], []
    for name, (npar, has, own) in sorted(decls().items()):
        hit = next((v for rx, v in seams if rx.fullmatch(name)), False)
        if hit is not False:
            off = hit if hit and re.fullmatch(r'-?\d+', hit) else '0'
            seam_arms.append(f'        "{name}" => {off}i64 as u64,')
            continue
        cands = [c for c in defs.get(name, []) if c[2] == npar]
        pick = gl.choose(cands, name, cmap, own)
        if not pick:
            # a `pub use` re-export and its original: the original (only real definitions have a parameter list)
            real = [c for c in cands if c[4] != own and (c[0], c[1], name) in gl.PLIST]
            if len(real) == 1: pick = real[0]
        if not pick:
            skipped.append((name, "no unique port")); continue
        crate, mp, _, _, rel = pick
        if crate == 'melee_modes' and mp.split('::')[-1] in ('gmtoulib',) and False: pass
        plist = gl.PLIST.get((crate, mp, name), '')
        params = [p for p in gl.split_top(plist, ',') if p.strip()]
        types = [p.split(':', 1)[1] for p in params]
        args = [conv_arg(ty, f'a[{i}]') for i, ty in enumerate(types)]
        if any(x is None for x in args):
            skipped.append((name, "param " + plist)); continue
        # the return type, from the original text (blank() keeps offsets)
        src = open(os.path.join(gl.RUST, crate, 'src', rel), encoding='utf8', errors='replace').read()
        m = re.search(r'(?m)^[ \t]*pub\s+(?:unsafe\s+)?(?:extern\s+"C"\s+)?fn\s+' + re.escape(name) + r'\s*\(', src)
        rty = ''
        if m:
            cl = gl.match_close(gl.blank(src), m.end() - 1, '(', ')')
            after = src[cl + 1:src.find('{', cl)]
            after = after.split('where')[0]
            rm = re.match(r'\s*->\s*(.+?)\s*$', after, re.S)
            rty = rm.group(1) if rm else ''
        ret = conv_ret(rty)
        if ret is None:
            skipped.append((name, "ret " + rty)); continue
        path = f'{crate}::{mp}::{name}'
        arms.append(f'        "{name}" => {{ let r = {path}({", ".join(args)}); {ret} }}')
    with open(OUT, 'w', newline='\n') as fh:
        fh.write('// @generated by tools/rust-port/engine/gen_ext_dispatch.py. Do not edit.\n')
        fh.write('#![allow(non_snake_case, unused_unsafe, unused_variables, clippy::all, unreachable_code, unused_braces, path_statements)]\n\n')
        fh.write('#[inline(always)]\nfn w2p<T>(w: u64) -> T { assert!(core::mem::size_of::<T>() == 8); unsafe { core::mem::transmute_copy(&(w as usize)) } }\n')
        fh.write('''#[inline(always)]\nfn w2v<T: Copy>(w: u64) -> T { assert!(core::mem::size_of::<T>() <= 8); unsafe { core::mem::transmute_copy(&w) } }
#[inline(always)]
fn v2w<T: Copy>(r: T) -> u64 { assert!(core::mem::size_of::<T>() <= 8); let mut w = 0u64; unsafe { core::ptr::copy_nonoverlapping(&r as *const T as *const u8, &mut w as *mut u64 as *mut u8, core::mem::size_of::<T>()) }; w }
#[inline(always)]
fn p2w<T>(r: T) -> u64 { assert!(core::mem::size_of::<T>() == 8); let w: usize = unsafe { core::mem::transmute_copy(&r) }; core::mem::forget(r); w as u64 }\n

''')
        fh.write('/// The canonical port of a string-hook ext callee, called with its arguments as words. None: not resolved here.\n')
        fh.write('pub unsafe fn dispatch(name: &str, a: &[u64]) -> Option<u64> {\n    Some(match name {\n')
        fh.write('\n'.join(arms) + '\n')
        fh.write('        _ => return None,\n    })\n}\n\n')
        fh.write('/// The host-seam callees of the string-hook stubs (seams.txt): the "feature off" value.\n')
        fh.write('pub fn seam_off(name: &str) -> Option<u64> {\n    Some(match name {\n')
        fh.write('\n'.join(seam_arms) + '\n')
        fh.write('        _ => return None,\n    })\n}\n')
        fh.write('\n/// Points every unported `melee_modes::tu_ext` hook at the by-name ext path (see `seams::ext_real`).\n')
        fh.write('pub unsafe fn install_tu_ext() {\n    use melee_modes::tu_ext as m;\n    use melee_types::*;\n')
        fh.write('\n'.join(tu_ext_installer()) + '\n}\n')
    if '-v' in sys.argv:
        for n, why in skipped: print(f'  left out {n}: {why}')
    print(f'{len(arms)} resolved, {len(seam_arms)} seams, {len(skipped)} left out -> {OUT}')


if __name__ == '__main__':
    main()
