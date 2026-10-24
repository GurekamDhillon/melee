"""Driver descriptions of the shard 801-808 item-kind TUs (itthunder, ittogepy, ittomato, ittosakinto, ittools,
itunknown, itwhispyapple, ittincle); same conventions as specs_cfg.py."""
import re
from specs_cfg import CFG, per_item, sbuf

# -------------------------------------------------------------------------------------- itthunder
_th = "IT[k].xDD4_itemVar.thunder"
CFG["itthunder"] = dict(
    spec=[("""        itThunderPokemonAttributes* a = (itThunderPokemonAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = (u32) (rnd() % 12);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itThunderPokemonAttributes;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rnd() % 12;""")],
    tweak=per_item(
        f"""        {_th}.x60 = (int) (rnd() % 5) - 1;
        {_th}.x64 = (int) (rnd() & 1);
        {_th}.x68 = rg();
        IT[k].xDAC_itcmd_var0 = (int) (rnd() & 1);
        IT[k].xDB0_itcmd_var1 = (int) (rnd() & 1);""",
        """        IT[k].xDD4_itemVar.thunder.x60 = (rnd() % 5) as i32 - 1;
        IT[k].xDD4_itemVar.thunder.x64 = (rnd() & 1) as i32;
        IT[k].xDD4_itemVar.thunder.x68 = rg();
        IT[k].xDAC_itcmd_var0 = rnd() & 1;
        IT[k].xDB0_itcmd_var1 = rnd() & 1;"""),
    ptrs=("", ""),
)

# ---------------------------------------------------------------------------------------- ittogepy
CFG["ittogepy"] = dict(
    spec=[("""        itPokemonAttributes* a = (itPokemonAttributes*) SPEC[0];
        a->x0 = rw(); a->timer = (int) (rnd() % 6) - 1; a->max = (int) (rnd() % 24) - 2; a->xC = (int) (rnd() % 8);
        a->x10 = (int) (rnd() % 8); a->x14 = (int) (rnd() % 8); a->x18 = (int) (rnd() % 8);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itPokemonAttributes;
            (*a).x0 = rw(); (*a).timer = (rnd() % 6) as i32 - 1; (*a).max = (rnd() % 24) as i32 - 2; (*a).xC = (rnd() % 8) as i32;
            (*a).x10 = (rnd() % 8) as i32; (*a).x14 = (rnd() % 8) as i32; (*a).x18 = (rnd() % 8) as i32;""")],
    tweak=per_item("        IT[k].xDD4_itemVar.pokemon.timer = (int) (rnd() % 5) - 2;\n        IT[k].xDAC_itcmd_var0 = rnd() & 1;",
                   "        IT[k].xDD4_itemVar.pokemon.timer = (rnd() % 5) as i32 - 2;\n        IT[k].xDAC_itcmd_var0 = rnd() & 1;"),
    ptrs=("", ""),
)

# ----------------------------------------------------------------------------------------- ittomato
CFG["ittomato"] = dict(
    spec=[("""        MaximTomatoSpecialAttr* a = (MaximTomatoSpecialAttr*) SPEC[0];
        a->heal_amount_0 = (int) (rnd() % 30); a->heal_amount_1 = (int) (rnd() % 30); a->x14 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut MaximTomatoSpecialAttr;
            (*a).heal_amount_0 = (rnd() % 30) as i32; (*a).heal_amount_1 = (rnd() % 30) as i32; (*a).x14 = rw();""")],
    tweak=per_item("""        IT[k].xDD4_itemVar.tomato.x8 = rnd() % 4;
        IT[k].xDD4_itemVar.tomato.x4_b0 = rnd() & 1;
        gm_80473A18.x90[k] = (u8) (1 + rnd() % 200);
        gm_80473A18.x90[3] = (u8) (1 + rnd() % 200);""",
                   """        IT[k].xDD4_itemVar.tomato.x8 = rnd() % 4;
        IT[k].xDD4_itemVar.tomato.set_x4_b0((rnd() & 1) as i32);
        m::ext::gm_80473A18.x90[k] = (1 + rnd() % 200) as u8;
        m::ext::gm_80473A18.x90[3] = (1 + rnd() % 200) as u8;"""),
    ptrs=("""    for (k = 0; k < 4; k++) { HM(gm_80473A18.x90[k]); gm_80473A18.x90[k] = 0; }
    for (k = 0; k < 3; k++) {
        HM(IT[k].xDD4_itemVar.tomato.heal_amount); HM(IT[k].xDD4_itemVar.tomato.x4_b0); HM(IT[k].xDD4_itemVar.tomato.x8);
        zero(&IT[k].xDD4_itemVar.tomato, sizeof IT[k].xDD4_itemVar.tomato);
    }""",
          """    for k in 0..4 { hm(m::ext::gm_80473A18.x90[k] as u32); m::ext::gm_80473A18.x90[k] = 0; }
    for k in 0..3 {
        let tm = &mut IT[k].xDD4_itemVar.tomato;
        hm(tm.heal_amount as u32); hm(tm.x4_b0() as u32); hm(tm.x8);
        core::ptr::write_bytes(tm as *mut _ as *mut u8, 0, core::mem::size_of::<itTomato_ItemVars>());
    }"""),
)

