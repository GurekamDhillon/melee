"""Per-TU oracle specs for gen_tu.py / run_tu.py (melee_items::tu). Keys as in melee_world/gen/mtu/specs.py, plus
`pools` (record types the generated fill functions point their pointer fields into, see mk_driver.py)."""
from mk_driver import WORLD_TYPES, POOL_TYPES, COMMON_GLOBALS

COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
}
SPECS = {}


def add(stem, src, types=(), **kw):
    kw["manual_data"] = list(kw.get("manual_data", [])) + list(COMMON_GLOBALS)
    SPECS[stem] = dict(src=src, types=list(WORLD_TYPES) + list(types), pools=POOL_TYPES, common_ret=COMMON_RET, **kw)


add("ithitbox", "src/melee/it/ithitbox.c")

_SPAWN_C = ("T((u32) a0->kind); T((u32) a0->x3C_damage); T((u32) a0->x40); T(B(a0->vel.x)); T(B(a0->vel.y)); T(B(a0->vel.z)); "
            "T(B(a0->prev_pos.x)); T(B(a0->prev_pos.y)); T(B(a0->prev_pos.z)); T(B(a0->pos.x)); T(B(a0->pos.y)); T(B(a0->pos.z)); "
            "T(B(a0->facing_dir)); T(PI(a0->x0_parent_gobj)); T(PI(a0->x4_parent_gobj2)); T((u32) a0->x44_flag.b0);")
_SPAWN_R = ("t((*a0).kind as u32); t((*a0).x3C_damage as u32); t((*a0).x40 as u32); t((*a0).vel.x.to_bits()); t((*a0).vel.y.to_bits()); "
            "t((*a0).vel.z.to_bits()); t((*a0).prev_pos.x.to_bits()); t((*a0).prev_pos.y.to_bits()); t((*a0).prev_pos.z.to_bits()); "
            "t((*a0).pos.x.to_bits()); t((*a0).pos.y.to_bits()); t((*a0).pos.z.to_bits()); t((*a0).facing_dir.to_bits()); "
            "t(pi((*a0).x0_parent_gobj)); t(pi((*a0).x4_parent_gobj2)); t((*a0).x44_flag.anon_0.b0() as u32);")
add("itdrop", "src/melee/it/itdrop.c",
    ret={"Item_80268B18": ("pick_igo()", "pick_igo()")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R),
             "ftLib_800874CC": ("{ int ii; for (ii = 0; ii < a2 && ii < 8; ii++) T(((int*) a1)[ii]); }",
                                "for ii in 0..(a2.min(8).max(0)) { t(*(a1 as *const i32).add(ii as usize) as u32); }")},
)
