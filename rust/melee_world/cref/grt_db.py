"""Callee database for gen_grt_stages.py: the externals of the gr/grt*.c per-stage TUs.

Each entry carries the Rust `ext` signature, the C recorder stub (oracle side) and the Rust
recorder hook (oracle side). Both recorders append the same text to the case trace and draw
their return values from the same seeded xorshift32, so a divergence of the port from the C
shows up as a different trace.
"""

# home: 'inl' = lives in gr/inlines.rs ext (callees of the gr/inlines.h inlines + shared),
#       'tu'  = lives in each TU's own ext.
# rp: rust params "name: type, ..." ; rr: rust return ("" or "-> T")
# c: C stub definition; r: Rust hook body (a block using the param names)

CALLEES = []


def callee(name, home, rp, rr, tag, c, r):
    CALLEES.append(dict(name=name, home=home, rp=rp, rr=rr, tag=tag, c=c, r=r))


# ---- inlines.h callees -------------------------------------------------------------------
callee("grAnime_801C8138", "inl", "g: *mut HSD_GObj, m: i32, b: i32", "", "group2: gr/granime.c",
       'void grAnime_801C8138(HSD_GObj* g, enum_t m, bool b) { emit(" anim(%s,%d,%d)", sym(g), m, b); }',
       'rec(&format!(" anim({},{},{})", sym(g as usize), m, b));')
callee("Ground_InitMapColl", "inl", "j: *mut HSD_JObj, m: i32", "-> i32", "group2: gr/ground.c",
       'bool Ground_InitMapColl(HSD_JObj* j, s32 m) { emit(" imc(%s,%d)", sym(j), m); return 1; }',
       'rec(&format!(" imc({},{})", sym(j as usize), m)); 1')
callee("Ground_UpdateMapColl", "inl", "g: *mut HSD_GObj", "-> i32", "group2: gr/ground.c",
       'bool Ground_UpdateMapColl(Ground_GObj* g) { emit(" umc(%s)", sym(g)); return 1; }',
       'rec(&format!(" umc({})", sym(g as usize))); 1')
callee("lb_800115F4", "inl", "", "", "group1: lb/lb_00F9.c",
       'void lb_800115F4(void) { emit(" lb()"); }',
       'rec(" lb()");')
callee("GObj_SetupGXLink", "inl", "g: *mut HSD_GObj, cb: GObj_RenderFunc, gx: u8, pri: u32", "", "group1: baselib/gobjgxlink.c",
       'void GObj_SetupGXLink(HSD_GObj* g, GObj_RenderFunc cb, u8 gx, u32 pri) { emit(" gxlink(%s,%s,%d,%u)", sym(g), fname((void*) cb), gx, pri); }',
       'rec(&format!(" gxlink({},{},{},{})", sym(g as usize), fname(cb.map_or(0, |f| f as usize)), gx, pri));')
callee("Mex_GrBind", "inl", "f: *mut c_void", "-> *mut c_void", "group2: pc/mex Mex_GrBind",
       'void* Mex_GrBind(void* f) { emit(" mex(%s)", fname(f)); return f; }',
       'rec(&format!(" mex({})", fname(f as usize))); f')
callee("HSD_GObj_SetupProc", "inl", "g: *mut HSD_GObj, f: HSD_GObjEvent, pri: u8", "", "group1: baselib/gobjproc.c",
       'HSD_GObjProc* HSD_GObj_SetupProc(HSD_GObj* g, HSD_GObjEvent f, u8 pri) { emit(" proc(%s,%s,%d)", sym(g), fname((void*) f), pri); return 0; }',
       'rec(&format!(" proc({},{},{})", sym(g as usize), fname(f.map_or(0, |f| f as usize)), pri));')
callee("stage_info", "inl", "", "-> *mut StageInfo", "group2: gr/stage.c global stage_info",
       '', 'W().stage_info')
callee("Ground_801C39C0", "inl", "", "", "group2: gr/ground.c",
       'void Ground_801C39C0(void) { emit(" g39C0()"); }', 'rec(" g39C0()");')
