"""Per-TU oracle specs for gen_it.py / run_it.py / mk_it_driver.py (melee_items::tu). Keys as in
melee_world/gen/mtu_it/specs.py (src, types, ret, effects, skip_data, patch, custom_stubs, ...); the
driver side (fixtures, ops) is in cfg_it.py."""
SPECS = {}
# the walker's state types every item TU shares
IT_TYPES = ["HSD_GObj", "Item", "ItemAttr", "ItemCommonData"]
# locals of header inlines that gen_tu mistakes for extern data
IT_SKIP = ["item", "ip", "jobj", "attrs", "attr"]

PTR = lambda v: f"if fv() & 3 == 0 {{ core::ptr::null_mut() }} else {{ &raw mut super::{v} }}"

COMMON_RET = {
    # HSD_Randi(n): 0 .. n-1
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
    # the item a spawn produced: the second fixture item, or none
    "Item_80268B18": ("((fv() & 3) == 0 ? 0 : &G2)", PTR("G2")),
    # a joint of the fixture
    "it_80272CC0": ("&JO[fv() % 3]", "&raw mut super::JO[(fv() % 3) as usize]"),
}


def flags(p):
    c = " | ".join(f"((u32) {p}.b{i} << {i})" for i in range(8))
    r = " | ".join(f"(({p}.anon_0.b{i}() as u32) << {i})" for i in range(8))
    return c, r


def spawn_effect(arg="a0"):
    """T() of every SpawnItem field the callee sees (the spawn record is a local of the caller)."""
    c, r = [], []
    for f in ("kind", "hold_kind", "x10", "x3C_damage", "x3E", "x40", "x48_ground_or_air"):
        c.append(f"T((u32) {arg}->{f});")
        r.append(f"t((*{arg}).{f} as u32);")
    for f in ("pos", "prev_pos", "vel"):
        for a in "xyz":
            c.append(f"T(B({arg}->{f}.{a}));")
            r.append(f"t((*{arg}).{f}.{a}.to_bits());")
    c.append(f"T(B({arg}->facing_dir));")
    r.append(f"t((*{arg}).facing_dir.to_bits());")
    for f in ("x0_parent_gobj", "x4_parent_gobj2"):
        c.append(f"T(PI({arg}->{f}));")
        r.append(f"t(pi((*{arg}).{f}));")
    for n in (44, 45, 46, 47):
        fc, fr = flags(f"{arg}->x{n}_flag")
        c.append(f"T({fc});")
        r.append(f"t({flags(f'(*{arg}).x{n}_flag')[1]});")
    return "{ " + " ".join(c) + " }", "{ " + " ".join(r) + " }"


def efsync(anchor):
    """efSync_Spawn is variadic: the compiled copy of the TU counts the arguments after the gobj."""
    patch = [(anchor,
              "#define EFS_N_(_1,_2,_3,_4,_5,N,...) N\n"
              "#define efSync_Spawn(g, o, ...) efSync_Spawn_N(EFS_N_(__VA_ARGS__,5,4,3,2,1,0), g, o, __VA_ARGS__)\n"
              "void* efSync_Spawn_N(int n, s32 gfx, HSD_GObj* g, ...);\n" + anchor)]
    stub = {"efSync_Spawn_N": dict(
        c="void* efSync_Spawn_N(int a0, s32 a1, HSD_GObj* a2, ...) { __builtin_va_list ap; int i; T(@ID@); T((u32) a0); T((u32) a1); T(PI(a2)); "
          "__builtin_va_start(ap, a2); for (i = 0; i < a0; i++) T(PI(__builtin_va_arg(ap, void*))); __builtin_va_end(ap); return 0; }",
        ext="fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;",
        hook="|a0, a1, a2| { t(@ID@); t(a2.len() as u32); t(a0 as u32); t(pi(a1)); for v in a2 { t(pi(*v as *const c_void)); } core::ptr::null_mut() }")}
    return patch, stub


def spec(stem, extra_types=(), **kw):
    p = dict(crate="melee_items", src=f"src/melee/it/kinds/{stem}.c", types=IT_TYPES + list(extra_types), skip_data=IT_SKIP,
             common_ret=COMMON_RET)
    p.update(kw)
    SPECS[stem] = p
    return p


spec("itfire", ["itFireAttributes"])

_p, _s = efsync("ItemStateTable it_803F5988[] = {")
spec("itegg", ["itEgg_ItemVars"], patch=_p, custom_stubs=_s, effects={"Item_80268B18": spawn_effect()})

