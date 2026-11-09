/* charger.c - vanilla-charger's fighter code for the geno 12 `wasm` block: a line-for-line port of
 * pc/geno/mods/vanilla-charger/lua/charger.lua. Hold B to charge (up to 60 frames), release to strike: the dash is faster
 * and the hit harder the longer it was held. Numbers are doubles as Lua's are. */
#include "gd_geno.h"

#define MAX_CHARGE 60
#define BASE_SPEED 1.6
#define SPEED_PER_FRAME 0.03
#define BASE_DAMAGE 6
#define DAMAGE_PER_FRAME 0.15

GD_EXPORT(charge_enter) void charge_enter(void)
{
    gd_state_set("charge", 0);
    gd_state_set("charged", 0);
}

GD_EXPORT(charge_frame) void charge_frame(void)
{
    if (gd_flag(GD_HELD(GD_BTN_SPECIAL)) && gd_state("charge") < MAX_CHARGE) {
        gd_state_set("charge", gd_state("charge") + 1);
        gd_state_set("charged", gd_state("charge") >= MAX_CHARGE);
        if (gd_flag(GD_ANIM_ENDED)) gd_geno_loop(); /* the clip is shorter than the charge: play it again */
    } else {
        gd_go("Release");
    }
}

GD_EXPORT(release_enter) void release_enter(void)
{
    gd_geno_velocity(BASE_SPEED + SPEED_PER_FRAME * gd_state("charge"), 0);
}

GD_EXPORT(release_frame) void release_frame(void)
{
    gd_geno_hitbox_damage(3, BASE_DAMAGE + DAMAGE_PER_FRAME * gd_state("charge"));
}
