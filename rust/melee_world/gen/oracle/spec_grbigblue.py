"""State-oracle spec of src/melee/gr/grbigblue.c (rust/melee_world/src/tu/grbigblue.rs).

The C is compiled natively (LP64 pointers), so the GameCube layout tricks of the source are patched to the field accesses
they stand for (c_patches / c_regex / c_fn_regex below); the Rust port uses the same fields. What is patched:
  * `(u8*) gp + 0xNN` accesses of the Ground's union (platform / manager / car-lane views),
  * the `grBigBlue_CarPhysics` / `grBb_CarGround` overlays and `iter += 0x40` lane walks,
  * `grBigBlue_801E8B84`'s `gp = (Ground*) &gp->x20[8]` lane step,
  * HSD_MemAlloc(120) (30 pointers) and the manager's 4-byte `event_extra` slot (`pad_3`).
"""

import re


def both(*parts):
    return ("".join(p[0] + " " for p in parts), "".join(p[1] + " " for p in parts))


def raw(c, r):
    return (c, r)


# ---------------------------------------------------------------------------------------------- world pieces
CG = raw(
    """{ int j_, k_;
  NN = 1; R32 = 1; R33 = 2; R34 = 3; R36 = 4;
  yakumono_param->x10 = (int) (rnd() % 40); yakumono_param->x14 = (int) (rnd() % 40); yakumono_param->x18 = (int) (rnd() % 5);
  yakumono_param->x1C = (int) (rnd() % 120); yakumono_param->x20 = (int) (rnd() % 50);
  yakumono_param->x88 = (int) (rnd() % 60); yakumono_param->x8C = (int) (rnd() % 40);
  yakumono_param->xB0 = (int) (rnd() % 30); yakumono_param->xB4 = (int) (rnd() % 30); yakumono_param->xB8 = (int) (rnd() % 4);
  yakumono_param->xDC = (int) (rnd() % 12); yakumono_param->xE0 = (int) (rnd() % 12);
  yakumono_param->x10C = (int) (rnd() % 12); yakumono_param->x110 = (int) (rnd() % 12);
  yakumono_param->x11C = (int) (rnd() % 12); yakumono_param->x120 = (int) (rnd() % 12);
  yakumono_param->x5C = (float) (int) (rnd() % 30); yakumono_param->x60 = (float) (int) (rnd() % 30);
  for (j_ = 0; j_ < NGO; j_++) {
    for (k_ = 0; k_ < 4; k_++) {
      OGP[j_].u.bigblue.car.lanes[k_].state = rnd() % 11;
      OGP[j_].u.bigblue.car.lanes[k_].direction = rnd() % 2;
      OGP[j_].u.bigblue.car.lanes[k_].collision_slot = rnd() % 30;
    }
  }
  for (j_ = 0; j_ < 12; j_++) {
    grBb_TrackEntries[j_].jobj_index = TE_IDX[j_][0]; grBb_TrackEntries[j_].start_index = TE_IDX[j_][1]; grBb_TrackEntries[j_].end_index = TE_IDX[j_][2];
    grBb_TrackEntries[j_].pad = 0;
    grBb_TrackEntries[j_].delta.x = rf(); grBb_TrackEntries[j_].delta.y = rf(); grBb_TrackEntries[j_].delta.z = rf();
  }
  if (rnd() % 2) OJO[NGO].child = &OJO[3 * NGO + 1];
}""",
    """{
  NN = 1; R32 = 1; R33 = 2; R34 = 3; R36 = 4;
  let y_ = t::yakumono_param;
  (*y_).x10 = (rnd() % 40) as i32; (*y_).x14 = (rnd() % 40) as i32; (*y_).x18 = (rnd() % 5) as i32;
  (*y_).x1C = (rnd() % 120) as i32; (*y_).x20 = (rnd() % 50) as i32;
  (*y_).x88 = (rnd() % 60) as i32; (*y_).x8C = (rnd() % 40) as i32;
  (*y_).xB0 = (rnd() % 30) as i32; (*y_).xB4 = (rnd() % 30) as i32; (*y_).xB8 = (rnd() % 4) as i32;
  (*y_).xDC = (rnd() % 12) as i32; (*y_).xE0 = (rnd() % 12) as i32;
  (*y_).x10C = (rnd() % 12) as i32; (*y_).x110 = (rnd() % 12) as i32;
  (*y_).x11C = (rnd() % 12) as i32; (*y_).x120 = (rnd() % 12) as i32;
  (*y_).x5C = (rnd() % 30) as i32 as f32; (*y_).x60 = (rnd() % 30) as i32 as f32;
  for j_ in 0..NGO {
    for k_ in 0..4usize {
      let l_ = addr_of_mut!(OGP[j_].u.bigblue.anon_0.car.lanes[k_].anon_0);
      (*l_).anon_0.set_state((rnd() % 11) as u8);
      (*l_).anon_0.set_direction((rnd() % 2) as u8);
      (*l_).anon_0_0.set_collision_slot((rnd() % 30) as u16);
    }
  }
  for j_ in 0..12usize {
    t::grBb_TrackEntries[j_].jobj_index = TE_IDX[j_][0]; t::grBb_TrackEntries[j_].start_index = TE_IDX[j_][1]; t::grBb_TrackEntries[j_].end_index = TE_IDX[j_][2];
    t::grBb_TrackEntries[j_].pad = 0;
    t::grBb_TrackEntries[j_].delta.x = rf(); t::grBb_TrackEntries[j_].delta.y = rf(); t::grBb_TrackEntries[j_].delta.z = rf();
  }
  if rnd() % 2 != 0 { OJO[NGO].child = addr_of_mut!(OJO[3 * NGO + 1]); }
}""",
)


def ix(g):
    """(C index expr, Rust index expr) of a ground slot: an int literal or a role variable"""
    return (str(g), str(g) if isinstance(g, int) else f"{g} as usize")


def roles(r32=1, r33=2, r34=3, r36=4):
    return raw(f"R32 = {r32}; R33 = {r33}; R34 = {r34}; R36 = {r36};", f"R32 = {r32}; R33 = {r33}; R34 = {r34}; R36 = {r36};")


