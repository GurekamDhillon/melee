"""State-oracle spec of src/melee/gr/grcorneria.c (rust/melee_world/src/tu/grcorneria.rs)."""


def both(*parts):
    return ("".join(p[0] + " " for p in parts), "".join(p[1] + " " for p in parts))


def raw(c, r):
    return (c, r)


NNs = raw("NN = 1;", "NN = 1;")
NN_RND = raw("NN = (rnd() % 4) != 0;", "NN = (rnd() % 4 != 0) as i32;")


def nid(*ids):
    m = 0
    for i in ids:
        m |= 1 << i
    return raw(f"NIDM = {m}u;", f"NIDM = {m}u32;")


# A bitfield byte (and the byte of a union that aliases one) is written palindromic: the C packs bitfields LSB-first, the Rust
# records MSB-first like the PPC, so only a palindromic byte reads the same flags (b0..b2 of xC4 / xC6 / offset_y) in both.
def PALC(e):
    return f"({{ unsigned n_ = {e}; (n_ | ((n_ & 1u) << 7) | ((n_ & 2u) << 5) | ((n_ & 4u) << 3) | ((n_ & 8u) << 1)); }})"


def PALR(e):
    return f"({{ let n_ = {e}; n_ | ((n_ & 1) << 7) | ((n_ & 2) << 5) | ((n_ & 4) << 3) | ((n_ & 8) << 1) }})"


# the world every function starts from: the module statics are filled by kind, so the indices/pointers they hold
# (arwing slots, group / type ids, the yakumono ints that are divisors) are put into range here
CG = raw(
    """{ int j_;
  yakumono_param->x74 = 1 + (int) (rnd() % 5); yakumono_param->x78 = (int) (rnd() % 4);
  yakumono_param->x7C = 1 + (int) (rnd() % 5); yakumono_param->x80 = (int) (rnd() % 4);
  for (j_ = 0; j_ < 3; j_++) {
      arwing_types[j_] = (int) (rnd() % 14); arwing_groups[j_] = (int) (rnd() % 5);
      arwing_gobjs[j_] = (rnd() % 3) ? &OGO[rnd() % NGO] : (HSD_GObj*) 0;
  }
  grCn_804D69A4 = rnd() % 3; grCn_804D69A8 = (int) (rnd() % 6) - 2; grCn_804D69AC = (rnd() % 3) == 0; grCn_804D69B0 = (int) (rnd() % 2);
  for (j_ = 0; j_ < NGO; j_++) OGP[j_].u.arwing.xC8 = rnd() % 3;
  NIDM = 0; }""",
    """{
  (*t::yakumono_param).x74 = 1 + (rnd() % 5) as i32; (*t::yakumono_param).x78 = (rnd() % 4) as i32;
  (*t::yakumono_param).x7C = 1 + (rnd() % 5) as i32; (*t::yakumono_param).x80 = (rnd() % 4) as i32;
  for j_ in 0..3usize {
      t::arwing_types[j_] = (rnd() % 14) as i32; t::arwing_groups[j_] = (rnd() % 5) as i32;
      t::arwing_gobjs[j_] = if rnd() % 3 != 0 { addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) } else { null_mut() };
  }
  t::grCn_804D69A4 = rnd() % 3; t::grCn_804D69A8 = (rnd() % 6) as i32 - 2; t::grCn_804D69AC = (rnd() % 3 == 0) as i32; t::grCn_804D69B0 = (rnd() % 2) as i32;
  for j_ in 0..NGO { OGP[j_].u.arwing.xC8 = rnd() % 3; }
  NIDM = 0; }""",
)

# offset_y is an f32 that shares its byte with flags.b0: a normal float whose low byte is palindromic
_OFFY_C = ("  for (j_ = 0; j_ < NGO; j_++) { float f_ = rf(); u32 w_; __builtin_memcpy(&w_, &f_, 4); w_ = (w_ & ~0xFFu) | "
           + PALC("rnd() % 16") + "; __builtin_memcpy(&OGP[j_].u.corneria.offset_y, &w_, 4); }\n")
_OFFY_R = ("  for j_ in 0..NGO { let f_ = rf(); let w_ = (f_.to_bits() & !0xFFu32) | "
           + PALR("rnd() % 16") + "; *(addr_of_mut!(OGP[j_].u.corneria.offset_y) as *mut u32) = w_; }\n")
