"""Oracle specs of the `if` TUs (shard 625-632); keys as in specs.py plus the pool-driver keys of
if_drv.py (pools, rules, statics, ops, setup_c / setup_rs, defaults)."""
COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
    "HSD_Randf": ("(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
}

SPECS = {}

SPECS["ifhazard"] = dict(
    src="src/melee/if/ifhazard.c",
    pools={"HSD_GObj": 4, "HSD_JObj": 4, "DynamicModelDesc": 2},
    rules={"HSD_GObj.hsd_obj": ("ptr", "HSD_JObj", False)},
    statics=[dict(c="un_804D6D80", kind="ptr", fill="p:HSD_GObj"), dict(c="un_804D6D84", kind="int", fill="r:0:3"),
             dict(c="un_804D6D88", kind="int", fill="r:0:2"), dict(c="un_804D6D8C", kind="int", fill="r:0:2"),
             dict(c="HSD_GObj_JObjKind", ext=True, kind="int", fill="k:20")],
    skip_data=["result"],
    ret={"GObj_Create": ("pick_HSD_GObj(0)", "pick_HSD_GObj(0)"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj(0)", "pick_HSD_JObj(0)")},
    null_ret_ok=["HSD_GObj_SetupProc"],
    ops=[dict(name="un_802FD604"), "un_802FD65C", "un_802FD668", "un_802FD674", "fn_802FD680", "fn_802FD6CC",
         "un_802FD704", "un_802FD708", "un_802FD8A0", "un_802FD8C4", "un_802FD8E8", "un_802FD90C"],
)

for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