# -------------------------------------------------------------------------------------- ittosakinto
_ts = "IT[k].xDD4_itemVar.tosakinto"
CFG["ittosakinto"] = dict(
    spec=[("""        itTosakinto_Attrs* a = (itTosakinto_Attrs*) SPEC[0];
        a->x0_scale = rw(); a->x4_life_timer = rw(); a->x8_vel_x = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itTosakinto_Attrs;
            (*a).x0_scale = rw(); (*a).x4_life_timer = rw(); (*a).x8_vel_x = rw();""")],
    tweak=per_item(f"""        {_ts}.x60_msid = (int) (rnd() % 5);
        {_ts}.x64_vel_x = rg();
        {_ts}.x68_sfx_id = (int) fv();
        IT[k].xD44_lifeTimer = (rnd() & 1) ? (float) (int) (rnd() % 4) : rg();""",
                   """        IT[k].xDD4_itemVar.tosakinto.x60_msid = (rnd() % 5) as i32;
        IT[k].xDD4_itemVar.tosakinto.x64_vel_x = rg();
        IT[k].xDD4_itemVar.tosakinto.x68_sfx_id = fv() as i32;
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { (rnd() % 4) as i32 as f32 } else { rg() };"""),
    ptrs=("", ""),
)

# ----------------------------------------------------------------------------------------- ittools
CFG["ittools"] = dict(
    spec=[("""        itToolsAttributes* a = (itToolsAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = (float) (rnd() % 60); a->x8 = rw(); a->xC = (int) (rnd() % 100);
        for (i = 0; i < 4; i++) {
            float* m = (float*) &a->motions[0] + 7 * i;
            m[0] = rw(); m[1] = rw(); m[2] = rw(); m[3] = rw(); m[4] = rw(); m[5] = rw(); m[6] = rw();
        }""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itToolsAttributes;
            (*a).x0 = rw(); (*a).x4 = (rnd() % 60) as f32; (*a).x8 = rw(); (*a).xC = (rnd() % 100) as i32;
            for i in 0..4 {
                let m = ((*a).motions.as_mut_ptr() as *mut f32).add(7 * i);
                *m = rw(); *m.add(1) = rw(); *m.add(2) = rw(); *m.add(3) = rw(); *m.add(4) = rw(); *m.add(5) = rw(); *m.add(6) = rw();
            }""")],
    tweak=per_item("""        IT[k].xDD4_itemVar.tools.x0 = (int) (rnd() % 4);
        IT[k].xD44_lifeTimer = (rnd() & 1) ? (float) (int) (rnd() % 8) : rg();
        IT[k].xD48_halfLifeTimer = rg();
        IT[k].xCA0 = (int) (rnd() % 50);
        IT[k].xCCC_incDamageDirection = rg();
        IT[k].x5D4_hitboxes[0].hit.damage = (float) (int) (rnd() % 30);
        IT[k].xCEC_fighterGObj = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;""",
                   """        IT[k].xDD4_itemVar.tools.x0 = (rnd() % 4) as i32;
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { (rnd() % 8) as i32 as f32 } else { rg() };
        IT[k].xD48_halfLifeTimer = rg();
        IT[k].xCA0 = (rnd() % 50) as i32;
        IT[k].xCCC_incDamageDirection = rg();
        IT[k].x5D4_hitboxes[0].hit.damage = (rnd() % 30) as i32 as f32;
        IT[k].xCEC_fighterGObj = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(IT[k].xCEC_fighterGObj)); IT[k].xCEC_fighterGObj = 0; }""",
          """    for k in 0..3 { hm(pi(IT[k].xCEC_fighterGObj)); IT[k].xCEC_fighterGObj = null_mut(); }"""),
    hints={"arg1": ("(int) (rnd() % 4)", "(rnd() % 4) as i32")},
)

