/* riposte.c - vanilla-riposte's fighter code for the geno 12 `wasm` block: a line-for-line port of
 * pc/geno/mods/vanilla-riposte/lua/riposte.lua (the down special as a counter with a follow-up; the counter window
 * itself is declared in geno.json and run by the engine). Numbers are doubles as Lua's are; math.min / math.max keep
 * their first argument unless a later one beats it. */
#include "gd_geno.h"

#define WINDOW_END 24   /* the last frame of the counter window in geno.json */
#define CANCEL_AFTER 8  /* B may be let go from this frame on */
#define STANCE_END 30   /* the stance's total length */
#define MIN_POWER 9
#define MAX_POWER 30
#define SCALE 1.5
#define MAX_STREAK 4
#define FOLLOW_FROM 6
#define FOLLOW_TO 16
#define FOLLOW_SCALE 0.6

/* nothing to set: the streak survives from the last riposte until a whiff breaks it */
GD_EXPORT(parry_enter) void parry_enter(void) {}

GD_EXPORT(parry_frame) void parry_frame(void)
{
    double f = gd_geno_ctx(GD_ACTION_FRAME);
    if (f > WINDOW_END && gd_state("streak") != 0) gd_state_set("streak", 0); /* the window passed with no counter: a whiff */
    if (f >= STANCE_END) {
        gd_go("auto");
    } else if (f >= CANCEL_AFTER && f <= WINDOW_END && !gd_flag(GD_HELD(GD_BTN_SPECIAL))) {
        gd_state_set("streak", 0);
        gd_go("auto");
    } else if (gd_flag(GD_ANIM_ENDED)) {
        gd_geno_loop(); /* the clip is shorter than the stance: play it again */
    }
}

GD_EXPORT(riposte_enter) void riposte_enter(void)
{
    double n, p, hi;
    n = gd_state("streak") + 1;
    gd_state_set("streak", MAX_STREAK < n ? MAX_STREAK : n);
    gd_state_set("follow", 0);
    p = SCALE * gd_geno_ctx(GD_HIT_DAMAGE);
    hi = MIN_POWER < p ? p : MIN_POWER;
    gd_state_set("power", (hi < MAX_POWER ? hi : MAX_POWER) + (gd_state("streak") - 1));
    if (gd_geno_ctx(GD_HIT_FROM) < 0) gd_geno_turn(); /* the attacker is behind: face them */
    gd_geno_velocity(0.9, 0);
}

GD_EXPORT(riposte_frame) void riposte_frame(void)
{
    double f = gd_geno_ctx(GD_ACTION_FRAME);
    gd_geno_hitbox_damage(3, gd_state("power"));
    if (gd_flag(GD_PRESSED(GD_BTN_ATTACK)) && !gd_state("follow") && f >= FOLLOW_FROM && f <= FOLLOW_TO) {
        gd_state_set("follow", 1);
        gd_go("Follow");
    }
}

GD_EXPORT(follow_enter) void follow_enter(void)
{
    gd_geno_velocity(1.6, 0);
}

GD_EXPORT(follow_frame) void follow_frame(void)
{
    gd_geno_hitbox_damage(3, FOLLOW_SCALE * gd_state("power"));
}
