/* Fixtures for the interrupt window: the rule function, the key table, the open core (repeat guard,
 * multi-hit, the gates), script ownership, and snapshot exactness (the window is plain BSS: a
 * memcpy of the table is the whole of what the snapshot does). Fake fighters; no engine calls. */
int ScriptGame_IntrWinTest(void)
{
    IntrWin saved[12], snap[12];
    static Fighter fake;
    Fighter* fp = &fake;
    int rc = 0;
    unsigned v1 = GW_TURBO_V1;
    memcpy(saved, ft_intrwin, sizeof saved);
    memset(ft_intrwin, 0, sizeof ft_intrwin);
    memset(fp, 0, sizeof *fp);

    /* the word */
    if (!gw_turbo_valid(0) || !gw_turbo_valid(v1) || gw_turbo_valid(GW_TURBO_SHIELD_HIT) ||
        gw_turbo_valid(v1 | GW_TURBO_HITLAG) || gw_turbo_valid(v1 | 0x10000u) || gw_turbo_valid(v1 | 0x800u) ||
        gw_turbo_valid(v1 | (200u << GW_TURBO_FRAMES_SHIFT)) || gw_turbo_frames(v1) != 30 ||
        gw_turbo_frames(v1 | (12u << GW_TURBO_FRAMES_SHIFT)) != 12) rc |= 1;

    /* the key table: smash startup/charge/release are one action family; multi-hit jabs are one */
    if (intrwin_key(ftCo_MS_Attack11) != IW_K_JAB || intrwin_key(ftCo_MS_Attack13) != IW_K_JAB ||
        intrwin_key(ftCo_MS_AttackS4Hi) != IW_K_FSMASH || intrwin_key(ftCo_MS_AttackS4Lw) != IW_K_FSMASH ||
        intrwin_key(ftCo_MS_AttackHi4) != IW_K_USMASH || intrwin_key(ftCo_MS_AttackAirLw) != IW_K_DAIR ||
        intrwin_key(ftCo_MS_Count) != IW_K_SPECIAL || intrwin_key(ftCo_MS_ThrowF) != IW_K_NONE ||
        intrwin_key(ftCo_MS_Wait) != IW_K_NONE || intrwin_key(ftCo_MS_CatchAttack) != IW_K_NONE) rc |= 2;

    /* each exclusion; and what stays allowed */
    if (intrwin_allows(v1, IW_K_JAB, 1, IW_X_JAB, IW_K_JAB) ||                 /* no self-cancel */
        !intrwin_allows(v1, IW_K_JAB, 1, IW_X_TILT, IW_K_FTILT) ||
        intrwin_allows(v1, IW_K_FSMASH, 1, IW_X_SMASH, IW_K_FSMASH) ||
        intrwin_allows(v1, IW_K_NAIR, 0, IW_X_AERIAL, IW_K_NAIR) ||
        !intrwin_allows(v1, IW_K_NAIR, 0, IW_X_AERIAL, IW_K_FAIR) ||
        intrwin_allows(v1, IW_K_SPECIAL, 1, IW_X_SPECIAL, IW_K_NONE)) rc |= 4;
    if (intrwin_allows(v1, IW_K_FTILT, 1, IW_X_DASH, IW_K_NONE) ||             /* dash: jab/dash attack only */
        !intrwin_allows(v1, IW_K_JAB, 1, IW_X_DASH, IW_K_NONE) ||
        !intrwin_allows(v1, IW_K_DASHATK, 1, IW_X_DASH, IW_K_NONE)) rc |= 8;
    if (intrwin_allows(v1, IW_K_USMASH, 1, IW_X_CROUCH, IW_K_NONE) ||          /* smash stays still */
        intrwin_allows(v1, IW_K_DSMASH, 1, IW_X_JUMP, IW_K_NONE) ||
        !intrwin_allows(v1, IW_K_DSMASH, 1, IW_X_ESCAPE, IW_K_NONE) ||          /* ...but rolls stay */
        !intrwin_allows(v1, IW_K_FTILT, 1, IW_X_JUMP, IW_K_NONE)) rc |= 16;
    if (intrwin_allows(v1, IW_K_JAB, 1, IW_X_SHIELD, IW_K_NONE) ||             /* no shield from the ground */
        !intrwin_allows(v1, IW_K_JAB, 1, IW_X_ESCAPE, IW_K_NONE)) rc |= 32;
    if (intrwin_allows(v1, IW_K_DAIR, 0, IW_X_AIRDODGE, IW_K_NONE) ||          /* no air dodge from air moves */
        intrwin_allows(v1, IW_K_SPECIAL, 0, IW_X_AIRDODGE, IW_K_NONE) ||
        !intrwin_allows(v1, IW_K_DAIR, 0, IW_X_AIRJUMP, IW_K_NONE)) rc |= 64;
    if (intrwin_allows(v1, IW_K_JAB, 1, IW_X_WALK, IW_K_NONE) || intrwin_allows(v1, IW_K_UAIR, 0, IW_X_TURN, IW_K_NONE) ||
        !intrwin_allows(GW_TURBO_ON, IW_K_JAB, 1, IW_X_WALK, IW_K_NONE)) rc |= 262144; /* steering never takes it */
    /* variants are data: drop the bits and the exits come back */
    if (!intrwin_allows(GW_TURBO_ON, IW_K_JAB, 1, IW_X_JAB, IW_K_JAB) ||
        !intrwin_allows(GW_TURBO_ON, IW_K_FSMASH, 1, IW_X_CROUCH, IW_K_NONE) ||
        !intrwin_allows(GW_TURBO_ON, IW_K_JAB, 1, IW_X_SHIELD, IW_K_NONE)) rc |= 128;

    /* opening: gates, the repeat guard, multi-hit, jump restore */
    fp->motion_id = ftCo_MS_AttackAirF; fp->ground_or_air = GA_Air; fp->x1968_jumpsUsed = 2;
    fp->x2068_attackID = 7; fp->x206C_attack_instance = 3;
    if (intrwin_open(1, fp, IW_OPEN_HIT, 0) || intrwin_open(1, fp, IW_OPEN_HIT, v1 & ~GW_TURBO_ON) ||
        intrwin_open(1, fp, IW_OPEN_ITEM, v1) ||                                    /* ITEM_HIT is not in V1 */
        intrwin_open(0, fp, IW_OPEN_HIT, v1) || intrwin_open(13, fp, IW_OPEN_HIT, v1)) rc |= 256;
    if (!intrwin_open(1, fp, IW_OPEN_SHIELD, v1) || ScriptGame_IntrWinRead(1, 0) != 30 ||
        ScriptGame_IntrWinRead(1, 3) != IW_K_FAIR || fp->x1968_jumpsUsed != 1) rc |= 512;
    if (intrwin_open(1, fp, IW_OPEN_HIT, v1) || ScriptGame_IntrWinRead(1, 4) != 1) rc |= 1024; /* open: no stomp */
    intrwin_close(&ft_intrwin[0], 1);
    if (intrwin_open(1, fp, IW_OPEN_HIT, v1)) rc |= 2048;                           /* same instance: multi-hit once */
    fp->x206C_attack_instance = 4;
    if (!intrwin_open(1, fp, IW_OPEN_HIT, v1) || ScriptGame_IntrWinRead(1, 4) != 2 ||
        ScriptGame_IntrWinRead(1, 5) != 1) rc |= 4096;                              /* a new move, a new window */
    fp->motion_id = ftCo_MS_ThrowF; intrwin_close(&ft_intrwin[0], 0); fp->x206C_attack_instance = 5;
    if (intrwin_open(1, fp, IW_OPEN_HIT, v1)) rc |= 8192;                           /* throws never grant */
    fp->motion_id = ftCo_MS_AttackAirN; fp->x1968_jumpsUsed = 1;
    if (!intrwin_open(2, fp, IW_OPEN_ITEM, v1 | GW_TURBO_ITEM_HIT) ||               /* the ITEM_HIT variant */
        ScriptGame_IntrWinRead(2, 0) != 30) rc |= 16384;

    /* snapshot exactness: restore a copy and the table is bit-identical, mid-window */
    memcpy(snap, ft_intrwin, sizeof snap);
    intrwin_close(&ft_intrwin[1], 1);
    memcpy(ft_intrwin, snap, sizeof snap);
    if (memcmp(ft_intrwin, snap, sizeof snap) || ScriptGame_IntrWinRead(2, 0) != 30 ||
        ScriptGame_IntrWinHashWord(2) == 0 || ScriptGame_IntrWinHashWord(5) != 0) rc |= 32768;

    /* ownership/release: a script's window is its own; release(owner) clears only its own */
    memset(ft_intrwin, 0, sizeof ft_intrwin);
    ft_intrwin[3].owner = 5; ft_intrwin[3].frames = 9; ft_intrwin[3].opened = 1;
    ft_intrwin[4].owner = 6; ft_intrwin[4].frames = 9; ft_intrwin[4].opened = 1;
    ScriptGame_IntrWinRelease(5);
    if (ft_intrwin[3].frames || !ft_intrwin[4].frames || ScriptGame_IntrWinRead(5, 1) != 6) rc |= 65536;
    ScriptGame_IntrWinRelease(0);
    if (ft_intrwin[4].frames) rc |= 131072;

    memcpy(ft_intrwin, saved, sizeof saved);
    return rc;
}
