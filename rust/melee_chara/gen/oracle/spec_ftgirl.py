SPEC = dict(
    src="src/melee/ft/kinds/ftZakoGirl/ftgirl.c",
    rust_mod="ftgirl",
    ext_mods=["ftinl"],
    legacy_stubs=True,
    attr_types=["s32"],
    funcs=["ftGl_Init_OnDeath", "ftGl_Init_OnItemInvisible", "ftGl_Init_OnItemVisible", "ftGl_Init_OnLoad", "ftGl_Init_OnItemPickup", "ftGl_Init_OnItemDrop", "ftGl_Init_LoadSpecialAttrs"],
)
