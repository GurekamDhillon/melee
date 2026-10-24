"""Specs and driver descriptions of the it/kinds TU ports (melee_items src/tu/<stem>.rs), shard 681-688:
itfoxblaster itfoxillusion itfoxlaser itfreeze itfreezer itfushigibana itgamewatchbreath itgamewatchchef.

Two consumers: specs.py (SPECS entries for gen_tu.py / run_tu.py: stub return values and effects, the state
types whose fill / hash are generated) and mk_itkinds.py (the C driver and the Rust test, written from the
one description CFG below so the two halves cannot drift).

Driver description (CFG[stem]):
  var      (union member, record type) of Item::xDD4_itemVar this kind uses (None: none)
  attr     record type of the kind's special attributes (ATTR buffer is filled with floats first)
  fix      statements applied after the generic world is built (see the helpers: I / CH / PT / RW / RAW)
  data     {extern data symbol: (C expr, Rust expr)} set during setup
  tables   [(name, kind, n)] data tables of the TU compared field by field (kind: state | f32 | i32)
  ops      [(function, argument tokens, return kind, [per-op fix statements])]
Argument tokens: G (&G), GN (&G or NULL), REF (&REFG), REFN, PAR (&PARG), IT (&ITEM), JO (&JO[0]), NULL,
VEC / VEC1 (a filled Vec3, hashed afterwards), f (rw()), i (fv() as int), u (fv() as u32), i:lo:hi, u:lo:hi.
Return kinds: v (void), i (integer), f (float bits), p (pointer id).
"""

# ------------------------------------------------------------------------------------ stub helpers


def vec_out(i):
    return (f"a{i}->x = rw(); a{i}->y = rw(); a{i}->z = rw();",
            f"(*a{i}).x = rw(); (*a{i}).y = rw(); (*a{i}).z = rw();")


COMMON_TYPES = ["HSD_GObj", "Item", "HSD_JObj", "SpawnItem", "ItemAttr", "ItemCommonData"]
SKIP_FIELDS = {"Item": ["xDD4_itemVar"]}
SKIP_DATA = ["result", "dir", "ip", "vel_x", "item", "jobj"]

COMMON_RET = {
    "Item_80268B18": ("((fv() & 3) == 0 ? 0 : &NEWG)",
                      "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::NEWG }"),
}
COMMON_EFFECTS = {
    "Item_80268B18": ("hash_SpawnItem(a0);", "hash_SpawnItem(&*a0);"),
    "it_8026BB68": vec_out(1),
}


def _merge(a, b):
    r = dict(a)
    r.update(b or {})
    return r


def mk(stem, var=None, attr=None, ret=None, effects=None, skip_data=None, extra_types=(), **kw):
    types = list(COMMON_TYPES)
    if var:
        types.append(var[1])
    if attr:
        types.append(attr)
    types += list(extra_types)
    return dict(crate="melee_items", src=f"src/melee/it/kinds/{stem}.c", types=types, skip_fields=SKIP_FIELDS,
                skip_data=SKIP_DATA + list(skip_data or []), ret=_merge(COMMON_RET, ret),
                effects=_merge(COMMON_EFFECTS, effects), **kw)


