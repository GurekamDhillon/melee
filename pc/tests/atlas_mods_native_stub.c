/* The tests that include gw_script_ui.inc but have no mods folder link this instead of gw_ui_mods_native.c (which reads gw_Mods_*): an empty source. Only
 * atlas_mods_door_test.c runs the real accessors, over a fake folder. */
#include <string.h>
#include "../platform/gw_ui_mods_native.h"

void at_mods_native_src(AtModsSrc *out, const AtModsHooks *hooks)
{
    (void) hooks;
    memset(out, 0, sizeof *out);
}
