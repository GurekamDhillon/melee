#!/usr/bin/env python3
"""State oracle for melee_world::tu::efn::{eflib, efasync, efdata} (the mechanical ports of src/melee/ef/*.c).

    python rust/melee_world/gen/mtu/mk_ef.py gen|run|all          (from the worktree root)

The three game TUs are compiled as ONE C translation unit (statics shared), with the SHIPPING float
contraction. gen_tu.py (spec "ef" of specs_ef.py) writes the recording stubs of every callee outside
them and the field-by-field fill / hash of the state records; this file writes the driver around it,
both halves from the one table below so they cannot drift:
  gen/mtu/tu/ef.c            C driver
  tests/tu_ef.rs             Rust driver, draw for draw
  tests/data/mtu_ef.txt      C output (op case ret state-hash trace-hash), computed from C source only

A case starts from a small object world: 9 GObjs (6 effect gobjs owning EF_Effects, 2 fighter gobjs, an
item gobj), 16 JObjs (effect roots with a sibling chain below them, a parent forest), 2 Fighters, the
effect description table, queued effects, generators / AppSRTs, the PLink lists, the param table and the
statics of the three files. The variadic Dispatch / Spawn calls get argument pointers of the kinds the
case needs (read from efasync.c).
"""
import os, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
os.chdir(ROOT)
CASES = 400

OPS = []


def op(c, r, ret="v", name=None):
    OPS.append((name or c.split("(")[0].split()[-1], c, r, ret))


def dispatch_table():
    """(case ids, kinds string) of efAsync_Dispatch: the argument pointers each case reads, in order"""
    t = (ROOT / "src/melee/ef/efasync.c").read_text(encoding="utf-8").replace("\r\n", "\n")
    i = t.index("void* efAsync_Dispatch")
    j = t.index("static char efAsync_803BFD68")
    body = t[i:j]
    k0 = body.index("switch (gfx_id)")
    body = body[k0:body.index("#if 1")]
    tk = {"Vec3*": "V", "f32*": "F", "HSD_JObj*": "J", "void*": "P", "u32": "U"}
    callee = {"efLib_CreateGenerator_AppSRT_SetScale": "JF", "efLib_CreateGenerator_AppSRT_SetFacingDirScale": "JFF",
              "efLib_CreateGenerator_AppSRT_SetFacingDir": "JF", "efLib_CreateGenerator_Attach_Scale": "J"}
    blocks = {}
    for m in re.finditer(r"\n    (block_\d+): \{", body):
        blocks[m.group(1)] = m.start()
    parts = re.split(r"\n    (?=case 0x[0-9A-Fa-f]+:)", body)[1:]

    def kinds_of(txt):
        out = ""
        for m in re.finditer(r"va_arg\(vlist, ([^)]+)\)|(efLib_CreateGenerator_\w+)\([^;]*\bvlist\b", txt):
            if m.group(1):
                out += tk[m.group(1)]
            elif m.group(2) in callee:
                out += callee[m.group(2)]
        return out
    ids, kinds, pend = [], [], []
    for p in parts:
        labs = re.findall(r"case (0x[0-9A-Fa-f]+):", p.split("\n")[0] if False else p[:p.index("\n")] if "\n" in p else p)
        first = p[:p.index("\n")] if "\n" in p else p
        lab = re.match(r"case (0x[0-9A-Fa-f]+):", first).group(1)
        rest = p[len(first):]
        pend.append(int(lab, 16))
        if rest.strip() == "":
            continue
        kd = kinds_of(rest)
        for g in re.findall(r"goto (block_\d+);", rest):
            b = blocks[g]
            kd += kinds_of(body[b:body.index("break;", b) + 6])
        for x in pend:
            ids.append(x)
            kinds.append(kd.ljust(6, "V"))
        pend = []
    return ids, kinds


DIDS, DKINDS = dispatch_table()
ND = len(DIDS)

EL = "el"
EA = "ea"
CBS = ["SetOffsetY_FromParamY", "SetScale_FromParamX", "SetRotYAndTransition", "SetJObjOffsetZ", "SetRotY_FromFighterDir",
       "SetRotYZ_FromFighter", "Fall_FromParamY", "SetOffset_FromParams", "LifetimeEndSpawn", "SetScaleRotY_FromFighter",
       "SetRotYZ_FromParamZ_FighterDir", "ftMr_SpecialLw", "ftLg_SpecialLw", "ftKp_SpecialHi", "ftCo_Bury",
       "ApplyStoredAlpha", "AccumOffset_FromParams"]
SPL = [0x4A38, 0x4A39, 0x4A3A, 0xB798, 0xB799, 0xB79A, 0x2D, 0x2E, 0x31, 0x127, 0x2, 0x6, 0xA3, 0xA7, 0xAA, 0xF2, 0x12A,
       0x132, 0x133, 0x16D, 0x16E, 0x16F, 0x170, 0x7E2, 0xD4, 0x243, 0xE3, 0xFC, 0xFF, 0xF7, 0x7918, 0x121, 0x55, 0x1E5]

# ------------------------------------------------------------------ eflib state functions
op("efLib_Init();", "el::efLib_Init();")
op("{ s32 f = (s32) fv(); efLib_SetFlags(&GO[6 + (rnd() & 1)], f); }", "{ let f = fv() as i32; el::efLib_SetFlags(&raw mut GO[6 + (rnd() & 1) as usize], f); }", "v", "efLib_SetFlags")
op("efLib_Destroy(&GO[rnd() % 8]);", "el::efLib_Destroy(&raw mut GO[(rnd() % 8) as usize]);")
op("efLib_DestroyAll(&GO[6 + (rnd() & 1)]);", "el::efLib_DestroyAll(&raw mut GO[6 + (rnd() & 1) as usize]);")
op("efLib_PauseAll(&GO[6 + (rnd() & 1)]);", "el::efLib_PauseAll(&raw mut GO[6 + (rnd() & 1) as usize]);")
op("efLib_ResumeAll(&GO[6 + (rnd() & 1)]);", "el::efLib_ResumeAll(&raw mut GO[6 + (rnd() & 1) as usize]);")
op("efLib_remove_user_data(&EFF[rnd() % 6]);", "el::efLib_remove_user_data(&raw mut EFF[(rnd() % 6) as usize] as *mut c_void);")
op("efLib_EffectCount = rnd() % 64; efLib_RemoveLast();", "ed::efLib_EffectCount = rnd() % 64; el::efLib_RemoveLast();", "v", "efLib_RemoveLast")
op("efLib_Update(&GO[rnd() % 6]);", "el::efLib_Update(&raw mut GO[(rnd() % 6) as usize]);")
op("{ int g = GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; ret = PI(efLib_Create(g, p)); }",
   "{ let g = gfx(); let p = &raw mut GO[6 + (rnd() & 1) as usize]; ret = pi(el::efLib_Create(g, p)); }", "p", "efLib_Create")
for fn in ["efLib_Create_Attach", "efLib_Create_AttachChild", "efLib_Create_Attach_Scale", "efLib_Create_AttachChild_Scale",
           "efLib_Create_Attach_Scale_FacingDir"]:
    op(f"{{ u32 g = (u32) GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI({fn}(g, p, j)); }}",
       f"{{ let g = gfx() as u32; let p = &raw mut GO[6 + (rnd() & 1) as usize]; let j = &raw mut JO[8 + (rnd() % 8) as usize]; ret = pi(el::{fn}(g, p, j)); }}", "p", fn)
op("{ u32 g = (u32) GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; ret = PI(efLib_Create_Attach_Pos(g, p, &V3[0])); }",
   "{ let g = gfx() as u32; let p = &raw mut GO[6 + (rnd() & 1) as usize]; ret = pi(el::efLib_Create_Attach_Pos(g, p, &raw mut V3[0])); }", "p", "efLib_Create_Attach_Pos")
op("efLib_render_callback(&GO[rnd() % 8], (int) (rnd() % 5) - 1);", "el::efLib_render_callback(&raw mut GO[(rnd() % 8) as usize], (rnd() % 5) as i32 - 1);")
op("efLib_particles_proc_main(&GO[0]);", "el::efLib_particles_proc_main(&raw mut GO[0]);")
op("efLib_particles_proc_aux(&GO[0]);", "el::efLib_particles_proc_aux(&raw mut GO[0]);")
op("{ int g = GG(); ret = PI(efLib_CreateGenerator(g, &V3[0])); }", "{ let g = gg(); ret = pi(el::efLib_CreateGenerator(g, &raw mut V3[0])); }", "p", "efLib_CreateGenerator")
op("{ int g = GFX(); ret = PI(efLib_CreateGenerator_AddAppSRT(g)); }", "{ let g = gfx(); ret = pi(el::efLib_CreateGenerator_AddAppSRT(g)); }", "p", "efLib_CreateGenerator_AddAppSRT")
op("{ int g = GFX(); float d = RNN(); ret = PI(efLib_CreateGenerator_Translate_FacingDir(g, &V3[1], d)); }",
   "{ let g = gfx(); let d = rnn(); ret = pi(el::efLib_CreateGenerator_Translate_FacingDir(g, &raw mut V3[1], d)); }", "p", "efLib_CreateGenerator_Translate_FacingDir")
