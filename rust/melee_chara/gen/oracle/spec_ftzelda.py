SPEC = dict(
    src="src/melee/ft/kinds/ftZelda/ftzelda.c",
    rust_mod="ftzelda",
    ext_mods=["ftinl"],
    legacy_stubs=True,
    attr_types=["ftZelda_DatAttrs"],
    includes=["ft/types.h", "ft/forward.h", "ft/fighter.h", "ft/inlines.h", "ft/ftcommon.h",
              "sysdolphin/baselib/gobj.h", "sysdolphin/baselib/jobj.h"],
    funcs=["ftZd_Init_OnDeath", "ftZd_Init_OnItemInvisible", "ftZd_Init_OnItemVisible", "ftZd_Init_OnKnockbackEnter",
           "ftZd_Init_OnKnockbackExit", "ftZd_Init_OnLoad", "ftZd_Init_801393AC", "ftZd_Init_OnItemPickup", "ftZd_Init_OnItemDrop",
           "ftZd_Init_LoadSpecialAttrs", "ftZd_Init_801395C8"],
)
