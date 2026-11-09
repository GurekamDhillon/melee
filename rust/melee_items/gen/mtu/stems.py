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

_ITEM_C = r"""
struct it_MObjInfo { char pad[64]; };
struct it_MObjInfo it_mobj;
static HSD_AnimJoint AJ[4]; static HSD_MatAnimJoint MJ[4]; static HSD_ShapeAnimJoint SJ[4];
static ItemModelDesc MD[2]; static ItemDynamics DY[2]; static BoneDynamicsDesc BDD[4];
static ItemStateArray STA; static HSD_Joint JNT[2];
static Article* A24[43]; static Article* A38[118]; static Article* A30[47];
static Fighter_804D653C_t CA04[4];
static s32 pick_kind(int spawn)
{
    u32 r = rnd();
    switch (r & 7) {
    case 0: return 0x1000 + (r >> 8) % 0x200;
    case 1: return 0x1200 + (r >> 8) % 0x50;
    case 2: return spawn ? 5000 + (r >> 8) % 40 : (s32) ((r >> 8) % 300);
    case 3: return 0x20000 + (r >> 8) % 20;
    default: return (r >> 8) % 300;
    }
}
"""
_ITEM_R = r"""
const _: () = assert!(core::mem::size_of::<HSD_DObj>() <= 64);
static mut AJ: [HSD_AnimJoint; 4] = unsafe { core::mem::zeroed() };
static mut MJ: [HSD_MatAnimJoint; 4] = unsafe { core::mem::zeroed() };
static mut SJ: [HSD_ShapeAnimJoint; 4] = unsafe { core::mem::zeroed() };
static mut MD: [ItemModelDesc; 2] = unsafe { core::mem::zeroed() };
static mut DY: [ItemDynamics; 2] = unsafe { core::mem::zeroed() };
static mut BDD: [BoneDynamicsDesc; 4] = unsafe { core::mem::zeroed() };
static mut STA: ItemStateArray = unsafe { core::mem::zeroed() };
static mut JNT: [HSD_Joint; 2] = unsafe { core::mem::zeroed() };
static mut A24: [*mut Article; 43] = [null_mut(); 43];
static mut A38: [*mut Article; 118] = [null_mut(); 118];
static mut A30: [*mut Article; 47] = [null_mut(); 47];
static mut CA04: [Fighter_804D653C_t; 4] = unsafe { core::mem::zeroed() };
fn pick_kind(spawn: bool) -> i32 {
    let r = rnd();
    match r & 7 {
        0 => 0x1000 + ((r >> 8) % 0x200) as i32,
        1 => 0x1200 + ((r >> 8) % 0x50) as i32,
        2 => if spawn { 5000 + ((r >> 8) % 40) as i32 } else { ((r >> 8) % 300) as i32 },
        3 => 0x20000 + ((r >> 8) % 20) as i32,
        _ => ((r >> 8) % 300) as i32,
    }
}
"""
_ITEM_SETUP_C = r"""
{
    int k;
    zero(AJ, sizeof AJ); zero(MJ, sizeof MJ); zero(SJ, sizeof SJ); zero(MD, sizeof MD); zero(DY, sizeof DY); zero(BDD, sizeof BDD);
    zero(&STA, sizeof STA); zero(JNT, sizeof JNT); zero(DOBBUF, sizeof DOBBUF); zero(CA04, sizeof CA04);
    for (i = 0; i < 4; i++) {
        AJ[i].child = (i < 3 && (rnd() & 1)) ? &AJ[i + 1 + rnd() % (3 - i)] : 0;
        AJ[i].next = (i < 3 && (rnd() & 1)) ? &AJ[i + 1 + rnd() % (3 - i)] : 0;
        MJ[i].child = (i < 3 && (rnd() & 1)) ? &MJ[i + 1 + rnd() % (3 - i)] : 0;
        MJ[i].next = (i < 3 && (rnd() & 1)) ? &MJ[i + 1 + rnd() % (3 - i)] : 0;
        SJ[i].child = (i < 3 && (rnd() & 1)) ? &SJ[i + 1 + rnd() % (3 - i)] : 0;
        SJ[i].next = (i < 3 && (rnd() & 1)) ? &SJ[i + 1 + rnd() % (3 - i)] : 0;
        ((HSD_DObj*) DOBBUF[i])->next = i < 3 ? (HSD_DObj*) DOBBUF[i + 1] : 0;
        ((HSD_DObj*) DOBBUF[i])->mobj = (rnd() & 1) ? (HSD_MObj*) &U8A[0] : 0;
        BDD[i].bone_id = rnd() % 4;
        CA04[i].unk5 = (u8) (rnd() & 1);
        STT[i].anim_id = (int) (rnd() % 9) - 1;
    }
    for (k = 0; k < 8; k++) {
        STA.x0_itemStateDesc[k].x0_anim_joint = (rnd() & 3) ? &AJ[rnd() % 4] : 0;
        STA.x0_itemStateDesc[k].x4_matanim_joint = (rnd() & 3) ? &MJ[rnd() % 4] : 0;
        STA.x0_itemStateDesc[k].x8_parameters = (rnd() & 3) ? &SJ[rnd() % 4] : 0;
    }
    for (i = 0; i < 3; i++) {
        SDS[i].x0_anim_joint = (rnd() & 3) ? &AJ[rnd() % 4] : 0;
        SDS[i].x4_matanim_joint = (rnd() & 3) ? &MJ[rnd() % 4] : 0;
        SDS[i].x8_parameters = (rnd() & 3) ? &SJ[rnd() % 4] : 0;
    }
    for (i = 0; i < 2; i++) {
        int bc = (rnd() % 3 == 0) ? 0 : (int) (rnd() % 4) + 1;
        ART[i].x0_common_attr = &ATTR[i];
        ART[i].xC_itemStates = &STA;
        ART[i].x10_modelDesc = &MD[i];
        MD[i].x0_joint = (rnd() & 1) ? (HSD_Joint*) &JNT[rnd() & 1] : 0;
        MD[i].x4_bone_count = bc;
        DY[i].count = rnd() % 4;
        DY[i].dyn_descs = BDD;
        ART[i].x14_dynamics = (bc && (rnd() & 1)) ? &DY[i] : 0;
    }
    for (k = 0; k < 43; k++) A24[k] = (CUR_OP == OP_Item_8026862C && k % 17 == 5) ? 0 : &ART[rnd() % 2];
    for (k = 0; k < 118; k++) A38[k] = (CUR_OP == OP_Item_8026862C && k % 17 == 5) ? 0 : &ART[rnd() % 2];
    for (k = 0; k < 47; k++) A30[k] = (CUR_OP == OP_Item_8026862C && k % 17 == 5) ? 0 : &ART[rnd() % 2];
    for (k = 0; k < 30; k++) it_804A0F60[k] = &ART[rnd() % 2];
    it_804D6D24 = A24; it_804D6D38 = A38; it_804D6D30 = A30; it_804D6D04 = CA04;
    for (i = 0; i < 4; i++) {
        IT[i].kind = pick_kind(0);
        IT[i].x374_dynamicBonesNum = rnd() % 5;
        IT[i].atk_victim = pick_igo();
        IT[i].grab_victim = pick_igo();
        IT[i].grab_dealt = (__typeof__(IT[0].grab_dealt)) pick_cb(1);
        IT[i].grabbed_for_victim = (__typeof__(IT[0].grabbed_for_victim)) pick_cb(3);
        IT[i].xC8_joint = (rnd() & 1) ? &JNT[rnd() & 1] : 0;
        IT[i].x548_colorOverlay.x28_colanim.i = rnd() % 4;
        IT[i].xC6C = (float) (rnd() % 5) * 0.5f;
        /* xD90.x2070_int <- xC74 is a whole-word write over a bitfield overlay (x86 LSB-first vs GC MSB-first views): only bytes 0 and 3 vary */
        IT[i].xC74 = (s32) ((rnd() & 0xFF) | ((rnd() & 0xFF) << 24));
    }
    for (i = 0; i < 8; i++) JO[i].flags &= ~JOBJ_USE_QUATERNION;
    SP.kind = pick_kind(1);
    for (i = 0; i < 26; i++) ((s32*) &Item_804A0C64)[i] = (s32) (fv() & 7);
    Item_804A0CCC.count = rnd(); Item_804A0CCC.x154.x0.b0 = rnd() & 1; Item_804A0CCC.x154.x0.b1 = rnd() & 1; Item_804A0CCC.x154.x0.b2 = rnd() & 1; Item_804A0CCC.x154.x0.b3 = rnd() & 1; Item_804A0CCC.x154.x0.b4 = rnd() & 1; Item_804A0CCC.x154.x0.b5 = rnd() & 1; Item_804A0CCC.x154.x0.b6 = rnd() & 1; Item_804A0CCC.x154.x0.b7 = rnd() & 1; for (i = 0; i < 0xB0; i++) ((u8*) &Item_804A0CCC)[i] = (u8) rnd();
    Item_804A0E24.last_kind = (s32) rnd(); Item_804A0E24.previous_kind = (s32) rnd(); Item_804A0E24.rare_spawned = rnd() & 1;
}
"""

