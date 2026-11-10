"""particle.c spec: particle spawn (psGenerateParticle0 / hsd_80398F0C), the per-frame update of a particle without a command list (timers, life, physics,
tornado, delete) hsd_8039930C, the rotate helper hsd_80398F8C and the generator teardown hsd_8039D0A0. Particles carry no command list here: the
bytecode interpreter's opcode paths need the tex / cmd group tables and are not driven."""
from lbspecs import add

POOLS = [('HSD_Particle', 'P0', 4), ('HSD_Generator', 'GEN', 2), ('HSD_psAppSRT', 'SRT', 2), ('_PSUserFunc', 'UF', 2), ('HSD_JObj', 'J', 2)]
ONLY = ["psGenerateParticle0", "hsd_80398F0C", "hsd_80398F8C", "hsd_8039930C", "hsd_8039D0A0"]
OWN = ["numActiveParticles", "numPeakParticles", "psPointJObj", "hsd_804D0908", "hsd_804D0F60"]
FILL = ["numActiveParticles", "numPeakParticles"]
C_DECL = r'''
static int cb_hook(HSD_Particle* p) { T(8401); T(PI(p)); return (int) (fv() % 3) - 1; }
static int cb_part(HSD_Particle* p) { T(8402); T(PI(p)); return (fv() & 1) ? -1 : 0; }
'''
RS_EXTRA = r'''
unsafe extern "C" fn cb_hook(p: *mut HSD_Particle) -> i32 { t(8401); t(pi(p)); (fv() % 3) as i32 - 1 }
unsafe extern "C" fn cb_part(p: *mut HSD_Particle) -> i32 { t(8402); t(pi(p)); if fv() & 1 != 0 { -1 } else { 0 } }
'''
_SETUP_C = r'''{ int q;
 for (q = 0; q < 2; q++) { UF[q].hookCreate = (fv() & 1) ? cb_hook : 0; UF[q].hookDelete = (fv() & 1) ? cb_hook : 0; }
 for (q = 0; q < 2; q++) { GEN[q].userfunc = (fv() & 1) ? &UF[fv() & 1] : 0; GEN[q].appsrt = (fv() & 1) ? &SRT[fv() & 1] : 0; GEN[q].linkNo = (u8) (fv() % 16); GEN[q].idnum = (u16) (fv() % 3); GEN[q].numChild = fv() % 8; }
 for (q = 0; q < 4; q++) { P0[q].next = (q < 3 && (fv() & 1)) ? &P0[q + 1] : 0; P0[q].gen = &GEN[fv() & 1]; P0[q].appsrt = (fv() & 1) ? &SRT[fv() & 1] : 0; P0[q].idnum = (u16) (fv() % 3); P0[q].linkNo = (u8) (fv() % 16);
   P0[q].cmdList = 0; P0[q].cmdWait = 0; P0[q].life = (u16) (1 + fv() % 3); P0[q].kind = (fv() ^ (fv() << 9)) & ~0x8800u; P0[q].callback = (fv() & 1) ? cb_part : 0;
   if (P0[q].sizeCount > 100) P0[q].sizeCount = 0; if (P0[q].rotateCount > 100) P0[q].rotateCount = 0; }
 for (q = 0; q < 16; q++) hsd_804D0908[q] = (fv() & 1) ? &P0[fv() & 3] : 0; for (q = 0; q < 8; q++) psPointJObj[q] = (fv() & 1) ? &J[fv() & 1] : 0; }'''.replace("\n", " ")