op("{ int g = GFX(); ret = PI(efLib_CreateGenerator_Attach(g, &JO[8 + rnd() % 8])); }", "{ let g = gfx(); ret = pi(el::efLib_CreateGenerator_Attach(g, &raw mut JO[8 + (rnd() % 8) as usize])); }", "p", "efLib_CreateGenerator_Attach")
op("{ int g = GFX(); ret = PI(efLib_CreateGenerator_Attach_AddAppSRT(g, &JO[8 + rnd() % 8])); }", "{ let g = gfx(); ret = pi(el::efLib_CreateGenerator_Attach_AddAppSRT(g, &raw mut JO[8 + (rnd() % 8) as usize])); }", "p", "efLib_CreateGenerator_Attach_AddAppSRT")
op("{ int g = GFX(); HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(w_attach_scale(g, &GO[6 + (rnd() & 1)], j)); }",
   "{ let g = gfx(); let j = &raw mut JO[8 + (rnd() % 8) as usize]; let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a = [j as *const c_void]; let mut va = Va::new(&a); ret = pi(el::efLib_CreateGenerator_Attach_Scale(g, &mut va, go)); }", "p", "efLib_CreateGenerator_Attach_Scale")
op("{ int g = GFX(); HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(w_setscale(g, j, &FL[rnd() % 6])); }",
   "{ let g = gfx(); let j = &raw mut JO[8 + (rnd() % 8) as usize]; let f = &raw mut FL[(rnd() % 6) as usize]; let a = [j as *const c_void, f as *const c_void]; let mut va = Va::new(&a); ret = pi(el::efLib_CreateGenerator_AppSRT_SetScale(g, &mut va)); }", "p", "efLib_CreateGenerator_AppSRT_SetScale")
op("{ int g = GFX(); HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(w_facing(g, j, &FL[rnd() % 6])); }",
   "{ let g = gfx(); let j = &raw mut JO[8 + (rnd() % 8) as usize]; let f = &raw mut FL[(rnd() % 6) as usize]; let a = [j as *const c_void, f as *const c_void]; let mut va = Va::new(&a); ret = pi(el::efLib_CreateGenerator_AppSRT_SetFacingDir(g, &mut va)); }", "p", "efLib_CreateGenerator_AppSRT_SetFacingDir")
op("{ int g = GFX(); HSD_JObj* j = &JO[8 + rnd() % 8]; float* f1 = &FL[rnd() % 6]; float* f2 = &FL[rnd() % 6]; ret = PI(w_facingscale(g, j, f1, f2)); }",
   "{ let g = gfx(); let j = &raw mut JO[8 + (rnd() % 8) as usize]; let f1 = &raw mut FL[(rnd() % 6) as usize]; let f2 = &raw mut FL[(rnd() % 6) as usize]; let a = [j as *const c_void, f1 as *const c_void, f2 as *const c_void]; let mut va = Va::new(&a); ret = pi(el::efLib_CreateGenerator_AppSRT_SetFacingDirScale(g, &mut va)); }", "p", "efLib_CreateGenerator_AppSRT_SetFacingDirScale")
op("{ int g = GFX(); HSD_GObj* go = &GO[6 + (rnd() & 1)]; HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(efLib_CreateGenerator_AppSRT_SetPos(g, go, j, &V3[2])); }",
   "{ let g = gfx(); let go = &raw mut GO[6 + (rnd() & 1) as usize]; let j = &raw mut JO[8 + (rnd() % 8) as usize]; ret = pi(el::efLib_CreateGenerator_AppSRT_SetPos(g, go, j, &raw mut V3[2])); }", "p", "efLib_CreateGenerator_AppSRT_SetPos")
op("{ int g = SPG(); int fl = (int) (rnd() & 1); efLib_SpawnParticleEffect(g / 1000, g, &JO[8 + rnd() % 8], fl); }",
   "{ let g = spg(); let fl = (rnd() & 1) as i32; el::efLib_SpawnParticleEffect(g / 1000, g, &raw mut JO[8 + (rnd() % 8) as usize], fl); }", "v", "efLib_SpawnParticleEffect")
op("{ int b = (rnd() & 3) == 0 ? 0x1E : (int) (rnd() % 50); int g = SPG(); efLib_Cb_SPtcl((int) (rnd() % 3), b, g, &JO[8 + rnd() % 8]); }",
   "{ let b = if rnd() & 3 == 0 { 0x1E } else { (rnd() % 50) as i32 }; let g = spg(); let l = (rnd() % 3) as i32; el::efLib_Cb_SPtcl(l, b, g, &raw mut JO[8 + (rnd() % 8) as usize]); }", "v", "efLib_Cb_SPtcl")
op("{ int b = (rnd() & 3) == 0 ? 0x1E : (int) (rnd() % 50); int g = SPG(); efLib_Cb_DPtcl((int) (rnd() % 3), b, g, &JO[8 + rnd() % 8]); }",
   "{ let b = if rnd() & 3 == 0 { 0x1E } else { (rnd() % 50) as i32 }; let g = spg(); let l = (rnd() % 3) as i32; el::efLib_Cb_DPtcl(l, b, g, &raw mut JO[8 + (rnd() % 8) as usize]); }", "v", "efLib_Cb_DPtcl")
op("efLib_Cb_ParticleRender(&PT[rnd() % 2]);", "el::efLib_Cb_ParticleRender(&raw mut PT[(rnd() % 2) as usize]);")
op("efLib_Cb_PtclAppSRTHook(&GEN[rnd() % 3]);", "el::efLib_Cb_PtclAppSRTHook(&raw mut GEN[(rnd() % 3) as usize]);")
for cb in CBS:
    pre = "EFF[0].next = &EFF[1]; " if cb == "ftKp_SpecialHi" else ""
    prer = "EFF[0].next = &raw mut EFF[1]; " if cb == "ftKp_SpecialHi" else ""
    idx = "0" if cb == "ftKp_SpecialHi" else "rnd() % 6"
    op(f"{{ {pre}efLib_Cb_{cb}(&EFF[{idx}]); }}", f"{{ {prer}el::efLib_Cb_{cb}(&raw mut EFF[({idx.replace('rnd() % 6', 'rnd() % 6')}) as usize]); }}".replace("[(0) as usize]", "[0]"), "v", "efLib_Cb_" + cb)
op("{ s32 c = (s32) (rnd() % 4); u32 k = fv() * 0x10101u; u32 t = fv() * 0x01010101u; efLib_SetTevKonstColor(&JO[rnd() % 16], c, k, t); }",
   "{ let c = (rnd() % 4) as i32; let k = fv().wrapping_mul(0x10101); let t = fv().wrapping_mul(0x01010101); el::efLib_SetTevKonstColor(&raw mut JO[(rnd() % 16) as usize], c, k, t); }", "v", "efLib_SetTevKonstColor")
op("{ u8 a = (u8) fv(); efLib_SetParamAlpha(&GO[rnd() % 8], a); }", "{ let a = fv() as u8; el::efLib_SetParamAlpha(&raw mut GO[(rnd() % 8) as usize], a); }", "v", "efLib_SetParamAlpha")
op("{ s32 g = GFX(); efLib_SetParamGfxId(&GO[rnd() % 8], g); }", "{ let g = gfx(); el::efLib_SetParamGfxId(&raw mut GO[(rnd() % 8) as usize], g); }", "v", "efLib_SetParamGfxId")
op("{ float z = rw(); HSD_JObjSetTranslateZ(&JO[rnd() % 16], z); }", "{ let z = rw(); el::HSD_JObjSetTranslateZ(&raw mut JO[(rnd() % 16) as usize], z); }", "v", "HSD_JObjSetTranslateZ")

# ------------------------------------------------------------------ efasync
op("efAsync_LoadAsync((int) (rnd() % 72) - 3);", "ea::efAsync_LoadAsync((rnd() % 72) as i32 - 3);")
op("{ u32 len = fv(); int idx = (int) (rnd() % 65); efAsync_OnLoad(&ARC[0], (u8*) &V3[0], len, idx); }",
   "{ let len = fv(); let idx = (rnd() % 65) as i32; ea::efAsync_OnLoad(&raw mut ARC[0], &raw mut V3[0] as *mut u8, len, idx); }", "v", "efAsync_OnLoad")
op("efAsync_LoadSync((int) (rnd() % 72) - 3);", "ea::efAsync_LoadSync((rnd() % 72) as i32 - 3);")
op("efAsync_QueueFlush(&GO[6 + (rnd() & 1)], &QH[0]);", "ea::efAsync_QueueFlush(&raw mut GO[6 + (rnd() & 1) as usize], &raw mut QH[0] as *mut c_void);")
op("efAsync_QueueClear(&QH[0]);", "ea::efAsync_QueueClear(&raw mut QH[0] as *mut c_void);")
op("efAsync_QueueInit();", "ea::efAsync_QueueInit();")
op("efAsync_MexInit();", "ea::efAsync_MexInit();")
op("efAsync_QueueProcessDeferred(&GO[6 + (rnd() & 1)], &QE[rnd() % 4]);", "ea::efAsync_QueueProcessDeferred(&raw mut GO[6 + (rnd() & 1) as usize], &raw mut QE[(rnd() % 4) as usize]);")
op("{ int b = (int) (rnd() % 70) - 2; efAsync_MexNoteArchive(b, (rnd() & 1) ? &ARC[0] : 0); }",
   "{ let b = (rnd() % 70) as i32 - 2; ea::efAsync_MexNoteArchive(b, if rnd() & 1 != 0 { &raw mut ARC[0] } else { null_mut() }); }", "v", "efAsync_MexNoteArchive")
