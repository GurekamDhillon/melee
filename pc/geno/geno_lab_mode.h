#ifndef GENO_LAB_MODE_H
#define GENO_LAB_MODE_H
/*
 * geno_lab_mode.h - LAB, the Geno Lab's own game mode (private, Geno build). geno_lab_mode.c.
 *
 * A game mode of its own, past the retail terminator like the port's GM_FRONTEND (GM_COUNT + 1),
 * so no vanilla id is reused and every retail table walk that stops at GM_COUNT still works.
 * Flow: SOLO > LAB -> the kit's character select -> the kit's stage select -> the match -> back to
 * LAB's character select. Rules: any fighters on any ports (humans and CPUs), no timer, no stocks
 * (time mode with the clock off: a KO respawns, forever), any stage, no items, Melee's own pause
 * off (the Lab draws its own pause menu, geno-lab/scripts/lab.lua).
 */
#include <melee/gm/forward.h>
#include <melee/gm/types.h>

#define GM_LAB (GM_COUNT + 2)

/* where a LAB match goes when it ends (GenoLab_Leave) */
enum {
    GENO_LAB_TO_CSS = 0,  ///< the default: LAB's character select
    GENO_LAB_TO_SSS = 1,  ///< the stage select, same fighters
    GENO_LAB_TO_MENU = 2, ///< quit: the menus (no contest)
    GENO_LAB_TO_MATCH = 3, ///< the same match again (the loading screen, then the match): restart
};

extern GameModeState gm_Mode_Lab_States[];
void gm_Mode_Lab_OnInit(void);
void gm_Mode_Lab_OnLoad(void);

/* for native code (gw_script.c calls the gw_-prefixed names) */
int GenoLab_ModeActive(void);   ///< 1 while LAB is the running game mode
int GenoLab_InMatch(void);      ///< 1 from a LAB match's start to its end (loading included)
int GenoLab_Leave(int where);   ///< end the LAB match (no contest) and go to `where`; 0 if no match
/* for tests (geno_tests.c) */
void GenoLab_ApplyRules(struct StartMeleeData* start);
int GenoLab_NextAfterMatch(void);

/* ---- debug movement (geno_lab_mode.c; any offline mode, not only LAB) ---------------------------
 * A fighter flies (noclip) while its state callbacks are the fly ones: it sits in Fall with no
 * gravity, no stage collision, no ledges, no blast-zone KO and (by default) no hurtboxes; the stick
 * moves it. Any action change (a KO, a respawn, the match ending) ends the flight by itself, so
 * nothing can stay stuck. Everything lives in the Fighter and in this TU's statics, both in a
 * snapshot, so it rewinds and re-simulates exactly. gw_script.c refuses all of it online. */
enum {
    GENO_FLY_OFF = 0,   ///< stop flying: Fall where it is
    GENO_FLY_ON = 1,    ///< start flying
    GENO_FLY_PLACE = 2, ///< stop flying on the floor below (Fall where it is when there is none)
};
int GenoFly_Set(int slot, int mode); ///< 0 ok; -1 no fighter, -2 its state cannot fly, 1 PLACE found no floor
int GenoFly_Get(int slot);           ///< 1 flying, 0 not, -1 no fighter on that port
int GenoFly_Any(void);               ///< 1 while any fighter flies (the camera leaves its bounds)
int GenoFly_HoldHitbox(int slot, int on, int action, int frame); ///< 0 ok, -1 no fighter, -3 not flying
int GenoFly_Holding(int slot);
int GenoFly_HoldHits(int slot); ///< connects counted per rehit interval since the hold began
int GenoFly_Teleport(int slot, int x_bits, int y_bits); ///< 0 ok, -1 no fighter, -2 state refused
void GenoFly_SetSpeed(int speed_bits);                  ///< units per frame at full stick
float GenoFly_Speed(void);
void GenoFly_SetSolid(int solid); ///< 1: hurtboxes stay on while flying (default 0: intangible)
int GenoFly_Solid(void);

#endif
