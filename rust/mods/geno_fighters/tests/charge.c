/* charge.c - the game-half suite's fighter-code fixture (geno_tests: test_geno_lua_charge / fault / digest, the cover
 * test): the geno 12 port of the inline Lua those tests carried (pc/geno/geno_tests.c, slice 5), line for line. */
#include "../gd_geno.h"

GD_EXPORT(enter) void enter(void) { gd_state_set("charge", 0); }

GD_EXPORT(frame) void frame(void)
{
    if (gd_flag(GD_HELD(GD_BTN_SPECIAL)) && gd_state("charge") < 60) {
        gd_state_set("charge", gd_state("charge") + 1);
        gd_state_set("armed", gd_state("charge") >= 3);
    } else {
        gd_go("Release");
    }
}

GD_EXPORT(rel) void rel(void) { gd_geno_velocity(1.6 + 0.03 * gd_state("charge"), 0); }

GD_EXPORT(relf) void relf(void) { gd_geno_hitbox_damage(3, 6 + 0.15 * gd_state("charge")); }

/* ctx.state.charge = 77 error('boom'): the write lands, then the call faults (GENO_LUA_FAULT_ERROR) */
GD_EXPORT(boom) void boom(void)
{
    gd_state_set("charge", 77);
    __builtin_trap();
}