op("{ HSD_GObj* g = MXG(); int id = MXID(); ret = (u32) efAsync_MexResolve(g, id, &OUTI[0], &OUTI[1], (HSD_GObj**) &OUTP[0]); HM((u32) OUTI[0]); HM((u32) OUTI[1]); HM(PI(OUTP[0])); }",
   "{ let g = mxg(); let id = mxid(); ret = ea::efAsync_MexResolve(g, id, &raw mut OUTI[0], &raw mut OUTI[1], &raw mut OUTP[0]) as u32; hm(OUTI[0] as u32); hm(OUTI[1] as u32); hm(pi(OUTP[0])); }", "i", "efAsync_MexResolve")
op("{ HSD_GObj* g = MXG(); int id = MXID(); float f = rw(); float o = rw(); efAsync_MexSpawn(g, &QH[0], id, &JO[8 + rnd() % 8], &V3[3], f, o); }",
   "{ let g = mxg(); let id = mxid(); let f = rw(); let o = rw(); ea::efAsync_MexSpawn(g, &raw mut QH[0] as *mut c_void, id, &raw mut JO[8 + (rnd() % 8) as usize], &raw mut V3[3], f, o); }", "v", "efAsync_MexSpawn")
op("efAsync_MexProcessDeferred(&GO[6 + (rnd() & 1)], &QE[rnd() % 4]);", "ea::efAsync_MexProcessDeferred(&raw mut GO[6 + (rnd() & 1) as usize], &raw mut QE[(rnd() % 4) as usize]);")
op("{ EF_Effect* e = &EFF[rnd() % 6]; efAsync_SetEffectScaleXYZ(e, rw()); }", "{ let e = &raw mut EFF[(rnd() % 6) as usize]; ea::efAsync_SetEffectScaleXYZ(e, rw()); }", "v", "efAsync_SetEffectScaleXYZ")
op("{ EF_Effect* e = &EFF[rnd() % 6]; efAsync_SetEffectRotationZFromPtr(e, &FL[rnd() % 6]); }", "{ let e = &raw mut EFF[(rnd() % 6) as usize]; ea::efAsync_SetEffectRotationZFromPtr(e, &raw mut FL[(rnd() % 6) as usize]); }", "v", "efAsync_SetEffectRotationZFromPtr")
op("{ EF_Effect* e = &EFF[rnd() % 6]; efAsync_SetEffectFacingDir(e, rw()); }", "{ let e = &raw mut EFF[(rnd() % 6) as usize]; ea::efAsync_SetEffectFacingDir(e, rw()); }", "v", "efAsync_SetEffectFacingDir")
op("efAsync_SetEffectRandomRotationZ(&EFF[rnd() % 6]);", "ea::efAsync_SetEffectRandomRotationZ(&raw mut EFF[(rnd() % 6) as usize]);")
op("efAsync_SetEffectScale(&EFF[rnd() % 6], &V3[4]);", "ea::efAsync_SetEffectScale(&raw mut EFF[(rnd() % 6) as usize], &raw mut V3[4]);")
# efAsync_Spawn: every kind, the variadic pointer arguments of that kind
op("{ u32 kind = rnd() % 9; u32 g = (u32) GFX(); HSD_GObj* go = &GO[6 + (rnd() & 1)]; void* a1 = ARGP((int) (rnd() % 3)); void* a2 = ARGP((int) (rnd() % 3)); void* a3 = ARGP(1); "
   "efAsync_Spawn(go, &QH[0], kind, g, &JO[8 + rnd() % 8], a1, a2, a3); }",
   "{ let kind = rnd() % 9; let g = gfx() as u32; let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a1 = argp((rnd() % 3) as i32); let a2 = argp((rnd() % 3) as i32); let a3 = argp(1); "
   "ea::efAsync_Spawn(go, &raw mut QH[0] as *mut c_void, kind, g, &raw mut JO[8 + (rnd() % 8) as usize], &[a1, a2, a3]); }", "v", "efAsync_Spawn")
for off in (0, 48, 96):
    op(f"{{ int d = (int) ((k + {off}) % {ND}); ret = PI(disp(DID[d], &GO[6 + (rnd() & 1)], ARGD(d, 0), ARGD(d, 1), ARGD(d, 2), ARGD(d, 3), ARGD(d, 4), ARGD(d, 5))); }}",
       f"{{ let d = ((k + {off}) % {ND}) as usize; let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a = [argd(d, 0), argd(d, 1), argd(d, 2), argd(d, 3), argd(d, 4), argd(d, 5)]; let mut va = Va::new(&a); ret = pi(ea::efAsync_Dispatch(DID[d], go, &mut va)); }}",
       "p", f"efAsync_Dispatch_{off}")
op("{ HSD_GObj* go = &GO[6 + (rnd() & 1)]; FL[0] = (float) (rnd() % 100000) / 7.0f; ret = PI(disp(0x3E8, go, &V3[0], &FL[0])); }",
   "{ let go = &raw mut GO[6 + (rnd() & 1) as usize]; FL[0] = (rnd() % 100000) as f32 / 7.0; let a = [&raw mut V3[0] as *const c_void, &raw mut FL[0] as *const c_void]; let mut va = Va::new(&a); ret = pi(ea::efAsync_Dispatch(0x3E8, go, &mut va)); }", "p", "efAsync_Dispatch_3E8_fused")
op("{ int g = (int) (rnd() % 1500); if (is_d(g)) g += 5000; HSD_GObj* go = &GO[6 + (rnd() & 1)]; void* a = ARGP(0); ret = PI(disp(g, go, a, a, a, a)); }",
   "{ let mut g = (rnd() % 1500) as i32; if is_d(g) { g += 5000; } let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a = argp(0); let b = [a, a, a, a]; let mut va = Va::new(&b); ret = pi(ea::efAsync_Dispatch(g, go, &mut va)); }", "p", "efAsync_Dispatch_any")

