"""Driver descriptions of the Ness item TUs (shard 753-760); imported by specs_cfg.py's users (mk_driver.py)."""
from specs_cfg import CFG, per_item

HINT_KIND = ("(int) (rnd() % 150)", "(rnd() % 150) as i32")


def spec_all(cty, c_fill, r_fill):
    """Fill the typed special-attribute block of all three articles (the spawned item is IT[2])."""
    return ("        { int j; for (j = 0; j < 3; j++) { %s* a = (%s*) SPEC[j];\n            %s } }" % (cty, cty, c_fill),
            "            for j in 0..3 {\n                let a = SPECB.0[j].as_mut_ptr() as *mut %s;\n                %s\n            }" % (cty, r_fill))


def pick_ptr(c_dst, r_dst):
    """dst = NULL / owner / another gobj (one draw)."""
    return ("        i = (int) (rnd() %% 4);\n        %s = (i == 0) ? 0 : (i == 1) ? IT[k].owner : &GO[(k + i) %% 3];" % c_dst,
            "        let i = rnd() %% 4;\n        %s = if i == 0 { null_mut() } else if i == 1 { IT[k].owner } else { &raw mut GO[(k + i as usize) %% 3] };" % r_dst)


_nb = "IT[k].xDD4_itemVar.nessbat"
_p = pick_ptr(_nb + ".x0", _nb + ".x0")
CFG["itnessbat"] = dict(
    spec=[("", "")],
    tweak=per_item(
        _p[0] + "\n        if (rnd() & 1) { IT[k].xDAC_itcmd_var0 = 0; IT[k].xDB0_itcmd_var1 = 0; IT[k].xDB4_itcmd_var2 = 0; IT[k].xDB8_itcmd_var3 = 0; }",
        _p[1] + "\n        if rnd() & 1 != 0 { IT[k].xDAC_itcmd_var0 = 0; IT[k].xDB0_itcmd_var1 = 0; IT[k].xDB4_itcmd_var2 = 0; IT[k].xDB8_itcmd_var3 = 0; }"),
    ptrs=("    for (k = 0; k < 3; k++) { HM(PI(%s.x0)); %s.x0 = 0; }" % (_nb, _nb),
          "    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.nessbat.x0)); IT[k].xDD4_itemVar.nessbat.x0 = null_mut(); }"),
)

_FP_ATTR = spec_all("itNessPKFirepillarAttributes", "a->x0 = (float) (rnd() % 60); a->x4 = rw(); a->scale = rw();",
                    "(*a).x0 = (rnd() % 60) as f32; (*a).x4 = rw(); (*a).scale = rw();")
CFG["itnesspkfire"] = dict(spec=[_FP_ATTR], tweak=("", ""), ptrs=("", ""))
CFG["itnesspkfirepillar"] = dict(spec=[_FP_ATTR], tweak=("", ""), ptrs=("", ""))

_pf = "IT[k].xDD4_itemVar.pkflush"
_p = pick_ptr(_pf + ".xDE0_PKFlash_Owner", _pf + ".xDE0_PKFlash_Owner")
CFG["itnesspkflash"] = dict(
    spec=[spec_all("itFlashAttributes",
                   "int q; float* f = (float*) a; for (q = 0; q < 11; q++) f[q] = rw(); a->x28_FLASH_EXPLOSION_DELAY = (float) (rnd() % 8); a->x4_FLASH_HITBOX_SIZE_MUL = (float) (rnd() % 6);",
                   "let f = a as *mut f32; for q in 0..11 { *f.add(q) = rw(); } (*a).x28_FLASH_EXPLOSION_DELAY = (rnd() % 8) as f32; (*a).x4_FLASH_HITBOX_SIZE_MUL = (rnd() % 6) as f32;")],
    tweak=per_item(
        _p[0] + """
        IT[k].xDD4_itemVar.pkflush.xDD8_PKFlash = rw();
        IT[k].xDD4_itemVar.pkflush.xDDC_PKFlash = (int) (rnd() & 1);
        i = (int) (rnd() % 3);
        IT[k].kind = (i == 0) ? It_Kind_Ness_PKFlush : (i == 1) ? It_Kind_Kirby_NessPKFlush : It_Kind_Ness_Bat;
        IT[k].msid = (int) (rnd() % 4);
        IT[k].xDAC_itcmd_var0 = (int) (rnd() % 8);
        IT[k].x40_vel.x = rg(); IT[k].x40_vel.y = rg();""",
        _p[1] + """
        IT[k].xDD4_itemVar.pkflush.xDD8_PKFlash = rw();
        IT[k].xDD4_itemVar.pkflush.xDDC_PKFlash = (rnd() & 1) as i32;
        let i = rnd() % 3;
        IT[k].kind = if i == 0 { It_Kind_Ness_PKFlush } else if i == 1 { It_Kind_Kirby_NessPKFlush } else { It_Kind_Ness_Bat };
        IT[k].msid = (rnd() % 4) as i32;
        IT[k].xDAC_itcmd_var0 = rnd() % 8;
        IT[k].x40_vel.x = rg(); IT[k].x40_vel.y = rg();"""),
    ptrs=("    for (k = 0; k < 3; k++) { HM(PI(%s.xDE0_PKFlash_Owner)); %s.xDE0_PKFlash_Owner = 0; }" % (_pf, _pf),
          "    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.pkflush.xDE0_PKFlash_Owner)); IT[k].xDD4_itemVar.pkflush.xDE0_PKFlash_Owner = null_mut(); }"),
    hints={"kind": HINT_KIND},
    over={
        "it_802AA810": ("ret = (u32) it_802AA810(&GO[0], &IT[0].x378_itemColl);",
                        "ret = m::it_802AA810(&raw mut GO[0], &raw mut IT[0].x378_itemColl) as u32;"),
    },
)

