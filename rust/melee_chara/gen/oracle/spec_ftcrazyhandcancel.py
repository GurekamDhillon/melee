SPEC = dict(
    src="src/melee/ft/kinds/ftCrazyHand/ftcrazyhandcancel.c",
    rust_mod="ftcrazyhandcancel",
    ext_mods=["ftinl"],
    legacy_stubs=True,
    attr_types=["ftCrazyHand_DatAttrs"],
    provides=["__assert"],
    data_c="""
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
""",
    bitfields=[("invisible", "invisible")],
    funcs=[dict(name="ftCh_Init_8015A030", bf=["invisible"])],
)