# ----------------------------------------------------------------------------------- C driver
C_HEAD = r'''/* TU: src/melee/ef/{efdata,eflib,efasync}.c as one TU */
/* C oracle for the Rust ports of these TUs: the game's files included verbatim (the efSync_Spawn call sites
 * counted, see specs_ef.py); every callee out of them is a recording stub. Written by gen/mtu/mk_ef.py.
 * Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#define __assert mtu_unused_assert
#include "mtu_common.h"
#undef __assert
extern void setbuf(void*, char*);
extern void* __acrt_iob_func(unsigned);
extern int fprintf(void*, const char*, ...);
extern char* getenv(const char*);
extern void RaiseException(unsigned long, unsigned long, unsigned long, const void*);
static int ASLINE[512], LASTAL; static const char* LASTC; static const char* ASCOND[512];
void __assert(const char* file, u32 line, const char* cond) { (void) file; LASTC = cond; LASTAL = (int) line; RaiseException(0xE0A55E87ul, 1, 0, 0); }
void* memset(void* d, int c, unsigned long n) { unsigned char* p = (unsigned char*) d; while (n--) *p++ = (unsigned char) c; return d; }
void memzero(void* d, int n) { unsigned char* p = (unsigned char*) d; while (n-- > 0) *p++ = 0; }
int __HI(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (c.u >> 32); }
int __LO(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (u32) c.u; }
static EF_Effect EFF[10];
static EF_QueuedEffect QE[8];
static EF_QueuedEffect QH[1];
static HSD_GObj GO[9];
static HSD_JObj JO[16];
static Fighter FT[2];
static Item ITM[1];
static HSD_DObj DO[4];
static HSD_MObj MO[4];
static HSD_TObj TO[8];
static HSD_TObjTev TV[8];
static HSD_Generator GEN[3];
static HSD_psAppSRT APS[3];
static HSD_Particle PT[2];
static HSD_GObjProc PROC[2];
static EF_EffectDesc DESC[64];
static EF_DAT_Entry DE[2];
static EF_DAT_Entry DEINIT[65];
static EfMexBehavior BHV;
static u8 BT1[16], BT2[16];
static char STRS[3][24] = { "MexA.dat", "MexB.dat", "MexC.dat" };
static HSD_Archive ARC[1];
static HSD_GObj* PLH[64];
static Vec3 V3[6];
static float FL[6];
static struct _HSD_PSCmdList CMD[5];
static struct _HSD_PSCmdList* PREF[0x220];
static u8 AJ[4];
static int EFN, QN;
static s32 OUTI[4];
static void* OUTP[2];
static int mtu_dbgp;
static int ASSN[512];
static void fprintf_stub(int op) { fprintf(__acrt_iob_func(2), "ASSERTS %d %d line %d %s%c", op, ASSN[op], ASLINE[op], ASCOND[op] ? ASCOND[op] : "", 10); }
static float RNN(void) { static const float k[4] = { 1.0f, 13.0f, 100.0f, 1000.0f }; float a = (float) ((int) (rnd() % 41) - 20) / 20.0f; return a * k[rnd() % 4]; }
#include "ef_gen.h"
/* the Fighter fields the ef callbacks read (the full record has bitfield overlays the walker cannot map) */
static void fill_FighterEf(Fighter* p)
{
    p->facing_dir = rg(); p->motion_id = (s32) fv(); p->cmd_vars[0] = fv(); p->cmd_vars[1] = fv(); p->cmd_vars[2] = fv(); p->cmd_vars[3] = fv();
    p->mv.co.common.x4.x = rg(); p->mv.co.common.x4.y = rg(); p->mv.co.common.x4.z = rg();
    p->mv.co.common.x10 = (rnd() & 1) ? (void*) &AJ[0] : 0;
    p->coll_data.floor.normal.x = rg(); p->coll_data.floor.normal.y = rg();
    p->u.kb.hat.kind = (int) fv(); p->kind = (int) fv();
}
static void hash_FighterEf(const Fighter* p)
{
    HM(B(p->facing_dir)); HM((u32) p->motion_id); HM(p->cmd_vars[0]); HM(p->cmd_vars[1]); HM(p->cmd_vars[2]); HM(p->cmd_vars[3]);
    HM(B(p->mv.co.common.x4.x)); HM(B(p->mv.co.common.x4.y)); HM(B(p->mv.co.common.x4.z)); HM(PI(p->mv.co.common.x10));
    HM(B(p->coll_data.floor.normal.x)); HM(B(p->coll_data.floor.normal.y)); HM((u32) p->u.kb.hat.kind); HM((u32) p->kind);
}

static const int DID[] = { @DIDS@ };
static const char* DKIND[] = { @DKINDS@ };
static int is_d(int g) { unsigned i; for (i = 0; i < sizeof DID / sizeof DID[0]; i++) if (DID[i] == g) return 1; return 0; }
static const int SPLIST[] = { @SPL@ };
static void (*const UPD[])(EF_Effect*) = { @UPDC@ };
#define NUPD (int) (sizeof UPD / sizeof UPD[0])

static int GFX(void)
{
    u32 r = rnd();
    int bank = (int) (rnd() % 48);
    int id = (int) (rnd() % 64);
    if (r % 24 == 0) return (int) (fv() % 70) * 1000 + id;
    if (r % 24 == 1) return 65000 + id;
    return bank * 1000 + id;
}
static int GG(void)
{
    u32 r = rnd();
    static const int sp[] = { 0x121, 0xFF, 0x7918, 0xFC, 0xF7 };
    if (r % 3 == 0) return sp[rnd() % 5];
    return GFX();
}
static int SPG(void)
{
    u32 r = rnd();
    if (r % 6 == 0) return GFX();
    return SPLIST[rnd() % (sizeof SPLIST / sizeof SPLIST[0])];
}
static HSD_GObj* MXG(void)
{
    u32 r = rnd() % 6;
    if (r == 0) return 0;
    if (r == 1) return &GO[8];
    return &GO[6 + (rnd() & 1)];
}
static int MXID(void)
{
    u32 r = rnd() % 8;
    if (r == 0) return (int) (rnd() % 5000);
    if (r == 1) return 5000 + 1000 * (int) (rnd() % 5);
    return 5000 + (int) (rnd() % 4100);
}
static void* ARGP(int kind)
{
    switch (kind) {
    case 0: return &V3[rnd() % 6];
    case 1: return &FL[rnd() % 6];
    default: return &JO[rnd() % 16];
    }
}
static void* ARGD(int d, int i)
{
    char k = DKIND[d][i];
    switch (k) {
    case 'V': return &V3[rnd() % 6];
    case 'F': return &FL[rnd() % 6];
    case 'J': return &JO[rnd() % 16];
    case 'U': return (void*) (unsigned long long) (fv() * 0x10101u);
    default: return &V3[rnd() % 6];
    }
}
static void* disp(s32 g, HSD_GObj* o, ...) { __builtin_va_list ap; void* r; __builtin_va_start(ap, o); r = efAsync_Dispatch(g, o, ap); __builtin_va_end(ap); return r; }
static HSD_Generator* w_attach_scale(s32 g, HSD_GObj* o, ...) { __builtin_va_list ap; HSD_Generator* r; __builtin_va_start(ap, o); r = efLib_CreateGenerator_Attach_Scale(g, ap, o); __builtin_va_end(ap); return r; }
static HSD_Generator* w_setscale(s32 g, ...) { __builtin_va_list ap; HSD_Generator* r; __builtin_va_start(ap, g); r = efLib_CreateGenerator_AppSRT_SetScale(g, ap); __builtin_va_end(ap); return r; }
static HSD_Generator* w_facing(s32 g, ...) { __builtin_va_list ap; HSD_Generator* r; __builtin_va_start(ap, g); r = efLib_CreateGenerator_AppSRT_SetFacingDir(g, ap); __builtin_va_end(ap); return r; }
static HSD_Generator* w_facingscale(s32 g, ...) { __builtin_va_list ap; HSD_Generator* r; __builtin_va_start(ap, g); r = efLib_CreateGenerator_AppSRT_SetFacingDirScale(g, ap); __builtin_va_end(ap); return r; }

static void setup(void)
{
    int i, j, ord[6], split;
    zero(EFF, sizeof EFF); zero(QE, sizeof QE); zero(QH, sizeof QH); zero(GO, sizeof GO); zero(JO, sizeof JO); zero(FT, sizeof FT);
    zero(ITM, sizeof ITM); zero(DO, sizeof DO); zero(MO, sizeof MO); zero(TO, sizeof TO); zero(TV, sizeof TV); zero(GEN, sizeof GEN);
    zero(APS, sizeof APS); zero(PT, sizeof PT); zero(PROC, sizeof PROC); zero(DESC, sizeof DESC); zero(DE, sizeof DE); zero(PLH, sizeof PLH);
    zero(V3, sizeof V3); zero(FL, sizeof FL); zero(OUTI, sizeof OUTI); zero(OUTP, sizeof OUTP); zero(ARC, sizeof ARC); zero(CMD, sizeof CMD);
    EFN = 6; QN = 4;
    for (i = 0; i < 9; i++) { fill_HSD_GObj(&GO[i]); GO[i].hsd_obj = &JO[i]; GO[i].gx_link = (rnd() & 1) ? 7 : 8; GO[i].obj_kind = (rnd() % 3 == 0) ? (u8) rnd() : 2; }
    for (i = 0; i < 16; i++) { fill_HSD_JObj(&JO[i]); JO[i].flags &= ~(1u << 17); if (rnd() & 1) JO[i].flags |= (1 << 6); if (rnd() % 4 == 0) JO[i].flags |= (1 << 25); }
    fill_FighterEf(&FT[0]); fill_FighterEf(&FT[1]);
    for (i = 0; i < 10; i++) fill_EF_Effect(&EFF[i]);
    for (i = 0; i < 8; i++) fill_EF_QueuedEffect(&QE[i]);
    fill_EF_QueuedEffect(&QH[0]);
    for (i = 0; i < 4; i++) { fill_HSD_DObj(&DO[i]); fill_HSD_MObj(&MO[i]); }
    for (i = 0; i < 8; i++) { fill_HSD_TObj(&TO[i]); fill_HSD_TObjTev(&TV[i]); }
    for (i = 0; i < 3; i++) { fill_HSD_Generator(&GEN[i]); fill_HSD_psAppSRT(&APS[i]); }
    for (i = 0; i < 2; i++) { fill_HSD_Particle(&PT[i]); fill_HSD_GObjProc(&PROC[i]); fill_EF_DAT_Entry(&DE[i]); }
    for (i = 0; i < 64; i++) fill_EF_EffectDesc(&DESC[i]);
    for (i = 0; i < 6; i++) { V3[i].x = RNN(); V3[i].y = RNN(); V3[i].z = RNN(); FL[i] = RNN(); }
    for (i = 0; i < 16; i++) { BT1[i] = (u8) (rnd() % 9); BT2[i] = (u8) (rnd() % 9); }
    BHV.mdl_num = (s32) (rnd() % 12); BHV.mdl_type = BT1; BHV.ptcl_num = (s32) (rnd() % 12); BHV.ptcl_type = BT2;
    /* effects and their gobjs */
    for (i = 0; i < 6; i++) {
        GO[i].user_data = &EFF[i]; EFF[i].gobj = &GO[i]; EFF[i].parent_gobj = &GO[6 + (rnd() & 1)];
        EFF[i].attach_jobj = (rnd() % 4) ? &JO[8 + rnd() % 8] : 0;
        EFF[i].update = (rnd() % 4) ? UPD[rnd() % NUPD] : 0;
        EFF[i].user_data = (rnd() % 3) ? (void*) &JO[rnd() % 16] : 0;
        EFF[i].next = &EFF[rnd() % 6];
        if (rnd() % 4 == 0) EFF[i].lifetime = 1;
        EFF[i].state_flags = (rnd() % 3 == 0) ? (u8) (rnd() % 3) : 0;
        EFF[i].scale_flags = (u8) (rnd() % 3);
        EFF[i].is_async = (u8) (rnd() & 1);
    }
    GO[6].user_data = &FT[0]; GO[7].user_data = &FT[1]; GO[6].classifier = 4; GO[7].classifier = 4;
    GO[8].classifier = 6; GO[8].user_data = &ITM[0]; ITM[0].mex_original_owner = (rnd() & 3) ? &GO[6 + (rnd() & 1)] : 0;
    for (i = 0; i < 2; i++) {
        u32 m = rnd() % 5;
        if (m == 0) FT[i].motion_id = 349; else if (m == 1) FT[i].motion_id = 357; else if (m == 2) FT[i].motion_id = 359;
        FT[i].kind = (rnd() % 3 == 0) ? 4 : (int) (rnd() % 127);
    }
    /* jobj forest: parents point to lower indices; effect roots own a sibling chain 8..15 */
    for (i = 1; i < 16; i++) JO[i].parent = (rnd() & 1) ? &JO[rnd() % i] : 0;
    for (i = 0; i < 6; i++) JO[i].child = &JO[8 + (rnd() & 1)];
    for (i = 8; i < 15; i++) JO[i].next = &JO[i + 1];
    JO[6].child = (rnd() & 1) ? &JO[10] : 0; JO[7].child = (rnd() & 1) ? &JO[11] : 0; JO[8].child = (rnd() & 1) ? &JO[12] : 0;
    for (i = 0; i < 4; i++) { DO[i].mobj = (rnd() % 4) ? &MO[i] : 0; }
    for (i = 0; i < 8; i++) { TO[i].next = &TO[(i + 1) % 8]; TO[i].tev = &TV[i]; }
    for (i = 0; i < 3; i++) {
        GEN[i].appsrt = (rnd() & 1) ? &APS[rnd() % 3] : 0;
        GEN[i].cmdList = (rnd() % 3) ? (u8*) CMD[rnd() % 5].cmdList : 0;
    }
    for (i = 0; i < 2; i++) { PT[i].appsrt = (rnd() & 1) ? &APS[rnd() % 3] : 0; PROC[i].s_link = (u8) (rnd() % 12); }
    for (i = 0; i < 0x220; i++) PREF[i] = &CMD[4];
    PREF[0x96] = &CMD[0]; PREF[0x97] = &CMD[1]; PREF[0x98] = &CMD[2]; PREF[0x21B] = &CMD[3];
    for (i = 0; i < 65; i++) ptclref_804D0E5C[i] = 0;
    ptclref_804D0E5C[0] = PREF;
    for (i = 0; i < 64; i++) {
        DESC[i].model_desc.joint = (rnd() % 4) ? (HSD_Joint*) &AJ[0] : 0;
        DESC[i].model_desc.animjoint = (rnd() & 1) ? (HSD_AnimJoint*) &AJ[1] : 0;
        DESC[i].model_desc.matanim_joint = (rnd() % 4 == 0) ? (HSD_MatAnimJoint*) &AJ[2] : 0;
        DESC[i].model_desc.shapeanim_joint = (rnd() % 4 == 0) ? (HSD_ShapeAnimJoint*) &AJ[3] : 0;
        if (rnd() & 1) DESC[i].lifetime = (float) (rnd() % 700) / 10.0f;
    }
    for (i = 0; i < 2; i++) { DE[i].ef_DAT_file = (rnd() & 1) ? STRS[0] : 0; DE[i].effDataTable_name = (rnd() & 1) ? STRS[1] : 0; }
    for (i = 0; i < 65; i++) {
        efAsync_DatEntries[i] = DEINIT[i];
        efAsync_DatEntries[i].data = (rnd() % 8) ? (void*) DESC : 0;
        efAsync_MexBhv[i] = (rnd() & 1) ? &BHV : 0;
    }
    efAsync_MexReady = (int) (rnd() & 1);
    for (i = 0; i < 128; i++) ftData_UnkBytePerCharacter[i] = (rnd() % 6 == 0) ? 0xFF : (u8) (rnd() % 70);
    /* efLib */
    efLib_EffectCount = rnd() % 64; efLib_LoadKind = rnd() & 1; efLib_AnimCount = (s32) (rnd() % 8);
    zero(efLib_AnimQueue, sizeof efLib_AnimQueue);
    for (i = 0; i < efLib_AnimCount; i++) ((HSD_JObj**) efLib_AnimQueue)[i] = &JO[rnd() % 16];
    for (i = 0; i < 8; i++) {
        fill_EF_ParamEntry(&efLib_ParamTable[i]);
        efLib_ParamTable[i].gobj = (rnd() & 3) ? &GO[rnd() % 8] : 0;
        j = (int) (rnd() % 4);
        if (j == 0) efLib_ParamTable[i].gfx_id = 0x417; else if (j == 1) efLib_ParamTable[i].gfx_id = 0x419;
    }
    hsd_804D7900 = 0;
    HSD_GObj_JObjKind = 2;
    HSD_GObj_CurrentInvokedProc = (rnd() & 1) ? &PROC[0] : 0;
    HSD_GObjPLinkHead = PLH;
    for (i = 0; i < 6; i++) ord[i] = i;
    for (i = 5; i > 0; i--) { j = (int) (rnd() % (u32) (i + 1)); { int t = ord[i]; ord[i] = ord[j]; ord[j] = t; } }
    split = (int) (rnd() % 7);
    PLH[11] = split ? &GO[ord[0]] : 0;
    for (i = 0; i + 1 < split; i++) GO[ord[i]].next = &GO[ord[i + 1]];
    PLH[12] = split < 6 ? &GO[ord[split]] : 0;
    for (i = split; i + 1 < 6; i++) GO[ord[i]].next = &GO[ord[i + 1]];
    /* the queue: QE[0..3] chained below the head */
    for (i = 0; i < 4; i++) {
        QE[i].spawn_kind = (u8) (rnd() % 10); QE[i].gfx_id = GFX(); QE[i].jobj = &JO[8 + rnd() % 8];
        if (QE[i].spawn_kind == 9) QE[i].gfx_id = 5000 + (s32) (rnd() % 4100);
        QE[i].next = (i < 3) ? &QE[i + 1] : 0;
    }
    QH[0].next = (rnd() % 4) ? &QE[0] : 0;
}

static void hash_all(void)
{
    int i;
    for (i = 0; i < 10; i++) hash_EF_Effect(&EFF[i]);
    for (i = 0; i < 8; i++) hash_EF_QueuedEffect(&QE[i]);
    hash_EF_QueuedEffect(&QH[0]);
    for (i = 0; i < 9; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < 16; i++) hash_HSD_JObj(&JO[i]);
    hash_FighterEf(&FT[0]); hash_FighterEf(&FT[1]);
    for (i = 0; i < 4; i++) { hash_HSD_DObj(&DO[i]); hash_HSD_MObj(&MO[i]); }
    for (i = 0; i < 8; i++) { hash_HSD_TObj(&TO[i]); hash_HSD_TObjTev(&TV[i]); }
    for (i = 0; i < 3; i++) { hash_HSD_Generator(&GEN[i]); hash_HSD_psAppSRT(&APS[i]); }
    for (i = 0; i < 2; i++) { hash_HSD_Particle(&PT[i]); hash_HSD_GObjProc(&PROC[i]); hash_EF_DAT_Entry(&DE[i]); }
    for (i = 0; i < 6; i++) { HM(B(V3[i].x)); HM(B(V3[i].y)); HM(B(V3[i].z)); HM(B(FL[i])); }
    HM((u32) EFN); HM((u32) QN);
    HM(efLib_EffectCount); HM(efLib_LoadKind); HM((u32) efLib_AnimCount);
    for (i = 0; i < 16; i++) HM(PI(((void**) efLib_AnimQueue)[i]));
    for (i = 0; i < 8; i++) hash_EF_ParamEntry(&efLib_ParamTable[i]);
    for (i = 0; i < 3; i++) HM(FI((const void*) lbl_803BF810[i]));
    HM(FI((const void*) hsd_804D7900));
    for (i = 0; i < 65; i++) { HM(SH(efAsync_DatEntries[i].ef_DAT_file)); HM(SH(efAsync_DatEntries[i].effDataTable_name)); HM(PI(efAsync_DatEntries[i].data)); HM(PI(efAsync_MexBhv[i])); }
    HM((u32) efAsync_MexReady);
    HM((u32) OUTI[2]);
}

int main(void)
{
    int op, k;
    setbuf(__acrt_iob_func(1), 0);
    for (op = 0; op < 65; op++) DEINIT[op] = efAsync_DatEntries[op];
    preg(EFF, sizeof EFF); preg(QE, sizeof QE); preg(QH, sizeof QH); preg(GO, sizeof GO); preg(JO, sizeof JO); preg(FT, sizeof FT);
    preg(ITM, sizeof ITM); preg(DO, sizeof DO); preg(MO, sizeof MO); preg(TO, sizeof TO); preg(TV, sizeof TV); preg(GEN, sizeof GEN);
    preg(APS, sizeof APS); preg(PT, sizeof PT); preg(PROC, sizeof PROC); preg(DESC, sizeof DESC); preg(DE, sizeof DE);
    preg(&BHV, sizeof BHV); preg(ARC, sizeof ARC); preg(PLH, sizeof PLH); preg(V3, sizeof V3); preg(FL, sizeof FL);
    preg(CMD, sizeof CMD); preg(PREF, sizeof PREF); preg(AJ, sizeof AJ); preg(STRS, sizeof STRS); preg(BT1, sizeof BT1); preg(BT2, sizeof BT2);
    freg_all();
    for (op = 0; op < @NOPS@; op++) {
        for (k = 0; k < @CASES@; k++) {
            u32 ret = 0;
            tu_case(op, k);
            mtu_dbgp = (op == DBG_OP && k == DBG_K);
            setup();
            __try {
            switch (op) {
@COPS@
            }
            } __except (_exception_code() == 0xE0A55E87ul) { HM(0xA55E87u); ASSN[op]++; ASLINE[op] = LASTAL; ASCOND[op] = LASTC; }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    if (getenv("MTU_STAT")) { for (op = 0; op < @NOPS@; op++) fprintf_stub(op); }
    return 0;
}
'''