callee("Ground_801C3BB4", "inl", "", "", "group2: gr/ground.c",
       'void Ground_801C3BB4(void) { emit(" g3BB4()"); }', 'rec(" g3BB4()");')
callee("Ground_801C4210", "inl", "", "", "group2: gr/ground.c",
       'void Ground_801C4210(void) { emit(" g4210()"); }', 'rec(" g4210()");')
callee("Ground_801C42AC", "inl", "", "", "group2: gr/ground.c",
       'void Ground_801C42AC(void) { emit(" g42AC()"); }', 'rec(" g42AC()");')

# ---- TU-specific callees -----------------------------------------------------------------
callee("OSReport", "tu", "fmt: &CStr, file: &CStr, line: i32, id: i32", "", "os: OSReport (varargs: fmt, __FILE__, line, id)",
       'void OSReport(const char* fmt, ...) { va_list ap; const char* f; int l; int id; if (strncmp(fmt, "grtrace", 7) == 0) { emit(" OSReport(%s)", fmt); return; } va_start(ap, fmt); f = va_arg(ap, const char*); l = va_arg(ap, int); id = va_arg(ap, int); va_end(ap); emit(" OSReport(%s,%s,%d,%d)", fmt, f, l, id); }',
       'rec(&format!(" OSReport({},{},{},{})", fmt.to_str().unwrap(), file.to_str().unwrap(), line, id));')
callee("Ground_GetStageGObj", "tu", "id: i32", "-> *mut HSD_GObj", "group2: gr/ground.c",
       'Ground_GObj* Ground_GetStageGObj(int id) { unsigned r = rnd(); HSD_GObj* g = ((r & 3) == 0 && !nonull) ? NULL : &fx_G[(r >> 2) % 3]; emit(" GetStageGObj(%d)=%s", id, sym(g)); return g; }',
       'let r = rnd(); let g = if r & 3 == 0 && !NONULL.load(std::sync::atomic::Ordering::Relaxed) { null_mut() } else { W().g.add(((r >> 2) % 3) as usize) }; rec(&format!(" GetStageGObj({})={}", id, sym(g as usize))); g')
callee("Ground_GetYakumonoParam", "tu", "", "-> *mut c_void", "group2: gr/ground.c",
       'void* Ground_GetYakumonoParam(void) { emit(" GetYakumonoParam()=%s", sym(fx_yak)); return fx_yak; }',
       'rec(&format!(" GetYakumonoParam()={}", sym(W().yak as usize))); W().yak.cast()')
callee("grZakoGenerator_801CAE04", "tu", "d: *mut c_void", "", "group2: gr/grzakogenerator.c",
       'HSD_GObj* grZakoGenerator_801CAE04(void* d) { emit(" zako(%s)", sym(d)); return NULL; }',
       'rec(&format!(" zako({})", sym(d as usize)));')
callee("mpJointFromLine", "tu", "line: i32", "-> i32", "group2: mp/mplib.c",
       'int mpJointFromLine(int line) { static const int t[] = { -1, 0, 1, 2, 0 }; int r = t[rnd() % 5]; emit(" mpJointFromLine(%d)=%d", line, r); return r; }',
       'const T: [i32; 5] = [-1, 0, 1, 2, 0]; let r = T[(rnd() % 5) as usize]; rec(&format!(" mpJointFromLine({})={}", line, r)); r')
callee("mpLineGetKind", "tu", "line: i32", "-> i32", "group2: mp/mplib.c",
       'enum_t mpLineGetKind(int line) { static const int t[] = { 1, 2, 4, 8, 0, 16 }; int r = t[rnd() % 6]; emit(" mpLineGetKind(%d)=%d", line, r); return r; }',
       'const T: [i32; 6] = [1, 2, 4, 8, 0, 16]; let r = T[(rnd() % 6) as usize]; rec(&format!(" mpLineGetKind({})={}", line, r)); r')
callee("HSD_PadMasterStatus", "tu", "", "-> *mut HSD_PadStatus", "group1: baselib/controller.c global HSD_PadMasterStatus[4]",
       '', 'W().pad')
