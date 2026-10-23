"""Per-TU driver descriptions for mk_driver.py (see there)."""
STEMS = {}
IDX4 = ("fv() & 3", "(fv() & 3) as i32")
STEMS["ithitbox"] = dict(
    ov={"it_80275594": {1: IDX4}, "it_8027570C": {1: IDX4}, "it_8027572C": {1: IDX4}, "it_80275820": {3: IDX4}},
)

STEMS["itdrop"] = dict(
    c_extra="static struct it_8026F3D4_arg1_t A1;",
    rs_extra="static mut A1: it_8026F3D4_arg1_t = unsafe { core::mem::zeroed() };",
    pregs=("preg(&A1, sizeof A1);", "preg(&raw const A1, core::mem::size_of_val(&A1));"),
    setup=("zero(&A1, sizeof A1);", "A1 = core::mem::zeroed();"),
    hash=("HM((u32) A1.x0); HM((u32) A1.x4); HM(PI(A1.x8));", "hm(A1.x0 as u32); hm(A1.x4 as u32); hm(pi(A1.x8));"),
    ov={"it_8026F3D4": {1: ("((rnd() & 1) ? &A1 : 0)", "if rnd() & 1 != 0 { &raw mut A1 } else { null_mut() }"),
                        2: ("(s32) (fv() % 9)", "(fv() % 9) as i32")}},
    hash_ft_all=False,
)

import re as _re
from pathlib import Path as _P
from specs_ef import EF_PRE as _EF_PRE
_ids = sorted({int(x, 16) for x in _re.findall(r"0x([0-9A-Fa-f]{3,4})\b", (_P(__file__).resolve().parents[4] / "src/melee/it/iteffect.c").read_text(encoding="utf-8"))
               if 0x200 <= int(x, 16) <= 0x600} | {0x24F, 0x250, 0x251, 0x3FF, 0x400, 0x401, 0x1387, 0x1388, 0x1B57, 0x1B58, 0x2327, 0x2328})