R_HEAD = r'''//! Written by gen/mtu/mk_ef.py from the same table as gen/mtu/tu/ef.c.
//! tu::efn::{eflib,efasync} vs the C (eflib.c + efasync.c + efdata.c, shipping contraction; regenerate the
//! expected file with `python rust/melee_world/gen/mtu/mk_ef.py run`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu as m;
use melee_world::tu::efn::eflib as el;
use melee_world::tu::efn::efasync as ea;
use melee_world::tu::efn::efdata as ed;
use melee_world::tu::va::Va;
use mtu_harness::*;
#[path = "mtu/ef_gen.rs"]
mod g;

static mut EFF: [EF_Effect; 10] = unsafe { core::mem::zeroed() };
static mut QE: [EF_QueuedEffect; 8] = unsafe { core::mem::zeroed() };
static mut QH: [EF_QueuedEffect; 1] = unsafe { core::mem::zeroed() };
static mut GO: [HSD_GObj; 9] = unsafe { core::mem::zeroed() };
static mut JO: [HSD_JObj; 16] = unsafe { core::mem::zeroed() };
static mut FT: [Fighter; 2] = unsafe { core::mem::zeroed() };
static mut ITM: [Item; 1] = unsafe { core::mem::zeroed() };
static mut DO: [HSD_DObj; 4] = unsafe { core::mem::zeroed() };
static mut MO: [HSD_MObj; 4] = unsafe { core::mem::zeroed() };
static mut TO: [HSD_TObj; 8] = unsafe { core::mem::zeroed() };
static mut TV: [HSD_TObjTev; 8] = unsafe { core::mem::zeroed() };
static mut GEN: [HSD_Generator; 3] = unsafe { core::mem::zeroed() };
static mut APS: [HSD_psAppSRT; 3] = unsafe { core::mem::zeroed() };
static mut PT: [HSD_Particle; 2] = unsafe { core::mem::zeroed() };
static mut PROC: [HSD_GObjProc; 2] = unsafe { core::mem::zeroed() };
static mut DESC: [EF_EffectDesc; 64] = unsafe { core::mem::zeroed() };
static mut DE: [EF_DAT_Entry; 2] = unsafe { core::mem::zeroed() };
static mut DEINIT: [EF_DAT_Entry; 65] = unsafe { core::mem::zeroed() };
static mut BHV: ea::EfMexBehavior = unsafe { core::mem::zeroed() };
static mut BT1: [u8; 16] = [0; 16];
static mut BT2: [u8; 16] = [0; 16];
static STRS: [[u8; 24]; 3] = [*b"MexA.dat\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0", *b"MexB.dat\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0", *b"MexC.dat\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"];
static mut ARC: [HSD_Archive; 1] = unsafe { core::mem::zeroed() };
static mut PLH: [*mut HSD_GObj; 64] = [null_mut(); 64];
static mut V3: [Vec3; 6] = unsafe { core::mem::zeroed() };
static mut FL: [f32; 6] = [0.0; 6];
static mut CMD: [_HSD_PSCmdList; 5] = unsafe { core::mem::zeroed() };
static mut PREF: [*mut _HSD_PSCmdList; 0x220] = [null_mut(); 0x220];
static mut AJ: [u8; 4] = [0; 4];
static mut EFN: usize = 0;
static mut QN: usize = 0;
static mut OUTI: [i32; 4] = [0; 4];
static mut OUTP: [*mut HSD_GObj; 2] = [null_mut(); 2];
static DID: [i32; @ND@] = [@DIDS@];
static DKIND: [&str; @ND@] = [@DKINDS_R@];
fn is_d(g: i32) -> bool { DID.contains(&g) }
static SPLIST: [i32; @NSPL@] = [@SPL@];
static UPD: [unsafe extern "C" fn(*mut EF_Effect); @NUPD@] = [@UPDR@];

fn rnn() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 100.0, 1000.0];
    let a = ((rnd() % 41) as i32 - 20) as f32 / 20.0;
    a * K[(rnd() % 4) as usize]
}
fn gfx() -> i32 {
    let r = rnd();
    let bank = (rnd() % 48) as i32;
    let id = (rnd() % 64) as i32;
    if r % 24 == 0 { return (fv() % 70) as i32 * 1000 + id; }
    if r % 24 == 1 { return 65000 + id; }
    bank * 1000 + id
}
fn gg() -> i32 {
    let r = rnd();
    const SP: [i32; 5] = [0x121, 0xFF, 0x7918, 0xFC, 0xF7];
    if r % 3 == 0 { return SP[(rnd() % 5) as usize]; }
    gfx()
}
fn spg() -> i32 {
    let r = rnd();
    if r % 6 == 0 { return gfx(); }
    SPLIST[(rnd() % SPLIST.len() as u32) as usize]
}
unsafe fn mxg() -> *mut HSD_GObj {
    let r = rnd() % 6;
    if r == 0 { return null_mut(); }
    if r == 1 { return &raw mut GO[8]; }
    &raw mut GO[6 + (rnd() & 1) as usize]
}
fn mxid() -> i32 {
    let r = rnd() % 8;
    if r == 0 { return (rnd() % 5000) as i32; }
    if r == 1 { return 5000 + 1000 * (rnd() % 5) as i32; }
    5000 + (rnd() % 4100) as i32
}
unsafe fn argp(kind: i32) -> *const c_void {
    match kind {
        0 => &raw mut V3[(rnd() % 6) as usize] as *const c_void,
        1 => &raw mut FL[(rnd() % 6) as usize] as *const c_void,
        _ => &raw mut JO[(rnd() % 16) as usize] as *const c_void,
    }
}
unsafe fn argd(d: usize, i: usize) -> *const c_void {
    match DKIND[d].as_bytes()[i] {
        b'V' => &raw mut V3[(rnd() % 6) as usize] as *const c_void,
        b'F' => &raw mut FL[(rnd() % 6) as usize] as *const c_void,
        b'J' => &raw mut JO[(rnd() % 16) as usize] as *const c_void,
        b'U' => fv().wrapping_mul(0x10101) as usize as *const c_void,
        _ => &raw mut V3[(rnd() % 6) as usize] as *const c_void,
    }
}

unsafe fn fill_fighter(p: &mut Fighter) {
    p.facing_dir = rg(); p.motion_id = fv() as _; p.cmd_vars[0] = fv(); p.cmd_vars[1] = fv(); p.cmd_vars[2] = fv(); p.cmd_vars[3] = fv();
    p.mv.co.common.x4.x = rg(); p.mv.co.common.x4.y = rg(); p.mv.co.common.x4.z = rg();
    p.mv.co.common.x10 = if rnd() & 1 != 0 { &raw mut AJ[0] as *mut c_void } else { null_mut() };
    p.coll_data.floor.normal.x = rg(); p.coll_data.floor.normal.y = rg();
    p.u.kb.hat.kind = fv() as _; p.kind = fv() as _;
}
unsafe fn hash_fighter(p: &Fighter) {
    hm(p.facing_dir.to_bits()); hm(p.motion_id as u32); hm(p.cmd_vars[0]); hm(p.cmd_vars[1]); hm(p.cmd_vars[2]); hm(p.cmd_vars[3]);
    hm(p.mv.co.common.x4.x.to_bits()); hm(p.mv.co.common.x4.y.to_bits()); hm(p.mv.co.common.x4.z.to_bits()); hm(pi(p.mv.co.common.x10));
    hm(p.coll_data.floor.normal.x.to_bits()); hm(p.coll_data.floor.normal.y.to_bits()); hm(p.u.kb.hat.kind as u32); hm(p.kind as u32);
}

unsafe fn setup() {
    EFF = core::mem::zeroed(); QE = core::mem::zeroed(); QH = core::mem::zeroed(); GO = core::mem::zeroed(); JO = core::mem::zeroed();
    FT = core::mem::zeroed(); ITM = core::mem::zeroed(); DO = core::mem::zeroed(); MO = core::mem::zeroed(); TO = core::mem::zeroed();
    TV = core::mem::zeroed(); GEN = core::mem::zeroed(); APS = core::mem::zeroed(); PT = core::mem::zeroed(); PROC = core::mem::zeroed();
    DESC = core::mem::zeroed(); DE = core::mem::zeroed(); PLH = [null_mut(); 64]; V3 = core::mem::zeroed(); FL = [0.0; 6];
    OUTI = [0; 4]; OUTP = [null_mut(); 2]; ARC = core::mem::zeroed(); CMD = core::mem::zeroed();
    EFN = 6; QN = 4;
    for i in 0..9 {
        g::fill_HSD_GObj(&mut GO[i]); GO[i].hsd_obj = &raw mut JO[i] as *mut c_void; GO[i].gx_link = if rnd() & 1 != 0 { 7 } else { 8 };
        GO[i].obj_kind = if rnd() % 3 == 0 { rnd() as u8 } else { 2 };
    }
    for i in 0..16 {
        g::fill_HSD_JObj(&mut JO[i]); JO[i].flags &= !(1u32 << 17);
        if rnd() & 1 != 0 { JO[i].flags |= 1 << 6; }
        if rnd() % 4 == 0 { JO[i].flags |= 1 << 25; }
    }
    fill_fighter(&mut FT[0]); fill_fighter(&mut FT[1]);
    for i in 0..10 { g::fill_EF_Effect(&mut EFF[i]); }
    for i in 0..8 { g::fill_EF_QueuedEffect(&mut QE[i]); }
    g::fill_EF_QueuedEffect(&mut QH[0]);
    for i in 0..4 { g::fill_HSD_DObj(&mut DO[i]); g::fill_HSD_MObj(&mut MO[i]); }
    for i in 0..8 { g::fill_HSD_TObj(&mut TO[i]); g::fill_HSD_TObjTev(&mut TV[i]); }
    for i in 0..3 { g::fill_HSD_Generator(&mut GEN[i]); g::fill_HSD_psAppSRT(&mut APS[i]); }
    for i in 0..2 { g::fill_HSD_Particle(&mut PT[i]); g::fill_HSD_GObjProc(&mut PROC[i]); g::fill_EF_DAT_Entry(&mut DE[i]); }
    for i in 0..64 { g::fill_EF_EffectDesc(&mut DESC[i]); }
    for i in 0..6 { V3[i].x = rnn(); V3[i].y = rnn(); V3[i].z = rnn(); FL[i] = rnn(); }
    for i in 0..16 { BT1[i] = (rnd() % 9) as u8; BT2[i] = (rnd() % 9) as u8; }
    BHV.mdl_num = (rnd() % 12) as i32; BHV.mdl_type = BT1.as_mut_ptr(); BHV.ptcl_num = (rnd() % 12) as i32; BHV.ptcl_type = BT2.as_mut_ptr();
    for i in 0..6 {
        GO[i].user_data = &raw mut EFF[i] as *mut c_void; EFF[i].gobj = &raw mut GO[i]; EFF[i].parent_gobj = &raw mut GO[6 + (rnd() & 1) as usize];
        EFF[i].attach_jobj = if rnd() % 4 != 0 { &raw mut JO[8 + (rnd() % 8) as usize] } else { null_mut() };
        EFF[i].update = if rnd() % 4 != 0 { Some(UPD[(rnd() % UPD.len() as u32) as usize]) } else { None };
        EFF[i].user_data = if rnd() % 3 != 0 { &raw mut JO[(rnd() % 16) as usize] as *mut c_void } else { null_mut() };
        EFF[i].next = &raw mut EFF[(rnd() % 6) as usize];
        if rnd() % 4 == 0 { EFF[i].lifetime = 1; }
        EFF[i].state_flags = if rnd() % 3 == 0 { (rnd() % 3) as u8 } else { 0 };
        EFF[i].scale_flags = (rnd() % 3) as u8;
        EFF[i].is_async = (rnd() & 1) as u8;
    }
    GO[6].user_data = &raw mut FT[0] as *mut c_void; GO[7].user_data = &raw mut FT[1] as *mut c_void; GO[6].classifier = 4; GO[7].classifier = 4;
    GO[8].classifier = 6; GO[8].user_data = &raw mut ITM[0] as *mut c_void;
    ITM[0].mex_original_owner = if rnd() & 3 != 0 { &raw mut GO[6 + (rnd() & 1) as usize] } else { null_mut() };
    for i in 0..2 {
        let mm = rnd() % 5;
        if mm == 0 { FT[i].motion_id = 349; } else if mm == 1 { FT[i].motion_id = 357; } else if mm == 2 { FT[i].motion_id = 359; }
        FT[i].kind = if rnd() % 3 == 0 { 4 } else { (rnd() % 127) as _ };
    }
    for i in 1..16usize { JO[i].parent = if rnd() & 1 != 0 { &raw mut JO[(rnd() % i as u32) as usize] } else { null_mut() }; }
    for i in 0..6 { JO[i].child = &raw mut JO[8 + (rnd() & 1) as usize]; }
    for i in 8..15 { JO[i].next = &raw mut JO[i + 1]; }
    JO[6].child = if rnd() & 1 != 0 { &raw mut JO[10] } else { null_mut() };
    JO[7].child = if rnd() & 1 != 0 { &raw mut JO[11] } else { null_mut() };
    JO[8].child = if rnd() & 1 != 0 { &raw mut JO[12] } else { null_mut() };
    for i in 0..4 { DO[i].mobj = if rnd() % 4 != 0 { &raw mut MO[i] } else { null_mut() }; }
    for i in 0..8 { TO[i].next = &raw mut TO[(i + 1) % 8]; TO[i].tev = &raw mut TV[i]; }
    for i in 0..3 {
        GEN[i].appsrt = if rnd() & 1 != 0 { &raw mut APS[(rnd() % 3) as usize] } else { null_mut() };
        GEN[i].cmdList = if rnd() % 3 != 0 { CMD[(rnd() % 5) as usize].cmdList.as_mut_ptr() } else { null_mut() };
    }
    for i in 0..2 {
        PT[i].appsrt = if rnd() & 1 != 0 { &raw mut APS[(rnd() % 3) as usize] } else { null_mut() };
        PROC[i].s_link = (rnd() % 12) as u8;
    }
    for i in 0..0x220 { PREF[i] = &raw mut CMD[4]; }
    PREF[0x96] = &raw mut CMD[0]; PREF[0x97] = &raw mut CMD[1]; PREF[0x98] = &raw mut CMD[2]; PREF[0x21B] = &raw mut CMD[3];
    for i in 0..65 { el::ext::ptclref_804D0E5C[i] = null_mut(); }
    el::ext::ptclref_804D0E5C[0] = PREF.as_mut_ptr();
    for i in 0..64 {
        DESC[i].model_desc.joint = if rnd() % 4 != 0 { &raw mut AJ[0] as *mut HSD_Joint } else { null_mut() };
        DESC[i].model_desc.animjoint = if rnd() & 1 != 0 { &raw mut AJ[1] as *mut HSD_AnimJoint } else { null_mut() };
        DESC[i].model_desc.matanim_joint = if rnd() % 4 == 0 { &raw mut AJ[2] as *mut HSD_MatAnimJoint } else { null_mut() };
        DESC[i].model_desc.shapeanim_joint = if rnd() % 4 == 0 { &raw mut AJ[3] as *mut HSD_ShapeAnimJoint } else { null_mut() };
        if rnd() & 1 != 0 { DESC[i].lifetime = (rnd() % 700) as f32 / 10.0; }
    }
    for i in 0..2 {
        DE[i].ef_DAT_file = if rnd() & 1 != 0 { STRS[0].as_ptr() as *mut i8 } else { null_mut() };
        DE[i].effDataTable_name = if rnd() & 1 != 0 { STRS[1].as_ptr() as *mut i8 } else { null_mut() };
    }
    for i in 0..65 {
        ea::efAsync_DatEntries[i] = core::ptr::read(&raw const DEINIT[i]);
        ea::efAsync_DatEntries[i].data = if rnd() % 8 != 0 { DESC.as_mut_ptr() as *mut c_void } else { null_mut() };
        ea::efAsync_MexBhv[i] = if rnd() & 1 != 0 { &raw mut BHV } else { null_mut() };
    }
    ea::efAsync_MexReady = (rnd() & 1) as i32;
    for i in 0..128 { ea::ext::ftData_UnkBytePerCharacter[i] = if rnd() % 6 == 0 { 0xFF } else { (rnd() % 70) as u8 }; }
    ed::efLib_EffectCount = rnd() % 64; ed::efLib_LoadKind = rnd() & 1; ed::efLib_AnimCount = (rnd() % 8) as i32;
    el::efLib_AnimQueue = core::mem::zeroed();
    for i in 0..ed::efLib_AnimCount as usize { *(el::efLib_AnimQueue.as_mut_ptr() as *mut *mut HSD_JObj).add(i) = &raw mut JO[(rnd() % 16) as usize]; }
    for i in 0..8 {
        g::fill_EF_ParamEntry(&mut el::efLib_ParamTable[i]);
        el::efLib_ParamTable[i].gobj = if rnd() & 3 != 0 { &raw mut GO[(rnd() % 8) as usize] as *mut c_void } else { null_mut() };
        let j = rnd() % 4;
        if j == 0 { el::efLib_ParamTable[i].gfx_id = 0x417; } else if j == 1 { el::efLib_ParamTable[i].gfx_id = 0x419; }
    }
    el::ext::hsd_804D7900 = None;
    el::ext::HSD_GObj_JObjKind = 2;
    ea::ext::HSD_GObj_CurrentInvokedProc = if rnd() & 1 != 0 { &raw mut PROC[0] } else { null_mut() };
    el::ext::HSD_GObjPLinkHead = PLH.as_mut_ptr();
    let mut ord = [0usize, 1, 2, 3, 4, 5];
    for i in (1..6usize).rev() { let j = (rnd() % (i as u32 + 1)) as usize; ord.swap(i, j); }
    let split = (rnd() % 7) as usize;
    PLH[11] = if split != 0 { &raw mut GO[ord[0]] } else { null_mut() };
    for i in 0..split.saturating_sub(1) { GO[ord[i]].next = &raw mut GO[ord[i + 1]]; }
    PLH[12] = if split < 6 { &raw mut GO[ord[split]] } else { null_mut() };
    for i in split..5 { GO[ord[i]].next = &raw mut GO[ord[i + 1]]; }
    for i in 0..4 {
        QE[i].spawn_kind = (rnd() % 10) as u8; QE[i].gfx_id = gfx();
        QE[i].jobj = &raw mut JO[8 + (rnd() % 8) as usize];
        if QE[i].spawn_kind == 9 { QE[i].gfx_id = 5000 + (rnd() % 4100) as i32; }
        QE[i].next = if i < 3 { &raw mut QE[i + 1] } else { null_mut() };
    }
    QH[0].next = if rnd() % 4 != 0 { &raw mut QE[0] } else { null_mut() };
}

unsafe fn hash_all() {
    for i in 0..10 { g::hash_EF_Effect(&EFF[i]); }
    for i in 0..8 { g::hash_EF_QueuedEffect(&QE[i]); }
    g::hash_EF_QueuedEffect(&QH[0]);
    for i in 0..9 { g::hash_HSD_GObj(&GO[i]); }
    for i in 0..16 { g::hash_HSD_JObj(&JO[i]); }
    hash_fighter(&FT[0]); hash_fighter(&FT[1]);
    for i in 0..4 { g::hash_HSD_DObj(&DO[i]); g::hash_HSD_MObj(&MO[i]); }
    for i in 0..8 { g::hash_HSD_TObj(&TO[i]); g::hash_HSD_TObjTev(&TV[i]); }
    for i in 0..3 { g::hash_HSD_Generator(&GEN[i]); g::hash_HSD_psAppSRT(&APS[i]); }
    for i in 0..2 { g::hash_HSD_Particle(&PT[i]); g::hash_HSD_GObjProc(&PROC[i]); g::hash_EF_DAT_Entry(&DE[i]); }
    for i in 0..6 { hm(V3[i].x.to_bits()); hm(V3[i].y.to_bits()); hm(V3[i].z.to_bits()); hm(FL[i].to_bits()); }
    hm(EFN as u32); hm(QN as u32);
    hm(ed::efLib_EffectCount); hm(ed::efLib_LoadKind); hm(ed::efLib_AnimCount as u32);
    for i in 0..16 { hm(pi(*(el::efLib_AnimQueue.as_ptr() as *const *const c_void).add(i))); }
    for i in 0..8 { g::hash_EF_ParamEntry(&el::efLib_ParamTable[i]); }
    for i in 0..3 { hm(fi(el::lbl_803BF810[i])); }
    hm(fi(el::ext::hsd_804D7900));
    for i in 0..65 {
        hm(sh(ea::efAsync_DatEntries[i].ef_DAT_file as *const u8)); hm(sh(ea::efAsync_DatEntries[i].effDataTable_name as *const u8));
        hm(pi(ea::efAsync_DatEntries[i].data)); hm(pi(ea::efAsync_MexBhv[i]));
    }
    hm(ea::efAsync_MexReady as u32);
    hm(OUTI[2] as u32);
}

#[test]
fn ef_matches_c() {
    unsafe {
        std::panic::set_hook(Box::new(|i| { let m = i.to_string(); eprintln!("{}", m); }));
        g::install();
        m::efn::eflib::ext::hook::OSReport = Some(|_| {});
        m::efn::efasync::ext::hook::OSReport = Some(|_| {});
        for i in 0..65 { DEINIT[i] = core::ptr::read(&raw const ea::efAsync_DatEntries[i]); }
        preg(&raw const EFF, core::mem::size_of_val(&EFF)); preg(&raw const QE, core::mem::size_of_val(&QE)); preg(&raw const QH, core::mem::size_of_val(&QH));
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const FT, core::mem::size_of_val(&FT));
        preg(&raw const ITM, core::mem::size_of_val(&ITM)); preg(&raw const DO, core::mem::size_of_val(&DO)); preg(&raw const MO, core::mem::size_of_val(&MO));
        preg(&raw const TO, core::mem::size_of_val(&TO)); preg(&raw const TV, core::mem::size_of_val(&TV)); preg(&raw const GEN, core::mem::size_of_val(&GEN));
        preg(&raw const APS, core::mem::size_of_val(&APS)); preg(&raw const PT, core::mem::size_of_val(&PT)); preg(&raw const PROC, core::mem::size_of_val(&PROC));
        preg(&raw const DESC, core::mem::size_of_val(&DESC)); preg(&raw const DE, core::mem::size_of_val(&DE));
        preg(&raw const BHV, core::mem::size_of_val(&BHV)); preg(&raw const ARC, core::mem::size_of_val(&ARC)); preg(&raw const PLH, core::mem::size_of_val(&PLH));
        preg(&raw const V3, core::mem::size_of_val(&V3)); preg(&raw const FL, core::mem::size_of_val(&FL));
        preg(&raw const CMD, core::mem::size_of_val(&CMD)); preg(&raw const PREF, core::mem::size_of_val(&PREF)); preg(&raw const AJ, core::mem::size_of_val(&AJ));
        preg(STRS.as_ptr(), core::mem::size_of_val(&STRS)); preg(&raw const BT1, 16); preg(&raw const BT2, 16);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ef.txt"));
        for op in 0..@NOPS@u32 {
            for k in 0..@CASES@u32 {
                case(op, k);
                if std::env::var("MTU_TRACE_CASE").is_ok() { eprintln!("case {} {}", op, k); }
                setup();
                let mut ret = 0u32;
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match op {
@ROPS@
                    _ => unreachable!(),
                }
                }));
                if let Err(e) = r {
                    let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    if !(msg.contains(": assertion failed") || msg.contains("lineID") || msg.contains("__assert")) { std::panic::resume_unwind(e); }
                    hm(0xA55E87);
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), @NOPS@ * @CASES@);
    }
}
'''