callee("Ground_801C3FA4", "tu", "g: *mut HSD_GObj, depth: i32", "-> *mut HSD_JObj", "group2: gr/ground.c",
       'HSD_JObj* Ground_801C3FA4(HSD_GObj* g, int depth) { unsigned r = rnd(); HSD_JObj* j = r % 3 == 0 ? NULL : &fx_J[(r >> 3) % 3]; emit(" g3FA4(%s,%d)=%s", sym(g), depth, sym(j)); return j; }',
       'let r = rnd(); let j = if r % 3 == 0 { null_mut() } else { W().j.add(((r >> 3) % 3) as usize) }; rec(&format!(" g3FA4({},{})={}", sym(g as usize), depth, sym(j as usize))); j')
callee("HSD_JObjGetFlags", "tu", "j: *mut HSD_JObj", "-> u32", "group1: baselib/jobj.c",
       'u32 HSD_JObjGetFlags(HSD_JObj* j) { u32 f = j ? j->flags : 0; emit(" getflags(%s)=%x", sym(j), f); return f; }',
       'let f = if j.is_null() { 0 } else { (*j).flags }; rec(&format!(" getflags({})={:x}", sym(j as usize), f)); f')
callee("HSD_JObjSetFlags", "tu", "j: *mut HSD_JObj, f: u32", "", "group1: baselib/jobj.c",
       'void HSD_JObjSetFlags(HSD_JObj* j, u32 f) { if (j) j->flags |= f; emit(" setflags(%s,%x)", sym(j), f); }',
       'if !j.is_null() { (*j).flags |= f; } rec(&format!(" setflags({},{:x})", sym(j as usize), f));')
callee("HSD_JObjClearFlags", "tu", "j: *mut HSD_JObj, f: u32", "", "group1: baselib/jobj.c",
       'void HSD_JObjClearFlags(HSD_JObj* j, u32 f) { if (j) j->flags &= ~f; emit(" clearflags(%s,%x)", sym(j), f); }',
       'if !j.is_null() { (*j).flags &= !f; } rec(&format!(" clearflags({},{:x})", sym(j as usize), f));')
callee("HSD_JObjSetFlagsAll", "tu", "j: *mut HSD_JObj, f: u32", "", "group1: baselib/jobj.c",
       'void HSD_JObjSetFlagsAll(HSD_JObj* j, u32 f) { if (j) j->flags |= f; emit(" setall(%s,%x)", sym(j), f); }',
       'if !j.is_null() { (*j).flags |= f; } rec(&format!(" setall({},{:x})", sym(j as usize), f));')
callee("HSD_JObjSetMtxDirtySub", "tu", "j: *mut HSD_JObj", "", "group1: baselib/jobj.c",
       'void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { j->flags |= JOBJ_MTX_DIRTY; emit(" dirty(%s)", sym(j)); }',
       '(*j).flags |= JOBJ_MTX_DIRTY as u32; rec(&format!(" dirty({})", sym(j as usize)));')
callee("Ground_801C33C0", "tu", "a: i32, b: i32", "-> i32", "group2: gr/ground.c",
       's32 Ground_801C33C0(s32 a, s32 b) { static const int t[] = { -1, 0, 3, 7, 12 }; int r = t[rnd() % 5]; emit(" g33C0(%d,%d)=%d", a, b, r); return r; }',
       'const T: [i32; 5] = [-1, 0, 3, 7, 12]; let r = T[(rnd() % 5) as usize]; rec(&format!(" g33C0({},{})={}", a, b, r)); r')
