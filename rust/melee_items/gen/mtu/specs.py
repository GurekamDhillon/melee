"""Per-TU oracle specs for gen_tu.py / run_tu.py (melee_items::tu). Keys as in melee_world/gen/mtu/specs.py, plus
`pools` (record types the generated fill functions point their pointer fields into, see mk_driver.py)."""
from mk_driver import WORLD_TYPES, POOL_TYPES

COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
}
SPECS = {}


def add(stem, src, types=(), **kw):
    SPECS[stem] = dict(src=src, types=list(WORLD_TYPES) + list(types), pools=POOL_TYPES, common_ret=COMMON_RET, **kw)


add("ithitbox", "src/melee/it/ithitbox.c")