def fill(s, d):
    for a, b in d.items():
        s = s.replace(a, b)
    return s


def gen():
    from specs_ef import SPECS
    import gen_tu
    gen_tu.build("ef", SPECS["ef"], False)
    cops = "\n".join(f"            case {i}: {{ {c} }} break;" for i, (n, c, r, kd) in enumerate(OPS))
    rops = "\n".join(f"                    {i} => {{ {r} }}" for i, (n, c, r, kd) in enumerate(OPS))
    ccb = ", ".join(f"efLib_Cb_{c}" for c in CBS)
    rcb = ", ".join(f"el::efLib_Cb_{c}" for c in CBS)
    common = {"@NOPS@": str(len(OPS)), "@CASES@": str(CASES), "@DIDS@": ", ".join(str(x) for x in DIDS),
              "@SPL@": ", ".join(str(x) for x in SPL), "@ND@": str(ND), "@NSPL@": str(len(SPL)), "@NUPD@": str(len(CBS)),
              "@UIDS@": ", ".join(str(i) for i, k in zip(DIDS, DKINDS) if "U" in k) or "-1", "@NU@": str(max(1, sum(1 for k in DKINDS if "U" in k)))}
    c = fill(C_HEAD.replace("@COPS@", cops), common)
    c = c.replace("@DKINDS@", ", ".join(f'"{k}"' for k in DKINDS)).replace("@UPDC@", ccb)
    r = fill(R_HEAD.replace("@ROPS@", rops), common)
    r = r.replace("@DKINDS_R@", ", ".join(f'"{k}"' for k in DKINDS)).replace("@UPDR@", rcb)
    (HERE / "tu" / "ef.c").write_text(c, encoding="utf-8", newline="\n")
    (ROOT / "rust/melee_world/tests/tu_ef.rs").write_text(r, encoding="utf-8", newline="\n")
    print(f"mk_ef: {len(OPS)} ops x {CASES} cases, {ND} Dispatch ids")