CG = raw(CG[0].replace("  NIDM = 0; }", _OFFY_C + "  NIDM = 0; }"), CG[1].replace("  NIDM = 0; }", _OFFY_R + "  NIDM = 0; }"))

# Ground.u words the ship / backgrounds / star fox articles read
SHIP = raw(
    """{ OGP[0].u.corneria.xC4.value = (rnd() % 3) == 0 ? 0 : (u8) @@C1@@; OGP[0].u.corneria.xC5 = (u8) (rnd() % 3);
  OGP[0].u.corneria.xC6.value = (rnd() % 2) ? 0 : (u8) @@C2@@;
  OGP[0].u.corneria.x128 = &OGO[rnd() % NGO]; OGP[0].u.corneria.x12C = (rnd() % 2) ? &OJO[0] : &OJO[rnd() % NJO];
  OGP[0].u.corneria.x108 = rnd() % 6; OGP[0].u.corneria.x10C = (int) (rnd() % 4) - 1; OGP[0].u.corneria.x110 = (int) (rnd() % 12) - 1;
  OGP[0].u.corneria.x118 = (s8) ((int) (rnd() % 4) - 1); OGP[0].u.corneria.x119 = (u8) (rnd() % 2); OGP[0].u.corneria.x11A = (u8) (rnd() % 2);
  OGP[0].u.corneria.x100 = (int) (rnd() % 0x400); OGP[0].u.corneria.x104 = (int) (rnd() % 5) - 1;
  OGP[0].u.corneria.xC8 = (grZakoGenerator_Config*) OP_grZakoGenerator_Config[0]; OGP[0].u.corneria.xCC = (grZakoGenerator_Config*) OP_grZakoGenerator_Config[1]; }""",
    """{ OGP[0].u.corneria.xC4.value = if rnd() % 3 == 0 { 0 } else { @@R1@@ as u8 }; OGP[0].u.corneria.xC5 = (rnd() % 3) as u8;
  OGP[0].u.corneria.xC6.value = if rnd() % 2 != 0 { 0 } else { @@R2@@ as u8 };
  OGP[0].u.corneria.x128 = addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]);
  OGP[0].u.corneria.x12C = if rnd() % 2 != 0 { addr_of_mut!(OJO[0]) } else { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) };
  OGP[0].u.corneria.x108 = rnd() % 6; OGP[0].u.corneria.x10C = (rnd() % 4) as i32 - 1; OGP[0].u.corneria.x110 = (rnd() % 12) as i32 - 1;
  OGP[0].u.corneria.x118 = ((rnd() % 4) as i32 - 1) as i8; OGP[0].u.corneria.x119 = (rnd() % 2) as u8; OGP[0].u.corneria.x11A = (rnd() % 2) as u8;
  OGP[0].u.corneria.x100 = (rnd() % 0x400) as i32; OGP[0].u.corneria.x104 = (rnd() % 5) as i32 - 1;
  OGP[0].u.corneria.xC8 = addr_of_mut!(OP_grZakoGenerator_Config[0]) as *mut grZakoGenerator_Config; OGP[0].u.corneria.xCC = addr_of_mut!(OP_grZakoGenerator_Config[1]) as *mut grZakoGenerator_Config; }""",
)

SHIP = raw(SHIP[0].replace("@@C1@@", PALC("rnd() % 16")).replace("@@C2@@", PALC("rnd() % 16")),
           SHIP[1].replace("@@R1@@", PALR("rnd() % 16")).replace("@@R2@@", PALR("rnd() % 16")))

