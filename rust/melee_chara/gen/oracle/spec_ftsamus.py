SPEC = dict(
    src="src/melee/ft/kinds/ftSamus/ftsamus.c",
    rust_mod="ftsamus",
    legacy_stubs=True,
    attr_types=["ftSs_DatAttrs"],
    # the stub generator would redeclare this with the probe's spelling of the enum parameter
    pre_c="#define HSD_ForeachAnim HSD_ForeachAnim_not_under_test",
    funcs=["ftSs_Init_80128428"],
)
