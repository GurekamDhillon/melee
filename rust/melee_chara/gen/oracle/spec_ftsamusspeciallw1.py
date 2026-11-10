SPEC = dict(
    src="src/melee/ft/kinds/ftSamus/ftsamusspeciallw1.c",
    rust_mod="ftsamusspeciallw1",
    ext_mods=["ftinl"],
    legacy_stubs=True,
    attr_types=["ftSs_DatAttrs"],
    provides=["__assert"],
    data_c="""
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
""",
    outs={"lb_8000B1CC": [2]},
    bitfields=[("x2210.x0.throw_flags_b0", "x2210.x0.throw_flags_b0")],
    funcs=[dict(name="ftSs_SpecialLw_8012ADF0", bf=["x2210.x0.throw_flags_b0"])],
)
