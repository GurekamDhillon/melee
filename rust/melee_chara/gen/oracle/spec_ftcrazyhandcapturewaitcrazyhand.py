SPEC = dict(
    src="src/melee/ft/kinds/ftCrazyHand/ftcrazyhandcapturewaitcrazyhand.c",
    rust_mod="ftcrazyhandcapturewaitcrazyhand",
    legacy_stubs=True,
    provides=["__assert"],
    data_c="""
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
""",
    bitfields=[("invisible", "invisible")],
    funcs=[dict(name="ftCh_GrabUnk1_8015B800", bf=["invisible"])],
)