_ITEM_SETUP_R = r"""
{
    AJ = core::mem::zeroed(); MJ = core::mem::zeroed(); SJ = core::mem::zeroed(); MD = core::mem::zeroed(); DY = core::mem::zeroed();
    BDD = core::mem::zeroed(); STA = core::mem::zeroed(); JNT = core::mem::zeroed(); DOBBUF = [[0; 8]; 4]; CA04 = core::mem::zeroed();
    for i in 0..4usize {
        AJ[i].child = if i < 3 && (rnd() & 1) != 0 { &raw mut AJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        AJ[i].next = if i < 3 && (rnd() & 1) != 0 { &raw mut AJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        MJ[i].child = if i < 3 && (rnd() & 1) != 0 { &raw mut MJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        MJ[i].next = if i < 3 && (rnd() & 1) != 0 { &raw mut MJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        SJ[i].child = if i < 3 && (rnd() & 1) != 0 { &raw mut SJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        SJ[i].next = if i < 3 && (rnd() & 1) != 0 { &raw mut SJ[i + 1 + (rnd() % (3 - i) as u32) as usize] } else { null_mut() };
        (*((&raw mut DOBBUF[i]) as *mut HSD_DObj)).next = if i < 3 { (&raw mut DOBBUF[i + 1]) as *mut HSD_DObj } else { null_mut() };
        (*((&raw mut DOBBUF[i]) as *mut HSD_DObj)).mobj = if rnd() & 1 != 0 { (&raw mut U8A[0]) as *mut HSD_MObj } else { null_mut() };
        BDD[i].bone_id = (rnd() % 4) as _;
        CA04[i].unk5 = (rnd() & 1) as u8;
        STT[i].anim_id = (rnd() % 9) as i32 - 1;
    }
    for k in 0..8usize {
        STA.x0_itemStateDesc[k].x0_anim_joint = if rnd() & 3 != 0 { &raw mut AJ[(rnd() % 4) as usize] } else { null_mut() };
        STA.x0_itemStateDesc[k].x4_matanim_joint = if rnd() & 3 != 0 { &raw mut MJ[(rnd() % 4) as usize] } else { null_mut() };
        STA.x0_itemStateDesc[k].x8_parameters = if rnd() & 3 != 0 { &raw mut SJ[(rnd() % 4) as usize] } else { null_mut() };
    }
    for i in 0..3usize {
        SDS[i].x0_anim_joint = if rnd() & 3 != 0 { &raw mut AJ[(rnd() % 4) as usize] } else { null_mut() };
        SDS[i].x4_matanim_joint = if rnd() & 3 != 0 { &raw mut MJ[(rnd() % 4) as usize] } else { null_mut() };
        SDS[i].x8_parameters = if rnd() & 3 != 0 { &raw mut SJ[(rnd() % 4) as usize] } else { null_mut() };
    }
    for i in 0..2usize {
        let bc: i32 = if rnd() % 3 == 0 { 0 } else { (rnd() % 4) as i32 + 1 };
        ART[i].x0_common_attr = &raw mut ATTR[i];
        ART[i].xC_itemStates = &raw mut STA;
        ART[i].x10_modelDesc = &raw mut MD[i];
        MD[i].x0_joint = if rnd() & 1 != 0 { (&raw mut JNT[(rnd() & 1) as usize]) as *mut _ } else { null_mut() };
        MD[i].x4_bone_count = bc as _;
        DY[i].count = (rnd() % 4) as i32;
        DY[i].dyn_descs = (&raw mut BDD) as *mut BoneDynamicsDesc;
        ART[i].x14_dynamics = if bc != 0 && (rnd() & 1) != 0 { &raw mut DY[i] } else { null_mut() };
    }
    for k in 0..43usize { A24[k] = if CUR_OP == OP_Item_8026862C && k % 17 == 5 { null_mut() } else { &raw mut ART[(rnd() % 2) as usize] }; }
    for k in 0..118usize { A38[k] = if CUR_OP == OP_Item_8026862C && k % 17 == 5 { null_mut() } else { &raw mut ART[(rnd() % 2) as usize] }; }
    for k in 0..47usize { A30[k] = if CUR_OP == OP_Item_8026862C && k % 17 == 5 { null_mut() } else { &raw mut ART[(rnd() % 2) as usize] }; }
    for k in 0..30usize { m::ext::it_804A0F60[k] = &raw mut ART[(rnd() % 2) as usize]; }
    m::ext::it_804D6D24 = (&raw mut A24) as *mut *mut Article;
    m::ext::it_804D6D38 = (&raw mut A38) as *mut *mut Article;
    m::ext::it_804D6D30 = (&raw mut A30) as *mut *mut Article;
    m::ext::it_804D6D04 = (&raw mut CA04) as *mut Fighter_804D653C_t;
    for i in 0..4usize {
        IT[i].kind = pick_kind(false) as _;
        IT[i].x374_dynamicBonesNum = (rnd() % 5) as i32;
        IT[i].atk_victim = pick_igo();
        IT[i].grab_victim = pick_igo();
        IT[i].grab_dealt = core::mem::transmute::<usize, _>(pick_cb(1));
        IT[i].grabbed_for_victim = core::mem::transmute::<usize, _>(pick_cb(3));
        IT[i].xC8_joint = if rnd() & 1 != 0 { &raw mut JNT[(rnd() & 1) as usize] } else { null_mut() };
        IT[i].x548_colorOverlay.x28_colanim.i = (rnd() % 4) as _;
        IT[i].xC6C = (rnd() % 5) as f32 * 0.5;
        IT[i].xC74 = ((rnd() & 0xFF) | ((rnd() & 0xFF) << 24)) as i32;
    }
    for i in 0..8usize { JO[i].flags &= !(JOBJ_USE_QUATERNION as u32); }
    SP.kind = pick_kind(true) as _;
    for i in 0..26usize { *((&raw mut m::Item_804A0C64) as *mut i32).add(i) = (fv() & 7) as i32; }
    m::Item_804A0CCC.count = rnd() as _; m::Item_804A0CCC.x154.anon_0.set_b0((rnd() & 1) as u8); m::Item_804A0CCC.x154.anon_0.set_b1((rnd() & 1) as u8); m::Item_804A0CCC.x154.anon_0.set_b2((rnd() & 1) as u8); m::Item_804A0CCC.x154.anon_0.set_b3((rnd() & 1) as u8); m::Item_804A0CCC.x154.anon_0.set_b4((rnd() & 1) as u8); m::Item_804A0CCC.x154.anon_0.set_b5((rnd() & 1) as u8); m::Item_804A0CCC.x154.anon_0.set_b6((rnd() & 1) as u8); m::Item_804A0CCC.x154.anon_0.set_b7((rnd() & 1) as u8); for i in 0..0xB0usize { *((&raw mut m::Item_804A0CCC) as *mut u8).add(i) = rnd() as u8; }
    m::Item_804A0E24.last_kind = rnd() as i32; m::Item_804A0E24.previous_kind = rnd() as i32; m::Item_804A0E24.rare_spawned = (rnd() & 1) as _;
}
"""