_efids_c = "static const int EFIDS[%d] = { %s };" % (len(_ids), ", ".join(hex(i) for i in _ids))
_efids_r = "const EFIDS: [i32; %d] = [%s];" % (len(_ids), ", ".join(hex(i) for i in _ids))
EFID_C = "(int) ((fv() & 1) ? (u32) EFIDS[rnd() % %d] : (rnd() %% 0x540))".replace("%d", str(len(_ids))).replace("%%", "%")
EFID_R = "(if fv() & 1 != 0 { EFIDS[(rnd() %% %d) as usize] } else { (rnd() %% 0x540) as i32 })".replace("%d", str(len(_ids))).replace("%%", "%")
STEMS["iteffect"] = dict(
    c_pre=_EF_PRE,
    c_extra=_efids_c + "\nstatic it_804D6D20_t ARC;\nstatic Article* ARCA[3][4];\nstatic it_804D6D40_t ARC40;\nstatic Fighter_804D653C_t ARC53C[2];",
    rs_extra=_efids_r + "\npub static mut ARC: it_804D6D20_t = unsafe { core::mem::zeroed() };\nstatic mut ARCA: [[*mut Article; 4]; 3] = [[null_mut(); 4]; 3];\n"
             "static mut ARC40: it_804D6D40_t = unsafe { core::mem::zeroed() };\nstatic mut ARC53C: [Fighter_804D653C_t; 2] = unsafe { core::mem::zeroed() };",
    pregs=("preg(&ARC, sizeof ARC); preg(ARCA, sizeof ARCA); preg(&ARC40, sizeof ARC40); preg(ARC53C, sizeof ARC53C);",
           "preg(&raw const ARC, core::mem::size_of_val(&ARC)); preg(&raw const ARCA, core::mem::size_of_val(&ARCA)); "
           "preg(&raw const ARC40, core::mem::size_of_val(&ARC40)); preg(&raw const ARC53C, core::mem::size_of_val(&ARC53C));"),
    setup=("{ int j; for (j = 0; j < 15; j++) { it_803F1ED8[j] = 'a' + rnd() % 26; it_803F1EE4[j] = 'a' + rnd() % 26; it_803F1EF0[j] = 'a' + rnd() % 26; } it_803F1ED8[15] = it_803F1EE4[15] = it_803F1EF0[15] = 0; ARCP = &ARC; zero(&ARC, sizeof ARC); zero(&ARC40, sizeof ARC40); zero(ARC53C, sizeof ARC53C); ARC.x0 = &ICD[0];"
           " for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) ARCA[i][j] = pick_Article();"
           " ARC.x4 = ARCA[0]; ARC.x8 = ARCA[1]; ARC.xC = ARCA[2]; ARC.x10 = &ARC40; ARC.x14 = &ARC53C[rnd() & 1];"
           " ARC40.x0 = (s32) fv(); ARC40.x4 = rw(); ARC40.x8 = rw(); ARC40.xC = rw(); ARC40.x10 = rw(); ARC40.x14 = rw(); ARC40.x18 = rw(); }",
           "{ for j in 0..15 { m::ext::it_803F1ED8[j] = b'a' as i8 + (rnd() % 26) as i8; m::ext::it_803F1EE4[j] = b'a' as i8 + (rnd() % 26) as i8; m::ext::it_803F1EF0[j] = b'a' as i8 + (rnd() % 26) as i8; } m::ext::it_803F1ED8[15] = 0; m::ext::it_803F1EE4[15] = 0; m::ext::it_803F1EF0[15] = 0; ARC = core::mem::zeroed(); ARC40 = core::mem::zeroed(); ARC53C = core::mem::zeroed(); ARC.x0 = &raw mut ICD[0];"
           " for i in 0..3 { for j in 0..4 { ARCA[i][j] = pick_Article(); } }"
           " ARC.x4 = (&raw mut ARCA[0]) as *mut *mut Article; ARC.x8 = (&raw mut ARCA[1]) as *mut *mut Article; ARC.xC = (&raw mut ARCA[2]) as *mut *mut Article;"
           " ARC.x10 = &raw mut ARC40; ARC.x14 = &raw mut ARC53C[(rnd() & 1) as usize];"
           " ARC40.x0 = fv() as i32; ARC40.x4 = rw(); ARC40.x8 = rw(); ARC40.xC = rw(); ARC40.x10 = rw(); ARC40.x14 = rw(); ARC40.x18 = rw(); }"),
    hash=("hash_it_804D6D40_t(&ARC40);", "g::hash_it_804D6D40_t(&ARC40);"),
    ov={"it_80278800": {1: (EFID_C, EFID_R)}},
)

STEMS["itdraw"] = dict(
    c_pre="static void* COBP;",
    c_extra="static HSD_CObj COB; static u8 IDXA[16]; static it_266F_ItemVars IV[2];",
    rs_extra="pub static mut COB: HSD_CObj = unsafe { core::mem::zeroed() }; static mut IDXA: [u8; 16] = [0; 16];"
             " static mut IV: [it_266F_ItemVars; 2] = unsafe { core::mem::zeroed() };",
    pregs=("preg(&COB, sizeof COB); preg(IDXA, sizeof IDXA); preg(IV, sizeof IV);",
           "preg(&raw const COB, core::mem::size_of_val(&COB)); preg(&raw const IDXA, 16); preg(&raw const IV, core::mem::size_of_val(&IV));"),
    setup=("{ int q; COBP = &COB; zero(&COB, sizeof COB); zero(IV, sizeof IV); for (q = 0; q < 16; q++) IDXA[q] = rnd() & 3;"
           " for (q = 0; q < 12; q++) ((float*) COB.view_mtx)[q] = rw();"
           " for (q = 0; q < 2; q++) { IV[q].x0 = fv() % 5; IV[q].x4 = IDXA; IV[q].x8 = fv() % 5; IV[q].xC = IDXA + 4; }"
           " for (q = 0; q < 4; q++) { IT[q].xAC8_hurtboxNum = rnd() % 3; if (rnd() % 3 == 0) IT[q].kind = It_Kind_Unk4;"
           " *(void**) &IT[q].xDD4_itemVar = &IV[q & 1]; } }",
           "{ COB = core::mem::zeroed(); IV = core::mem::zeroed(); for q in 0..16usize { IDXA[q] = (rnd() & 3) as u8; }"
           " for q in 0..12usize { *(&raw mut COB.view_mtx as *mut f32).add(q) = rw(); }"
           " for q in 0..2usize { IV[q].x0 = (fv() % 5) as u16; IV[q].x4 = (&raw mut IDXA) as *mut u8; IV[q].x8 = (fv() % 5) as u16; IV[q].xC = ((&raw mut IDXA) as *mut u8).add(4); }"
           " for q in 0..4usize { IT[q].xAC8_hurtboxNum = (rnd() % 3) as u8; if rnd() % 3 == 0 { IT[q].kind = It_Kind_Unk4; }"
           " *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = &raw mut IV[q & 1]; } }"),
    ov={"it_8026EBC8": {1: ("(u16) (fv() % 5)", "(fv() % 5) as u16"), 2: ("IDXA", "(&raw mut IDXA) as *mut u8")}, "it_8026EC54": {1: ("(u16) (fv() % 5)", "(fv() % 5) as u16"), 2: ("IDXA", "(&raw mut IDXA) as *mut u8")}},
)