callee("grMaterial_801C8CFC", "tu",
       "a: i32, b: i32, gp: *mut Ground, j: *mut HSD_JObj, cb4: *const c_void, cb5: Option<unsafe extern \"C\" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)>, cb6: *const c_void",
       "-> *mut HSD_GObj", "group2: gr/grmaterial.c",
       'Item_GObj* grMaterial_801C8CFC(int a, int b, Ground* gp, HSD_JObj* j, void* cb4, void (*cb5)(Item_GObj*, Ground*, Vec3*, HSD_GObj*, f32), void* cb6) { unsigned r = rnd(); Item_GObj* it = r % 3 == 0 ? NULL : &fx_IG[(r >> 2) % 2]; emit(" mat8CFC(%d,%d,%s,%s,%s,%s,%s)=%s", a, b, sym(gp), sym(j), fname(cb4), fname((void*) cb5), fname(cb6), sym(it)); return it; }',
       'let r = rnd(); let it = if r % 3 == 0 { null_mut() } else { W().ig.add(((r >> 2) % 2) as usize) }; rec(&format!(" mat8CFC({},{},{},{},{},{},{})={}", a, b, sym(gp as usize), sym(j as usize), fname(cb4 as usize), fname(cb5.map_or(0, |f| f as usize)), fname(cb6 as usize), sym(it as usize))); it')
callee("grMaterial_801C8DE0", "tu", "g: *mut HSD_GObj, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32, h: f32", "", "group2: gr/grmaterial.c",
       'void grMaterial_801C8DE0(Item_GObj* g, float a, float b, float c, float d, float e, float f, float h) { emit(" mat8DE0(%s,%08x,%08x,%08x,%08x,%08x,%08x,%08x)", sym(g), fb(a), fb(b), fb(c), fb(d), fb(e), fb(f), fb(h)); }',
       'rec(&format!(" mat8DE0({},{:08x},{:08x},{:08x},{:08x},{:08x},{:08x},{:08x})", sym(g as usize), a.to_bits(), b.to_bits(), c.to_bits(), d.to_bits(), e.to_bits(), f.to_bits(), h.to_bits()));')
callee("grMaterial_801C8E08", "tu", "g: *mut HSD_GObj", "", "group2: gr/grmaterial.c",
       'void grMaterial_801C8E08(Item_GObj* g) { emit(" mat8E08(%s)", sym(g)); }',
       'rec(&format!(" mat8E08({})", sym(g as usize)));')
callee("grMaterial_801C8CDC", "tu", "g: *mut HSD_GObj", "", "group2: gr/grmaterial.c",
       'void grMaterial_801C8CDC(HSD_GObj* g) { emit(" mat8CDC(%s)", sym(g)); }',
       'rec(&format!(" mat8CDC({})", sym(g as usize)));')
callee("Ground_801C53EC", "tu", "a: u32", "", "group2: gr/ground.c",
       'void Ground_801C53EC(u32 a) { emit(" g53EC(%u)", a); }',
       'rec(&format!(" g53EC({})", a));')
callee("Camera_RequestQuake", "tu", "kind: i32, pos: *mut Vec3", "", "group2: cm/camera.c",
       'void Camera_RequestQuake(int kind, Vec3* p) { emit(" quake(%d,%s)", kind, sym(p)); }',
       'rec(&format!(" quake({},{})", kind, sym(pos as usize)));')
callee("efSync_Spawn", "tu", "id: i32, g: *mut HSD_GObj, pos: *mut Vec3", "", "group2: ef/efsync.c (varargs: Vec3*)",
       'void* efSync_Spawn(s32 id, HSD_GObj* g, ...) { va_list ap; Vec3* p; va_start(ap, g); p = va_arg(ap, Vec3*); va_end(ap); emit(" efSync(%d,%s,%08x,%08x,%08x)", id, sym(g), fb(p->x), fb(p->y), fb(p->z)); return NULL; }',
       'rec(&format!(" efSync({},{},{:08x},{:08x},{:08x})", id, sym(g as usize), (*pos).x.to_bits(), (*pos).y.to_bits(), (*pos).z.to_bits()));')
import os as _os
_C = _os.environ.get("GRT_SET", "a") == "c"  # set c's purin loop needs x on both sides of 130
_OFFC = 'rf() + 100.0f;' if _C else 'rf();'
_OFFR = 'rf() + 100.0f32;' if _C else 'rf();'
callee("lb_8000B1CC", "tu", "j: *mut HSD_JObj, a: *const Vec3, out: *mut Vec3", "", "group1: lb/lb_00B0.c",
       'void lb_8000B1CC(HSD_JObj* j, Vec3* a, Vec3* out) { out->x = ' + _OFFC + ' out->y = rf(); out->z = rf(); emit(" lb1CC(%s,%s)", sym(j), sym(a)); }',
       '(*out).x = ' + _OFFR + ' (*out).y = rf(); (*out).z = rf(); rec(&format!(" lb1CC({},{})", sym(j as usize), sym(a as usize)));')