_ITEM_PREG_C = ("preg(AJ, sizeof AJ); preg(MJ, sizeof MJ); preg(SJ, sizeof SJ); preg(MD, sizeof MD); preg(DY, sizeof DY); preg(BDD, sizeof BDD); "
                "preg(&STA, sizeof STA); preg(JNT, sizeof JNT); preg(DOBBUF, sizeof DOBBUF); preg(A24, sizeof A24); preg(A38, sizeof A38); preg(A30, sizeof A30); "
                "preg(CA04, sizeof CA04); preg(&item_alloc_data, sizeof item_alloc_data); preg(&item_dynamic_bones_alloc_data, sizeof item_dynamic_bones_alloc_data); "
                "preg(&item_link_alloc_data, sizeof item_link_alloc_data); preg(&Item_804A0C64, sizeof Item_804A0C64);")
_ITEM_PREG_R = ("preg(&raw const AJ, core::mem::size_of_val(&AJ)); preg(&raw const MJ, core::mem::size_of_val(&MJ)); preg(&raw const SJ, core::mem::size_of_val(&SJ)); "
                "preg(&raw const MD, core::mem::size_of_val(&MD)); preg(&raw const DY, core::mem::size_of_val(&DY)); preg(&raw const BDD, core::mem::size_of_val(&BDD)); "
                "preg(&raw const STA, core::mem::size_of_val(&STA)); preg(&raw const JNT, core::mem::size_of_val(&JNT)); preg(&raw const DOBBUF, core::mem::size_of_val(&DOBBUF)); "
                "preg(&raw const A24, core::mem::size_of_val(&A24)); preg(&raw const A38, core::mem::size_of_val(&A38)); preg(&raw const A30, core::mem::size_of_val(&A30)); "
                "preg(&raw const CA04, core::mem::size_of_val(&CA04)); preg(&raw const m::item_alloc_data, core::mem::size_of_val(&m::item_alloc_data)); "
                "preg(&raw const m::item_dynamic_bones_alloc_data, core::mem::size_of_val(&m::item_dynamic_bones_alloc_data)); "
                "preg(&raw const m::item_link_alloc_data, core::mem::size_of_val(&m::item_link_alloc_data)); preg(&raw const m::Item_804A0C64, core::mem::size_of_val(&m::Item_804A0C64));")