STARFOX = raw(
    """{ OGP[0].u.corneria2.xF4 = (int) (rnd() % 7); OGP[0].u.corneria2.xF8 = (int) (rnd() % 4) - 1; OGP[0].u.corneria2.xF0 = (int) (rnd() % 20);
  OGP[0].u.corneria2.xFC = (int) (rnd() % 2); OGP[0].u.corneria2.x100 = (int) (rnd() % 2);
  OGP[0].u.corneria2.xDC = (rnd() % 3) ? &OGO[1 + rnd() % (NGO - 1)] : (HSD_GObj*) 0;
  OGP[0].u.corneria2.xE0 = (rnd() % 2) ? &OGO[1 + rnd() % (NGO - 1)] : (HSD_GObj*) 0; OGP[0].u.corneria2.xE4 = (rnd() % 2) ? &OGO[1 + rnd() % (NGO - 1)] : (HSD_GObj*) 0;
  OGP[0].u.corneria2.xE8 = (rnd() % 2) ? &OGO[1 + rnd() % (NGO - 1)] : (HSD_GObj*) 0; OGP[0].u.corneria2.xEC = (rnd() % 2) ? &OGO[1 + rnd() % (NGO - 1)] : (HSD_GObj*) 0;
  OGP[0].u.corneria2.xC8 = (int) (rnd() % 3); }""",
    """{ OGP[0].u.corneria2.xF4 = (rnd() % 7) as i32; OGP[0].u.corneria2.xF8 = (rnd() % 4) as i32 - 1; OGP[0].u.corneria2.xF0 = (rnd() % 20) as i32;
  OGP[0].u.corneria2.xFC = (rnd() % 2) as i32; OGP[0].u.corneria2.x100 = (rnd() % 2) as i32;
  OGP[0].u.corneria2.xDC = if rnd() % 3 != 0 { addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize]) } else { null_mut() };
  OGP[0].u.corneria2.xE0 = if rnd() % 2 != 0 { addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize]) } else { null_mut() };
  OGP[0].u.corneria2.xE4 = if rnd() % 2 != 0 { addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize]) } else { null_mut() };
  OGP[0].u.corneria2.xE8 = if rnd() % 2 != 0 { addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize]) } else { null_mut() };
  OGP[0].u.corneria2.xEC = if rnd() % 2 != 0 { addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize]) } else { null_mut() };
  OGP[0].u.corneria2.xC8 = (rnd() % 3) as i32; }""",
)

TALK = raw(
    """{ OGP[0].u.starfox.xC4.word = rnd() % 20; OGP[0].u.starfox.arwing_slot = (int) (rnd() % 5); OGP[0].u.starfox.xCC = (int) (rnd() % 4) - 1; }""",
    """{ OGP[0].u.starfox.xC4.word = rnd() % 20; OGP[0].u.starfox.arwing_slot = (rnd() % 5) as i32; OGP[0].u.starfox.xCC = (rnd() % 4) as i32 - 1; }""",
)

SMASH = raw(
    """{ OGP[0].u.smashtaunt.state = (s16) (rnd() % 4); OGP[0].u.smashtaunt.timer = (s16) ((int) (rnd() % 5) - 1);
  OGP[0].u.smashtaunt.line = (s16) (rnd() % 5);
  OGP[0].u.smashtaunt.jobj0 = (rnd() % 3) ? &OJO[rnd() % NJO] : (HSD_JObj*) 0; OGP[0].u.smashtaunt.jobj1 = (rnd() % 3) ? &OJO[rnd() % NJO] : (HSD_JObj*) 0;
  OGP[0].u.smashtaunt.jobj2 = (rnd() % 3) ? &OJO[rnd() % NJO] : (HSD_JObj*) 0;
  OGP[0].u.smashtaunt.joint_idx0 = (s16) (rnd() % 9); OGP[0].u.smashtaunt.joint_idx1 = (s16) (rnd() % 9); OGP[0].u.smashtaunt.joint_idx2 = (s16) (rnd() % 9);
  OGP[0].u.smashtaunt.text = (rnd() % 3) ? (HSD_Text*) OP_HSD_Text[rnd() % 2] : (HSD_Text*) 0; }""",
    """{ OGP[0].u.smashtaunt.state = (rnd() % 4) as i16; OGP[0].u.smashtaunt.timer = ((rnd() % 5) as i32 - 1) as i16;
  OGP[0].u.smashtaunt.line = (rnd() % 5) as i16;
  OGP[0].u.smashtaunt.jobj0 = if rnd() % 3 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) } else { null_mut() };
  OGP[0].u.smashtaunt.jobj1 = if rnd() % 3 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) } else { null_mut() };
  OGP[0].u.smashtaunt.jobj2 = if rnd() % 3 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) } else { null_mut() };
  OGP[0].u.smashtaunt.joint_idx0 = (rnd() % 9) as i16; OGP[0].u.smashtaunt.joint_idx1 = (rnd() % 9) as i16; OGP[0].u.smashtaunt.joint_idx2 = (rnd() % 9) as i16;
  OGP[0].u.smashtaunt.text = if rnd() % 3 != 0 { addr_of_mut!(OP_HSD_Text[(rnd() % 2) as usize]) as *mut HSD_Text } else { null_mut() }; }""",
)