_PC = "{ int q; for (q = 0; q < 4; q++) *(void**) &IT[q].xDD4_itemVar = 0; }"
_PR = "for q in 0..4usize { *((&raw mut IT[q].xDD4_itemVar) as *mut *mut it_266F_ItemVars) = null_mut(); }"
STEMS["itdraw"]["post"] = {n: (_PC, _PR) for n in ("it_8026EB18", "it_8026EBC8", "it_8026EC54", "it_8026ECE0", "it_8026EECC",
                           "it_8026EECC_inline_0", "it_8026EECC_inline_1", "it_8026EECC_inline_2", "it_8026EECC_inline_3", "it_8026EECC_inline_sw")}

_PFT = ("(void*) &FT[rnd() % 3]", "(&raw mut FT[(rnd() % 3) as usize]) as *mut c_void")
_PIT = ("(void*) &IT[rnd() % 4]", "(&raw mut IT[(rnd() % 4) as usize]) as *mut c_void")
_ANG = ("(float) ((int) (fv() % 200) - 100) * 0.37f", "((fv() % 200) as i32 - 100) as f32 * 0.37")
STEMS["itcoll"] = dict(
    c_pre=_EF_PRE,
    c_extra="static ItHurtBoneList HBL[2]; static ItHurtBoneDesc HBD[2][2]; static ItCollDynamics DYN[2]; static ItCollDynamicsDesc DYD[2][2]; static struct Fighter_804D6524_t F6524;",
    rs_extra="static mut HBL: [ItHurtBoneList; 2] = unsafe { core::mem::zeroed() }; static mut HBD: [[ItHurtBoneDesc; 2]; 2] = unsafe { core::mem::zeroed() };"
             " static mut DYN: [m::ItCollDynamics; 2] = unsafe { core::mem::zeroed() }; static mut DYD: [[m::ItCollDynamicsDesc; 2]; 2] = unsafe { core::mem::zeroed() }; static mut F6524: Fighter_804D6524_t = unsafe { core::mem::zeroed() };",
    pregs=("preg(HBL, sizeof HBL); preg(HBD, sizeof HBD); preg(DYN, sizeof DYN); preg(DYD, sizeof DYD);",
           "preg(&raw const HBL, core::mem::size_of_val(&HBL)); preg(&raw const HBD, core::mem::size_of_val(&HBD)); preg(&raw const DYN, core::mem::size_of_val(&DYN)); preg(&raw const DYD, core::mem::size_of_val(&DYD));"),
    setup=("{ int q, k; u32 r; zero(&F6524, sizeof F6524); F6524.x4 = rw(); Fighter_804D6524 = &F6524; it_804D6D18 = rnd() % 15; Item_804A0CCC.count = rnd() % 12;"
           " for (q = 0; q < 15; q++) { r = rnd(); it_804A0E70[q].x0 = (r & 1) ? 2 : 1;"
           " it_804A0E70[q].x4 = (r & 1) ? (void*) &IT[(r >> 4) % 4] : (void*) &FT[(r >> 4) % 3]; it_804A0E70[q].x8 = pick_hit(); it_804A0E70[q].xC = pick_hurt(); }"
           " for (q = 0; q < 4; q++) { IT[q].xAC4_ignoreItemID = rnd() % 3; IT[q].xAC8_hurtboxNum = rnd() % 3; IT[q].xB68 = rnd() % 3;"
           " for (k = 0; k < 4; k++) IT[q].x5D4_hitboxes[k].hit.element = rnd() % 17; }"
           " for (q = 0; q < 3; q++) { FT[q].hurt_capsules_len = rnd() % 16; for (k = 0; k < 4; k++) FT[q].x914[k].element = rnd() % 17; }"
           " zero(HBL, sizeof HBL); zero(HBD, sizeof HBD); zero(DYN, sizeof DYN); zero(DYD, sizeof DYD);"
           " for (q = 0; q < 2; q++) { HBL[q].count = rnd() % 3; HBL[q].descs = HBD[q]; DYN[q].count = rnd() % 3; DYN[q].descs = DYD[q];"
           " for (k = 0; k < 2; k++) { HBD[q][k].bone_id = (rnd() % 4) as i32; HBD[q][k].a_offset.x = rw(); HBD[q][k].b_offset.y = rw(); HBD[q][k].scale = rw();"
           " DYD[q][k].bone_id = rnd() % 4; DYD[q][k].offset.x = rw(); DYD[q][k].size = rw(); }"
           " ART[q].x8_hurtbones = (rnd() & 1) ? &HBL[q] : 0; ART[q].x14_dynamics = (rnd() & 1) ? (void*) &DYN[q] : 0; } }",
           "{ F6524 = core::mem::zeroed(); F6524.x4 = rw(); m::ext::Fighter_804D6524 = &raw mut F6524; m::ext::it_804D6D18 = rnd() % 15; m::ext::Item_804A0CCC.count = (rnd() % 12) as usize;"
           " for q in 0..15usize { let r = rnd(); m::ext::it_804A0E70[q].x0 = if r & 1 != 0 { 2 } else { 1 };"
           " m::ext::it_804A0E70[q].x4 = if r & 1 != 0 { (&raw mut IT[((r >> 4) % 4) as usize]) as *mut c_void } else { (&raw mut FT[((r >> 4) % 3) as usize]) as *mut c_void };"
           " m::ext::it_804A0E70[q].x8 = pick_hit(); m::ext::it_804A0E70[q].xC = pick_hurt(); }"
           " for q in 0..4usize { IT[q].xAC4_ignoreItemID = rnd() % 3; IT[q].xAC8_hurtboxNum = (rnd() % 3) as u8; IT[q].xB68 = (rnd() % 3) as u8;"
           " for k in 0..4usize { IT[q].x5D4_hitboxes[k].hit.element = rnd() % 17; } }"
           " for q in 0..3usize { FT[q].hurt_capsules_len = (rnd() % 16) as u8; for k in 0..4usize { FT[q].x914[k].element = rnd() % 17; } }"
           " HBL = core::mem::zeroed(); HBD = core::mem::zeroed(); DYN = core::mem::zeroed(); DYD = core::mem::zeroed();"
           " for q in 0..2usize { HBL[q].count = (rnd() % 3) as i32; HBL[q].descs = (&raw mut HBD[q]) as *mut ItHurtBoneDesc; DYN[q].count = (rnd() % 3) as i32; DYN[q].descs = (&raw mut DYD[q]) as *mut m::ItCollDynamicsDesc;"
           " for k in 0..2usize { HBD[q][k].bone_id = (rnd() % 4) as i32; HBD[q][k].a_offset.x = rw(); HBD[q][k].b_offset.y = rw(); HBD[q][k].scale = rw();"
           " DYD[q][k].bone_id = (rnd() % 4) as i32; DYD[q][k].offset.x = rw(); DYD[q][k].size = rw(); }"
           " ART[q].x8_hurtbones = if rnd() & 1 != 0 { &raw mut HBL[q] } else { null_mut() }; ART[q].x14_dynamics = if rnd() & 1 != 0 { (&raw mut DYN[q]) as *mut ItemDynamics } else { null_mut() }; } }"),
    ov={"it_8026F9AC": {1: _PFT}, "it_8026F9AC_noinline": {1: _PFT},
        "it_8026FAC4": {3: _PIT}, "it_8026FAC4_noinline": {3: _PIT},
        "it_8027129C": {1: IDX4}, "it_80271534": {1: ("fv() & 1", "(fv() & 1) as i32")}, "it_80271590": {1: ("fv() & 1", "(fv() & 1) as i32")},
        "it_80271830": {1: _ANG}},
    hash=("HM(PI(Fighter_804D6524));", "hm(pi(m::ext::Fighter_804D6524));"),
    pregs_unused=0,
    hash_ft={"it_802703E8"},
)
STEMS["itgroundcoll"] = dict()
STEMS["itmaplib"] = dict(
    setup=("for (i = 0; i < 8; i++) JO[i].flags &= ~JOBJ_USE_QUATERNION;", "for i in 0..8 { JO[i].flags &= !(JOBJ_USE_QUATERNION as u32); }"),
)

