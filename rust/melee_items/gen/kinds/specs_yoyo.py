"""Driver description of itnessyoyo (shard 753-760): a chain of string links (LK), their gobjs / jobjs, a fighter
for the yo-yo's owner. Imported by mk_driver.py after specs_ness."""
from specs_cfg import CFG
from specs_ness import spec_all

_nv = "IT[k].xDD4_itemVar.nessyoyo"
_YA = "(itYoyoAttributes*) SPEC[0]"
_YR = "SPECB.0[0].as_mut_ptr() as *mut itYoyoAttributes"
_C_DECL = """static ItemLink LK[6];
static HSD_GObj LG[6];
static HSD_JObj LJ[6];
static HSD_JObj LB[6];
static Fighter FPL;
static HSD_GObj GF;
static FighterBone PARTS[64];
static unsigned CNT[4];
"""
_R_DECL = """static mut LK: [ItemLink; 6] = unsafe { core::mem::zeroed() };
static mut LG: [HSD_GObj; 6] = unsafe { core::mem::zeroed() };
static mut LJ: [HSD_JObj; 6] = unsafe { core::mem::zeroed() };
static mut LB: [HSD_JObj; 6] = unsafe { core::mem::zeroed() };
static mut FPL: Fighter = unsafe { core::mem::zeroed() };
static mut GF: HSD_GObj = unsafe { core::mem::zeroed() };
static mut PARTS: [FighterBone; 64] = unsafe { core::mem::zeroed() };
static mut CNT: [usize; 4] = [0; 4];
"""
_TW_C = """    {
        int n, j;
        zero(LK, sizeof LK); zero(LG, sizeof LG); zero(LJ, sizeof LJ); zero(LB, sizeof LB);
        zero(&FPL, sizeof FPL); zero(&GF, sizeof GF); zero(PARTS, sizeof PARTS); zero(CNT, sizeof CNT);
        for (j = 0; j < 6; j++) {
            fill_ItemLink(&LK[j]); fill_HSD_GObj(&LG[j]); fill_HSD_JObj(&LJ[j]); fill_HSD_JObj(&LB[j]);
            LJ[j].flags &= ~JOBJ_USE_QUATERNION; LB[j].flags &= ~JOBJ_USE_QUATERNION;
            LJ[j].child = (rnd() & 1) ? &LB[j] : 0;
        }
        n = 1 + (int) (rnd() % 4);
        for (j = 0; j < 6; j++) {
            LK[j].prev = (j < n && j + 1 < n) ? &LK[j + 1] : 0;
            LK[j].next = (j < n && j > 0) ? &LK[j - 1] : 0;
            LK[j].gobj = &LG[j];
            LK[j].jobj = &LB[j];
            LG[j].hsd_obj = &LJ[j];
        }
        j = (int) (rnd() % (u32) n);
        LK[j].x2C_b0 = 1;
        FPL.u.ns.yoyo_hitbox_pos.x = rw(); FPL.u.ns.yoyo_hitbox_pos.y = rw(); FPL.u.ns.yoyo_hitbox_pos.z = rw();
        FPL.u.ns.x223C = rw();
        FPL.parts = PARTS;
        PARTS[FtPart_R2ndNa].joint = &LB[0];
        GF.user_data = &FPL;
        for (k = 0; k < 3; k++) {
            i = (int) (rnd() % 3);
            IT[k].xDD4_itemVar.nessyoyo.x0 = (i == 0) ? 0x156 : (i == 1) ? 0x159 : (int) (rnd() % 0x20);
            IT[k].xDD4_itemVar.nessyoyo.x4 = rw();
            IT[k].xDD4_itemVar.nessyoyo.x8 = &LK[n - 1];
            IT[k].xDD4_itemVar.nessyoyo.xC = &LK[0];
            IT[k].xDD4_itemVar.nessyoyo.x10 = &GF;
            i = (int) (rnd() % 4);
            IT[k].xDD4_itemVar.nessyoyo.x18 = (i == 0) ? 0 : &LJ[(int) (rnd() % (u32) n)];
        }
    }"""
