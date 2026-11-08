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

spec("itlucky", ["itLuckyAttributes", "itLuckyEggAttributes"], skip_data=IT_SKIP + ["attr", "spawn", "vel", "pos"],
     effects={"Item_80268B18": spawn_effect()},
     ret={"it_80288C88": ("((fv() & 3) == 0 ? 0 : &G2)", PTR("G2"))})

_WR3_2 = ("if (a2) { a2->x = rw(); a2->y = rw(); a2->z = rw(); }",
          "if !a2.is_null() { (*a2).x = rw(); (*a2).y = rw(); (*a2).z = rw(); }")
_p, _s = efsync("ItemStateTable it_803F7EE8[] = {")
spec("itlugia", ["itLugiaAttributes"], skip_stubs=["sinf", "cosf", "__frsqrte"], skip_data=IT_SKIP + ["attr", "spawn", "pos", "target", "diff"],
     effects={"Item_80268B18": spawn_effect(), "it_8026BB88": BB68, "lbVector_Diff": _WR3_2})

spec("itlinkboomerang", ["itLinkBoomerangAttributes"], skip_stubs=["sinf", "cosf", "__frsqrte", "sqrtf"],
     skip_data=IT_SKIP + ["attr", "stack", "quat", "rot", "pos", "sp14", "spawn", "child", "hobj", "mtx", "coll", "owner", "idx", "joint", "anim_bundle", "item_state_desc", "gobj2"],
     effects={"Item_80268B18": spawn_effect(), "it_8026BB68": BB68, "ftLk_SpecialHi_GetPosWithAdjustedY": BB68,
              "ftLib_GetCameraBonePos": BB68, "HSD_MtxGetRotation": BB68},
     ret={"HSD_JObjLoadJoint": ("&JO[fv() % 4]", "&raw mut super::JO[(fv() % 4) as usize]"),
          "ftLib_GetPartJObj": ("&JO[fv() % 4]", "&raw mut super::JO[(fv() % 4) as usize]"),
          "lbVector_CosAngle": ("((float) ((int) (rnd() % 2001) - 1000) / 1000.0f)", "((rnd() % 2001) as i32 - 1000) as f32 / 1000.0"),
          "ftLib_GetKind": ("((fv() & 1) ? 6 : 1)", "(if fv() & 1 != 0 { 6 } else { 1 })")})

_p, _s = efsync("ItemStateTable it_803F6998[] = {")
_MIDENT = ("if (a0) { int r_, c_; for (r_ = 0; r_ < 3; r_++) for (c_ = 0; c_ < 4; c_++) a0[r_][c_] = (r_ == c_) ? 1.0f : 0.0f; }",
           "if !a0.is_null() { let m_ = a0 as *mut f32; for r_ in 0..3usize { for c_ in 0..4usize { *m_.add(r_ * 4 + c_) = if r_ == c_ { 1.0 } else { 0.0 }; } } }")
_MCONCAT = ("if (a2) { int i_; for (i_ = 0; i_ < 12; i_++) ((float*) a2)[i_] = rw(); }",
            "if !a2.is_null() { for i_ in 0..12usize { *(a2 as *mut f32).add(i_) = rw(); } }")
_MCOPY = ("if (a0 && a1) { int i_; for (i_ = 0; i_ < 12; i_++) ((float*) a1)[i_] = ((float*) a0)[i_]; }",
          "if !a0.is_null() && !a1.is_null() { for i_ in 0..12usize { *(a1 as *mut f32).add(i_) = *(a0 as *const f32).add(i_); } }")
_NORM = ("if (a0) { a0->x = rw(); { u32 q_ = rnd() % 4; a0->y = q_ == 0 ? 1.0f : q_ == 1 ? -1.0f : rw(); } a0->z = rw(); }",
         "if !a0.is_null() { (*a0).x = rw(); { let q_ = rnd() % 4; (*a0).y = if q_ == 0 { 1.0 } else if q_ == 1 { -1.0 } else { rw() }; } (*a0).z = rw(); }")
_MPCOLL = ("if (a0) { static const int E_[5] = { 0, 0x8000, 0x3F, 0xFC0, 0x2000 }; a0->env_flags = E_[rnd() % 5]; a0->cur_pos.x = rw(); a0->cur_pos.y = rw(); }",
           "if !a0.is_null() { const E_: [i32; 5] = [0, 0x8000, 0x3F, 0xFC0, 0x2000]; (*a0).env_flags = E_[(rnd() % 5) as usize]; (*a0).cur_pos.x = rw(); (*a0).cur_pos.y = rw(); }")
_p, _s = efsync("ItemStateTable it_803F6998[] = {")
spec("itlinkhookshot", ["ItemLink"], patch=_p, custom_stubs=_s, skip_stubs=["sinf", "cosf", "__frsqrt", "sqrtf", "__frsqrte"],
     skip_data=IT_SKIP + ["result", "guess", "y", "attr", "m", "pos", "vec", "link", "coll", "fp", "lk_attr", "head", "anchor", "cur", "dir", "speed", "saved_pos", "vec2", "item_link", "mp_island"],
     effects={"Item_80268B18": spawn_effect(), "PSMTXIdentity": _MIDENT, "PSMTXConcat": _MCONCAT, "PSMTXCopy": _MCOPY,
              "PSVECCrossProduct": _WR3_2, "lbVector_Normalize": _NORM, "mpColl_800471F8": _MPCOLL, "mpColl_8004B108": _MPCOLL,
              "mpGetSpeed": _WR3_2},
     ret={"GObj_Create": ("(((fv() & 15) == 0) ? 0 : &LG[GCNT++ % 16])", "(if fv() & 15 == 0 { core::ptr::null_mut() } else { let r_ = &raw mut super::LG[super::GCNT % 16]; super::GCNT += 1; r_ })"),
          "HSD_ObjAlloc": ("&LK[ACNT++ % 16]", "{ let r_ = &raw mut super::LK[super::ACNT % 16]; super::ACNT += 1; r_ as *mut c_void }"),
          "HSD_JObjLoadJoint": ("&JO[fv() % 4]", "&raw mut super::JO[(fv() % 4) as usize]"),
          "ft_80082E3C": ("((fv() & 1) ? &G2 : 0)", "(if fv() & 1 != 0 { &raw mut super::G2 } else { core::ptr::null_mut() })"),
          "mpIsland_8005AB54": ("((fv() & 1) ? &MPU : 0)", "(if fv() & 1 != 0 { &raw mut super::MPU } else { core::ptr::null_mut() })"),
          "ftParts_GetBoneIndex": ("(fv() % 150)", "((fv() % 150) as _)")})