callee("grAnime_801C7FF8", "tu", "g: *mut HSD_GObj, a: i32, b: i32, c: i32, d: f32, e: f32", "", "group2: gr/granime.c",
       'void grAnime_801C7FF8(HSD_GObj* g, int a, int b, int c, float d, float e) { emit(" anim7FF8(%s,%d,%d,%d,%08x,%08x)", sym(g), a, b, c, fb(d), fb(e)); }',
       'rec(&format!(" anim7FF8({},{},{},{},{:08x},{:08x})", sym(g as usize), a, b, c, d.to_bits(), e.to_bits()));')

# ---- grtmario.c -------------------------------------------------------------------------
callee("ftCo_800C07F8", "tu", "g: *mut HSD_GObj, k: u32, f: *const c_void", "", "group3: ft/ftdevice.c",
       'void ftCo_800C07F8(Ground_GObj* g, u32 k, void* f) { emit(" ftdev(%s,%u,%s)", sym(g), k, fname(f)); }',
       'rec(&format!(" ftdev({},{},{})", sym(g as usize), k, fname(f as usize)));')
callee("Ground_801C10B8", "tu", "g: *mut HSD_GObj, f: HSD_GObjEvent", "", "group2: gr/ground.c",
       'void Ground_801C10B8(HSD_GObj* g, HSD_GObjEvent f) { emit(" g10B8(%s,%s)", sym(g), fname((void*) f)); }',
       'rec(&format!(" g10B8({},{})", sym(g as usize), fname(f.map_or(0, |f| f as usize))));')
callee("Ground_801C0498", "tu", "", "-> f32", "group2: gr/ground.c",
       'float Ground_801C0498(void) { static const float t[] = { 1.0f, 0.5f, 2.0f, 0.75f }; last498 = t[rnd() % 4]; emit(" g0498=%08x", fb(last498)); return last498; }',
       'const T: [f32; 4] = [1.0, 0.5, 2.0, 0.75]; let v = T[(rnd() % 4) as usize]; LAST498 = v; rec(&format!(" g0498={:08x}", v.to_bits())); v')
callee("ftLib_GetPos", "tu", "g: *mut HSD_GObj, out: *mut Vec3", "", "group3: ft/ftlib.c",
       'void ftLib_GetPos(HSD_GObj* g, Vec3* out) { out->x = rf(); out->y = ypick(0); out->z = rf(); emit(" lib6644(%s,%08x,%08x,%08x)", sym(g), fb(out->x), fb(out->y), fb(out->z)); }',
       '(*out).x = rf(); (*out).y = ypick(false); (*out).z = rf(); rec(&format!(" lib6644({},{:08x},{:08x},{:08x})", sym(g as usize), (*out).x.to_bits(), (*out).y.to_bits(), (*out).z.to_bits()));')
callee("ftLib_GetPrevPos", "tu", "g: *mut HSD_GObj, out: *mut Vec3", "", "group3: ft/ftlib.c",
       'void ftLib_GetPrevPos(HSD_GObj* g, Vec3* out) { out->x = rf(); out->y = ypick(1); out->z = rf(); emit(" lib6684(%s,%08x,%08x,%08x)", sym(g), fb(out->x), fb(out->y), fb(out->z)); }',
       '(*out).x = rf(); (*out).y = ypick(true); (*out).z = rf(); rec(&format!(" lib6684({},{:08x},{:08x},{:08x})", sym(g as usize), (*out).x.to_bits(), (*out).y.to_bits(), (*out).z.to_bits()));')