def mgr(g, x1_three=False):
    gc, gr = ix(g)
    x1c = "(u8) ((rnd() % 3 == 0) ? 3 : rnd() % 4)" if x1_three else "(u8) (rnd() % 5)"
    x1r = "(if rnd() % 3 == 0 { 3 } else { (rnd() % 4) as u8 })" if x1_three else "((rnd() % 5) as u8)"
    return raw(
        f"""{{ int j_, p_[3]; Ground* m_ = &OGP[{gc}]; p_[0] = 0; p_[1] = 1; p_[2] = 2;
  {{ int a_ = (int) (rnd() % 3); int b_ = (int) (rnd() % 3); int t_ = p_[a_]; p_[a_] = p_[b_]; p_[b_] = t_; }}
  m_->u.bigblue.manager.flags = rnd() % 12;
  m_->u.bigblue.x0 = (u8) (rnd() % 4);
  {{ u32 t_ = rnd() % 3; m_->u.bigblue.x1 = (u8) (t_ == 0 ? 0 : t_ == 1 ? 1 : 0xFF); }}
  for (j_ = 0; j_ < 3; j_++) {{
    m_->u.bigblue.xD4[j_] = &OJO[rnd() % NJO];
    m_->u.bigblue.data[j_].index = (u8) p_[j_];
    m_->u.bigblue.data[j_].x1 = {x1c};
    m_->u.bigblue.data[j_].x2 = (s8) ((int) (rnd() % 3) - 1);
    m_->u.bigblue.data[j_].x4 = (int) (rnd() % 9) - 3;
    m_->u.bigblue.data[j_].x34 = (int) (rnd() % 4);
    m_->u.bigblue.data[j_].x2C = (int) (rnd() % 3);
    m_->u.bigblue.data[j_].x30 = (int) (rnd() % 3);
    m_->u.bigblue.data[j_].x50 = 0;
  }}
  for (j_ = 0; j_ < 3; j_++) {{
    u32 v_ = rnd() % 4;
    m_->u.bigblue.manager.event_data[j_] = v_ == 0 ? (void*) 0 : v_ == 1 ? (void*) 1 : (void*) &OJO[rnd() % NJO];
  }}
}}""",
        f"""{{ let mut p_ = [0usize, 1, 2]; let m_ = addr_of_mut!(OGP[{gr}]);
  let s_ = addr_of_mut!((*m_).u.bigblue.anon_0.anon_0); let mg_ = addr_of_mut!((*m_).u.bigblue.anon_0.manager);
  {{ let a_ = (rnd() % 3) as usize; let b_ = (rnd() % 3) as usize; p_.swap(a_, b_); }}
  (*mg_).flags = rnd() % 12;
  (*s_).anon_0.anon_0.x0 = (rnd() % 4) as u8;
  {{ let t_ = rnd() % 3; (*s_).anon_0.anon_0.x1 = if t_ == 0 {{ 0 }} else if t_ == 1 {{ 1 }} else {{ 0xFF }}; }}
  for j_ in 0..3usize {{
    (*s_).xD4[j_] = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]);
    (*s_).data[j_].index = p_[j_] as u8;
    (*s_).data[j_].x1 = {x1r};
    (*s_).data[j_].x2 = ((rnd() % 3) as i32 - 1) as i8;
    (*s_).data[j_].x4 = (rnd() % 9) as i32 - 3;
    (*s_).data[j_].x34 = (rnd() % 4) as i32;
    (*s_).data[j_].x2C = (rnd() % 3) as i32;
    (*s_).data[j_].x30 = (rnd() % 3) as i32;
    (*s_).data[j_].x50 = 0;
  }}
  for j_ in 0..3usize {{
    let v_ = rnd() % 4;
    (*mg_).event_data[j_] = if v_ == 0 {{ null_mut() }} else if v_ == 1 {{ 1usize as *mut c_void }} else {{ addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut c_void }};
  }}
}}""",
    )


def car(g):
    gc, gr = ix(g)
    return raw(
        f"""{{ int j_, k_; Ground* c_ = &OGP[{gc}];
  c_->u.bigblue.car.collision_jobjs = OJAR; c_->u.bigblue.car.ranks = ORK;
  for (j_ = 0; j_ < 32; j_++) {{ OJAR[j_] = &OJO[rnd() % NJO]; ORK[j_] = (u8) (rnd() % 3); }}
  c_->u.bigblue.car.spawn_timer = (s16) ((int) (rnd() % 14) - 4);
  for (k_ = 0; k_ < 4; k_++) {{
    struct grBigBlue_CarLane* l_ = &c_->u.bigblue.car.lanes[k_];
    l_->state = rnd() % 11; l_->direction = rnd() % 2; l_->collision_slot = rnd() % 30;
    l_->threshold = (int) (rnd() % 30) - 5;
    {{ u32 t_ = rnd() % 4; if (t_ == 0) l_->alpha = 0.0f; else if (t_ == 1) l_->alpha = 1.0f; }}
  }}
}}""",
        f"""{{ let c_ = addr_of_mut!(OGP[{gr}].u.bigblue.anon_0.car);
  (*c_).collision_jobjs = addr_of_mut!(OJAR) as *mut *mut HSD_JObj; (*c_).ranks = addr_of_mut!(ORK) as *mut u8;
  for j_ in 0..32usize {{ OJAR[j_] = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); ORK[j_] = (rnd() % 3) as u8; }}
  (*c_).spawn_timer = ((rnd() % 14) as i32 - 4) as i16;
  for k_ in 0..4usize {{
    let l_ = addr_of_mut!((*c_).lanes[k_]);
    (*l_).anon_0.anon_0.set_state((rnd() % 11) as u8); (*l_).anon_0.anon_0.set_direction((rnd() % 2) as u8); (*l_).anon_0.anon_0_0.set_collision_slot((rnd() % 30) as u16);
    (*l_).threshold = (rnd() % 30) as i32 - 5;
    {{ let t_ = rnd() % 4; if t_ == 0 {{ (*l_).alpha = 0.0; }} else if t_ == 1 {{ (*l_).alpha = 1.0; }} }}
  }}
}}""",
    )