for _s in ("itzako", "it_26B1", "it_2725", "it_279C"):
    STEMS.setdefault(_s, dict())

_TB = ("&TB[rnd() & 1]", "&raw mut TB[(rnd() & 1) as usize]")
_TB0 = ("&TB[0]", "&raw mut TB[0]")
_CNT = ("CNT", "(&raw mut CNT) as *mut i32")
STEMS["itspawn"] = dict(
    skip_static=["bisectValue", "it_8026C88C_inline", "it_8026D018_inline", "it_8026D018_inline2", "it_8026D018_inline3"],
    c_pre=_EF_PRE + "static int CNT[64]; static int GMV; static unsigned long long FM64; static unsigned char AB[2048]; static unsigned AO;",
    c_extra="static ItemPickTable TB[2]; static u8 TBK[2][8]; static u16 TBW[2][8]; static struct it_8026C47C_arg0_t B47C;",
    rs_extra="pub static mut CNT: [i32; 64] = [0; 64]; pub static mut GMV: i32 = 0; pub static mut FM64: u64 = 0; pub static mut AB: [u8; 2048] = [0; 2048]; pub static mut AO: u32 = 0;"
             " static mut TB: [ItemPickTable; 2] = unsafe { core::mem::zeroed() }; static mut TBK: [[u8; 8]; 2] = [[0; 8]; 2]; static mut TBW: [[u16; 8]; 2] = [[0; 8]; 2];"
             " static mut B47C: it_8026C47C_arg0_t = unsafe { core::mem::zeroed() };",
    pregs=("preg(CNT, sizeof CNT); preg(AB, sizeof AB); preg(TBK, sizeof TBK); preg(TBW, sizeof TBW);",
           "preg(&raw const CNT, core::mem::size_of_val(&CNT)); preg(&raw const AB, 2048); preg(&raw const TBK, 16); preg(&raw const TBW, 32);"),
    setup=("{ int q, k; u32 cum; for (q = 0; q < 64; q++) CNT[q] = (rnd() & 3) == 0 ? 0 : (s32) (fv() % 40); GMV = (rnd() % 8 == 0) ? -1 : (s32) (rnd() % 5);"
           " FM64 = (rnd() % 6 == 0) ? 0 : fv64(); AO = 0; zero(AB, sizeof AB); zero(&B47C, sizeof B47C);"
           " zero(&it_804A0E30, sizeof it_804A0E30); zero(&it_804A0E50, sizeof it_804A0E50); zero(&it_804A0E60, sizeof it_804A0E60);"
           " it_804A0E30.x0 = (s32) (fv() % 4); it_804A0E30.x18 = FM64;"
           " for (k = 0; k < 2; k++) { u32 r = rnd(); zero(&TB[k], sizeof TB[k]); TB[k].size = 1 + (r % 7); TB[k].x4 = TBK[k]; TB[k].xC = TBW[k];"
           " TB[k].x8 = ((r >> 8) % 5 == 0) ? 0 : (u16) (1 + (r >> 12) % 60); cum = 0;"
           " for (q = 0; q < 8; q++) { TBK[k][q] = (rnd() % 4 == 0) ? 0x22 : (u8) (rnd() % 40); TBW[k][q] = (u16) cum; cum += rnd() % 12; } }"
           " it_804D6D28->x128[0] = (s32) (fv() % 20); }",
           "{ for q in 0..64usize { CNT[q] = if rnd() & 3 == 0 { 0 } else { (fv() % 40) as i32 }; } GMV = if rnd() % 8 == 0 { -1 } else { (rnd() % 5) as i32 };"
           " FM64 = if rnd() % 6 == 0 { 0 } else { fv64() }; AO = 0; AB = [0; 2048]; B47C = core::mem::zeroed();"
           " m::it_804A0E30 = core::mem::zeroed(); m::it_804A0E50 = core::mem::zeroed(); m::it_804A0E60 = core::mem::zeroed();"
           " m::it_804A0E30.x0 = (fv() % 4) as i32; m::it_804A0E30.x18 = FM64;"
           " for k in 0..2usize { let r = rnd(); TB[k] = core::mem::zeroed(); TB[k].size = (1 + (r % 7)) as u8; TB[k].x4 = (&raw mut TBK[k]) as *mut u8; TB[k].xC = (&raw mut TBW[k]) as *mut u16;"
           " TB[k].x8 = if (r >> 8) % 5 == 0 { 0 } else { (1 + (r >> 12) % 60) as u16 }; let mut cum = 0u32;"
           " for q in 0..8usize { TBK[k][q] = if rnd() % 4 == 0 { 0x22 } else { (rnd() % 40) as u8 }; TBW[k][q] = cum as u16; cum += rnd() % 12; } }"
           " (*m::ext::it_804D6D28).x128[0] = (fv() % 20) as i32; }"),
    hash=("{ int q; hash_RandomItemSpawner(&it_804A0E30); hash_ItemPickTable(&it_804A0E50); hash_ItemPickTable(&it_804A0E60); hash_ItemPickTable(&TB[0]); hash_ItemPickTable(&TB[1]);"
          " for (q = 0; q < 64; q++) HM((u32) CNT[q]); HM((u32) GMV); HM64((double) 0); HM((u32) FM64); HM((u32) (FM64 >> 32)); HM(AO); HM(RH(AB, sizeof AB)); HM(RH(TBK, sizeof TBK)); HM(RH(TBW, sizeof TBW)); HM(RH(&B47C, sizeof B47C)); }",
          "{ g::hash_RandomItemSpawner(&m::it_804A0E30); g::hash_ItemPickTable(&m::it_804A0E50); g::hash_ItemPickTable(&m::it_804A0E60); g::hash_ItemPickTable(&TB[0]); g::hash_ItemPickTable(&TB[1]);"
          " for q in 0..64usize { hm(CNT[q] as u32); } hm(GMV as u32); hm64(0.0); hm(FM64 as u32); hm((FM64 >> 32) as u32); hm(AO); hm(rh(&AB)); hm(rh(&TBK)); hm(rh(&TBW)); hm(rh(&B47C)); }"),
    ov={"it_8026C47C": {0: ("&B47C", "&raw mut B47C")},
        "it_8026C65C": {0: _TB0}, "it_8026C75C": {0: _TB},
        "it_8026CA4C": {0: _TB, 1: _CNT, 2: ("fv64()", "fv64()"), 3: ("(s32) (fv() % 12)", "(fv() % 12) as i32")},
        "it_8026CB9C": {0: _CNT, 1: ("fv64()", "fv64()")},
        "it_8026CD50": {0: _CNT, 1: ("fv64()", "fv64()")},
        "fn_8026C88C": {0: ("(GMV = (s32) (rnd() % 5), &GO[rnd() % 4])", "{ GMV = (rnd() % 5) as i32; &raw mut GO[(rnd() % 4) as usize] }")}},
)

