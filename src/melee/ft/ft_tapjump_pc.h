#ifndef MELEE_FT_TAPJUMP_PC_H
#define MELEE_FT_TAPJUMP_PC_H

/* Per-player Tap jump off (the CSS Controls panel, agreed in netplay setup and frozen at match start).
 * Only the stick-up jump request is gated; the stick values, X/Y and every other use of stick up stay. */
#if defined(TARGET_PC)
extern int FighterControls_TapJumpOff(int port);
#define FT_TAP_JUMP_OK(fp) (!FighterControls_TapJumpOff((fp)->player_id))
#else
#define FT_TAP_JUMP_OK(fp) 1
#endif

#endif