def plat(g, one=False):
    gc, gr = ix(g)
    return raw(
        f"""{{ Ground* p_ = &OGP[{gc}];
  p_->u.bigblue.x0 = (u8) (rnd() % 4);
  {{ u32 t_ = rnd() % 3; p_->u.bigblue.x1 = (u8) (t_ == 0 ? 0 : t_ == 1 ? 1 : 0xFF); }}
  p_->u.bigblue.platform.xC8_timer = (int) (rnd() % 6) - 1;
  if (rnd() % 2) p_->u.bigblue.platform.xCC_timer = (int) (rnd() % 6) - 1;
  if (rnd() % 2) p_->u.bigblue.platform.xD0_timer = (int) (rnd() % 6) - 1;
}}""",
        f"""{{ let p_ = addr_of_mut!(OGP[{gr}]);
  (*p_).u.bigblue.anon_0.anon_0.anon_0.anon_0.x0 = (rnd() % 4) as u8;
  {{ let t_ = rnd() % 3; (*p_).u.bigblue.anon_0.anon_0.anon_0.anon_0.x1 = if t_ == 0 {{ 0 }} else if t_ == 1 {{ 1 }} else {{ 0xFF }}; }}
  (*p_).u.bigblue.anon_0.platform.xC8_timer = (rnd() % 6) as i32 - 1;
  if rnd() % 2 != 0 {{ (*p_).u.bigblue.anon_0.platform.xCC_timer = (rnd() % 6) as i32 - 1; }}
  if rnd() % 2 != 0 {{ (*p_).u.bigblue.anon_0.platform.xD0_timer = (rnd() % 6) as i32 - 1; }}
}}""",
    )


def road(g):
    gc, gr = ix(g)
    return raw(
        f"""{{ Ground* r_ = &OGP[{gc}];
  r_->u.bigblue.cur_lane = rnd() % 12; r_->u.bigblue.next_lane = rnd() % 12; r_->u.bigblue.prev_lane = rnd() % 12;
  r_->u.bigblue.nibble_hi = rnd() % 5;
  r_->u.bigblue.road.direction = (s16) ((int) (rnd() % 3) - 1);
  r_->u.bigblue.road.position.y = -100000.0f;
}}""",
        f"""{{ let r_ = addr_of_mut!(OGP[{gr}]); let l_ = addr_of_mut!((*r_).u.bigblue.anon_0.anon_0.anon_0.anon_0_0_);
  (*l_).set_cur_lane(rnd() % 12); (*l_).set_next_lane(rnd() % 12); (*l_).set_prev_lane(rnd() % 12);
  (*l_).set_nibble_hi(rnd() % 5);
  (*r_).u.bigblue.anon_0.road.direction = ((rnd() % 3) as i32 - 1) as i16;
  (*r_).u.bigblue.anon_0.road.position.y = -100000.0;
}}""",
    )


def ai(expr_c, expr_rs, idx=0):
    return raw(f"AI[{idx}] = {expr_c};", f"a.i[{idx}] = {expr_rs};")


NN_ = lambda v: raw(f"NN = {v};", f"NN = {v};")

HDR = ["u.bigblue.b0", "u.bigblue.b1", "u.bigblue.b2", "u.bigblue.prev_lane", "u.bigblue.cur_lane", "u.bigblue.next_lane",
       "u.bigblue.nibble_hi", "u.bigblue.nibble_lo", "u.bigblue.x0_b1"]
# the lane word's padding bitfields and `state_hi` alias the other lane flags differently in the LSB-first C and the MSB-first Rust
LANEX = [f"u.bigblue.car.lanes[{k}].{n}" for k in range(4) for n in ("pad_slot_0", "pad_slot_1", "state_hi")]

# the manager's data[] (and the platform / road views) overlay the car lanes: the lane flag bytes then hold ints / floats that
# the LSB-first C and the MSB-first Rust read as different bitfield values (the bytes themselves are compared)
LANEALL = [f"u.bigblue.car.lanes[{k}].{n}" for k in range(4) for n in ("state", "direction", "collision_slot")]
MGR_VIEW = {"grBigBlue_801E57C0", "grBigBlue_801E59F8", "grBigBlue_801E6904", "grBigBlue_801E6C60", "fn_801E8560", "grBigBlue_801E8794",
            "grBigBlue_801E8978", "grBigBlue_801E89DC", "grBigBlue_801E8A1C", "grBigBlue_801E8D64", "grBigBlue_801E93D8",
            "grBigBlue_801E9F3C", "grBigBlue_801EA05C", "grBigBlue_801EAB50", "grBigBlue_801EACE8"}

funcs = []


def F(name, parts=(), args=None, n=None, road_hdr=False, ignore_hdr=False, base=True):
    d = dict(name=name, u=["bigblue"])
    if road_hdr:
        d["bfx"] = list(LANEX)  # header bitfields compared by value (road functions)
    elif ignore_hdr:
        d["bfx"] = list(LANEX) + list(HDR)
    else:
        d["bfs"] = list(HDR)  # header bytes compared raw (the manager / platform byte views)
        d["bfx"] = list(LANEX)
    if name in MGR_VIEW:
        d["bfx"] = list(d["bfx"]) + LANEALL
    if args:
        d["c_args"] = [x[0] for x in args]
        d["rs_args"] = [x[1] for x in args]
    c, r = both(*(([CG] if base else []) + list(parts)))
    d["pre_c"] = c
    d["pre_rs"] = r
    if n:
        d["n"] = n
    funcs.append(d)


G0 = ("&OGO[0]", "g")
TRIV_G = ["grBigBlue_801E5B10", "grBigBlue_801E5B18", "grBigBlue_801E5B1C", "grBigBlue_801E6114", "grBigBlue_801E611C",
          "grBigBlue_801E6120", "grBigBlue_801E61BC", "grBigBlue_801E61FC", "grBigBlue_801E6288", "grBigBlue_801E6290",
          "grBigBlue_801E6294", "grBigBlue_801E6354", "grBigBlue_801E635C", "grBigBlue_801E6360", "grBigBlue_801E687C",
          "grBigBlue_801E6C58", "grBigBlue_801E855C", "grBigBlue_801E93D0", "grBigBlue_801E9F38", "grBigBlue_801EA054",
          "grBigBlue_801EAB4C"]