_TW_R = """    {
        LK = core::mem::zeroed(); LG = core::mem::zeroed(); LJ = core::mem::zeroed(); LB = core::mem::zeroed();
        FPL = core::mem::zeroed(); GF = core::mem::zeroed(); PARTS = core::mem::zeroed(); CNT = [0; 4];
        for j in 0..6 {
            g::fill_ItemLink(&mut LK[j]); g::fill_HSD_GObj(&mut LG[j]); g::fill_HSD_JObj(&mut LJ[j]); g::fill_HSD_JObj(&mut LB[j]);
            LJ[j].flags &= !(JOBJ_USE_QUATERNION as u32); LB[j].flags &= !(JOBJ_USE_QUATERNION as u32);
            LJ[j].child = if rnd() & 1 != 0 { &raw mut LB[j] } else { null_mut() };
        }
        let n = 1 + (rnd() % 4) as usize;
        for j in 0..6usize {
            LK[j].prev = if j < n && j + 1 < n { &raw mut LK[j + 1] } else { null_mut() };
            LK[j].next = if j < n && j > 0 { &raw mut LK[j - 1] } else { null_mut() };
            LK[j].gobj = &raw mut LG[j];
            LK[j].jobj = &raw mut LB[j];
            LG[j].hsd_obj = &raw mut LJ[j] as *mut _;
        }
        let j = (rnd() % n as u32) as usize;
        LK[j].set_x2C_b0(1);
        FPL.u.ns.yoyo_hitbox_pos.x = rw(); FPL.u.ns.yoyo_hitbox_pos.y = rw(); FPL.u.ns.yoyo_hitbox_pos.z = rw();
        FPL.u.ns.x223C = rw();
        FPL.parts = &raw mut PARTS as *mut FighterBone;
        PARTS[FtPart_R2ndNa as usize].joint = &raw mut LB[0];
        GF.user_data = &raw mut FPL as *mut _;
        for k in 0..3 {
            let i = rnd() % 3;
            IT[k].xDD4_itemVar.nessyoyo.x0 = if i == 0 { 0x156 } else if i == 1 { 0x159 } else { (rnd() % 0x20) as i32 };
            IT[k].xDD4_itemVar.nessyoyo.x4 = rw();
            IT[k].xDD4_itemVar.nessyoyo.x8 = &raw mut LK[n - 1];
            IT[k].xDD4_itemVar.nessyoyo.xC = &raw mut LK[0];
            IT[k].xDD4_itemVar.nessyoyo.x10 = &raw mut GF;
            let i = rnd() % 4;
            IT[k].xDD4_itemVar.nessyoyo.x18 = if i == 0 { null_mut() } else { let q = (rnd() % n as u32) as usize; &raw mut LJ[q] };
        }
    }"""
_PTRS_C = """    for (k = 0; k < 3; k++) {
        HM(PI(%s.x8)); %s.x8 = 0; HM(PI(%s.xC)); %s.xC = 0; HM(PI(%s.x10)); %s.x10 = 0; HM(PI(%s.x18)); %s.x18 = 0;
    }
    for (i = 0; i < 6; i++) { hash_ItemLink(&LK[i]); hash_HSD_GObj(&LG[i]); hash_HSD_JObj(&LJ[i]); hash_HSD_JObj(&LB[i]); }
    HM(B(FPL.u.ns.yoyo_hitbox_pos.x)); HM(B(FPL.u.ns.yoyo_hitbox_pos.y)); HM(B(FPL.u.ns.yoyo_hitbox_pos.z));
    HM(B(FPL.u.ns.x223C)); HM(CNT[0]); HM(CNT[1]); HM(CNT[2]);""" % ((_nv,) * 8)
_PTRS_R = """    for k in 0..3 {
        let nv = &mut IT[k].xDD4_itemVar.nessyoyo;
        hm(pi(nv.x8)); nv.x8 = null_mut(); hm(pi(nv.xC)); nv.xC = null_mut(); hm(pi(nv.x10)); nv.x10 = null_mut(); hm(pi(nv.x18)); nv.x18 = null_mut();
    }
    for i in 0..6 { g::hash_ItemLink(&LK[i]); g::hash_HSD_GObj(&LG[i]); g::hash_HSD_JObj(&LJ[i]); g::hash_HSD_JObj(&LB[i]); }
    hm(FPL.u.ns.yoyo_hitbox_pos.x.to_bits()); hm(FPL.u.ns.yoyo_hitbox_pos.y.to_bits()); hm(FPL.u.ns.yoyo_hitbox_pos.z.to_bits());
    hm(FPL.u.ns.x223C.to_bits()); hm(CNT[0] as u32); hm(CNT[1] as u32); hm(CNT[2] as u32);"""