LINK = ["pc/gameworld/gekko_fp.c"]


def run():
    import run_tu
    BUILD = run_tu.BUILD
    BUILD.mkdir(parents=True, exist_ok=True)
    from specs_ef import SPECS
    import gen_tu
    spec = SPECS["ef"]
    src = gen_tu.patched_source("ef", spec)
    drv = HERE / "tu" / "ef.c"
    exe = BUILD / "mtu_ef.exe"
    lls = []
    if os.environ.get("MTU_DBG"):
        dop, dk = os.environ["MTU_DBG"].split(":")
        dbg = [f"-DDBG_OP={dop}", f"-DDBG_K={dk}"]
    else:
        dbg = ["-DDBG_OP=-1", "-DDBG_K=-1"]
    for s in [drv] + [ROOT / x for x in LINK]:
        ll = BUILD / f"mtu_ef__{s.stem}.ll"
        lo = BUILD / f"mtu_ef__{s.stem}.lo.ll"
        run_tu.run([run_tu.CLANG, *run_tu.FLAGS, "-fms-extensions", "-I", str((ROOT / spec["src"]).parent), "-iquote", "src/melee/ef", "-I", str(HERE / "tu"),
                    '-DTU_SRC="' + str(ROOT / src).replace("\\", "/") + '"', *dbg, str(s), "-o", str(ll)], "ef")
        run_tu.run([sys.executable, str(run_tu.LOWER), str(ll), str(lo)], "ef")
        lls.append(str(lo))
    run_tu.run([run_tu.CLANG, "-O0", "-w", *lls, "-o", str(exe), "-Wl,/OPT:NOICF", "-Wl,/FORCE:UNRESOLVED", "-fuse-ld=lld"], "ef")
    r_ = subprocess.run([str(exe)], capture_output=True, text=True)
    out = r_.stdout
    if os.environ.get("MTU_STAT"):
        print(r_.stderr)
    if os.environ.get("MTU_DBG"):
        tr = [l for l in out.splitlines() if l.startswith(("T ", "H ", "B ", "ENT", "DBG"))]
        (BUILD / "dbg_ef.txt").write_text(chr(10).join(tr) + chr(10), encoding="utf-8")
        print(f"mk_ef: {len(tr)} trace words -> {BUILD / 'dbg_ef.txt'}")
        return
    if r_.returncode:
        print(f"mk_ef: exe exit code {r_.returncode} after {len(out.splitlines())} lines")
    dd = ROOT / "rust/melee_world/tests/data"
    (dd / "mtu_ef.txt").write_text(out, encoding="utf-8", newline="\n")
    print(f"mk_ef: {len(out.splitlines())} lines")


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode in ("gen", "all"):
        gen()
    if mode in ("run", "all"):
        run()
