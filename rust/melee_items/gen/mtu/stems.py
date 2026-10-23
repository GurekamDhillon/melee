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