# ---- stage entry points
F("grBigBlue_801E57BC", args=[("(int) (AI[0] & 1)", "(a.i[0] & 1) as i32")])
F("grBigBlue_801E57C0", parts=[NN_(1)], ignore_hdr=True, n=200)
F("grBigBlue_801E59C8")
F("grBigBlue_801E59CC")
F("grBigBlue_801E59F0")
F("grBigBlue_801E59F8",
  parts=[raw("{ static const int ids_[] = { 0, 1, 2, 3, 4, 31, 32, 33, 34, 35, 36, 37, 40 }; AI[0] = ids_[rnd() % 13]; NN = (AI[0] == 33) ? 1 : (rnd() % 4 != 0); }",
             "{ const IDS_: [u32; 13] = [0, 1, 2, 3, 4, 31, 32, 33, 34, 35, 36, 37, 40]; a.i[0] = IDS_[(rnd() % 13) as usize]; NN = if a.i[0] == 33 { 1 } else { (rnd() % 4 != 0) as i32 }; }")],
  args=[("(s32) AI[0]", "a.i[0] as i32")], ignore_hdr=True, n=400)
for nm in TRIV_G:
    F(nm)
F("grBigBlue_801EFC0C", args=[("(enum_t) AI[0]", "a.i[0] as _")])
F("grBigBlue_801E5AE4")
F("grBigBlue_801E5B20")
F("fn_801E6124", road_hdr=True)
F("grBigBlue_801E613C", road_hdr=True)
F("grBigBlue_801E61C4", parts=[road(0)], road_hdr=True, n=400)
F("grBigBlue_801E6200")
F("grBigBlue_801E6298")
F("grBigBlue_801E6364", parts=[car(0)], n=300)
F("grBigBlue_801E6884", parts=[car(0)], n=400)
F("grBigBlue_801E68B8", parts=[car(0)])
F("grBigBlue_801E6904")
F("grBigBlue_801E6C60", parts=[roles(r32=0), mgr(0, x1_three=True), car("R33"), plat("R36")], n=600)
F("fn_801E8560",
  parts=[mgr(0), raw("{ OCD.x34_flags.b1234 = (rnd() % 3 != 0) ? 1 : rnd() % 4; }", "{ OCD.x34_flags.set_b1234(if rnd() % 3 != 0 { 1 } else { (rnd() % 4) as u8 }); }")],
  args=[("(void*) &OGP[0]", "addr_of_mut!(OGP[0]) as *mut c_void"), ("(int) (AI[0] % 4)", "(a.i[0] % 4) as i32"), ("&OCD", "addr_of_mut!(OCD)"),
        ("(int) (AI[1] % 1500)", "(a.i[1] % 1500) as i32"), ("(mpLib_GroundEnum) (AI[2] % 3)", "a.i[2] % 3"), ("AF[0]", "a.f[0]")],
  n=500)
F("grBigBlue_801E8794",
  parts=[mgr("R32", x1_three=True)],
  args=[("(rnd() % 2) ? (void*) OGP[R32].u.bigblue.xD4[rnd() % 3] : (void*) &OJO[rnd() % NJO]",
         "if rnd() % 2 != 0 { OGP[R32 as usize].u.bigblue.anon_0.anon_0.xD4[(rnd() % 3) as usize] as *mut c_void } else { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut c_void }"),
        ("&AV3[0]", "&mut a.v3[0]"), ("(bool) (AI[0] & 1)", "a.i[0] & 1 != 0"), ("AF[0]", "a.f[0]"), ("AF[1]", "a.f[1]")],
  n=500)
F("grBigBlue_801E8978",
  parts=[],
  args=[("(int) (AI[0] % 3)", "(a.i[0] % 3) as i32"),
        ("(rnd() % 2) ? (void*) 1 : (void*) 0", "if rnd() % 2 != 0 { 1usize as *mut c_void } else { null_mut() }"),
        ("(rnd() % 2) ? (void*) &OJO[rnd() % NJO] : (void*) 0", "if rnd() % 2 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut c_void } else { null_mut() }")])
F("grBigBlue_801E89DC", parts=[mgr("R32")], args=[("(int) (AI[0] % 3)", "(a.i[0] % 3) as i32")])
F("grBigBlue_801E8A1C", parts=[mgr("R32")], args=[("(int) (AI[0] % 3)", "(a.i[0] % 3) as i32")])
F("grBigBlue_801E8B84", parts=[car("R33")],
  args=[("AF[0]", "a.f[0]"), ("AF[1]", "a.f[1]"), ("(float) ((int) (rnd() % 400) - 200)", "((rnd() % 400) as i32 - 200) as f32"),
        ("(float) ((int) (rnd() % 400) - 200)", "((rnd() % 400) as i32 - 200) as f32")], n=500)
F("grBigBlue_801E8D04", parts=[car("R33")], n=300)
F("grBigBlue_801E8D64", n=300)
F("grBigBlue_801E93D8", parts=[roles(r32=1, r33=2), plat(0), mgr("R32", x1_three=True), car("R33")], n=800)
F("grBigBlue_801E9F3C")
F("grBigBlue_801EA05C", parts=[roles(r32=1, r33=2, r36=0), plat(0), mgr("R32", x1_three=True), car("R33")], n=800)
F("grBigBlue_801EAB50", parts=[mgr("R32", x1_three=True)],
  args=[("&AV3[0]", "&mut a.v3[0]"), ("(s32) (AI[0] & 1)", "(a.i[0] & 1) as i32"), ("AF[0]", "a.f[0]"), ("AF[1]", "a.f[1]")], n=400)
F("grBigBlue_801EACE8", parts=[mgr("R32", x1_three=True), plat("R36")],
  args=[("(rnd() % 2) ? (HSD_JObj*) OGP[R32].u.bigblue.xD4[rnd() % 3] : (HSD_JObj*) &OJO[rnd() % NJO]",
         "if rnd() % 2 != 0 { OGP[R32 as usize].u.bigblue.anon_0.anon_0.xD4[(rnd() % 3) as usize] } else { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) }"),
        ("&AV3[0]", "&mut a.v3[0]"), ("&APF[0]", "&mut a.pf[0]"), ("(float) (rnd() % 120)", "(rnd() % 120) as f32"), ("(float) (rnd() % 60)", "(rnd() % 60) as f32")],
  n=600)
