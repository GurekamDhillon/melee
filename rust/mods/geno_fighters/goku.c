/* goku.c - Goku's fighter code for the geno 12 `wasm` block: a line-for-line port of goku-xv2's lua/goku.lua
 * (tools/rust-port/golden/coverage/goku-xv2-lua/goku.lua, the geno 10 package's text). Numbers are doubles as Lua's
 * are; an int slot is read and written as an integral double; `a and b or c` and math.min / math.max keep the Lua
 * evaluation order and results. Build: build.sh (clang --target=wasm32, no libc). */
#include "gd_geno.h"

#define TAP 20  /* charge frames: tap < 20 <= mid < 45 <= full; 60 is the cap */
#define MID 45
#define FULL 60

/* a stored charge (shield while charging) resumes where it was left */
GD_EXPORT(charge_enter) void charge_enter(void)
{
    gd_state_set("charge", gd_state("stored"));
    gd_state_set("stored", 0);
}

GD_EXPORT(charge_frame) void charge_frame(void)
{
    if (gd_flag(GD_PRESSED(GD_BTN_SHIELD)) && gd_state("charge") >= 10) {
        /* store the charge: Melee-style, B again resumes it */
        gd_state_set("stored", gd_state("charge"));
        gd_go("auto");
        return;
    }
    if (gd_flag(GD_HELD(GD_BTN_SPECIAL)) && gd_state("charge") < FULL) {
        gd_state_set("charge", gd_state("charge") + 1);
        if (gd_flag(GD_ANIM_ENDED)) gd_geno_loop();
        return;
    }
    if (gd_flag(GD_HELD(GD_BTN_SPECIAL))) {
        if (gd_flag(GD_ANIM_ENDED)) gd_geno_loop();
        return;
    }
    if (gd_state("charge") >= MID) { gd_go("KameFull"); return; }
    if (gd_state("charge") >= TAP) { gd_go("KameMid"); return; }
    gd_go("KameTap");
}

/* Dragon Rush / Super Dash: the dash is the move; in the air it keeps its height (a recovery) */
GD_EXPORT(rush_enter) void rush_enter(void)
{
    if (gd_flag(GD_AIR)) gd_geno_velocity(2.6, 0.3); else gd_geno_velocity(2.6, 0);
}

GD_EXPORT(rush_frame) void rush_frame(void)
{
    if (gd_geno_ctx(GD_ACTION_FRAME) == 20) gd_geno_velocity(0.5, gd_geno_ctx(GD_VEL_Y));
}

GD_EXPORT(rushcombo_enter) void rushcombo_enter(void)
{
    gd_geno_velocity(1.4, 0);
}

GD_EXPORT(rushcombo_frame) void rushcombo_frame(void)
{
    double f = gd_geno_ctx(GD_ACTION_FRAME);
    if (f == 8 || f == 15) gd_geno_velocity(1.4, gd_flag(GD_AIR) ? 0.2 : 0);
    if (f == 22) gd_geno_velocity(0.4, gd_geno_ctx(GD_VEL_Y));
}

/* Ground liftoff and vertical launch are the first two ftcmd writes. */
GD_EXPORT(recovery_enter) void recovery_enter(void)
{
    if (gd_flag(GD_AIR)) gd_geno_velocity(0.4, 3.6);
}

/* down B tap = Z-vanish counter, which costs ki: with too little ki it is only the blink (hold = ki charge) */
GD_EXPORT(counter_frame) void counter_frame(void)
{
    if (gd_geno_ctx(GD_ACTION_FRAME) == 12 && gd_flag(GD_HELD(GD_BTN_SPECIAL))) { gd_go("KiCharge"); return; }
}

GD_EXPORT(riposte_enter) void riposte_enter(void)
{
    double d, hi;
    gd_geno_turn(); /* Z-vanish: reappear facing the attacker */
    /* math.min(18, math.max(10, ctx.self.hit_damage * 1.25)): Lua keeps the first argument unless a later one beats it */
    d = gd_geno_ctx(GD_HIT_DAMAGE) * 1.25;
    hi = 10 < d ? d : 10;
    gd_state_set("counter_damage", hi < 18 ? hi : 18);
}

GD_EXPORT(riposte_frame) void riposte_frame(void)
{
    gd_geno_hitbox_damage(1, gd_state("counter_damage"));
}

GD_EXPORT(kicharge_frame) void kicharge_frame(void)
{
    if (gd_flag(GD_ANIM_ENDED)) gd_geno_loop(); /* hold the charge clip until released */
    if (!gd_flag(GD_HELD(GD_BTN_SPECIAL))) { gd_go("auto"); return; }
}

/* Not called: names the cape reappear states so the offline reachability check sees them (the engine picks them by
 * behaviour order). */
GD_EXPORT(it_targets) void it_targets(void)
{
    gd_go("ITEnd"); gd_go("ITEndAir"); gd_go("RushCombo"); gd_go("KameSuper");
}