STEMS["itmaterial"] = dict(
    c_extra="static unsigned char TBOG[16]; static GXColor GCOL[2]; static HSD_TExp TX[3];\n"
            "static void cb_stev(HSD_MObj* m, HSD_TObj* t, u32 r) { T(7010); T(PI(m)); T(PI(t)); T(r); }\n"
            "static void* IT0P;",
    rs_extra="static mut TBOG: [u8; 16] = [0; 16]; static mut GCOL: [GXColor; 2] = unsafe { core::mem::zeroed() }; static mut TX: [HSD_TExp; 3] = unsafe { core::mem::zeroed() };\n"
             "unsafe extern \"C\" fn cb_stev(m: *mut HSD_MObj, t: *mut HSD_TObj, r: u32) { t_(7010); t_pi(m); t_pi(t); t_(r); }\n"
             "fn t_(v: u32) { t(v) } fn t_pi<T>(p: *mut T) { t(pi(p)) }",
    pregs=("preg_xpools(); preg(TBOG, sizeof TBOG); preg(GCOL, sizeof GCOL); preg(TX, sizeof TX); freg((const void*) cb_stev);",
           "g::preg_xpools(); preg(&raw const TBOG, 16); preg(&raw const GCOL, 8); preg(&raw const TX, core::mem::size_of_val(&TX)); freg(cb_stev as usize);"),
    setup=("{ static int init; static struct it_MObjInfo p0; int q; if (!init) { p0 = it_mobj; init = 1; } it_mobj = p0; setup_xpools();"
           " zero(TX, sizeof TX); for (q = 0; q < 3; q++) fill_HSD_TECnst(&TX[q].cnst); for (q = 0; q < 16; q++) TBOG[q] = (u8) rnd(); for (q = 0; q < 2; q++) { GCOL[q].r = rnd(); GCOL[q].g = rnd(); GCOL[q].b = rnd(); GCOL[q].a = rnd(); }"
           " for (q = 0; q < 2; q++) MO[q].parent.class_info = (HSD_ClassInfo*) &MI[0];"
           " MI[0].setup_tev = cb_stev;"
           " hsdMObj.parent.release = (void*) cbev1; hsdMObj.parent.amnesia = (void*) cbev2; hsdMObj.load = (void*) cbp1; hsdMObj.make_texp = (void*) cbev3;"
           " for (q = 0; q < 5; q++) TO[q].next = (q < 2 && (rnd() & 1)) ? &TO[q + 1] : 0;"
           " TO[4].imagedesc = (rnd() & 1) ? (void*) TBOG : 0;"
           " for (q = 0; q < 2; q++) MO[q].tobj = (rnd() & 3) ? &TO[rnd() % 3] : 0;"
           " tobj_shadows = (rnd() & 1) ? &TO[3] : 0; tobj_toon = (rnd() & 1) ? &TO[4] : 0;"
           " for (q = 0; q < 2; q++) MO[q].mat = (rnd() % 5) ? &MA[rnd() & 1] : &MA[0];"
           " for (q = 0; q < 2; q++) MO[q].pe = (rnd() & 1) ? &PE[rnd() & 1] : 0;"
           " for (q = 0; q < 3; q++) DO[q].next = (q < 2 && (rnd() & 1)) ? &DO[q + 1] : 0;"
           " for (q = 0; q < 3; q++) DO[q].mobj = (rnd() % 4) ? &MO[rnd() & 1] : 0;"
           " HSD_GObj_804D7814 = &GO[rnd() % 4]; if (HDBG) printf(\"S %08X %08X %08X %08X%c\", PI(MO[0].tobj), PI(MO[1].tobj), PI(tobj_shadows), PI(tobj_toon), 10); }",
           "{ let q: usize; m::it_mobj_static_init(); g::setup_xpools();"
           " TX = core::mem::zeroed(); for q in 0..3usize { g::fill_HSD_TECnst(&mut TX[q].cnst); } for q in 0..16usize { TBOG[q] = rnd() as u8; } for q in 0..2usize { GCOL[q].r = rnd() as u8; GCOL[q].g = rnd() as u8; GCOL[q].b = rnd() as u8; GCOL[q].a = rnd() as u8; }"
           " for q in 0..2usize { g::MO[q].parent.class_info = (&raw mut g::MI[0]) as *mut HSD_ClassInfo; }"
           " g::MI[0].setup_tev = Some(cb_stev);"
           " m::ext::hsdMObj.parent.release = core::mem::transmute::<usize, _>(cbev1 as usize); m::ext::hsdMObj.parent.amnesia = core::mem::transmute::<usize, _>(cbev2 as usize);"
           " m::ext::hsdMObj.load = core::mem::transmute::<usize, _>(cbp1 as usize); m::ext::hsdMObj.make_texp = core::mem::transmute::<usize, _>(cbev3 as usize);"
           " for q in 0..5usize { g::TO[q].next = if q < 2 && (rnd() & 1) != 0 { &raw mut g::TO[q + 1] } else { null_mut() }; }"
           " g::TO[4].imagedesc = if rnd() & 1 != 0 { (&raw mut TBOG) as *mut _ } else { null_mut() };"
           " for q in 0..2usize { g::MO[q].tobj = if rnd() & 3 != 0 { &raw mut g::TO[(rnd() % 3) as usize] } else { null_mut() }; }"
           " m::ext::tobj_shadows = if rnd() & 1 != 0 { &raw mut g::TO[3] } else { null_mut() }; m::ext::tobj_toon = if rnd() & 1 != 0 { &raw mut g::TO[4] } else { null_mut() };"
           " for q in 0..2usize { g::MO[q].mat = if rnd() % 5 != 0 { &raw mut g::MA[(rnd() & 1) as usize] } else { &raw mut g::MA[0] }; }"
           " for q in 0..2usize { g::MO[q].pe = if rnd() & 1 != 0 { &raw mut g::PE[(rnd() & 1) as usize] } else { null_mut() }; }"
           " for q in 0..3usize { g::DO[q].next = if q < 2 && (rnd() & 1) != 0 { &raw mut g::DO[q + 1] } else { null_mut() }; }"
           " for q in 0..3usize { g::DO[q].mobj = if rnd() % 4 != 0 { &raw mut g::MO[(rnd() & 1) as usize] } else { null_mut() }; }"
           " m::ext::HSD_GObj_804D7814 = &raw mut GO[(rnd() % 4) as usize]; if HDBG { println!(\"S {:08X} {:08X} {:08X} {:08X}\", pi(g::MO[0].tobj), pi(g::MO[1].tobj), pi(m::ext::tobj_shadows), pi(m::ext::tobj_toon)); } }"),
    hash=("hash_xpools(); hash_it_MObjInfo(&it_mobj); hash_GXColor(&GCOL[0]); hash_GXColor(&GCOL[1]); for (int q = 0; q < 3; q++) hash_HSD_TECnst(&TX[q].cnst);", "g::hash_xpools(); g::hash_it_MObjInfo(&m::it_mobj); g::hash_GXColor(&GCOL[0]); g::hash_GXColor(&GCOL[1]); for q in 0..3usize { g::hash_HSD_TECnst(&TX[q].cnst); }"),
    ov={"fn_80277D8C": {0: ("&MO[rnd() & 1]", "&raw mut g::MO[(rnd() & 1) as usize]"), 1: ("fv()", "fv()"), 2: ("fv()", "fv()")},
        "it_80277F90": {1: ("&MO[rnd() & 1]", "&raw mut g::MO[(rnd() & 1) as usize]"), 2: ("&TX[rnd() % 3]", "&raw mut TX[(rnd() % 3) as usize]")},
        "it_80278108": {1: ("&MO[rnd() & 1]", "&raw mut g::MO[(rnd() & 1) as usize]"), 2: ("(rnd() & 1) ? &TX[rnd() % 3] : 0", "if rnd() & 1 != 0 { &raw mut TX[(rnd() % 3) as usize] } else { null_mut() }")},
        "it_80278574": {1: ("&GCOL[rnd() & 1]", "&raw mut GCOL[(rnd() & 1) as usize]")}},
)