_SFX = ("((rnd() % 4) == 0 ? 540000 + (int) (rnd() & 1) : (rnd() % 3) == 0 ? 5000 + (int) (rnd() % 5000) : (int) fv())",
        "(if rnd() % 4 == 0 { 540000 + (rnd() & 1) as i32 } else if rnd() % 3 == 0 { 5000 + (rnd() % 5000) as i32 } else { fv() as i32 }) as _")
STEMS["item"] = dict(
    c_pre="static unsigned char DOBBUF[4][64] __attribute__((aligned(16)));",
    c_extra=_ITEM_C,
    rs_extra=_ITEM_R + "pub static mut DOBBUF: [[u64; 8]; 4] = [[0; 8]; 4];",
    pregs=(_ITEM_PREG_C, _ITEM_PREG_R),
    setup=(_ITEM_SETUP_C, _ITEM_SETUP_R),
    hash=("HM(RH(&Item_804A0C64, sizeof Item_804A0C64)); HM((u32) Item_804A0CCC.count); HM((u32) Item_804A0CCC.x154.x0.b0); HM((u32) Item_804A0CCC.x154.x0.b1); HM((u32) Item_804A0CCC.x154.x0.b2); HM((u32) Item_804A0CCC.x154.x0.b3); HM((u32) Item_804A0CCC.x154.x0.b4); HM((u32) Item_804A0CCC.x154.x0.b5); HM((u32) Item_804A0CCC.x154.x0.b6); HM((u32) Item_804A0CCC.x154.x0.b7); HM(RH(Item_804A0CCC.ecb_offset_arr, sizeof Item_804A0CCC.ecb_offset_arr)); HM((u32) Item_804A0E24.last_kind); HM((u32) Item_804A0E24.previous_kind); HM((u32) Item_804A0E24.rare_spawned);",
          "hm(rh(&m::Item_804A0C64)); hm(m::Item_804A0CCC.count as u32); hm(m::Item_804A0CCC.x154.x0.b0() as u32); hm(m::Item_804A0CCC.x154.x0.b1() as u32); hm(m::Item_804A0CCC.x154.x0.b2() as u32); hm(m::Item_804A0CCC.x154.x0.b3() as u32); hm(m::Item_804A0CCC.x154.x0.b4() as u32); hm(m::Item_804A0CCC.x154.x0.b5() as u32); hm(m::Item_804A0CCC.x154.x0.b6() as u32); hm(m::Item_804A0CCC.x154.x0.b7() as u32); hm(rh(&m::Item_804A0CCC.ecb_offset_arr)); hm(m::Item_804A0E24.last_kind as u32); hm(m::Item_804A0E24.previous_kind as u32); hm(m::Item_804A0E24.rare_spawned as u32);"),
    ov={"Item_80268E5C": {1: ("fv() & 3", "(fv() & 3) as _")},
        "Item_80268BE0": {1: ("((rnd() & 3) ? &AJ[rnd() % 4] : 0)", "if rnd() & 3 != 0 { &raw mut AJ[(rnd() % 4) as usize] } else { null_mut() }"),
                          2: ("((rnd() & 3) ? &MJ[rnd() % 4] : 0)", "if rnd() & 3 != 0 { &raw mut MJ[(rnd() % 4) as usize] } else { null_mut() }"),
                          3: ("((rnd() & 3) ? &SJ[rnd() % 4] : 0)", "if rnd() & 3 != 0 { &raw mut SJ[(rnd() % 4) as usize] } else { null_mut() }")},
        "Item_8026AE84": {1: _SFX}, "Item_8026AF0C": {1: _SFX}, "Item_8026AFA0": {1: _SFX}},
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

_VP = ("&VA[rnd() % 4]", "&raw mut VA[(rnd() % 4) as usize]")
_VPN = ("((rnd() & 3) ? &VA[rnd() % 4] : 0)", "if rnd() & 3 != 0 { &raw mut VA[(rnd() % 4) as usize] } else { null_mut() }")
_F64 = ("(double) rw()", "rw() as f64")
STEMS["itzako"] = dict(
    skip_static=["it_8027B798_CalcAngle", "product_xyz", "itzako_sqrtf", "return_sqrt_value3"],
    c_extra="static it_804D6D40_t D40; static ftCommonData FCD;",
    rs_extra="static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() }; static mut FCD: ftCommonData = unsafe { core::mem::zeroed() };",
    pregs=("preg(&D40, sizeof D40); preg(&FCD, sizeof FCD);", "preg(&raw const D40, core::mem::size_of_val(&D40)); preg(&raw const FCD, core::mem::size_of_val(&FCD));"),
    setup=("{ int q; zero(&D40, sizeof D40); zero(&FCD, sizeof FCD); fill_it_804D6D40_t(&D40); fill_ftCommonData(&FCD); it_804D6D40 = &D40; p_ftCommonData = &FCD;"
           " D40.x0 = (s32) (fv() % 8); D40.x14 = (float) (fv() % 6); D40.x18 = (float) (fv() % 8);"
           " for (q = 0; q < 8; q++) JO[q].flags &= ~JOBJ_USE_QUATERNION; }",
           "{ D40 = core::mem::zeroed(); FCD = core::mem::zeroed(); g::fill_it_804D6D40_t(&mut D40); g::fill_ftCommonData(&mut FCD); m::ext::it_804D6D40 = &raw mut D40; m::ext::p_ftCommonData = &raw mut FCD;"
           " D40.x0 = (fv() % 8) as i32; D40.x14 = (fv() % 6) as f32; D40.x18 = (fv() % 8) as f32;"
           " for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } }"),
    hash=("hash_it_804D6D40_t(&D40);", "g::hash_it_804D6D40_t(&D40);"),
    noret=["it_8027BA54"],
    ov={"it_8027B5B0": {1: _VPN, 2: ("&JO[rnd() % 8]", "&raw mut JO[(rnd() % 8) as usize]"), 3: _VPN, 4: ("(int) (fv() % 3)", "(fv() % 3) as i32")},
        "it_8027BBF4": {2: _F64}, "it_8027C0F0": {1: _VPN, 2: _F64},
        "it_8027C824": {1: ("(s32 (*)(Item_GObj*)) pick_fn(2)", "unsafe { core::mem::transmute::<usize, _>(pick_fn(2)) }")}},
)

STEMS["itzako"]["post"] = {"it_8027B5B0": (
    "{ int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } }",
    "for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); }")}
STEMS["itzako"]["post"]["it_8027CE64"] = STEMS["itzako"]["post"]["it_8027B5B0"]
STEMS["itzako"]["n"] = {"it_8027C8D0": 6000, "it_8027BB1C": 1500}


def attrs_stem(words=16, ints=()):
    """Stem pieces for TUs that read ART[i].x4_specialAttributes: two attribute blocks of `words` 32-bit words,
    random floats (rw), except the word indexes in `ints` (small integers)."""
    ci = "".join(f" if (w == {i}) {{ ((int*) SATTR[q])[w] = (int) (fv() % 6); }}" + (" else" if False else "") for i in ints)
    c_fill = "((float*) SATTR[q])[w] = rw();"
    r_fill = "*(&raw mut SATTR[q] as *mut f32).add(w) = rw();"
    if ints:
        cond = " || ".join(f"w == {i}" for i in ints)
        c_fill = f"if ({cond}) ((int*) SATTR[q])[w] = (int) (fv() % 6); else ((float*) SATTR[q])[w] = rw();"
        r_fill = f"if {cond} {{ *(&raw mut SATTR[q] as *mut i32).add(w) = (fv() % 6) as i32; }} else {{ *(&raw mut SATTR[q] as *mut f32).add(w) = rw(); }}"
    return dict(
        c_extra=f"static u32 SATTR[2][{words}];",
        rs_extra=f"static mut SATTR: [[u32; {words}]; 2] = [[0; {words}]; 2];",
        pregs=("preg(SATTR, sizeof SATTR);", "preg(&raw const SATTR, core::mem::size_of_val(&SATTR));"),
        setup=(f"{{ int q, w; for (q = 0; q < 2; q++) {{ ART[q].x4_specialAttributes = SATTR[q]; for (w = 0; w < {words}; w++) {{ {c_fill} }} }} }}",
               f"{{ for q in 0..2usize {{ ART[q].x4_specialAttributes = (&raw mut SATTR[q]) as *mut c_void; for w in 0..{words}usize {{ {r_fill} }} }} }}"),
        hash=(f"{{ int q, w; for (q = 0; q < 2; q++) for (w = 0; w < {words}; w++) HM(SATTR[q][w]); }}",
              f"{{ for q in 0..2usize {{ for w in 0..{words}usize {{ hm(SATTR[q][w]); }} }} }}"),
    )

STEMS["itdkinoko"] = attrs_stem(4)

# union itemVar whose first word is a pointer: hashed by region id and cleared after the call (raw bytes hold addresses)
PTR_POST = ("{ int q; for (q = 0; q < 4; q++) { HM(PI(*(void**) &IT[q].xDD4_itemVar)); *(void**) &IT[q].xDD4_itemVar = 0; } }",
            "for q in 0..4usize { let pp = (&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void; hm(pi(*pp)); *pp = null_mut(); }")
STEMS["itclinkmilk"] = dict(
    post_all=PTR_POST,
    setup=("{ int q; for (q = 0; q < 4; q++) { u32 r = rnd(); void* p = (r & 3) == 0 ? (void*) 0 : (void*) &GO[4 + (r >> 8) % 3];"
           " *(void**) &IT[q].xDD4_itemVar = p; IT[q].owner = (r & 16) ? (HSD_GObj*) p : pick_fgo(); } }",
           "{ for q in 0..4usize { let r = rnd(); let p: *mut c_void = if r & 3 == 0 { null_mut() } else { &raw mut GO[4 + ((r >> 8) % 3) as usize] as *mut c_void };"
           " *((&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void) = p; IT[q].owner = if r & 16 != 0 { p as *mut HSD_GObj } else { pick_fgo() }; } }"),
)
STEMS["itcrazyhandbomb"] = attrs_stem(3)

_COIN_PC = ("{ int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = 0; } }",
            "for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.coin.x14)); IT[q].xDD4_itemVar.coin.x14 = null_mut(); }")
def merge(*cfgs):
    out = {}
    for c in cfgs:
        for k, v in c.items():
            if k in out and isinstance(v, tuple):
                out[k] = (out[k][0] + " " + v[0], out[k][1] + " " + v[1])
            elif k in out and isinstance(v, str):
                out[k] = out[k] + chr(10) + v
            else:
                out[k] = v
    return out

# the HSD_JObjSetRotation* asserts (C: __assert never returns) need non-quaternion jobjs
NOQUAT = dict(setup=("{ int q; for (q = 0; q < 8; q++) JO[q].flags &= ~JOBJ_USE_QUATERNION; }",
                     "{ for q in 0..8usize { JO[q].flags &= !(JOBJ_USE_QUATERNION as u32); } }"))
STEMS["itcoin"] = merge(attrs_stem(19), NOQUAT, {"post_all": _COIN_PC})

_PILL_PC = ("{ int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = 0; } }",
            "for q in 0..4usize { hm(pi(IT[q].xDD4_itemVar.drmariopill.x4)); IT[q].xDD4_itemVar.drmariopill.x4 = null_mut(); }")
_PILL_SETUP = ("{ int q; for (q = 0; q < 4; q++) { u32 r = rnd(); HSD_GObj* p = (r & 3) == 0 ? (HSD_GObj*) 0 : &GO[4 + (r >> 8) % 3];"
               " IT[q].xDD4_itemVar.drmariopill.x4 = p; IT[q].owner = (r & 16) ? p : pick_fgo(); } }",
               "{ for q in 0..4usize { let r = rnd(); let p: *mut HSD_GObj = if r & 3 == 0 { null_mut() } else { &raw mut GO[4 + ((r >> 8) % 3) as usize] };"
               " IT[q].xDD4_itemVar.drmariopill.x4 = p; IT[q].owner = if r & 16 != 0 { p } else { pick_fgo() }; } }")
STEMS["itdrmariopill"] = merge(attrs_stem(6), NOQUAT, {"setup": _PILL_SETUP, "post_all": _PILL_PC})

_ICE_SETUP = ("{ int q; for (q = 0; q < 4; q++) { u32 r = rnd(); IT[q].kind = It_Kind_IceClimber_Ice + (r & 1);"
              " *(void**) &IT[q].xDD4_itemVar = (r & 6) == 0 ? (void*) 0 : (void*) &GO[4 + (r >> 8) % 3]; } }",
              "{ for q in 0..4usize { let r = rnd(); IT[q].kind = It_Kind_IceClimber_Ice + (r & 1) as i32;"
              " *((&raw mut IT[q].xDD4_itemVar) as *mut *mut c_void) = if r & 6 == 0 { null_mut() } else { &raw mut GO[4 + ((r >> 8) % 3) as usize] as *mut c_void }; } }")
from specs_ef import EF_PRE as _EFPRE2
STEMS["itclimbersice"] = merge(attrs_stem(13, ints=(11, 12)), {"c_pre": _EFPRE2, "setup": _ICE_SETUP, "post_all": PTR_POST})

_CS_PC = ("{ int q; for (q = 0; q < 4; q++) { HM(PI(IT[q].xDD4_itemVar.climbersstring.x4)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x8)); HM(PI(IT[q].xDD4_itemVar.climbersstring.xC)); HM(PI(IT[q].xDD4_itemVar.climbersstring.x14));"
          " IT[q].xDD4_itemVar.climbersstring.x4 = 0; IT[q].xDD4_itemVar.climbersstring.x8 = 0; IT[q].xDD4_itemVar.climbersstring.xC = 0; IT[q].xDD4_itemVar.climbersstring.x14 = 0; } }",
          "for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.climbersstring; hm(pi(v.x4)); hm(pi(v.x8)); hm(pi(v.xC)); hm(pi(v.x14)); v.x4 = null_mut(); v.x8 = null_mut(); v.xC = null_mut(); v.x14 = null_mut(); }")
_CS_SETUP = ("{ int q, j; setup_xpools(); LKN = 0; zero(PARTS, sizeof PARTS); zero(OUTP, sizeof OUTP); zero(MXA, sizeof MXA);"
             " for (q = 0; q < 12; q++) MXA[q] = rw();"
             " for (q = 0; q < 32; q++) PARTS[q].joint = &JO[rnd() % 8];"
             " for (q = 0; q < 3; q++) { FT[q].parts = PARTS; FT[q].u.pp.x2240.x = (rnd() & 1) ? 0.0f : rw(); FT[q].u.pp.x2240.y = (rnd() & 1) ? 0.0f : rw(); }"
             " for (j = 0; j < 6; j++) { LK[j].next = j ? &LK[j - 1] : 0; LK[j].prev = j < 5 ? &LK[j + 1] : 0; LK[j].gobj = &GO[7 + (rnd() & 1)]; LK[j].jobj = &JO[rnd() % 8]; }"
             " LK[5].x2C_b0 = 1;"
             " for (q = 0; q < 4; q++) { u32 r = rnd(); IT[q].xDD4_itemVar.climbersstring.x0 = rw(); IT[q].xDD4_itemVar.climbersstring.x4 = &LK[5]; IT[q].xDD4_itemVar.climbersstring.x8 = &LK[0];"
             " IT[q].xDD4_itemVar.climbersstring.xC = &GO[4 + (r >> 8) % 3]; IT[q].xDD4_itemVar.climbersstring.x14 = &JO[(r >> 12) % 8]; } }",
             "{ setup_xpools_(); LKN = 0; PARTS = core::mem::zeroed(); OUTP = [null_mut(); 2]; MXA = [0.0; 12];"
             " for q in 0..12usize { MXA[q] = rw(); }"
             " for q in 0..32usize { PARTS[q].joint = &raw mut JO[(rnd() % 8) as usize]; }"
             " for q in 0..3usize { FT[q].parts = (&raw mut PARTS) as *mut FighterBone; FT[q].u.pp.x2240.x = if rnd() & 1 != 0 { 0.0 } else { rw() }; FT[q].u.pp.x2240.y = if rnd() & 1 != 0 { 0.0 } else { rw() }; }"
             " for j in 0..6usize { g::LK[j].next = if j != 0 { &raw mut g::LK[j - 1] } else { null_mut() }; g::LK[j].prev = if j < 5 { &raw mut g::LK[j + 1] } else { null_mut() }; g::LK[j].gobj = &raw mut GO[7 + (rnd() & 1) as usize]; g::LK[j].jobj = &raw mut JO[(rnd() % 8) as usize]; }"
             " g::LK[5].set_x2C_b0(1);"
             " for q in 0..4usize { let r = rnd(); let v = &mut IT[q].xDD4_itemVar.climbersstring; v.x0 = rw(); v.x4 = &raw mut g::LK[5]; v.x8 = &raw mut g::LK[0];"
             " v.xC = &raw mut GO[4 + ((r >> 8) % 3) as usize]; v.x14 = &raw mut JO[((r >> 12) % 8) as usize]; } }")
STEMS["itclimbersstring"] = merge(attrs_stem(14, ints=(0, 1, 6, 7, 8)), NOQUAT, dict(
    c_pre="static int LKN;", c_extra="static FighterBone PARTS[32]; static ItemLink* OUTP[2]; static float MXA[12];",
    rs_extra="static mut LKN: usize = 0; static mut PARTS: [FighterBone; 32] = unsafe { core::mem::zeroed() }; static mut OUTP: [*mut ItemLink; 2] = [null_mut(); 2]; static mut MXA: [f32; 12] = [0.0; 12];"
             " unsafe fn setup_xpools_() { g::setup_xpools(); }",
    pregs=("preg_xpools(); preg(PARTS, sizeof PARTS); preg(OUTP, sizeof OUTP); preg(MXA, sizeof MXA);",
           "g::preg_xpools(); preg(&raw const PARTS, core::mem::size_of_val(&PARTS)); preg(&raw const OUTP, core::mem::size_of_val(&OUTP)); preg(&raw const MXA, core::mem::size_of_val(&MXA));"),
    setup=_CS_SETUP, post_all=_CS_PC,
    hash=("hash_xpools(); { int q; for (q = 0; q < 32; q++) HM(PI(PARTS[q].joint)); HM(PI(OUTP[0])); HM(PI(OUTP[1])); for (q = 0; q < 12; q++) HM(B(MXA[q])); }",
          "g::hash_xpools(); { for q in 0..32usize { hm(pi(PARTS[q].joint)); } hm(pi(OUTP[0])); hm(pi(OUTP[1])); for q in 0..12usize { hm(MXA[q].to_bits()); } }"),
    ov={"it_802C248C_setupGObj": {2: ("(GObj_RenderFunc) pick_cb(1)", "core::mem::transmute::<usize, _>(pick_cb(1))")},
        "Item_InitLinkMtx": {0: ("(float (*)[4]) MXA", "(&raw mut MXA) as *mut Mtx")},
        "it_802C27D4": {0: ("&GO[4 + rnd() % 3]", "&raw mut GO[4 + (rnd() % 3) as usize]")},
        "it_2725_Logic70_EvtUnk": {1: ("&GO[rnd() % 7]", "&raw mut GO[(rnd() % 7) as usize]")}},
))

# itemVar is hashed raw here (dosei view only) and cleared, so bitfield-bearing union members cannot alias the dosei floats
_RAWVAR = ("{ int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } }",
           "for q in 0..4usize { let v = IT[q].xDD4_itemVar.dosei; hm(v.xDD4 as u32); hm(v.xDD8 as u32); hm(v.xDDC.to_bits()); hm(v.xDE0.to_bits()); hm(v.xDE4.x.to_bits()); hm(v.xDE4.y.to_bits()); hm(v.xDE4.z.to_bits()); hm(v.xDF0 as u32); hm(v.xDF4.to_bits()); hm(v.xDF8.x.to_bits()); hm(v.xDF8.y.to_bits()); hm(v.xDF8.z.to_bits()); core::ptr::write_bytes(&raw mut IT[q].xDD4_itemVar as *mut u8, 0, core::mem::size_of_val(&IT[q].xDD4_itemVar)); }")
_DOSEI_SETUP = ("{ int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; v->xDD4 = (int) (fv() % 5); v->xDD8 = (int) (fv() % 4); v->xDDC = rw(); v->xDE0 = rw();"
                " v->xDE4.x = rw(); v->xDE4.y = rw(); v->xDE4.z = rw(); v->xDF0 = (int) (fv() % 4); v->xDF4 = rw(); v->xDF8.x = rw(); v->xDF8.y = rw(); v->xDF8.z = rw(); } }",
                "{ for q in 0..4usize { let v = &mut IT[q].xDD4_itemVar.dosei; v.xDD4 = (fv() % 5) as i32; v.xDD8 = (fv() % 4) as i32; v.xDDC = rw(); v.xDE0 = rw();"
                " v.xDE4.x = rw(); v.xDE4.y = rw(); v.xDE4.z = rw(); v.xDF0 = (fv() % 4) as i32; v.xDF4 = rw(); v.xDF8.x = rw(); v.xDF8.y = rw(); v.xDF8.z = rw(); } }")
STEMS["itdosei"] = merge(attrs_stem(6, ints=(1, 3, 4, 5)), NOQUAT, {"setup": _DOSEI_SETUP, "post_all": _RAWVAR})
