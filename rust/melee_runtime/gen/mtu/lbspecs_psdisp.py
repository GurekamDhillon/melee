"""psdisp.c spec: particle display (tornado trail position, colour interpolation, channel / TEV register setup, the particle sort over the per-link
particle lists). The polygon / point drawing paths (psDispSub*, psDispSubAppSRT) need tex-group fixtures and are not driven here."""
from lbspecs import add

POOLS = [('HSD_Particle', 'P0', 4), ('HSD_Generator', 'GEN', 2), ('GXColor', 'P1', 2), ('HSD_psAppSRT', 'SRT', 2)]
ONLY = ["calcTornadoLastPos", "getColorPrimEnv", "setupChanCtrl", "setupChanReg", "setupTevReg", "particleSort", "psDispSub", "psDispSubMakePolygon", "psDispSubAppSRT"]
OWN = ["vmtx", "rvmtx", "pvmtx", "prj", "HSD_PSDisp_804D7914", "HSD_PSDisp_804D7918", "HSD_PSDisp_804D791C", "HSD_PSDisp_804D7920", "HSD_PSDisp_804D7924", "HSD_PSDisp_804D7928", "prevChanCtrl", "prevChanMat", "prevChanAmb", "prevColorPrim", "prevColorEnv", "prevColorMat", "prevPointSize", "prevLineWidth", "particle_list", "psFrameNum",
       "HSD_PSDisp_8040C360", "HSD_PSDisp_804D7948"]
FILL = ["vmtx", "rvmtx", "pvmtx", "prj", "HSD_PSDisp_804D7914", "HSD_PSDisp_804D7918", "HSD_PSDisp_804D791C", "HSD_PSDisp_804D7920", "HSD_PSDisp_804D7924", "HSD_PSDisp_804D7928", "prevChanCtrl", "prevChanMat", "prevChanAmb", "prevColorPrim", "prevColorEnv", "prevColorMat"]
_SETUP_C = r'''{ int q;
 for (q = 0; q < 4; q++) { P0[q].next = (q < 3 && (fv() & 1)) ? &P0[q + 1] : 0; P0[q].gen = &GEN[fv() & 1]; P0[q].appsrt = &SRT[fv() & 1];
   P0[q].kind = fv() ^ (fv() << 13) ^ (fv() << 26); if (fv() & 1) { P0[q].primColCount = (u16) (1 + fv() % 100); P0[q].primColRemain = (u16) (fv() % 101); } else P0[q].primColCount = 0;
   if (fv() & 1) { P0[q].envColCount = (u16) (1 + fv() % 100); P0[q].envColRemain = (u16) (fv() % 101); } else P0[q].envColCount = 0;
   if (fv() & 1) { P0[q].matColCount = (u16) (1 + fv() % 100); P0[q].matColRemain = (u16) (fv() % 101); } else P0[q].matColCount = 0;
   if (fv() & 1) { P0[q].ambColCount = (u16) (1 + fv() % 100); P0[q].ambColRemain = (u16) (fv() % 101); } else P0[q].ambColCount = 0; }
 for (q = 0; q < 16; q++) hsd_804D0908[q] = (fv() & 1) ? &P0[fv() & 3] : 0; for (q = 0; q < 17; q++) particle_list[q] = (fv() & 1) ? &P0[fv() & 3] : 0; }'''.replace("\n", " ")
_SETUP_R = r'''{
 for q in 0..4usize { P0[q].next = if q < 3 && fv() & 1 != 0 { &raw mut P0[q + 1] } else { null_mut() }; P0[q].r#gen = &raw mut GEN[(fv() & 1) as usize]; P0[q].appsrt = &raw mut SRT[(fv() & 1) as usize];
   P0[q].kind = fv() ^ (fv() << 13) ^ (fv() << 26); if fv() & 1 != 0 { P0[q].primColCount = (1 + fv() % 100) as u16; P0[q].primColRemain = (fv() % 101) as u16; } else { P0[q].primColCount = 0; }
   if fv() & 1 != 0 { P0[q].envColCount = (1 + fv() % 100) as u16; P0[q].envColRemain = (fv() % 101) as u16; } else { P0[q].envColCount = 0; }
   if fv() & 1 != 0 { P0[q].matColCount = (1 + fv() % 100) as u16; P0[q].matColRemain = (fv() % 101) as u16; } else { P0[q].matColCount = 0; }
   if fv() & 1 != 0 { P0[q].ambColCount = (1 + fv() % 100) as u16; P0[q].ambColRemain = (fv() % 101) as u16; } else { P0[q].ambColCount = 0; } }
 for q in 0..16usize { m::ext::hsd_804D0908[q] = if fv() & 1 != 0 { &raw mut P0[(fv() & 3) as usize] } else { null_mut() }; } for q in 0..17usize { m::particle_list[q] = if fv() & 1 != 0 { &raw mut P0[(fv() & 3) as usize] } else { null_mut() }; } }'''.replace("\n", " ")
add("psdisp", "src/sysdolphin/baselib/psdisp.c", dict(
    pub_only=True, pools=POOLS, only=ONLY, ncases=400, setup=(_SETUP_C, _SETUP_R),
    ov={"particleSort": {0: ("(s32) (fv() & 15)", "(fv() & 15) as i32"), 2: ("(HSD_Particle**) &PPA[0]", "(&raw mut PPA[0]) as *mut *mut HSD_Particle"), 3: ("(HSD_Particle**) &PPA[1]", "(&raw mut PPA[1]) as *mut *mut HSD_Particle")}},
    pregs=("", "melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());")),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=FILL, own_hash=OWN, own_only=ONLY,
    skip_stubs=["memcpy", "memset", "sqrtf", "__frsqrte", "sinf", "cosf", "tanf"],
    ret={"atan2f": ("rw()", "rw()")})