CFG["itnessyoyo"] = dict(
    spec=[spec_all("itYoyoAttributes",
                   "a->x0_CHARGE_SPAWN_POS = (int) (rnd() % 5); a->x4_UPSMASH_CHARGE_STRING_LENGTH = (int) (rnd() % 6); a->x8_DOWNSMASH_CHARGE_STRING_LENGTH = (int) (rnd() % 6); a->xC_SIZE = rw(); a->x10_UNK1 = rw(); a->x28_YOYO_PULL_STRENGTH = rw(); a->x2C_UNK3_MOD = rw(); a->x34_GRAVITY = rw(); a->x38_GRAVITY_MOD = rw(); a->x3C_YOYO_PULL_STRENGTH_2 = rw();",
                   "(*a).x0_CHARGE_SPAWN_POS = (rnd() % 5) as i32; (*a).x4_UPSMASH_CHARGE_STRING_LENGTH = (rnd() % 6) as i32; (*a).x8_DOWNSMASH_CHARGE_STRING_LENGTH = (rnd() % 6) as i32; (*a).xC_SIZE = rw(); (*a).x10_UNK1 = rw(); (*a).x28_YOYO_PULL_STRENGTH = rw(); (*a).x2C_UNK3_MOD = rw(); (*a).x34_GRAVITY = rw(); (*a).x38_GRAVITY_MOD = rw(); (*a).x3C_YOYO_PULL_STRENGTH_2 = rw();")],
    c_decl=_C_DECL, rs_decl=_R_DECL,
    c_preg=["preg(LK, sizeof LK); preg(LG, sizeof LG); preg(LJ, sizeof LJ); preg(LB, sizeof LB); preg(&FPL, sizeof FPL); preg(&GF, sizeof GF); preg(PARTS, sizeof PARTS);"],
    rs_preg=["preg(&raw const LK, core::mem::size_of_val(&LK)); preg(&raw const LG, core::mem::size_of_val(&LG)); preg(&raw const LJ, core::mem::size_of_val(&LJ)); preg(&raw const LB, core::mem::size_of_val(&LB)); preg(&raw const FPL, core::mem::size_of_val(&FPL)); preg(&raw const GF, core::mem::size_of_val(&GF)); preg(&raw const PARTS, core::mem::size_of_val(&PARTS));"],
    tweak=(_TW_C, _TW_R), ptrs=(_PTRS_C, _PTRS_R),
    over={
        "it_802BE5D8": ("it_802BE5D8(&GO[0], rw());", "m::it_802BE5D8(&raw mut GO[0] as *mut core::ffi::c_void, rw());"),
        "it_802BE9D8": ("{ float f = rw(); int a = (rnd() % 3 == 0) ? 0x159 : (int) (rnd() % 0x20); ret = PI(it_802BE9D8(&GF, f, &VA[0], a)); }",
                        "{ let f = rw(); let a = if rnd() % 3 == 0 { 0x159 } else { (rnd() % 0x20) as i32 }; ret = pi(m::it_802BE9D8(&raw mut GF, f, &raw mut VA[0], a)); }"),
        "it_802BF030": ("{ int ix = (int) (rnd() % 3); int a = (int) fv(); float o = rw(); ret = (u32) it_802BF030(&LK[ix], a, o); }",
                        "{ let ix = (rnd() % 3) as usize; let a = fv() as i32; let o = rw(); ret = m::it_802BF030(&raw mut LK[ix], a, o) as u32; }"),
        "it_802BF180": ("{ int ix = (int) (rnd() % 3); float l = rw(); it_802BF180(&LK[ix], &VA[0], %s, l); }".replace("%s", _YA),
                        "{ let ix = (rnd() % 3) as usize; let l = rw(); m::it_802BF180(&raw mut LK[ix], &raw mut VA[0], %s, l); }".replace("%s", _YR)),
        "it_802BF28C": ("{ int ix = (int) (rnd() % 3); ret = (u32) it_802BF28C(&LK[ix], &VA[0], %s, &IT[0]); }".replace("%s", _YA),
                        "{ let ix = (rnd() % 3) as usize; ret = m::it_802BF28C(&raw mut LK[ix], &raw mut VA[0], %s, &raw mut IT[0]) as u32; }".replace("%s", _YR)),
        "it_802BF4A0": ("{ int ix = (int) (rnd() % 3); ret = (u32) it_802BF4A0(&LK[ix], &VA[0], %s, &IT[0]); }".replace("%s", _YA),
                        "{ let ix = (rnd() % 3) as usize; ret = m::it_802BF4A0(&raw mut LK[ix], &raw mut VA[0], %s, &raw mut IT[0]) as u32; }".replace("%s", _YR)),
        "it_802BF800": ("{ int ix = (int) (rnd() % 3); float d = rw(); ret = (u32) it_802BF800(&LK[ix], &VA[0], %s, &IT[0], d); }".replace("%s", _YA),
                        "{ let ix = (rnd() % 3) as usize; let d = rw(); ret = m::it_802BF800(&raw mut LK[ix], &raw mut VA[0], %s, &raw mut IT[0], d) as u32; }".replace("%s", _YR)),
    },
)