GRKIND = raw(
    "stage_info.grkind = (rnd() % 3) ? Gr_Kind_Corneria : Gr_Kind_Venom;",
    "(*(addr_of_mut!(G_stage_info) as *mut StageInfo)).grkind = if rnd() % 3 != 0 { Gr_Kind_Corneria } else { Gr_Kind_Venom };",
)


def ai(expr_c, expr_rs, idx=0):
    return raw(f"AI[{idx}] = {expr_c};", f"a.i[{idx}] = {expr_rs};")


funcs = []
ALLU = ["corneria", "corneria2", "arwing", "starfox", "smashtaunt"]


def F(name, u=None, args=None, pre_parts=(), n=None, bfs=None, voidret=False, base=True):
    d = dict(name=name)
    d["u"] = u if u is not None else ALLU
    if bfs:
        d["bfs"] = bfs
    if voidret:
        d["voidret"] = True
    if args:
        d["c_args"] = [x[0] for x in args]
        d["rs_args"] = [x[1] for x in args]
    parts = ([CG] if base else []) + list(pre_parts)
    c, r = both(NNs, *parts)
    d["pre_c"] = c
    d["pre_rs"] = r
    if n:
        d["n"] = n
    funcs.append(d)


G0 = ("&OGO[0]", "g")
GOBJ_ARG = [G0]

# ---- whole-stage functions
F("grCorneria_801DCCFC", pre_parts=[])
F("grCorneria_801DCE1C", pre_parts=[SHIP], n=500)
F("grCorneria_801DD2C0", args=[("(int) AI[0]", "a.i[0] as i32")], pre_parts=[SHIP], n=300)
F("grCorneria_801DD350", pre_parts=[SHIP], n=300)
F("grCorneria_801DD478", pre_parts=[SHIP], n=300)
F("grCorneria_801DD508")
F("grCorneria_801DD52C")
F("grCorneria_801DD534", args=[("(int) AI[0]", "a.i[0] as i32")], pre_parts=[NN_RND, ai("rnd() % 19", "rnd() % 19"), SHIP], n=600)
# ---- stage gobj callbacks
for nm in ["grCorneria_801DD620", "grCorneria_801DD64C", "grCorneria_801DD654", "grCorneria_801DD658", "grCorneria_801DD9A0",
           "grCorneria_801DDAC0", "grCorneria_801DDCE8", "grCorneria_801DE4BC", "grCorneria_801DE560", "grCorneria_801DE8E0",
           "grCorneria_801DEC00", "grCorneria_801DF8CC", "grCorneria_801DFBC4", "grCorneria_801DFBEC", "stageGObj6_OnInit",
           "grCorneria_801DFC1C", "grCorneria_801DFC24", "grCorneria_801DFC28", "grCorneria_801DFC90", "grCorneria_801DFEB4",
           "grCorneria_801DFF18", "grCorneria_801E013C", "grCorneria_801E01A0", "grCorneria_801E03C4", "grCorneria_801E0D28",
           "grCorneria_801E0DE0", "grCorneria_801E0E0C", "grCorneria_801E0E3C", "grCorneria_801E0F30", "grCorneria_801E0F64",
           "grCorneria_801E1054", "grCorneria_801E1058", "grCorneria_801E12CC"]:
    F(nm, n=60, pre_parts=[])
F("grCorneria_801DD65C", pre_parts=[SHIP])
F("grCorneria_801DD674", pre_parts=[SHIP], n=300)
F("grCorneria_801DD9A8", pre_parts=[SHIP], n=500)
F("grCorneria_801DDAC4", pre_parts=[SHIP], n=300)
F("grCorneria_801DDCF0", n=100)
F("grCorneria_801DDD4C", n=100)
F("grCorneria_801DDDA8", args=[("(AI[0] & 1) ? &OGO[0] : (HSD_GObj*) 0", "if a.i[0] & 1 != 0 { g } else { null_mut() }"), ("&AV3[0]", "&mut a.v3[0]")],
  pre_parts=[SHIP], n=300)