_SETUP_R = r'''{
 for q in 0..2usize { UF[q].hookCreate = if fv() & 1 != 0 { Some(cb_hook) } else { None }; UF[q].hookDelete = if fv() & 1 != 0 { Some(cb_hook) } else { None }; }
 for q in 0..2usize { GEN[q].userfunc = if fv() & 1 != 0 { &raw mut UF[(fv() & 1) as usize] } else { null_mut() }; GEN[q].appsrt = if fv() & 1 != 0 { &raw mut SRT[(fv() & 1) as usize] } else { null_mut() }; GEN[q].linkNo = (fv() % 16) as u8; GEN[q].idnum = (fv() % 3) as u16; GEN[q].numChild = fv() % 8; }
 for q in 0..4usize { P0[q].next = if q < 3 && fv() & 1 != 0 { &raw mut P0[q + 1] } else { null_mut() }; P0[q].r#gen = &raw mut GEN[(fv() & 1) as usize]; P0[q].appsrt = if fv() & 1 != 0 { &raw mut SRT[(fv() & 1) as usize] } else { null_mut() }; P0[q].idnum = (fv() % 3) as u16; P0[q].linkNo = (fv() % 16) as u8;
   P0[q].cmdList = null_mut(); P0[q].cmdWait = 0; P0[q].life = (1 + fv() % 3) as u16; P0[q].kind = (fv() ^ (fv() << 9)) & !0x8800u32; P0[q].callback = if fv() & 1 != 0 { Some(cb_part) } else { None };
   if P0[q].sizeCount > 100 { P0[q].sizeCount = 0; } if P0[q].rotateCount > 100 { P0[q].rotateCount = 0; } }
 for q in 0..16usize { m::hsd_804D0908[q] = if fv() & 1 != 0 { &raw mut P0[(fv() & 3) as usize] } else { null_mut() }; } for q in 0..8usize { m::psPointJObj[q] = if fv() & 1 != 0 { &raw mut J[(fv() & 1) as usize] } else { null_mut() }; } }'''.replace("\n", " ")
add("particle", "src/sysdolphin/baselib/particle.c", dict(
    pub_only=True, pools=POOLS, only=ONLY, ncases=400, setup=(_SETUP_C, _SETUP_R), c_decl=C_DECL, rs_extra=RS_EXTRA,
    ov={"psGenerateParticle0": {0: ("(fv() & 1) ? (HSD_Particle**) &PPA[0] : 0", "if fv() & 1 != 0 { (&raw mut PPA[0]) as *mut *mut HSD_Particle } else { null_mut() }"), 1: ("(int) (fv() % 16)", "(fv() % 16) as i32"),
                                 2: ("(int) (fv() % 4)", "(fv() % 4) as i32"), 3: ("fv() & ~0x8804u", "fv() & !0x8804u32"), 5: ("(u8*) 0", "null_mut()"), 17: ("picko_HSD_Generator()", "picko_HSD_Generator()"), 18: ("(int) (fv() & 1)", "(fv() & 1) as i32")},
        "hsd_80398F0C": {0: ("(s32) (fv() % 16)", "(fv() % 16) as i32"), 1: ("(s32) (fv() % 4)", "(fv() % 4) as i32"), 2: ("(s32) (fv() & ~0x8804u)", "(fv() & !0x8804u32) as i32"), 4: ("(s32) 0", "null_mut()"), 7: ("(s32) 0", "null_mut()")},
        "hsd_8039930C": {0: ("&P0[fv() & 3]", "&raw mut P0[(fv() & 3) as usize]"), 1: ("(fv() & 1) ? &P0[fv() & 3] : 0", "if fv() & 1 != 0 { &raw mut P0[(fv() & 3) as usize] } else { null_mut() }")},
        "hsd_8039D0A0": {0: ("&GEN[fv() & 1]", "&raw mut GEN[(fv() & 1) as usize]")}},
    pregs=("freg((const void*) cb_hook); freg((const void*) cb_part);", "freg(cb_hook as usize); freg(cb_part as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());")),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=FILL, own_hash=OWN, own_only=ONLY,
    skip_stubs=["memcpy", "memset", "sqrtf", "__frsqrte", "sinf", "cosf", "tanf"],
    ret={"atan2f": ("rw()", "rw()"), "HSD_ObjAlloc": ("(fv() % 5) ? (void*) pick_HSD_Particle() : 0", "if fv() % 5 != 0 { pick_HSD_Particle() as *mut c_void } else { null_mut() }"), "psGetNewIDNum": ("(u16) (fv() % 3)", "(fv() % 3) as u16"),
         "psAttachParticleAppSRT": ("(int) (fv() & 1)", "(fv() & 1) as i32"), "psRemoveParticleAppSRT": ("(int) (fv() & 1)", "(fv() & 1) as i32"), "HSD_Randf": ("rw()", "rw()"),
         "HSD_JObjAlloc": ("pick_HSD_JObj()", "pick_HSD_JObj()")})
