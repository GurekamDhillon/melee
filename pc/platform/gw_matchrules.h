/* gw_matchrules.h - the match rule word shared by the simulation (game side, pc/gameworld) and the
 * native side (netplay handshake, scene token, menus). Plain defines and two tiny pure functions so
 * it compiles in both worlds. Design: _research/turbo-mode-projectm-2026-10-04.md; behaviour and
 * the decisions are in the header comment of pc/gameworld/script_fighter_interrupt.inc.
 *
 * TURBO is the first match rule: while a fighter's attack has connected, the fighter may cancel
 * the rest of that attack into a wider set of actions (an "interrupt window"). The rule is ONE
 * 32-bit word so variants are data: bit 0 turns it on, bits 1..8 are the individual rules, bits
 * 24..31 are the window length in logic frames (0 = the default). Zero means no rule.
 *
 * It is a MATCH RULE, not a script effect: both peers of an online match must hold the same word
 * (gw_net.c carries it in HELLO/ACCEPT, protocol 4; the host's scene string carries it as
 * `turbo=<hex>`), and the simulation reads it only through gw_MatchTurboRules().
 */
#ifndef GW_MATCHRULES_H
#define GW_MATCHRULES_H

#define GW_TURBO_ON          0x00000001u /* the rule is on (nothing below means anything without it) */
#define GW_TURBO_SHIELD_HIT  0x00000002u /* a hit that lands on a shield also opens the window */
#define GW_TURBO_ITEM_HIT    0x00000004u /* so does a projectile/item hit by the fighter's own move */
#define GW_TURBO_NO_SELF     0x00000008u /* a move cannot cancel into itself (specials: into specials) */
#define GW_TURBO_DASH_JAB    0x00000010u /* only jabs and dash attacks may cancel into a dash */
#define GW_TURBO_SMASH_STILL 0x00000020u /* smashes may not cancel into crouch, jump or dash */
#define GW_TURBO_NO_SHIELD   0x00000040u /* grounded attacks may not cancel into shield */
#define GW_TURBO_NO_AIRDODGE 0x00000080u /* aerials and specials may not cancel into air dodge */
#define GW_TURBO_AIR_JUMPS   0x00000100u /* a hit landed in the air restores air jumps */
#define GW_TURBO_HITLAG      0x00000200u /* RESERVED: window usable during hitlag (not built: refused) */
#define GW_TURBO_NO_STEER    0x00000400u /* walking and turning never take the window (stick held = steering, not intent) */
#define GW_TURBO_NO_MOVE_LOOP 0x00000800u /* NOT in V1 (owner's decision pending). A cancel into crouch, dash or a jump remembers the
                                             granting move for the rest of the window's original length: that same move, thrown
                                             again out of the crouch/dash/jump, grants no window until the memory ends or a
                                             different attack lands. Closes jab -> crouch -> jab -> crouch ... on a grounded
                                             opponent (the same-move guard compares the cancel target, and crouch is another move). */
#define GW_TURBO_FRAMES_SHIFT 24
#define GW_TURBO_FRAMES_MASK 0xFF000000u
#define GW_TURBO_DEFAULT_FRAMES 30       /* logic frames outside hitlag; 0 in the word = this */
#define GW_TURBO_MAX_FRAMES  120

/* The first rule set: everything above except ITEM_HIT, window 30 frames (0x5FB). */
#define GW_TURBO_V1 (GW_TURBO_ON | GW_TURBO_SHIELD_HIT | GW_TURBO_NO_SELF | GW_TURBO_DASH_JAB | \
                     GW_TURBO_SMASH_STILL | GW_TURBO_NO_SHIELD | GW_TURBO_NO_AIRDODGE | GW_TURBO_AIR_JUMPS | \
                     GW_TURBO_NO_STEER)
/* Bits this build implements. A word with any other bit set is refused, never half-applied. */
#define GW_TURBO_SUPPORTED (0x000001FFu | GW_TURBO_NO_STEER | GW_TURBO_NO_MOVE_LOOP | GW_TURBO_FRAMES_MASK)

static inline int gw_turbo_frames(unsigned w) {
    unsigned f = (w & GW_TURBO_FRAMES_MASK) >> GW_TURBO_FRAMES_SHIFT;
    return f == 0 ? GW_TURBO_DEFAULT_FRAMES : (int) f;
}
/* 1 if `w` is a rule word this build can apply (0 = off is valid). */
static inline int gw_turbo_valid(unsigned w) {
    if (w == 0) return 1;
    if (w & ~GW_TURBO_SUPPORTED) return 0;
    if (!(w & GW_TURBO_ON)) return 0;
    return gw_turbo_frames(w) <= GW_TURBO_MAX_FRAMES;
}

#endif