_p, _s = efsync("ItemStateTable it_803F7DC8[] = {")
spec("itentei", ["itPokemonAttributes"], patch=_p, custom_stubs=_s)

_p, _s = efsync("ItemStateTable it_803F6728[] = {")
spec("itevyoshiegg", ["itEvYoshiEgg_DatAttrs"], patch=_p, custom_stubs=_s, effects={"Item_80268B18": spawn_effect()})

_p, _s = efsync("ItemStateTable it_803F6090[] = {")
spec("itfflower", ["FFlowerAttr"], patch=_p, custom_stubs=_s, effects={"Item_80268B18": spawn_effect()},
     null_ret_ok=["it_802996D0"])

# it_8026BB68(gobj, &pos) writes the parent's position
BB68 = ("if (a1) { a1->x = rw(); a1->y = rw(); a1->z = rw(); }",
        "if !a1.is_null() { (*a1).x = rw(); (*a1).y = rw(); (*a1).z = rw(); }")
spec("itfflowerflame", [], effects={"Item_80268B18": spawn_effect(), "it_8026BB68": BB68})

spec("itfoods", [], effects={"Item_80268B18": spawn_effect(), "it_8026BB68": BB68})

_WR3 = lambda a: ("if (%s) { %s->x = rw(); %s->y = rw(); %s->z = rw(); }" % (a, a, a, a),
                  "if !%s.is_null() { (*%s).x = rw(); (*%s).y = rw(); (*%s).z = rw(); }" % (a, a, a, a))
spec("itflipper", ["itFlipper_DatAttrs"], skip_stubs=["__frsqrte"], skip_data=IT_SKIP + ["guess", "y"],
     effects={"Item_80268B18": spawn_effect(), "lb_8000B1CC": _WR3("a2"), "ftLib_800866DC": _WR3("a1"),
              "ftLib_80086BEC": _WR3("a1"), "lbVector_NormalizeXY": _WR3("a0")})

# ---- shard 713-720 (it/kinds: itkirby*, itkireihana, itklap, itkoopaflame, itkusudama, itkyasarin)
# efAsync_Spawn is variadic in C (extra args after the jobj); the TUs here call it with exactly five.
EFASYNC5 = {"efAsync_Spawn": dict(
    c="void efAsync_Spawn(HSD_GObj* a0, void* a1, u32 a2, u32 a3, HSD_JObj* a4, ...) { T(@ID@); T(PI(a0)); T(PI(a1)); T(a2); T(a3); T(PI(a4)); }",
    ext="fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj);",
    hook="|a0, a1, a2, a3, a4| { t(@ID@); t(pi(a0)); t(pi(a1)); t(a2); t(a3); t(pi(a4)); }")}

spec("itkirbyhammer", [], custom_stubs=EFASYNC5, effects={"Item_80268B18": spawn_effect(), "it_8026BB68": BB68})

spec("itkirbyyoshispecialn", [], null_ret_ok=["ftLib_80086630"], effects={"lb_8000B1CC": _WR3("a2")})

_WR2 = lambda a: ("if (%s) { %s->x = rw(); %s->y = rw(); }" % (a, a, a),
                  "if !%s.is_null() { (*%s).x = rw(); (*%s).y = rw(); }" % (a, a, a))
spec("itkirby_2F23", [], skip_stubs=["__frsqrte"], skip_data=IT_SKIP + ["guess", "y", "result"], effects={"ftKb_SpecialN_800F5B5C": _WR3("a1"), "ftKb_SpecialN_800F5B20": _WR2("a1")})

spec("itkireihana", ["itkireihanaAttributes"])

spec("itklap", [], skip_stubs=["sinf", "cosf"], skip_data=IT_SKIP + ["pos"], ret={"it_8027B5B0": COMMON_RET["Item_80268B18"]},
     effects={"lb_8000B1CC": _WR3("a2"),
              "Ground_801C4DA0": ("if (a0) { a0->x = rw(); a0->y = rw(); a0->z = rw(); } if (a1) *a1 = rw();",
                                  "if !a0.is_null() { (*a0).x = rw(); (*a0).y = rw(); (*a0).z = rw(); } if !a1.is_null() { *a1 = rw(); }"),
              "lbColl_80008D30": ("((struct SmallerHitCapsule*) a0)->damage = rw();", "(*(a0 as *mut SmallerHitCapsule)).damage = rw();")})
