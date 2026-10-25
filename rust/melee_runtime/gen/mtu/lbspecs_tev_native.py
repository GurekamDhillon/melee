"""tev.c spec: colour channel lists (cached ambient / material / control state), TEV descriptor lists, the stage counter and the TEV register flush.
The colour word compares read GXColor as a 32-bit word: the oracle patches the masks for the native byte order (console: big-endian word)."""
from lbspecs import add

POOLS = [("HSD_Chan", "CH", 3), ("HSD_TevDesc", "TD", 3)]
OWN = ["prev_ch", "TevReg", "render_alloc_data", "tevreg_alloc_data", "chan_alloc_data", "current_tev", "num_tex_gens", "prev_num_chans", "prev_mat_invalid",
       "prev_amb_invalid", "committed_tev_stages"]
add("tev_native", "src/sysdolphin/baselib/tev.c", dict(
    pub_only=True, pools=POOLS, ncases=400,
    setup=("{ int q; for (q = 0; q < 3; q++) { CH[q].chan = ((fv() % 8) == 7) ? 0xFF : fv() % 6; CH[q].next = (q < 2 && (fv() & 1)) ? &CH[q + 1] : 0; CH[q].enable = fv() & 1; CH[q].amb_src = fv() & 1; CH[q].mat_src = fv() & 1;"
           " TD[q].stage = fv() % 16; TD[q].next = (q < 2 && (fv() & 1)) ? &TD[q + 1] : 0; TD[q].flags = fv() & 1; TD[q].coord = fv() % 8; TD[q].map = fv() % 8; TD[q].color = fv() % 8; }"
           " current_tev = fv() % 16; for (q = 0; q < 2; q++) { prev_mat_invalid[q] = fv() & 1; prev_amb_invalid[q] = fv() & 1; } for (q = 0; q < 4; q++) TevReg[q].c = fv() & 1; }",
           "{ for q in 0..3usize { CH[q].chan = if fv() % 8 == 7 { 0xFF } else { fv() % 6 }; CH[q].next = if q < 2 && fv() & 1 != 0 { &raw mut CH[q + 1] } else { null_mut() }; CH[q].enable = (fv() & 1) as _; CH[q].amb_src = (fv() & 1) as _; CH[q].mat_src = (fv() & 1) as _;"
           " TD[q].stage = fv() % 16; TD[q].next = if q < 2 && fv() & 1 != 0 { &raw mut TD[q + 1] } else { null_mut() }; TD[q].flags = fv() & 1; TD[q].coord = fv() % 8; TD[q].map = fv() % 8; TD[q].color = fv() % 8; }"
           " m::current_tev = (fv() % 16) as i32; for q in 0..2usize { m::prev_mat_invalid[q] = (fv() & 1) as i32; m::prev_amb_invalid[q] = (fv() & 1) as i32; } for q in 0..4usize { m::TevReg[q].c = (fv() & 1) as i32; } }"),
    pregs=("", "melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());")),
    patch=[("return ((*d0 ^ *d1) & 0xFFFFFF00) != 0;", "return ((*d0 ^ *d1) & 0x00FFFFFF) != 0;"), ("*d = (*d & 0xff) | (*s & 0xffffff00);", "*d = (*d & 0xFF000000) | (*s & 0x00FFFFFF);")],
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=["prev_ch", "TevReg", "num_tex_gens", "prev_num_chans", "committed_tev_stages"], own_hash=OWN,
    skip_stubs=["memcpy", "memset"])
