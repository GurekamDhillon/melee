SPEC = dict(
    src="src/melee/ft/kinds/ftZakoBoy/ftboy.c",
    rust_mod="ftboy",
    ext_mods=["ftinl"],
    legacy_stubs=True,
    attr_types=["ftZakoboyAttributes"],
    funcs=["ftBo_Init_OnDeath", "ftBo_Init_OnItemInvisible", "ftBo_Init_OnItemVisible", "ftBo_Init_OnItemPickup", "ftBo_Init_OnItemDrop", "ftBo_Init_OnLoad", "ftBo_Init_LoadSpecialAttrs"],
)