F("grCorneria_801DDE88", pre_parts=[SHIP], n=500)
F("grCorneria_801DE024", pre_parts=[SHIP], n=600)
F("grCorneria_Arwing_801DE4C0", pre_parts=[SHIP], n=300)
F("grCorneria_801DE568", pre_parts=[SHIP], n=500)
F("grCorneria_801DE8E4", pre_parts=[STARFOX], n=400)
F("grCorneria_801DEC08", n=200)
F("grCorneria_801DEC94", n=300)
F("grCorneria_801DED50", pre_parts=[SHIP, STARFOX], n=1500)
F("grCorneria_801DF8D0", pre_parts=[STARFOX], n=400)
F("grCorneria_801DFBCC", pre_parts=[SHIP, STARFOX], n=500)
F("grCorneria_801DFC2C", pre_parts=[SHIP], n=200)
F("grCorneria_801DFC98", pre_parts=[SHIP], n=500)
F("grCorneria_801DFEB8", pre_parts=[SHIP], n=200)
F("grCorneria_801DFF20", pre_parts=[SHIP], n=500)
F("grCorneria_801E0140", pre_parts=[SHIP], n=200)
F("grCorneria_801E01A8", pre_parts=[SHIP], n=500)
F("grCorneria_801E03C8", voidret=True, args=[G0, ("(int) AI[0]", "a.i[0] as i32")],
  pre_parts=[SHIP, raw("{ static const int I_[] = { 8, 9, 4, 5, 1 }; AI[0] = I_[rnd() % 5]; }", "{ const I_: [u32; 5] = [8, 9, 4, 5, 1]; a.i[0] = I_[(rnd() % 5) as usize]; }")], n=300)
F("grCorneria_801E0678", pre_parts=[SHIP], n=200)
F("grCorneria_801E08CC", pre_parts=[SHIP], n=300)
F("grCorneria_801E0A74", n=300)
F("grCorneria_801E0C3C", pre_parts=[SHIP], n=200)
F("grCorneria_801E0D30", n=100)
F("grCorneria_801E0DE4", pre_parts=[SMASH], n=100)
F("grCorneria_801E0E14", pre_parts=[SMASH], n=500)
F("grCorneria_801E0E40", pre_parts=[nid(12)], n=600)
F("grCorneria_801E0F34", args=[G0, ("(int) AI[0]", "a.i[0] as i32")], pre_parts=[ai("(int) (rnd() % 30) - 5", "((rnd() % 30) as i32 - 5) as u32")], n=300)
F("grCorneria_801E0F6C", pre_parts=[TALK, nid(5)], n=600)
F("grCorneria_801E1030", n=60)
F("grCorneria_801E1060", n=200)
F("fn_801E12D0", args=[G0, ("&OGP[0]", "addr_of_mut!(OGP[0])")], n=60)
F("fn_801E12D4", args=[G0, ("&OGP[0]", "addr_of_mut!(OGP[0])"), ("&AV3[0]", "&mut a.v3[0]"), ("&OGO[1]", "addr_of_mut!(OGO[1])"), ("AF[0]", "a.f[0]")],
  pre_parts=[SHIP], n=300)
F("grCorneria_801E1348", pre_parts=[SHIP], n=1500)
F("grCorneria_801E1878", pre_parts=[SHIP], n=200)
F("grCorneria_801E1970", pre_parts=[SHIP], n=500)
F("grCorneria_801E1BF0", n=200)
F("grCorneria_801E2110", pre_parts=[], n=200)
F("grCorneria_801E2228", pre_parts=[SHIP], n=1000)
F("grCorneria_801E2454", n=100)
F("grCorneria_801E2480", n=100)
F("smashTaunt_801E24AC", args=[G0, ("(int) AI[0]", "a.i[0] as i32")], pre_parts=[SMASH], n=300)
F("smashTaunt_801E2550", args=[G0, ("&OGP[0].u.smashtaunt", "addr_of_mut!(OGP[0].u.smashtaunt)")], pre_parts=[SMASH], n=100)
F("grCorneria_801E2598", pre_parts=[ai("rnd() % 20", "rnd() % 20", 0), ai("rnd() % 5", "rnd() % 5", 1)],
  args=[("AI[0]", "a.i[0] as u32"), ("AI[1]", "a.i[1] as u32")], n=200)
F("grCorneria_801E25C4", args=[G0, ("&OGP[0].u.smashtaunt", "addr_of_mut!(OGP[0].u.smashtaunt)"), ("(int) AI[0]", "a.i[0] as i32"), ("(int) AI[1]", "a.i[1] as i32"), ("(int) AI[2]", "a.i[2] as i32")],
  pre_parts=[SMASH, ai("rnd() % 5", "rnd() % 5", 0)], n=300)
F("grCorneria_801E2738", args=[G0, ("&OGP[0].u", "addr_of_mut!(OGP[0].u) as *mut c_void"), ("AI[0]", "a.i[0] as u32"), ("AI[1]", "a.i[1] as u32")],
  pre_parts=[SMASH, ai("rnd() % 20", "rnd() % 20", 0), ai("rnd() % 5", "rnd() % 5", 1)], n=300)