SPECS_IT = {}
SPECS_IT["itfoxblaster"] = mk(
    "itfoxblaster", var=("foxblaster", "itFoxBlaster_ItemVars"),
    ret={"ftFx_SpecialN_GetBlasterAction": ("(int) (fv() % 11)", "(fv() % 11) as i32"),
         "ftKb_SpecialNFx_800FDD4C": ("(fv() % 11)", "fv() % 11"),
         "ftFx_SpecialN_CheckRemoveBlaster": ("(int) (fv() % 3)", "(fv() % 3) as i32"),
         "ftFx_SpecialN_CheckBlasterAction": ("(int) (fv() % 3)", "(fv() % 3) as i32"),
         "ftKb_SpecialNFx_800FDD14": ("(int) (fv() % 3)", "(fv() % 3) as i32"),
         "ftKb_SpecialNFx_800FDDF4": ("(int) (fv() % 3)", "(fv() % 3) as i32"),
         "ftCo_800BF228": ("(int) (fv() % 3)", "(fv() % 3) as i32"),
         "atan2f": ("rw()", "rw()")},
    effects={"ftLib_80086644": vec_out(1), "ftFx_SpecialN_FtGetHoldJoint": vec_out(1),
             "ftFx_SpecialN_ItGetHoldJoint": vec_out(1), "ftKb_SpecialNFx_800FDC00": vec_out(1),
             "ftKb_SpecialNFx_800FDC70": vec_out(1)},
    custom_stubs={"efSync_Spawn": dict(
        c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* p; void* q; T(@ID@); T((u32) a0); T(PI(a1)); "
          "__builtin_va_start(ap, a1); p = __builtin_va_arg(ap, void*); q = __builtin_va_arg(ap, void*); __builtin_va_end(ap); "
          "T(PI(p)); T(PI(q)); T(B(((Vec3*) p)->x)); T(B(((Vec3*) p)->y)); T(B(((Vec3*) p)->z)); T(B(*(float*) q)); "
          "return (void*) (unsigned long long) (fv() & 0xFFFF); }",
        ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec, a3: *mut f32) -> *mut c_void;",
        hook="|a0, a1, a2, a3| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); t(pi(a3)); t((*a2).x.to_bits()); t((*a2).y.to_bits()); "
             "t((*a2).z.to_bits()); t((*a3).to_bits()); (fv() & 0xFFFF) as usize as *mut c_void }")})
SPECS_IT["itfoxillusion"] = mk(
    "itfoxillusion", var=("foxillusion", "itFoxIllusion_ItemVars"),
    ret={"HSD_JObjLoadJoint": ("&LJO", "&raw mut super::LJO"),
         "ftFx_SpecialS_GetCmdVar2": ("(int) (fv() % 4)", "(fv() % 4) as i32"),
         "ftFx_SpecialS_CheckGhostRemove": ("(int) ((fv() & 3) == 0)", "((fv() & 3) == 0) as i32"),
         "ftLib_800865CC": ("(int) (fv() % 3)", "(fv() % 3) as i32")},
    effects={"ftFx_SpecialS_CopyGhostPosIndexed": vec_out(2)})
SPECS_IT["itfoxlaser"] = mk(
    "itfoxlaser", var=("foxlaser", "itFoxLaser_ItemVars"), attr="FoxLaserAttr", skip_stubs=["cosf", "sinf"],
    ret={"atan2f": ("rw()", "rw()"),
         "it_8026E9A4": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
         "it_80273130": ("(int) (fv() & 1)", "(fv() & 1) as i32")},
    effects={"lbVector_Mirror": ("a0->x = rw(); a0->y = rw(); a0->z = rw();",
                                 "(*a0).x = rw(); (*a0).y = rw(); (*a0).z = rw();")})
SPECS_IT["itfreeze"] = mk(
    "itfreeze", var=("freeze", "itFreeze_ItemVars"), attr="itUnkAttributes",
    effects={"it_80276408": vec_out(2)},
    ret={"it_8026DAA8": ("(int) (fv() & 0xF)", "(fv() & 0xF) as i32")})
SPECS_IT["itfreezer"] = mk(
    "itfreezer", var=("freezer", "itFreezer_ItemVars"), attr="itFreezerAttributes",
    ret={"it_80272C6C": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
         "it_8027A09C": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
         "it_8027A118": ("(int) (fv() & 1)", "(fv() & 1) as i32")})
SPECS_IT["itfushigibana"] = mk(
    "itfushigibana", var=("fushigibana", "itFushigibana_ItemVars"), attr="itFushigibanaAttributes",
    ret={"it_80272C6C": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
         "it_8027A09C": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
         "it_8027A118": ("(int) (fv() & 1)", "(fv() & 1) as i32")})
SPECS_IT["itgamewatchbreath"] = mk(
    "itgamewatchbreath",
    ret={"ftGw_AttackAirN_ItemCheckSparkyRemove": ("(int) (fv() & 1)", "(fv() & 1) as i32")})
