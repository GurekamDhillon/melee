"""Per-TU oracle specs for gen_it.py / run_it.py / mk_it_driver.py (melee_items::tu). Keys as in
melee_world/gen/mtu_it2/specs.py (src, types, ret, effects, skip_data, patch, custom_stubs, ...); the
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



_WR3 = lambda a: ("if (%s) { %s->x = rw(); %s->y = rw(); %s->z = rw(); }" % (a, a, a, a),
                  "if !%s.is_null() { (*%s).x = rw(); (*%s).y = rw(); (*%s).z = rw(); }" % (a, a, a, a))
BB68 = _WR3("a1")

# ---- shard 729-736 (it/kinds: itlinkboomerang, itlinkbow, itlinkhookshot, itlipstick, itlipstickspore, itlizardon, itlucky, itlugia)
spec("itlinkbow", [], skip_data=IT_SKIP + ["result"], effects={"Item_80268B18": spawn_effect()},
     ret={"ftLk_SpecialN_GetIndex": ("(u32) (fv() % 7)", "(fv() % 7) as u32"), "ftKb_SpecialNLk800FB394": ("(u32) (fv() % 7)", "(fv() % 7) as u32")})

spec("itlipstick", ["itLipstickAttributes"], skip_data=IT_SKIP + ["result"])

_p, _s = efsync("ItemStateTable it_803F6680[] = {")
spec("itlipstickspore", ["itLipstickSporeAttributes"], patch=_p, custom_stubs=_s, skip_data=IT_SKIP + ["attr", "hit", "abs_vel"],
     effects={"Item_80268B18": spawn_effect(), "it_8026BB68": BB68})

_p, _s = efsync("ItemStateTable it_803F7BC0[] = {")
spec("itlizardon", ["itLizardonAttributes"], skip_stubs=["sinf", "cosf"], patch=_p, custom_stubs=_s, skip_data=IT_SKIP + ["attr", "bone_pos", "spawn"],
     effects={"Item_80268B18": spawn_effect(), "it_8026BB88": BB68})