F("grCorneria_801E277C", args=[G0, ("&OGP[0].u.smashtaunt", "addr_of_mut!(OGP[0].u.smashtaunt)")], pre_parts=[SMASH], n=1000)
F("grCorneria_801E2A6C", pre_parts=[GRKIND, nid(12, 5)], n=300)
F("grCorneria_801E2AF4", pre_parts=[GRKIND, nid(12, 5)], n=300)
F("grCorneria_801E2B80", pre_parts=[GRKIND, nid(12), NN_RND, raw("NN = 1;", "NN = 1;")], n=300)
F("grCorneria_801E2C34", pre_parts=[GRKIND, nid(12), raw("NN = 1;", "NN = 1;")], n=300)
F("grCorneria_801E2CE8", pre_parts=[GRKIND], n=100)
F("grCorneria_801E2D14", pre_parts=[GRKIND, nid(12, 8)], n=200)
F("grCorneria_801E2D90", args=[("(enum_t) ((int) AI[0] - 1)", "(a.i[0] as i32 - 1)")], pre_parts=[GRKIND], n=300)
F("grCorneria_801E2E50", args=[("(int) AI[0] - 1", "(a.i[0] as i32 - 1)")], pre_parts=[GRKIND], n=300)
F("grCorneria_801E2EA0", pre_parts=[SHIP], n=200)
F("grCorneria_801E2EE4", args=[("(enum_t) AI[0]", "a.i[0] as enum_t")], n=60)
F("grCorneria_801E2EEC", args=[("&AV3[0]", "&mut a.v3[0]"), ("(int) AI[0]", "a.i[0] as i32"), ("&OJO[0]", "addr_of_mut!(OJO[0])")], pre_parts=[SHIP], n=500)
F("grCorneria_801E2FCC", pre_parts=[SHIP], n=200)

# These write a whole word / short of Ground.u through another member (arwing.xC4 = n, smashtaunt.state, ...), which the C (LSB-first
# bitfields) and the Rust (MSB-first like the PPC) read back as different flags: the flag views of that storage are not compared.
BFA = ["u.corneria.xC4.flags", "u.corneria.xC6.flags", "u.corneria.offset_y.flags", "u.corneria2.xC4.flags", "u.starfox.xC4.flags"]
WORD_WRITERS = ["grCorneria_801DD2C0", "grCorneria_801E0F34", "grCorneria_801DD9A8", "grCorneria_801DED50", "grCorneria_801DFBCC",
                "grCorneria_801E0DE4", "grCorneria_801E0E14", "grCorneria_801E0E40", "grCorneria_801E0F6C", "grCorneria_801E1970",
                "smashTaunt_801E24AC", "smashTaunt_801E2550", "grCorneria_801E25C4", "grCorneria_801E2738", "grCorneria_801E277C",
                "grCorneria_801E2B80", "grCorneria_801E2C34"]
# these also set xC6 / offset_y flag bits through the accessors, so those stay compared by value
BIT_AND_WORD = {"grCorneria_801DD2C0": [0, 1, 3, 4], "grCorneria_801DD9A8": [0, 3, 4]}
for _d in funcs:
    if _d["name"] in WORD_WRITERS:
        _d["bfs"] = [x for i, x in enumerate(BFA) if i not in BIT_AND_WORD.get(_d["name"], [])]

# on_init of the Arwing gobjs sets `xC4.flags.b0 = 1` (a bit write) and DD350 then stores `xC4.value = 1` into three more grounds: the
# whole byte differs by design between the LSB-first C and the MSB-first Rust, so the flag view is not compared
for _d in funcs:
    if _d["name"] in ("grCorneria_801DD350", "grCorneria_801DD2C0"):
        _d["bfx"] = ["u.corneria.xC4.flags", "u.corneria2.xC4.flags", "u.starfox.xC4.flags"]

