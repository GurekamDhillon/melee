/* counter.c - the suite's counter-window fixture (geno_tests: test_geno_lua_counter): the geno 12 port of its inline Lua. */
#include "../gd_geno.h"

/* s.power = math.min(30, math.max(9, 1.5 * hit_damage)) s.seen = countered s.from = hit_from */
GD_EXPORT(answer) void answer(void)
{
    double p = 1.5 * gd_geno_ctx(GD_HIT_DAMAGE), hi = 9 < p ? p : 9;
    gd_state_set("power", hi < 30 ? hi : 30);
    gd_state_set("seen", gd_geno_ctx(GD_COUNTERED));
    gd_state_set("from", gd_geno_ctx(GD_HIT_FROM));
}

GD_EXPORT(strike) void strike(void) { gd_geno_hitbox_damage(3, gd_state("power")); }

GD_EXPORT(spin) void spin(void)
{
    gd_geno_turn();
    gd_geno_velocity(2, 0);
}