# ---------------------------------------------------------------------------------------- itunknown
_uk = "IT[k].xDD4_itemVar.unknown"
CFG["itunknown"] = dict(
    spec_size=512,
    spec=[("""        itUnknownAttributes* a = (itUnknownAttributes*) SPEC[0];
        a->x0.f = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw();
        a->x18.i = (int) (rnd() % 6); a->x1C.i = (int) (rnd() % 6); a->x20.i = (int) (rnd() % 6);
        if (rnd() & 1) { a->x1C.f = rw(); a->x20.f = rw(); a->x18.f = rw(); a->x0.i = (int) (rnd() % 100); }
        for (i = 0; i < 26; i++) a->x24[i] = (rnd() & 1) ? (HSD_Joint*) &JO[3 + (rnd() % 3)] : 0;""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itUnknownAttributes;
            (*a).x0.f = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw();
            (*a).x18.i = (rnd() % 6) as i32; (*a).x1C.i = (rnd() % 6) as i32; (*a).x20.i = (rnd() % 6) as i32;
            if rnd() & 1 != 0 { (*a).x1C.f = rw(); (*a).x20.f = rw(); (*a).x18.f = rw(); (*a).x0.i = (rnd() % 100) as i32; }
            for i in 0..26 { (*a).x24[i] = if rnd() & 1 != 0 { &raw mut JO[3 + (rnd() % 3) as usize] as *mut HSD_Joint } else { null_mut() }; }""")],
    tweak=per_item(f"""        {_uk}.x60 = (rnd() & 1) ? 1.0f : rg();
        {_uk}.x64.i = (int) (rnd() % 5) - 2;
        {_uk}.x68.i = (int) (rnd() % 4);
        {_uk}.x6C.x = rw(); {_uk}.x6C.y = rw(); {_uk}.x6C.z = rw();
        {_uk}.x78.vec.x = rw(); {_uk}.x78.vec.y = rw(); {_uk}.x78.vec.z = rw();
        {_uk}.x84 = (int) (rnd() % 5);
        IT[k].xD44_lifeTimer = (rnd() & 1) ? (float) (int) (rnd() % 4) : rg();""",
                   """        IT[k].xDD4_itemVar.unknown.x60 = if rnd() & 1 != 0 { 1.0 } else { rg() };
        IT[k].xDD4_itemVar.unknown.x64.i = (rnd() % 5) as i32 - 2;
        IT[k].xDD4_itemVar.unknown.x68.i = (rnd() % 4) as i32;
        IT[k].xDD4_itemVar.unknown.x6C.x = rw(); IT[k].xDD4_itemVar.unknown.x6C.y = rw(); IT[k].xDD4_itemVar.unknown.x6C.z = rw();
        IT[k].xDD4_itemVar.unknown.x78.vec.x = rw(); IT[k].xDD4_itemVar.unknown.x78.vec.y = rw(); IT[k].xDD4_itemVar.unknown.x78.vec.z = rw();
        IT[k].xDD4_itemVar.unknown.x84 = (rnd() % 5) as i32;
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { (rnd() % 4) as i32 as f32 } else { rg() };"""),
    ptrs=("", ""),
)

# ------------------------------------------------------------------------------------ itwhispyapple
CFG["itwhispyapple"] = dict(
    spec=[("""        itWhispyAppleAttributes* a = (itWhispyAppleAttributes*) SPEC[0];
        a->x4 = (int) (rnd() % 30); a->x8 = (int) (rnd() % 30); a->x14 = rw(); a->x18 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itWhispyAppleAttributes;
            (*a).x4 = (rnd() % 30) as i32; (*a).x8 = (rnd() % 30) as i32; (*a).x14 = rw(); (*a).x18 = rw();""")],
    tweak=per_item("""        IT[k].xDD4_itemVar.whispyapple.xDD8 = (int) (rnd() % 3);
        IT[k].xD44_lifeTimer = (rnd() & 1) ? (float) (int) (rnd() % 4) : rg();
        IT[k].pos.z = (rnd() & 1) ? (float) (int) (rnd() % 3) - 1 : rg();
        it_804D6D28->x34 = rnd() % 4;
        if (rnd() & 1) IT[k].msid = 7;""",
                   """        IT[k].xDD4_itemVar.whispyapple.xDD8 = (rnd() % 3) as i32;
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { (rnd() % 4) as i32 as f32 } else { rg() };
        IT[k].pos.z = if rnd() & 1 != 0 { (rnd() % 3) as i32 as f32 - 1.0 } else { rg() };
        (*m::ext::it_804D6D28).x34 = rnd() % 4;
        if rnd() & 1 != 0 { IT[k].msid = 7; }"""),
    ptrs=("", ""),
    hints={"threshold1": ("(float) (rnd() % 10) / 10.0f", "(rnd() % 10) as f32 / 10.0"),
           "threshold2": ("(float) (rnd() % 10) / 10.0f", "(rnd() % 10) as f32 / 10.0")},
)

