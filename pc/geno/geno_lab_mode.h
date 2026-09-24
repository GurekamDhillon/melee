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
};

extern GameModeState gm_Mode_Lab_States[];
void gm_Mode_Lab_OnInit(void);
void gm_Mode_Lab_OnLoad(void);

/* for native code (gw_script.c calls the gw_-prefixed names) */
int GenoLab_ModeActive(void);   ///< 1 while LAB is the running game mode
int GenoLab_Leave(int where);   ///< end the LAB match (no contest) and go to `where`; 0 if no match
/* for tests (geno_tests.c) */
void GenoLab_ApplyRules(struct StartMeleeData* start);
int GenoLab_NextAfterMatch(void);

#endif