CALLBACK_NAMES = [
    "grCorneria_801DD620", "grCorneria_801DD64C", "grCorneria_801DD654", "grCorneria_801DD658", "grCorneria_801DDAC4",
    "grCorneria_801DDCE8", "grCorneria_801DE024", "grCorneria_801DE4BC", "grCorneria_801DE8E4", "grCorneria_801DEC00",
    "grCorneria_801DED50", "grCorneria_801DF8CC", "grCorneria_801DD674", "grCorneria_801DD9A0", "grCorneria_801DD9A8",
    "grCorneria_801DDAC0", "grCorneria_801E0140", "grCorneria_801E01A0", "grCorneria_801E01A8", "grCorneria_801E03C4",
    "grCorneria_801E0DE4", "grCorneria_801E0E0C", "grCorneria_801E0E14", "grCorneria_801E0E3C", "stageGObj6_OnInit",
    "grCorneria_801DFC1C", "grCorneria_801DFC24", "grCorneria_801DFC28", "grCorneria_801E0C3C", "grCorneria_801E0D28",
    "grCorneria_801E0D30", "grCorneria_801E0DE0", "grCorneria_801DFC2C", "grCorneria_801DFC90", "grCorneria_801DFC98",
    "grCorneria_801DFEB4", "grCorneria_801DFEB8", "grCorneria_801DFF18", "grCorneria_801DFF20", "grCorneria_801E013C",
    "grCorneria_801DF8D0", "grCorneria_801DFBC4", "grCorneria_801DFBCC", "grCorneria_801DFBEC", "grCorneria_801E1054",
    "grCorneria_801E1058", "grCorneria_801E1060", "grCorneria_801E12CC", "grCorneria_801E0F30", "grCorneria_801E0F64",
    "grCorneria_801E0F6C", "grCorneria_801E1030", "grCorneria_Arwing_801DE4C0", "grCorneria_801DE560", "grCorneria_801DE568",
    "grCorneria_801DE8E0", "grCorneria_801DD350", "grCorneria_801DD2C0", "grCorneria_801DD478", "grCorneria_801DD508",
    "grCorneria_801DD52C", "grCorneria_801E2EE4", "grCorneria_801E2EEC", "grCorneria_801DD65C", "grCorneria_801E2454",
    "grCorneria_801E2480", "fn_801E12D0", "fn_801E12D4", "smashTaunt_801E24AC",
]