SPECS_IT["itgamewatchchef"] = mk(
    "itgamewatchchef", var=("gamewatchchef", "itGamewatchchef_ItemVars"), attr="itGamewatchchefAttributes",
    ret={"it_8026DAA8": ("(int) (fv() & 0xF)", "(fv() & 0xF) as i32")})


def add(specs):
    specs.update(SPECS_IT)


# ----------------------------------------------------------------------------------- fix helpers
# Every helper returns (C text, Rust text); both consume the same draws in the same order.

def _isf(v):
    return isinstance(v, float) or (isinstance(v, str) and ("." in v or "e" in v) and not v.startswith("0x"))


def I(path, lo, hi):
    n = hi - lo + 1
    return (f"{path} = ({lo}) + (int) (rnd() % {n}u);", f"{path} = (({lo}) + (rnd() % {n}u) as i64) as _;")


def CH(path, vals):
    n = len(vals)
    if any(_isf(v) for v in vals):
        cv = ", ".join(f"{float(v)!r}f" for v in vals)
        rv = ", ".join(f"{float(v)!r}" for v in vals)
        return (f"{{ static const float t[{n}] = {{ {cv} }}; {path} = t[rnd() % {n}u]; }}",
                f"{{ const T: [f32; {n}] = [{rv}]; {path} = T[(rnd() % {n}u) as usize]; }}")
    cv = ", ".join(str(v) for v in vals)
    return (f"{{ static const int t[{n}] = {{ {cv} }}; {path} = t[rnd() % {n}u]; }}",
            f"{{ const T: [i64; {n}] = [{cv}]; {path} = T[(rnd() % {n}u) as usize] as _; }}")


def RW(path):
    return (f"{path} = rw();", f"{path} = rw();")


def PT(path, syms):
    """pointer choice: 0 or &SYM per entry (repeat an entry to weight it)"""
    n = len(syms)
    c = " ".join(f"case {i}: {path} = {s if s == '0' else '&' + s}; break;" for i, s in enumerate(syms))
    r = " ".join(f"{i} => {path} = {'core::ptr::null_mut()' if s == '0' else '&raw mut ' + s + ' as *mut _'}," for i, s in enumerate(syms))
    return (f"switch (rnd() % {n}u) {{ {c} }}", f"match rnd() % {n}u {{ {r} _ => unreachable!() }}")


def RAW(c, r):
    return (c, r)


def COPYF(dst, src, p=2):
    """dst = src (1 in p draws) else rw()"""
    return (f"{{ u32 r = rnd(); if (r % {p}u == 0) {dst} = {src}; else {dst} = rw(); }}",
            f"{{ let r = rnd(); if r % {p}u == 0 {{ {dst} = {src}; }} else {{ {dst} = rw(); }} }}")


OWNER_ANY = PT("ITEM.owner", ["0", "OWN", "OWN", "REFG"])

def RG(path):
    """small values (rg(): 0, -0, NaN, +-1 or k/20)"""
    return (f"{path} = rg();", f"{path} = rg();")


def RWZ(path, p=3):
    """0.0 in 1 of p draws, else rw()"""
    return (f"{{ u32 r = rnd(); if (r % {p}u == 0) {path} = 0.0f; else {path} = rw(); }}",
            f"{{ let r = rnd(); if r % {p}u == 0 {{ {path} = 0.0; }} else {{ {path} = rw(); }} }}")


UD_NULL = PT("G.user_data", ["ITEM", "ITEM", "ITEM", "0"])

# ------------------------------------------------------------------------------------ descriptions
CFG = {}