F("grBigBlue_801EB004", road_hdr=True, n=300)
F("grBigBlue_801EB4AC", parts=[road(0)], road_hdr=True, n=500)
F("grBigBlue_801EBAF8", parts=[road(0)], road_hdr=True, n=800)
F("grBigBlue_801EC58C",
  args=[("&AV3[0]", "&mut a.v3[0]"), ("(rnd() % 2) ? &AV3[1] : (Vec3*) 0", "if rnd() % 2 != 0 { &mut a.v3[1] as *mut Vec3 } else { null_mut() }"), ("AF[0]", "a.f[0]")])
F("grBigBlue_801EC6C0", parts=[car(0)], n=300)
F("grBigBlue_801ECB50", parts=[car(0)], n=800)
F("grBigBlue_801ED694", parts=[car(0), roles(r34=3)],
  args=[G0, ("(s32) (AI[0] % 4)", "(a.i[0] % 4) as i32")], n=1200)
F("grBigBlue_801EDF44", parts=[car(0)], args=[G0, ("(s32) (AI[0] % 4)", "(a.i[0] % 4) as i32")], n=600)
F("grBigBlue_801EE398", parts=[car(0)], args=[G0, ("(s32) (AI[0] % 4)", "(a.i[0] % 4) as i32"), ("(s32) (AI[1] % 12)", "(a.i[1] % 12) as i32")], n=800)
F("grBigBlue_801EEF00", parts=[car(0)], args=[G0, ("(s32) (AI[0] % 4)", "(a.i[0] % 4) as i32")], n=800)
F("grBigBlue_801EF424", parts=[car(0), roles(r34=3)], n=500)
F("fn_801EF60C",
  parts=[car(0), raw("{ OCD.x34_flags.b1234 = (rnd() % 3 != 0) ? 1 : rnd() % 4; }", "{ OCD.x34_flags.set_b1234(if rnd() % 3 != 0 { 1 } else { (rnd() % 4) as u8 }); }")],
  args=[("(void*) &OGP[0]", "addr_of_mut!(OGP[0]) as *mut c_void"),
        ("(int) lbl_803E2DFC[AI[0] % 30]", "t::lbl_803E2DFC[(a.i[0] % 30) as usize] as i32"),
        ("&OCD", "addr_of_mut!(OCD)"), ("(int) (AI[1] % 1500)", "(a.i[1] % 1500) as i32"),
        ("(mpLib_GroundEnum) ((rnd() % 4 != 0) ? 1 : 2)", "if rnd() % 4 != 0 { 1 } else { 2 }"), ("AF[0]", "a.f[0]")],
  n=500)
F("grBigBlue_801EF7D8", parts=[raw("if (rnd() % 3 == 0) OGO[R34].user_data = 0;", "if rnd() % 3 == 0 { OGO[R34 as usize].user_data = null_mut(); }")],
  args=[("&AV3[0]", "&mut a.v3[0]")])
F("grBigBlue_801EF844",
  parts=[raw("stage_info.grkind = (rnd() % 3) ? Gr_Kind_BigBlue : rnd() % 30;",
             "(*(addr_of_mut!(G_stage_info) as *mut StageInfo)).grkind = if rnd() % 3 != 0 { Gr_Kind_BigBlue } else { rnd() % 30 };")],
  args=[("(enum_t) ((AI[0] % 4 == 0) ? -1 : (int) (AI[1] % 70))", "if a.i[0] % 4 == 0 { -1 } else { (a.i[1] % 70) as i32 }")])
F("fn_801EFB9C", args=[G0, ("(int) AI[0]", "a.i[0] as i32")])
F("grBigBlue_801EFC14", args=[("&AV3[0]", "&mut a.v3[0]"), ("(int) AI[0]", "a.i[0] as i32"), ("&OJO[0]", "addr_of_mut!(OJO[0])")])

ALL_FNS = [d["name"] for d in funcs]

C_ROLE_MAP = "(a0 == 32 ? (void*) &OGO[R32] : a0 == 33 ? (void*) &OGO[R33] : a0 == 34 ? (void*) &OGO[R34] : a0 == 36 ? (void*) &OGO[R36] : ((rnd() % 3) ? (void*) &OGO[5] : (void*) 0))"
R_ROLE_MAP = ("(if a0 == 32 { addr_of_mut!(OGO[R32 as usize]) as *mut c_void } else if a0 == 33 { addr_of_mut!(OGO[R33 as usize]) as *mut c_void } "
              "else if a0 == 34 { addr_of_mut!(OGO[R34 as usize]) as *mut c_void } else if a0 == 36 { addr_of_mut!(OGO[R36 as usize]) as *mut c_void } "
              "else if rnd() % 3 != 0 { addr_of_mut!(OGO[5]) as *mut c_void } else { null_mut() })")

TE_IDX = [(4, 6, 5), (19, 21, 20), (7, 9, 8), (10, 12, 11), (44, 46, 45), (1, 3, 2), (32, 34, 33), (35, 37, 36), (38, 40, 39), (41, 43, 42), (29, 31, 30), (13, 18, 17)]