_fe = "IT[k].xDD4_itemVar.pkflushexplode"
CFG["itnesspkflashexplode"] = dict(
    spec=[spec_all("itFlashExplAttributes",
                   "a->x0_FLASH_EXPL_HITBOX_SIZE_MUL = (float) (1 + rnd() % 5); a->x4_FLASH_EXPL_GRAPHIC_SIZE_INIT = rw(); a->x8_FLASH_EXPL_GRAPHIC_SIZE_GROWTH = rw(); a->xC_FLASH_EXPL_BASE_DAMAGE = (float) (rnd() % 30); a->x10_FLASH_EXPL_DAMAGE_MUL = rw();",
                   "(*a).x0_FLASH_EXPL_HITBOX_SIZE_MUL = (1 + rnd() % 5) as f32; (*a).x4_FLASH_EXPL_GRAPHIC_SIZE_INIT = rw(); (*a).x8_FLASH_EXPL_GRAPHIC_SIZE_GROWTH = rw(); (*a).xC_FLASH_EXPL_BASE_DAMAGE = (rnd() % 30) as f32; (*a).x10_FLASH_EXPL_DAMAGE_MUL = rw();")],
    tweak=per_item(
        """        IT[k].xDD4_itemVar.pkflushexplode.xDD4 = (float) (rnd() % 6);
        IT[k].xDD4_itemVar.pkflushexplode.xDD8 = (rnd() & 1) ? 0.0f : rw();
        IT[k].xDD4_itemVar.pkflushexplode.xDDC = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDB4_itcmd_var2 = (int) (rnd() & 1);""",
        """        let fe = &mut IT[k].xDD4_itemVar.pkflushexplode;
        fe.xDD4 = (rnd() % 6) as f32;
        fe.xDD8 = if rnd() & 1 != 0 { 0.0 } else { rw() };
        fe.xDDC = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xDB4_itcmd_var2 = rnd() & 1;"""),
    ptrs=("    for (k = 0; k < 3; k++) { HM(PI(%s.xDDC)); %s.xDDC = 0; }" % (_fe, _fe),
          "    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.pkflushexplode.xDDC)); IT[k].xDD4_itemVar.pkflushexplode.xDDC = null_mut(); }"),
    hints={"kind": HINT_KIND},
)

_tt = "IT[k].xDD4_itemVar.nesspkthundertrail"
CFG["itnesspkthundertrail"] = dict(
    spec=[("", "")],
    tweak=per_item(
        """        IT[k].xDD4_itemVar.nesspkthundertrail.x0 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDD4_itemVar.nesspkthundertrail.x4 = (int) (rnd() % 6);
        IT[k].xDD4_itemVar.nesspkthundertrail.x8 = (int) (rnd() % 6);""",
        """        let tt = &mut IT[k].xDD4_itemVar.nesspkthundertrail;
        tt.x0 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        tt.x4 = (rnd() % 6) as i32;
        tt.x8 = (rnd() % 6) as i32;"""),
    ptrs=("    for (k = 0; k < 3; k++) { HM(PI(%s.x0)); %s.x0 = 0; }" % (_tt, _tt),
          "    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.nesspkthundertrail.x0)); IT[k].xDD4_itemVar.nesspkthundertrail.x0 = null_mut(); }"),
    hints={"trail_idx": ("(int) (rnd() % 8) - 1", "(rnd() % 8) as i32 - 1")},
    extra_ops=[("it_803F6C18", "{ int j; for (j = 0; j < 6; j++) HM((u32) it_803F6C18[j]); }",
                "for j in 0..6 { hm(m::it_803F6C18[j] as u32); }")],
)