SPEC = dict(
    src="src/melee/gr/grcorneria.c",
    rust_mod="grcorneria",
    ext_mods=["grinl"],
    u=ALLU,
    bf_skip=["StageCallbacks"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    variadic={"OSReport": "pii"},
    provides=["HSD_GObjPLinkHead", "HSD_PadMasterStatus", "HSD_SisLib_804D1124"],
    c_patches=[
        # the ship's ground is a `corneria` record: the arwing view of +0x18 (xDC) is its offset_x on the PPC
        ("vec->x = gp2->u.arwing.xDC + (-pos.z + lbl_803E2068[idx].x);", "vec->x = gp2->u.corneria.offset_x + (-pos.z + lbl_803E2068[idx].x);"),
    ],
    data_c="""static int NN;
static unsigned NIDM;
static HSD_GObj* OPLH[HSD_GOBJ_PLINK_FIGHTER + 1];
HSD_GObj** HSD_GObjPLinkHead = OPLH;
HSD_PadStatus HSD_PadMasterStatus[4];
SIS* HSD_SisLib_804D1124[5];""",
    use_rs="""static mut NN: i32 = 0;
static mut NIDM: u32 = 0;
static mut OPAD: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
static mut OSIS: [*mut SIS; 5] = [core::ptr::null_mut(); 5];
use core::ptr::null_mut;""",
    extra_pools=["grCorneria_YakumonoParam", "UnkArchiveStruct", "HSD_Text", "grZakoGenerator_Config"],
    statics=[
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grCn_804D69A4", rs="t::grCn_804D69A4", kind="u"),
        dict(c="grCn_804D69A8", rs="t::grCn_804D69A8", kind="i"),
        dict(c="grCn_804D69AC", rs="t::grCn_804D69AC", kind="i"),
        dict(c="grCn_804D69B0", rs="t::grCn_804D69B0", kind="i"),
        dict(c="grCn_804D466C", rs="t::grCn_804D466C", kind="i"),
        dict(c="arwing_gobjs", rs="t::arwing_gobjs", kind="p", n=3),
        dict(c="arwing_types", rs="t::arwing_types", kind="i", n=3),
        dict(c="arwing_groups", rs="t::arwing_groups", kind="i", n=3),
        dict(c="grCn_StageCallbacks", rs="t::grCn_StageCallbacks", ty="StageCallbacks", n=19),
        dict(c="grCn_StageData", rs="t::grCn_StageData", ty="StageData"),
        dict(c="grCn_803E1D38", rs="t::grCn_803E1D38", ty="GrJoint", n=8),
        dict(c="grCn_803E1F70", rs="t::grCn_803E1F70", ty="Vec3", n=5),
        dict(c="grCn_803E1FAC", rs="t::grCn_803E1FAC", ty="Vec3", n=5, fill=True),
        dict(c="HSD_PadMasterStatus", rs="OPAD", ty="HSD_PadStatus", n=4, fill=True),
        dict(c="HSD_SisLib_804D1124", rs="OSIS", kind="p", n=5),
    ],
    graph=[dict(kind="list", arr="OGO", n=6, field="next", z=0)],
    links_c="    NN = 0; NIDM = 0; yakumono_param = (void*) OP_grCorneria_YakumonoParam[0]; OPLH[HSD_GOBJ_PLINK_FIGHTER] = &OGO[0];",
    links_rs="    NN = 0; NIDM = 0; t::yakumono_param = addr_of_mut!(OP_grCorneria_YakumonoParam[0]) as *mut _;",
    hooks_rs="""    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);
    melee_world::tu::grcorneria::ext::GRDISPLAY_801C5DB0 = Some(|a0: *mut HSD_GObj, a1: i32| { lg_s("grDisplay_801C5DB0("); lg_ptr(a0 as *const u8 as usize); lg_int(a1 as u32); lg_s(")"); lg_c(';'); });
    melee_world::tu::grcorneria::ext::HSD_GOBJPLINKHEAD = Some(|_l: i32| addr_of_mut!(OGO[0]));
    melee_world::tu::grcorneria::ext::HSD_PADMASTERSTATUS = Some(|| addr_of_mut!(OPAD) as *mut HSD_PadStatus);
    melee_world::tu::grcorneria::ext::HSD_SISLIB_804D1124 = Some(|| addr_of_mut!(OSIS) as *mut *mut SIS);""",
    outs={"lb_8000B1CC": [2], "ftLib_GetPos": [1], "Camera_GetTransformInterest": [0], "Player_LoadPlayerCoords": [1]},
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "HSD_Randf": ("rf()", "rf()"),
        "Ground_801C3FA4": ("((NN || rnd() % 5) ? (void*) &OJO[rnd() % NJO] : (void*) 0)",
                            "(if NN != 0 || rnd() % 5 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _ } else { null_mut() })"),
        "Ground_GetStageGObj": ("((NN || rnd() % 4) ? (void*) &OGO[1 + rnd() % (NGO - 1)] : (void*) 0)",
                                "(if NN != 0 || rnd() % 4 != 0 { addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize]) as *mut _ } else { null_mut() })"),
        "Ground_GetMapGObj": ("((!((NIDM >> a0) & 1u) || (rnd() % 2)) ? (void*) &OGO[rnd() % NGO] : (void*) 0)",
                              "(if (NIDM >> a0) & 1 == 0 || rnd() % 2 != 0 { addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _ } else { null_mut() })"),
        "Ground_GetYakumonoParam": ("(void*) OP_grCorneria_YakumonoParam[0]", "addr_of_mut!(OP_grCorneria_YakumonoParam[0]) as *mut c_void"),
        "grDatFiles_GetArchive": ("(void*) OP_UnkArchiveStruct[rnd() % 2]", "addr_of_mut!(OP_UnkArchiveStruct[(rnd() % 2) as usize]) as *mut c_void"),
        "HSD_SisLib_803A5ACC": ("(void*) OP_HSD_Text[rnd() % 2]", "addr_of_mut!(OP_HSD_Text[(rnd() % 2) as usize]) as *mut c_void"),
        "Ground_801C0498": ("(float) (4 + rnd() % 4) / 4.0f", "((4 + rnd() % 4) as f32 / 4.0)"),
        "Stage_80225194": ("((rnd() % 4 == 0) ? 0xE9 : (rnd() % 4 == 0) ? 0x47 : (rnd() % 4 == 0) ? 0x46 : rnd() % 100)",
                           "(if rnd() % 4 == 0 { 0xE9 } else if rnd() % 4 == 0 { 0x47 } else if rnd() % 4 == 0 { 0x46 } else { rnd() % 100 })"),
        "mpJointFromLine": ("(int) (rnd() % 9)", "((rnd() % 9) as i32)"),
    },
    fnvals={**{n: f"t::{n}" for n in CALLBACK_NAMES}, "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0"},
    funcs=funcs,
)