SPEC = dict(
    src="src/melee/gr/grbigblue.c",
    rust_mod="grbigblue",
    ext_mods=["grinl"],
    u=["bigblue"],
    bf_skip=["StageCallbacks"],
    bfx=list(LANEX),
    globals=[dict(c="stage_info", ty="StageInfo")],
    variadic={"OSReport": "pii"},
    c_patches=[
        # 30 pointers (the C allocates the GC size 120)
        ("gp->u.bigblue.car.collision_jobjs = HSD_MemAlloc(120);", "gp->u.bigblue.car.collision_jobjs = HSD_MemAlloc(30 * sizeof(HSD_JObj*));"),
        # the 4-byte slot at GC +0xE0 (`pad_3` natively; `manager.event_extra` would cover data[0])
        ("gp->u.bigblue.manager.event_extra = extra;", "*(u32*) gp->u.bigblue.pad_3 = (u32) (unsigned long long) extra;"),
        # grBigBlue_801E8D64
        ("*(s32*) ((u8*) other_gp + 0xCC) = 1;", "*(s32*) &other_gp->u.bigblue.xCC = 1;"),
        ("*(f32*) ((u8*) gp + 0xD8) = 0.0F;", "gp->u.bigblue.platform.xD8 = 0.0F;"),
        ("*(s32*) ((u8*) gp + 0xC8) = (s32) (yakumono_param->xD8);", "gp->u.bigblue.platform.xC8_timer = (s32) (yakumono_param->xD8);"),
        ("*(u8*) ((u8*) gp + 0xC4) = 2;", "gp->u.bigblue.x0 = 2;"),
        # grBigBlue_801E9F3C / 801EA05C
        ("((u8*) gp)[0xC4] = 0;", "gp->u.bigblue.x0 = 0;"),
        ("*(s32*) ((u8*) GET_GROUND(Ground_GetMapGObj(32)) + 0xD0)", "*(s32*) &GET_GROUND(Ground_GetMapGObj(32))->u.bigblue.xD0", 2),
        # grBigBlue_801EACE8
        ("(int) ((u8*) gp)[0xE5 + i * 0x54] != 3", "(int) gp->u.bigblue.data[i].x1 != 3"),
    ],
    c_regex=[
        # grBigBlue_801E8B84: the lane walk
        (r"(?s)f32 grBigBlue_801E8B84\(f32 right, f32 left, f32 bottom, f32 top\)\n\{.*?\n\}\n",
         """f32 grBigBlue_801E8B84(f32 right, f32 left, f32 bottom, f32 top)
{
    Ground* gp = Ground_GetMapGObj(33)->user_data;
    f32 result = -3.4028235e38f;
    int k;
    for (k = 0; k < 4; k++) {
        if ((unsigned) gp->u.bigblue.car.lanes[k].state != 1U) {
            if (gp->u.bigblue.car.lanes[k].pos.y < right && gp->u.bigblue.car.lanes[k].pos.y > left) {
                if (gp->u.bigblue.car.lanes[k].pos.x < top && gp->u.bigblue.car.lanes[k].pos.x > bottom) {
                    if (gp->u.bigblue.car.lanes[k].pos.y > result) {
                        result = gp->u.bigblue.car.lanes[k].pos.y;
                    }
                }
            }
        }
    }
    return result;
}
""", 1),
    ],
    c_fn_regex=[
        # grBigBlue_801E93D8: the platform view through `bp`
        ("grBigBlue_801E93D8", r"bp\[0xC4\]", "gp->u.bigblue.x0"),
        ("grBigBlue_801E93D8", r"\*\(f32\*\) \(bp \+ 0xD0\)", "(*(f32*) &gp->u.bigblue.platform.xD0_timer)"),
        ("grBigBlue_801E93D8", r"\*\(f32\*\) \(bp \+ 0xD8\)", "gp->u.bigblue.platform.xD8"),
        ("grBigBlue_801E93D8", r"\*\(s32\*\) \(bp \+ 0xC8\)", "gp->u.bigblue.platform.xC8_timer"),
        ("grBigBlue_801E93D8", r"\*\(f32\*\) \(bp \+ 0xCC\)", "(*(f32*) &gp->u.bigblue.platform.xCC_timer)"),
        ("grBigBlue_801E93D8", r"\(\(grBb_GroundStateFlag\*\) Ground_GetMapGObj\(32\)->user_data\)\s*->xCC",
         "(*(s32*) &((Ground*) Ground_GetMapGObj(32)->user_data)->u.bigblue.xCC)", 1),
        ("grBigBlue_801E93D8", r"grBb_GroundStateFlag\* manager =\s*Ground_GetMapGObj\(32\)->user_data;\s*manager->xCC = 0;",
         "Ground* manager = Ground_GetMapGObj(32)->user_data; *(s32*) &manager->u.bigblue.xCC = 0;", 1),
        ("grBigBlue_801E93D8", r"\(s8\) p\[0xE5\]", "(s8) ((Ground*) mgp)->u.bigblue.data[idx].x1", 1),
        ("grBigBlue_801E93D8", r"slot\[0xE5\] = 2;", "((Ground*) mgp)->u.bigblue.data[idx].x1 = 2;", 1),
        ("grBigBlue_801E93D8", r"\*\(s32\*\) \(slot \+ 0xE8\) = 0;", "((Ground*) mgp)->u.bigblue.data[idx].x4 = 0;", 1),
        ("grBigBlue_801E93D8", r"\*\(u32\*\) \(mgp2 \+ 0xE0\) = \(u32\) jobj;",
         "*(u32*) ((Ground*) mgp2)->u.bigblue.pad_3 = (u32) (unsigned long long) jobj;", 1),
        ("grBigBlue_801E93D8", r"\*\(s32\*\) \(mgp2 \+ 0xC8\) = 1;", "*(s32*) &((Ground*) mgp2)->u.bigblue.xC8 = 1;", 1),
        # grBigBlue_801ED694: grBigBlue_CarPhysics overlay and the raw lane bytes
        ("grBigBlue_801ED694", r"grBigBlue_CarPhysics\* gp = gobj->user_data;", "Ground* gp = gobj->user_data;", 1),
        ("grBigBlue_801ED694", r"gp->data\.jobjs", "gp->u.bigblue.car.collision_jobjs"),
        ("grBigBlue_801ED694", r"gp->data\.lanes", "gp->u.bigblue.car.lanes"),
        ("grBigBlue_801ED694", r"u16 hw = \*\(u16\*\) lane_flags;\s*jobj = gp->u\.bigblue\.car\.collision_jobjs\[\(hw >> 4\) & 0x1F\];",
         "jobj = gp->u.bigblue.car.collision_jobjs[gp->u.bigblue.car.lanes[lane].collision_slot];", 1),
        ("grBigBlue_801ED694", r"\(lane_flags\[0\] >> 2\) & 0x3F", "gp->u.bigblue.car.lanes[lane].state"),
        ("grBigBlue_801ED694", r"\(lane_flags\[0\] >> 1\) & 1", "gp->u.bigblue.car.lanes[lane].direction"),
        ("grBigBlue_801ED694", r"\(\(grBb_ByteBits\*\) lane_flags\)->b6 = 0;", "gp->u.bigblue.car.lanes[lane].direction = 0;", 1),
        ("grBigBlue_801ED694", r"\(\(grBb_ByteBits\*\) lane_flags\)->b6 = 1;", "gp->u.bigblue.car.lanes[lane].direction = 1;", 1),
        ("grBigBlue_801ED694", r"iter = gp->raw", "iter = 0", 1),
        ("grBigBlue_801ED694", r"iter \+= 0x40", "iter += 0", 1),
        ("grBigBlue_801ED694", r"\(iter\[0xD4\] >> 2\) & 0x3F", "gp->u.bigblue.car.lanes[idx].state", 1),
        ("grBigBlue_801ED694", r"\*\(f32\*\) \(iter \+ 0xE0\)", "gp->u.bigblue.car.lanes[idx].pos.x", 1),
        ("grBigBlue_801ED694", r"\*\(f32\*\) \(gp2 \+ 0xCC\) - \*\(f32\*\) \(gp2 \+ 0xD8\)",
         "((Ground*) gp2)->u.bigblue.road.position.y - ((Ground*) gp2)->u.bigblue.road.previous_position.y", 1),
        # grBigBlue_801ECB50: the lane states at gp + 0x40 steps
        ("grBigBlue_801ECB50", r"u8\* bp = \(u8\*\) gp;", "u8* bp = (u8*) gp; int pl_ = 0;"),
        ("grBigBlue_801ECB50", r"p = bp \+ 0x40;", "p = bp + 0x40; pl_ = 1;"),
        ("grBigBlue_801ECB50", r"p \+= 0x40;", "p += 0x40; pl_ += 1;"),
        ("grBigBlue_801ECB50", r"\(bp\[0xD4\] >> 2\) & 0x3F", "gp->u.bigblue.car.lanes[0].state"),
        ("grBigBlue_801ECB50", r"\(p\[0xD4\] >> 2\) & 0x3F", "gp->u.bigblue.car.lanes[pl_].state"),
        ("grBigBlue_801ECB50", r"\(p\[0x114\] >> 2\) & 0x3F", "gp->u.bigblue.car.lanes[pl_ + 1].state"),
        # grBigBlue_801EEF00: grBb_CarGround overlay
        ("grBigBlue_801EEF00", r"grBb_CarGround\* gp;", "Ground* gp;", 1),
        ("grBigBlue_801EEF00", r"\(\*\(volatile u8\*\) \(gp->bytes \+ offset \+ 0xD4\) >> 2\) & 0x3F", "gp->u.bigblue.car.lanes[index].state", 1),
        ("grBigBlue_801EEF00", r"gp->typed\.cars\[index\]", "gp->u.bigblue.car.lanes[index]"),
        ("grBigBlue_801EEF00", r"u8\* car = gp->bytes \+ offset;\s*f32\* alpha = \(f32\*\) \(car \+ 0xEC\);\s*\*\(f32\*\) \(car \+ 0x100\) = 0\.0F;",
         "f32* alpha = &gp->u.bigblue.car.lanes[index].alpha; gp->u.bigblue.car.lanes[index].accel = 0.0F;", 1),
        # grBigBlue_801EF424
        ("grBigBlue_801EF424", r"\(car_i\[0xD4\] >> 2\) & 0x3F", "gp->u.bigblue.car.lanes[i].state", 1),
        ("grBigBlue_801EF424", r"\(car_j\[0xD4\] >> 2\) & 0x3F", "gp->u.bigblue.car.lanes[j].state", 1),
        ("grBigBlue_801EF424", r"\*\(f32\*\) \(car_i \+ 0xE0\)", "gp->u.bigblue.car.lanes[i].pos.x"),
        ("grBigBlue_801EF424", r"\*\(f32\*\) \(car_j \+ 0xE0\)", "gp->u.bigblue.car.lanes[j].pos.x"),
        ("grBigBlue_801EF424", r"\*\(f32\*\) \(car_i \+ 0xDC\)", "gp->u.bigblue.car.lanes[i].delta"),
        ("grBigBlue_801EF424", r"\*\(f32\*\) \(car_j \+ 0xDC\)", "gp->u.bigblue.car.lanes[j].delta"),
        # fn_801EF60C
        ("fn_801EF60C", r"hw = \*\(u16\*\) \(p \+ 0xD4\);", "hw = (u16) (gp->u.bigblue.car.lanes[i].collision_slot << 4);", 1),
        ("fn_801EF60C", r"\*\(f32\*\) \(p \+ 0xF4\)", "gp->u.bigblue.car.lanes[i].gravity", 1),
    ],
    data_c="""static int NN, R32 = 1, R33 = 2, R34 = 3, R36 = 4;
static HSD_JObj* OJAR[32];
static u8 ORK[32];
static HSD_JObj OJC[31];
static HSD_JObj OJD[30];
static CollData OCD;
static const s16 TE_IDX[12][3] = { %s };""" % ", ".join("{ %d, %d, %d }" % t for t in TE_IDX),
    use_rs="""static mut NN: i32 = 0;
static mut R32: i32 = 1;
static mut R33: i32 = 2;
static mut R34: i32 = 3;
static mut R36: i32 = 4;
static mut OJAR: [*mut HSD_JObj; 32] = [core::ptr::null_mut(); 32];
static mut ORK: [u8; 32] = [0; 32];
static mut OJC: [HSD_JObj; 31] = unsafe { core::mem::zeroed() };
static mut OJD: [HSD_JObj; 30] = unsafe { core::mem::zeroed() };
static mut OCD: CollData = unsafe { core::mem::zeroed() };
const TE_IDX: [[i16; 3]; 12] = [%s];
use core::ptr::null_mut;""" % ", ".join("[%d, %d, %d]" % t for t in TE_IDX),
    extra_pools=["grBb_YakumonoParam"],
    statics=[
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grBb_StageCallbacks", rs="t::grBb_StageCallbacks", ty="StageCallbacks", n=41),
        dict(c="grBb_StageData", rs="t::grBb_StageData", ty="StageData"),
        dict(c="grBb_803E2938", rs="t::grBb_803E2938", ty="GrJoint", n=28),
        dict(c="grBb_803E2D84", rs="t::grBb_803E2D84", kind="s", n=30, const=True),
        dict(c="grBb_803E2DC0", rs="t::grBb_803E2DC0", kind="s", n=30, const=True),
        dict(c="lbl_803E2DFC", rs="t::lbl_803E2DFC", kind="s", n=30, const=True),
        dict(c="lbl_803E3010", rs="t::lbl_803E3010", kind="u", n=4, const=True),
        dict(c="grBb_803B8108", rs="t::grBb_803B8108", ty="Vec3"),
        dict(c="grBb_803B8114", rs="t::grBb_803B8114", ty="Vec3"),
        dict(c="grBb_803B8120", rs="t::grBb_803B8120", kind="i", n=5, const=True),
        dict(c="grBb_803B8134", rs="t::grBb_803B8134", kind="i", n=32, const=True),
        dict(c="grBb_TrackEntries", rs="t::grBb_TrackEntries", ty="grBb_TrackEntry", n=12, fill=True),
        dict(c="OJAR", rs="OJAR", kind="p", n=32),
        dict(c="ORK", rs="ORK", kind="C", n=32),
        dict(c="OJC", rs="OJC", ty="HSD_JObj", n=31, fill=True),
        dict(c="OJD", rs="OJD", ty="HSD_JObj", n=30, fill=True),
        dict(c="OCD", rs="OCD", ty="CollData", fill=True),
    ],
    links_c="""    NN = 1; R32 = 1; R33 = 2; R34 = 3; R36 = 4; yakumono_param = (void*) OP_grBb_YakumonoParam[0];
    OGO[5].hsd_obj = &OJC[0]; OJC[0].child = &OJC[1];
    for (i = 1; i <= 30; i++) { OJC[i].next = (i < 30) ? &OJC[i + 1] : 0; OJC[i].child = &OJD[i - 1]; }""",
    links_rs="""    NN = 1; R32 = 1; R33 = 2; R34 = 3; R36 = 4; t::yakumono_param = addr_of_mut!(OP_grBb_YakumonoParam[0]) as *mut _;
    OGO[5].hsd_obj = addr_of_mut!(OJC[0]) as *mut _; OJC[0].child = addr_of_mut!(OJC[1]);
    for i in 1..=30usize { OJC[i].next = if i < 30 { addr_of_mut!(OJC[i + 1]) } else { null_mut() }; OJC[i].child = addr_of_mut!(OJD[i - 1]); }""",
    hooks_rs="""    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);""",
    outs={"lb_8000B1CC": [2], "lbVector_Add": [0], "lbVector_Sub": [0], "lbVector_Diff": [2], "lbVector_ApplyEulerRotation": [0],
          "mpCheckMultiple": [4, 7]},
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "HSD_Randf": ("(float) (rnd() % 1024) / 1024.0f", "((rnd() % 1024) as f32 / 1024.0)"),
        "Ground_801C0498": ("(float) (4 + rnd() % 4) / 4.0f", "((4 + rnd() % 4) as f32 / 4.0)"),
        "Ground_801C3FA4": ("(void*) &OJO[rnd() % NJO]", "addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _"),
        "Ground_801C2CF4": ("(void*) &OJO[rnd() % NJO]", "addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _"),
        "Ground_GetMapGObj": (C_ROLE_MAP, R_ROLE_MAP),
        "Ground_GetStageGObj": ("((NN || rnd() % 8) ? (a0 == 4 ? (void*) &OGO[5] : (void*) &OGO[rnd() % NGO]) : (void*) 0)",
                                "(if NN != 0 || rnd() % 8 != 0 { if a0 == 4 { addr_of_mut!(OGO[5]) as *mut c_void } else { addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut c_void } } else { null_mut() })"),
        "Ground_GetYakumonoParam": ("(void*) OP_grBb_YakumonoParam[0]", "addr_of_mut!(OP_grBb_YakumonoParam[0]) as *mut c_void"),
        "HSD_MemAlloc": ("(a0 == 240 ? (void*) OJAR : (void*) ORK)", "(if a0 as i64 == 240 { addr_of_mut!(OJAR) as *mut c_void } else { addr_of_mut!(ORK) as *mut c_void })"),
        "Stage_GetBlastZoneRightOffset": ("(float) (100 + rnd() % 200)", "((100 + rnd() % 200) as f32)"),
        "Stage_GetBlastZoneLeftOffset": ("-(float) (100 + rnd() % 200)", "(-((100 + rnd() % 200) as f32))"),
        "Stage_GetCamBoundsRightOffset": ("(float) (50 + rnd() % 150)", "((50 + rnd() % 150) as f32)"),
        "Stage_GetCamBoundsLeftOffset": ("-(float) (50 + rnd() % 150)", "(-((50 + rnd() % 150) as f32))"),
        "Stage_GetCamBoundsTopOffset": ("(float) (50 + rnd() % 200)", "((50 + rnd() % 200) as f32)"),
        "Stage_GetCamBoundsBottomOffset": ("-(float) (50 + rnd() % 200)", "(-((50 + rnd() % 200) as f32))"),
        "mpCheckMultiple": ("(int) (rnd() % 3 == 0)", "((rnd() % 3 == 0) as i32)"),
        "mpJointFromLine": ("(int) (rnd() % 70)", "((rnd() % 70) as i32)"),
        "it_8026D324": ("(int) (rnd() % 2)", "((rnd() % 2) as i32)"),
        "it_8026BE84": ("(void*) (unsigned long long) (1 + rnd() % 1000)", "((1 + rnd() % 1000) as usize as *mut c_void)"),
        "Camera_8003010C": ("(int) (rnd() % 2)", "((rnd() % 2) as i32)"),
        "gm_GetDbPauseFlag": ("(int) (rnd() % 2)", "((rnd() % 2) as i32)"),
    },
    fnvals={**{n: f"t::{n}" for n in sorted(set(ALL_FNS) | {"grBigBlue_801E57BC"}) if n.startswith(("grBigBlue_", "fn_"))},
            "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0_cb"},
    funcs=funcs,
)