_pk = "IT[k].xDD4_itemVar.pkthunder"
_p = pick_ptr(_pk + ".xF04", _pk + ".xF04")
CFG["itnesspkthunderball"] = dict(
    spec=[spec_all("itPKThunderAttributes",
                   "a->x0_PKTHUNDER_LIFETIME = rw(); a->x4_PKTHUNDER_SPEED = rw(); a->x8_PKTHUNDER_SPAWN_ANGLE = rw(); a->xC_PKTHUNDER_STICK_THRESHOLD = rg(); a->x10_PKTHUNDER_TURN_RADIUS = (float) (rnd() % 100);",
                   "(*a).x0_PKTHUNDER_LIFETIME = rw(); (*a).x4_PKTHUNDER_SPEED = rw(); (*a).x8_PKTHUNDER_SPAWN_ANGLE = rw(); (*a).xC_PKTHUNDER_STICK_THRESHOLD = rg(); (*a).x10_PKTHUNDER_TURN_RADIUS = (rnd() % 100) as f32;")],
    tweak=per_item(
        _p[0] + """
        { int j;
        for (j = 0; j < 6; j++) { IT[k].xDD4_itemVar.pkthunder.xDD4[j] = (rnd() & 1) ? &GO[(k + 1 + j) % 3] : 0; }
        IT[k].xDD4_itemVar.pkthunder.xEF4 = 0;
        IT[k].xDD4_itemVar.pkthunder.xF08 = 0;
        IT[k].xDD4_itemVar.pkthunder.xEF8 = (int) (rnd() % 14);
        IT[k].xDD4_itemVar.pkthunder.xEFC = (int) (rnd() % 8);
        IT[k].xDD4_itemVar.pkthunder.xF00 = (rnd() % 4) == 0;
        IT[k].xDD4_itemVar.pkthunder.xEEC = rw();
        for (j = 0; j < 16; j++) { IT[k].xDD4_itemVar.pkthunder.angles[j] = rg() * 4.0f; } }""",
        _p[1] + """
        let pk = &mut IT[k].xDD4_itemVar.pkthunder;
        for j in 0..6 { pk.xDD4[j] = if rnd() & 1 != 0 { &raw mut GO[(k + 1 + j) % 3] } else { null_mut() }; }
        pk.xEF4 = null_mut();
        pk.xF08 = null_mut();
        pk.xEF8 = (rnd() % 14) as i32;
        pk.xEFC = (rnd() % 8) as i32;
        pk.xF00 = (rnd() % 4 == 0) as i32;
        pk.xEEC = rw();
        for j in 0..16 { pk.angles[j] = rg() * 4.0; }"""),
    ptrs=("""    for (k = 0; k < 3; k++) {
        int j;
        for (j = 0; j < 6; j++) { HM(PI(%s.xDD4[j])); %s.xDD4[j] = 0; }
        HM(PI(%s.xEF4)); %s.xEF4 = 0; HM(PI(%s.xF04)); %s.xF04 = 0; HM(PI(%s.xF08)); %s.xF08 = 0;
    }""" % ((_pk,) * 8),
          """    for k in 0..3 {
        let pk = &mut IT[k].xDD4_itemVar.pkthunder;
        for j in 0..6 { hm(pi(pk.xDD4[j])); pk.xDD4[j] = null_mut(); }
        hm(pi(pk.xEF4)); pk.xEF4 = null_mut(); hm(pi(pk.xF04)); pk.xF04 = null_mut(); hm(pi(pk.xF08)); pk.xF08 = null_mut();
    }"""),
    hints={"idx": ("(int) (rnd() % 8)", "(rnd() % 8) as i32")},
    over={
        "it_802AB4B8": ("ret = (u32) it_802AB4B8(&GO[0], &IT[0].x378_itemColl);",
                        "ret = m::it_802AB4B8(&raw mut GO[0], &raw mut IT[0].x378_itemColl) as u32;"),
        "it_802AB468": ("it_802AB468(&GO[0], &VA[0].x, (int) fv());",
                        "m::it_802AB468(&raw mut GO[0], &raw mut VA[0].x, fv() as i32);"),
    },
    extra_ops=[
        ("it_802AB3F0(NULL)", "it_802AB3F0(0, &VA[0], 1);", "m::it_802AB3F0(null_mut(), &raw mut VA[0], 1);"),
        ("it_802AB3F0(out NULL)", "it_802AB3F0(&GO[0], 0, 1);", "m::it_802AB3F0(&raw mut GO[0], null_mut(), 1);"),
        ("it_802AB468(NULL)", "it_802AB468(0, &VA[0].x, 1);", "m::it_802AB468(null_mut(), &raw mut VA[0].x, 1);"),
        ("it_802AB568(NULL)", "ret = PI(it_802AB568(0));", "ret = pi(m::it_802AB568(null_mut()));"),
    ],
)