# ----------------------------------------------------------------------------------------- ittincle
_tn = "IT[k].xDD4_itemVar.tincle"
_I = lambda lo, hi: (f"(int) (rnd() % {hi - lo}) - {-lo}" if lo < 0 else f"(int) (rnd() % {hi - lo}) + {lo}",
                     f"(rnd() % {hi - lo}) as i32 - {-lo}" if lo < 0 else f"(rnd() % {hi - lo}) as i32 + {lo}")
_tn_ints = {"x20": (-2, 5), "x24": (0, 6), "x28": (0, 6), "x2C": (-1, 6)}
_tn_flts = ["x30", "x34", "x38", "x3C", "x40", "x44", "x48", "x4C", "x50", "x54", "x58", "x5C"]
_sa = {"x4": (0, 8), "x8": (0, 8), "x28": (1, 6), "x2C": (0, 6), "x30": (1, 4), "x3C": (0, 8), "x40": (0, 6)}
_sa_f = ["x0", "xC", "x10", "x14", "x18", "x1C", "x20", "x24", "x34", "x38", "x44", "x48", "x4C", "x50"]
CFG["ittincle"] = dict(
    spec=[("\n".join(
        ["        itTincleAttributes* a = (itTincleAttributes*) SPEC[0];"]
        + [f"        a->{n} = {_I(*r)[0]};" for n, r in _sa.items()]
        + [f"        a->{n} = rw();" for n in _sa_f]
        + ["        a->x54 = (s8) (rnd() % 6); a->x55 = (s8) (rnd() % 9);"]),
           "\n".join(
        ["            let a = SPECB.0[0].as_mut_ptr() as *mut itTincleAttributes;"]
        + [f"            (*a).{n} = {_I(*r)[1]};" for n, r in _sa.items()]
        + [f"            (*a).{n} = rw();" for n in _sa_f]
        + ["            (*a).x54 = (rnd() % 6) as i8; (*a).x55 = (rnd() % 9) as i8;"]))],
    tweak=per_item(
        "\n".join(f"        {_tn}.{n} = {_I(*r)[0]};" for n, r in _tn_ints.items())
        + "\n" + "\n".join(f"        {_tn}.{n} = (rnd() & 1) ? rg() : rw();" for n in _tn_flts)
        + f"\n        {_tn}.x64 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;\n        if (rnd() & 1) IT[k].msid = (int) (rnd() % 14);",
        "\n".join(f"        IT[k].xDD4_itemVar.tincle.{n} = {_I(*r)[1]};" for n, r in _tn_ints.items())
        + "\n" + "\n".join(f"        IT[k].xDD4_itemVar.tincle.{n} = if rnd() & 1 != 0 {{ rg() }} else {{ rw() }};" for n in _tn_flts)
        + "\n        IT[k].xDD4_itemVar.tincle.x64 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };"
          "\n        if rnd() & 1 != 0 { IT[k].msid = (rnd() % 14) as i32; }"),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.x64)); %s.x64 = 0; }""" % (_tn, _tn),
          """    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.tincle.x64)); IT[k].xDD4_itemVar.tincle.x64 = null_mut(); }"""),
    hints={"arg1": ("(int) (rnd() % 2)", "(rnd() % 2) as i32")},
    over={"it_802ECCA4": ("{ int kv = (int) fv(); it_802ECCA4(&GO[0], &kv, &VA[0]); ret = (u32) kv; }",
                          "{ let mut kv: i32 = fv() as i32; m::it_802ECCA4(&raw mut GO[0], &mut kv, &raw mut VA[0]); ret = kv as u32; }")},
)