callee("ftLib_80086B80", "tu", "g: *mut HSD_GObj", "-> f32", "group3: ft/ftlib.c",
       'float ftLib_80086B80(HSD_GObj* g) { float v = rf(); emit(" lib6B80(%s)=%08x", sym(g), fb(v)); return v; }',
       'let v = rf(); rec(&format!(" lib6B80({})={:08x}", sym(g as usize), v.to_bits())); v')
callee("grLib_801C96F8", "tu", "a: i32, b: i32, p: *mut Vec3", "-> *mut HSD_Generator", "group2: gr/grlib.c",
       'HSD_Generator* grLib_801C96F8(s32 a, s32 b, Vec3* p) { unsigned r = rnd(); HSD_Generator* gp = r % 3 == 0 ? NULL : &fx_GN[(r >> 2) % 2]; emit(" lib96F8(%d,%d,%s", a, b, p ? "V" : "0"); if (p) emit(",%08x,%08x,%08x", fb(p->x), fb(p->y), fb(p->z)); emit(")=%s", sym(gp)); return gp; }',
       'let r = rnd(); let gp = if r % 3 == 0 { null_mut() } else { W().gn.add(((r >> 2) % 2) as usize) }; rec(&format!(" lib96F8({},{},{}", a, b, if p.is_null() { "0" } else { "V" })); if !p.is_null() { rec(&format!(",{:08x},{:08x},{:08x}", (*p).x.to_bits(), (*p).y.to_bits(), (*p).z.to_bits())); } rec(&format!(")={}", sym(gp as usize))); gp')

# ---- shard 609-616 (grtpeach .. grtzelda) ------------------------------------------------
callee("Ground_801C32D4", "tu", "a: i32, b: i32", "-> i32", "group2: gr/ground.c",
       's32 Ground_801C32D4(s32 a, s32 b) { static const int t[] = { -1, 0, 5, 9, 2 }; int r = t[rnd() % 5]; emit(" g32D4(%d,%d)=%d", a, b, r); return r; }',
       'const T: [i32; 5] = [-1, 0, 5, 9, 2]; let r = T[(rnd() % 5) as usize]; rec(&format!(" g32D4({},{})={}", a, b, r)); r')
callee("mpJointListAdd", "tu", "joint: i32", "", "group2: mp/mplib.c",
       'void mpJointListAdd(int joint) { emit(" jlAdd(%d)", joint); }',
       'rec(&format!(" jlAdd({})", joint));')
callee("mpLib_80057BC0", "tu", "joint: i32", "", "group2: mp/mplib.c",
       'void mpLib_80057BC0(int joint) { emit(" lib7BC0(%d)", joint); }',
       'rec(&format!(" lib7BC0({})", joint));')
callee("Mex_GrCallbacks", "tu", "", "-> *mut c_void", "pc: m-ex (Mex_GrCallbacks, NULL for a vanilla stage)",
       'void* Mex_GrCallbacks(void) { void* p = (rnd() & 1) ? (void*) fx_MX : NULL; emit(" mexcbs=%s", sym(p)); return p; }',
       'let p: *mut c_void = if rnd() & 1 != 0 { w().mx.cast() } else { null_mut() }; rec(&format!(" mexcbs={}", sym(p as usize))); p')
callee("Mex_GrTrace", "tu", "", "-> i32", "pc: m-ex (Mex_GrTrace, the grtrace debug switch)",
       'int Mex_GrTrace(void) { int r = (rnd() & 3) == 0; emit(" mextrace=%d", r); return r; }',
       'let r = (rnd() & 3 == 0) as i32; rec(&format!(" mextrace={}", r)); r')
# grtseak.c's "grtrace" OSReport calls take %p arguments (pointer values are not comparable), so
# the Rust twin is a separate ext entry; the C recorder is the same OSReport stub (above), which
# prints the format only for "grtrace" messages.
callee("OSReport_grtrace", "tu", "fmt: &CStr, args: &[usize]", "", "os: OSReport (varargs, %p arguments)",
       '', 'rec(&format!(" OSReport({})", fmt.to_str().unwrap()));')

BY_NAME = {c["name"]: c for c in CALLEES}