# ---------------------------------------------------------------------------------- itfoxblaster
_V = "ITEM.xDD4_itemVar.foxblaster"
CFG["itfoxblaster"] = dict(
    var=("foxblaster", "itFoxBlaster_ItemVars"), attr=None, data={},
    fix=[CH("ITEM.kind", [74, 75, 138, 139, 74, 75, 138, 139, 0, 5]),
         CH("ITEM.msid", [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
         I(f"{_V}.xDD4", 0, 13), I(f"{_V}.xDD8", 0, 4), CH(f"{_V}.xDDC", [-2, -1, 0, 1, 2]),
         CH(f"{_V}.xDE0", [0, 1]), CH(f"{_V}.gfx_spawn_var", [0, 1]), CH(f"{_V}.set_sfx_var2", [0, 1, 2, 5]),
         PT(f"{_V}.owner", ["0", "OWN", "OWN"]), PT("ITEM.owner", ["0", "OWN", "OWN", "REFG"])],
    tables=[("it_803F6CA8", "state", 11), ("it_803F6D58", "f32", 14), ("it_803F6D90", "f32", 14),
            ("it_803F6DC8", "f32", 14), ("it_803F6E28", "f32", 5), ("it_803F6E3C", "i32", 11),
            ("it_803F6E68", "i32", 11)],
    ops=[
        ("itFoxBlaster_PlaySFX", "IT u u", "v", []),
        ("it_802ADDD0", "GN i:0:3", "v", [UD_NULL]),
        ("it_802ADEF0", "G", "v", []),
        ("itFoxBlaster_SetShotOffset", "IT VEC VEC1", "v", []),
        ("itFoxBlaster_SetShotAngle", "IT VEC VEC1", "v", []),
        ("it_802ADF10", "G", "v", []),
        ("it_802AE1D0", "G", "v", []),
        ("it_802AE200", "G", "v", [UD_NULL]),
        ("it_802AE538", "G", "v", []),
        ("it_802AE608", "G", "v", []),
        ("it_802AE63C", "G", "v", [UD_NULL]),
        ("itFoxBlaster_SetCommandVars", "IT i", "v", []),
        ("itFoxBlaster_ClearShot", "G i:0:5", "v", []),
        ("it_802AE7B8", "G", "v", []),
        ("it_802AE8A8", "f REFN VEC i i", "p", []),
        ("it_802AE994", "REFN i i", "p", []),
        ("it_802AEAB4", "GN", "v", []),
        ("itFoxBlaster_Logic96_PickedUp", "G", "v", []),
        ("clear_blaster_references", "G", "v", [UD_NULL]),
        ("clear_blaster", "G", "v", []),
        ("itFoxblaster_UnkMotion8_Anim", "G", "i", []),
        ("itFoxblaster_UnkMotion8_Phys", "G", "v", []),
        ("itFoxblaster_UnkMotion8_Coll", "G", "i", []),
        ("itFoxblaster_UnkMotion9_Anim", "G", "i", []),
        ("itFoxblaster_UnkMotion9_Phys", "G", "v", []),
        ("itFoxblaster_UnkMotion9_Coll", "G", "i", []),
        ("itFoxblaster_UnkMotion10_Anim", "GN", "i", [UD_NULL]),
        ("itFoxblaster_UnkMotion10_Phys", "G", "v", []),
        ("itFoxblaster_UnkMotion10_Coll", "G", "i", []),
        ("itFoxBlaster_Logic96_EvtUnk", "G REF", "v", []),
    ])

# --------------------------------------------------------------------------------- itfoxillusion
_V = "ITEM.xDD4_itemVar.foxillusion"
CFG["itfoxillusion"] = dict(
    var=("foxillusion", "itFoxIllusion_ItemVars"), attr=None, data={},
    fix=[PT(f"{_V}.xDD4", ["0", "JNT", "JNT"]), PT(f"{_V}.xDDC", ["0", "GJO", "GJO"]),
         PT("ITEM.owner", ["0", "OWN", "OWN"]), PT("MD.x0_joint", ["0", "JNT"]),
         CH("ITEM.xD44_lifeTimer", [0.0, 1.0, 2.0, -1.0, 5.0])],
    tables=[("it_803F6818", "state", 3)],
    ops=[
        ("it_8029CD18", "G i", "v", []),
        ("it_8029CD78", "G", "p", [PT(f"{_V}.xDDC", ["0"])]),
        ("it_8029CEB4", "PAR VEC i f", "p", []),
        ("itFoxIllusion_Logic14_DmgDealt", "G", "i", []),
        ("itFoxIllusion_Logic14_Destroyed", "G", "v", []),
        ("it_8029CFF0", "G", "v", []),
        ("itFoxillusion_UnkMotion0_Anim", "G", "i", []),
        ("itFoxillusion_Phys", "G VEC", "v", []),
        ("itFoxillusion_UnkMotion0_Phys", "G", "v", []),
        ("itFoxillusion_UnkMotion0_Coll", "G", "i", []),
        ("itFoxillusion_UnkMotion1_Anim", "G", "i", []),
        ("itFoxillusion_UnkMotion1_Phys", "G", "v", []),
        ("itFoxillusion_UnkMotion1_Coll", "G", "i", []),
        ("it_8029D798", "G", "v", []),
        ("itFoxillusion_UnkMotion2_Anim", "G", "i", []),
        ("itFoxillusion_UnkMotion2_Phys", "G", "v", []),
        ("itFoxillusion_UnkMotion2_Coll", "G", "i", []),
        ("it_8029D948", "G REF", "v", []),
    ])

# ----------------------------------------------------------------------------------- itfoxlaser
CFG["itfoxlaser"] = dict(
    var=("foxlaser", "itFoxLaser_ItemVars"), attr="FoxLaserAttr", data={},
    fix=[], tables=[("it_803F67D0", "state", 2)],
    ops=[
        ("it_8029C4D4", "G NULL", "i", []),
        ("it_8029C504", "PAR VEC i:0:3 i f f", "v", []),
        ("it_8029C6A4", "f f PAR VEC i", "v", []),
        ("it_8029C6CC", "f f PAR VEC i", "v", []),
        ("itFoxlaser_UnkMotion1_Anim", "G", "i", []),
        ("itFoxlaser_UnkMotion1_Phys", "G", "v", []),
        ("itFoxlaser_UnkMotion1_Coll", "G", "i", []),
        ("itFoxLaser_Logic94_Clanked", "G", "i", []),
        ("itFoxLaser_Logic94_Reflected", "G", "i", [COPYF("ITEM.xC68", "ITEM.facing_dir")]),
        ("itFoxLaser_Logic94_Absorbed", "G", "i", []),
        ("itFoxLaser_Logic94_ShieldBounced", "G", "i", []),
        ("itFoxLaser_Logic94_HitShield", "G", "i", []),
        ("itFoxLaser_Logic94_EvtUnk", "G REF", "v", []),
    ])

# ------------------------------------------------------------------------------------ itfreeze
_V = "ITEM.xDD4_itemVar.freeze"
CFG["itfreeze"] = dict(
    var=("freeze", "itFreeze_ItemVars"), attr="itUnkAttributes",
    data={"it_804D6D28": ("&ICD", "&raw mut ICD")},
    fix=[CH("ITEM.ground_or_air", [0, 1, 1]), RG(f"{_V}.x0"), RG(f"{_V}.x10"), RG(f"{_V}.x14"),
         CH(f"{_V}.x18", [-1, 0, 1, 2, 90]), PT(f"{_V}.unk_1C", ["0", "REFG"]),
         CH("ITEM.facing_dir", [-1.0, 1.0, 0.5])],
    tables=[("it_803F5D28", "state", 6)],
    ops=[
        ("it_8028EB88", "REFN VEC f f", "p", []),
        ("it_8028EC98", "G f", "v", []),
        ("it_8028ECE0", "G", "v", []),
        ("it_8028ECF0", "G VEC", "p", []),
        ("itFreeze_Logic17_Destroyed", "G", "v", []),
        ("it_3F14_Logic17_Spawned", "G", "v", []),
        ("it_8028EDBC", "G", "v", []),
        ("it_8028EF34", "G", "v", []),
        ("itFreeze_UnkMotion0_Anim", "G", "i", []),
        ("itFreeze_UnkMotion0_Phys", "G", "v", []),
        ("itFreeze_UnkMotion0_Coll", "G", "i", []),
        ("it_8028F1D8", "G", "v", []),
        ("itFreeze_UnkMotion3_Anim", "G", "i", []),
        ("itFreeze_UnkMotion1_Phys", "G", "v", []),
        ("itFreeze_UnkMotion1_Coll", "G", "i", []),
        ("itFreeze_Logic17_PickedUp", "G", "v", []),
        ("itFreeze_UnkMotion2_Anim", "G", "i", []),
        ("itFreeze_Logic17_Dropped", "G", "v", []),
        ("itFreeze_Logic17_Thrown", "G", "v", []),
        ("itFreeze_UnkMotion3_Phys", "G", "v", []),
        ("itFreeze_UnkMotion3_Coll", "G", "i", []),
        ("itFreeze_Logic17_DmgDealt", "G", "i", []),
        ("itFreeze_Logic17_Clanked", "G", "i", []),
        ("itFreeze_Logic17_HitShield", "G", "i", []),
        ("itFreeze_Logic17_Absorbed", "G", "i", []),
        ("itFreeze_Logic17_Reflected", "G", "i", []),
        ("itFreeze_Logic17_ShieldBounced", "G", "i", []),
        ("itFreeze_Logic17_DmgReceived", "G", "i", []),
        ("it_8028F434", "G f REFN", "v", []),
        ("itFreeze_UnkMotion4_Anim", "G", "i", []),
        ("itFreeze_UnkMotion4_Phys", "G", "v", []),
        ("itFreeze_ClearLinkedItem", "G", "v", []),
        ("itFreeze_ResetToMotion0", "G", "v", []),
        ("itFreeze_UnkMotion4_Coll", "G", "i", []),
        ("it_8028F7C8", "G", "v", []),
        ("itFreeze_UnkMotion5_Anim", "G", "i", []),
        ("itFreeze_UnkMotion5_Phys", "G", "v", []),
        ("itFreeze_UnkMotion5_Coll", "G", "i", []),
        ("itFreeze_Logic17_EvtUnk", "G REF", "v", []),
        ("it_8028F8E4", "G", "v", []),
        ("it_8028F968", "G", "v", []),
        ("it_8028F9B8", "G", "v", []),
    ])

# ----------------------------------------------------------------------------------- itfreezer
_V = "ITEM.xDD4_itemVar.freezer"
CFG["itfreezer"] = dict(
    var=("freezer", "itFreezer_ItemVars"), attr="itFreezerAttributes", data={},
    fix=[CH(f"{_V}.x60", [-2, -1, 0, 0, 1, 5]), CH("ITEM.xDB0_itcmd_var1", [0, 0, 1, 7]),
         CH("ITEM.xDAC_itcmd_var0", [0, 1, 3])],
    tables=[("it_803F7C70", "state", 3)],
    ops=[
        ("itFreezer_802CCF9C", "G", "v", []),
        ("itFreezer_802CCFFC", "G", "v", []),
        ("itFreezer_802CD000", "G REF", "v", []),
        ("itFreezer_UnkMotion1_Anim", "G", "i", []),
        ("itFreezer_UnkMotion1_Phys", "G", "v", []),
        ("itFreezer_UnkMotion1_Coll", "G", "i", []),
        ("itFreezer_802CD090", "G", "v", []),
        ("itFreezer_802CD12C", "G", "v", []),
        ("itFreezer_UnkMotion2_Anim_Inline", "G", "v", []),
        ("itFreezer_UnkMotion2_Anim", "G", "i", []),
        ("itFreezer_UnkMotion2_Phys", "G", "v", []),
        ("itFreezer_UnkMotion2_Coll", "G", "i", []),
        ("itFreezer_802CD290", "G", "v", []),
        ("itFreezer_802CD2EC", "G", "v", []),
        ("itFreezer_UnkMotion0_Anim", "G", "i", []),
        ("itFreezer_UnkMotion0_Phys_inline2", "G", "v", []),
        ("itFreezer_UnkMotion0_Phys_inline1", "G", "v", []),
        ("itFreezer_UnkMotion0_Phys", "G", "v", []),
        ("itFreezer_UnkMotion0_Coll", "G", "i", []),
    ])

# -------------------------------------------------------------------------------- itfushigibana
_V = "ITEM.xDD4_itemVar.fushigibana"
CFG["itfushigibana"] = dict(
    var=("fushigibana", "itFushigibana_ItemVars"), attr="itFushigibanaAttributes", data={},
    fix=[CH(f"{_V}.x64", [0.0, 0.0, 1.0, 2.5, -1.0]), CH("ITEM.xD44_lifeTimer", [0.0, 1.0, 1.5, 2.5, 10.0, -1.0])],
    tables=[("it_803F82F0", "state", 3)],
    ops=[
        ("itFushigibana_UnkMotion1_Anim_inline1", "G", "v", []),
        ("itFushigibana_UnkMotion1_Anim_inline2", "G", "i", []),
        ("itFushigibana_Logic29_Spawned", "G", "v", []),
        ("it_802D705C", "G", "v", []),
        ("it_802D70A4", "G", "v", []),
        ("itFushigibana_UnkMotion0_Anim", "G", "i", []),
        ("itFushigibana_UnkMotion0_Phys", "G", "v", []),
        ("itFushigibana_UnkMotion0_Coll", "G", "i", []),
        ("it_802D718C", "G", "v", []),
        ("itFushigibana_UnkMotion1_Anim", "G", "i", []),
        ("itFushigibana_UnkMotion1_Phys", "G", "v", []),
        ("itFushigibana_UnkMotion1_Coll", "G", "i", []),
        ("it_802D7328", "G", "v", []),
        ("itFushigibana_UnkMotion2_Anim", "G", "i", []),
        ("itFushigibana_UnkMotion2_Phys", "G", "v", []),
        ("itFushigibana_UnkMotion2_Coll", "G", "i", []),
    ])

# ------------------------------------------------------------------------------ itgamewatchbreath
CFG["itgamewatchbreath"] = dict(
    var=None, attr=None, data={},
    fix=[PT("ITEM.owner", ["0", "OWN", "OWN"]), CH("ITEM.xDAC_itcmd_var0", [0, 1, 5])],
    tables=[("it_803F7938", "state", 2)],
    ops=[
        ("it_802C720C", "PAR VEC i f", "p", []),
        ("itGameWatchBreath_Logic76_Destroyed", "G", "v", []),
        ("it_802C7340", "G", "v", []),
        ("it_802C738C", "G", "v", []),
        ("it_802C73AC", "G", "v", []),
        ("itGameWatchBreath_Logic76_PickedUp", "G", "v", []),
        ("it_802C7424", "G", "v", []),
        ("itGamewatchbreath_UnkMotion1_Anim", "G", "i", []),
        ("itGameWatchBreath_Logic76_EvtUnk", "G REF", "v", []),
    ])

# ---------------------------------------------------------------------------------- itgamewatchchef
_V = "ITEM.xDD4_itemVar.gamewatchchef"
CFG["itgamewatchchef"] = dict(
    var=("gamewatchchef", "itGamewatchchef_ItemVars"), attr="itGamewatchchefAttributes", data={},
    fix=[I(f"{_V}.x4", 0, 4), RWZ("ITEM.x40_vel.x", 3),
         CH("ITEM.xD44_lifeTimer", [0.0, 1.0, 1.5, 2.5, 3.5, 5.5, 10.0, -1.0])],
    tables=[("it_803F79E0", "state", 2)],
    ops=[
        ("it_802C837C", "PAR VEC i u:0:4 f", "p", []),
        ("itGameWatchChef_Logic112_DmgDealt", "G", "i", []),
        ("it_802C84A0", "G i:0:4", "v", []),
        ("itGamewatchchef_UnkMotion0_Anim", "G", "i", []),
        ("itGamewatchchef_UnkMotion0_Phys", "G", "v", []),
        ("itGamewatchchef_UnkMotion0_Coll", "G", "i", []),
        ("it_802C875C", "G", "v", []),
        ("itGamewatchchef_UnkMotion1_Anim", "G", "i", []),
        ("itGamewatchchef_UnkMotion1_Phys", "G", "v", []),
        ("itGamewatchchef_UnkMotion1_Coll", "G", "i", []),
        ("it_2725_Logic112_Clanked", "G", "i", []),
        ("it_2725_Logic112_HitShield", "G", "i", []),
        ("it_2725_Logic112_Absorbed", "G", "i", []),
        ("itGameWatchChef_Logic112_ShieldBounced", "G", "i", []),
        ("itGameWatchChef_Logic112_Reflected", "G", "i", []),
        ("itGameWatchChef_Logic112_EvtUnk", "G REF", "v", []),
    ])
