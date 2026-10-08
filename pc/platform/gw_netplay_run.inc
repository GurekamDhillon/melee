/* gw_netplay_run.inc - the stage run's director glue inside gw_netplay.c (included after lb_broadcast): the unlock-mask exchange, the stage-end barrier,
 * the abort, the interstitial's words and the read-only view for scripts. The pure rules (shared stocks, verdicts, the tuple, the Classic plan) are gw_netdir.h;
 * the record is gw_netrun.h; the lobby state it works on is `lb` (gw_netplay.c). Nothing here is in a snapshot or the rollback hash: the director writes
 * nothing while a stage runs, and what crosses a stage boundary is the record (digest-checked at the next scene) and the director state ("R"). */

/* The run is over for both: a disagreement, a refusal, or a barrier that did not close. Idempotent; the lobby then refuses READY with the reason. */
static void lb_run_abort(const char *why) {
    if (!lb.run_on) return;
    if (lb.rd.over == GW_ND_LIVE) lb.rd.over = GW_ND_ABORTED;
    snprintf(lb.run_fail, sizeof lb.run_fail, "%s", why);
    snprintf(lb.rd_last, sizeof lb.rd_last, "run ended: %s", why);
    gw_log("netrun: RUN ABORTED - %s", why);
    lb.ready[0] = lb.ready[1] = 0;
    lb.countdown = 0;
    lb.seq++;
}

/* ---- the agreed unlock mask: each side sends the fighters its save has unlocked; the agreed set is the intersection (lb_run_mask) ---- */
static void lb_run_send_mask(void) {
    char m[24];
    lb.mask_local = lb_run_local_mask();
    snprintf(m, sizeof m, "K %08x", (unsigned) lb.mask_local);
    lb_send(m);
    lb.mask_sent = 1;
    gw_log("netrun: my unlocked fighters 0x%08x", (unsigned) lb.mask_local);
}
static void lb_run_on_mask(const char *m) {
    lb.mask_peer_raw = (uint32_t) strtoul(m + 2, NULL, 16);
    lb.mask_raw_ok = 1;
    lb.seq++;
    gw_log("netrun: the other side's unlocked fighters 0x%08x", (unsigned) lb.mask_peer_raw);
}

/* ---- the stage-end barrier --------------------------------------------------------------------------------------------------------------
 * Before a peer lets a stage's scene go (gmVsMelee_ExitVs -> gw_Netplay_StageEnd) the two exchange (epoch, exit frame, a final hash from before the exit,
 * winner, stocks, shared stocks) on the lobby channel, which is still open: the session closes only when the next scene starts (gw_Netplay_MatchOver).
 * Equal tuples: both directors move on identically (pool pays, next stage / continue / over). Unequal, or the other side does not answer: the run ends for
 * both. The hash is the curated gameplay checksum (gw_rb_checksum_e) of the last simulated frame before the exit, once its inputs are confirmed. */
#define LB_RUN_HASH_BACK 1
#define LB_RUN_BARRIER_MS 5000u
static void lb_run_on_end(const char *m) {
    GwNdEnd e;
    if (!gw_nd_end_parse(m, &e)) {
        gw_log("netrun: a malformed stage-end message: %.60s", m);
        return;
    }
    lb.fz_peer = e;
    lb.fz_have_peer = 1;
}
static void lb_run_pump(int frame) {
    if (np.net != NULL) gw_net_poll(np.net, frame);
    np_rdv_service();
#ifdef _WIN32
    {
        MSG msg;
        while (PeekMessageA(&msg, NULL, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
#else
    gw_pump_events();
#endif
    Sleep(1);
}
/* The common part, testable without a transport: both tuples are in, compare, move the director on. Returns 1 when they agreed. */
static int lb_run_close_stage(const GwNdEnd *mine, const GwNdEnd *theirs) {
    const char *why = gw_nd_end_diff(mine, theirs);
    int before = lb.rd.pool, deaths, stage = lb.rd.stage;
    char msg[160];
    if (why != NULL) {
        snprintf(msg, sizeof msg, "the two sides ended stage %d differently: %s (mine: frame %d hash %08x winner %d stocks %d-%d pool %d; theirs: frame %d hash %08x winner %d stocks %d-%d pool %d)",
                 stage + 1, why, mine->frame, (unsigned) mine->hash, mine->winner, mine->s0, mine->s1, mine->pool, theirs->frame, (unsigned) theirs->hash, theirs->winner, theirs->s0, theirs->s1,
                 theirs->pool);
        lb_run_abort(msg);
        return 0;
    }
    deaths = gw_nd_deaths(lb.rd_each, mine->s0, mine->s1);
    if (mine->winner < 0) deaths = lb.rd.pool; /* cancellation or no clear loses this stage for the pair */
    gw_nd_after_stage(&lb.rd, deaths);
    lb.ends++;
    snprintf(lb.rd_last, sizeof lb.rd_last, "stage %d ended: epoch %d, exit frame %d, hash %08x, winner %d, stocks %d-%d, shared stocks %d -> %d (-%d)%s", stage + 1, mine->epoch, mine->frame,
             (unsigned) mine->hash, mine->winner, mine->s0, mine->s1, before, lb.rd.pool, deaths,
             lb.rd.over == GW_ND_LIVE ? (gw_nd_asking(&lb.rd) ? ", continue offered" : "") : lb.rd.over == GW_ND_CLEARED ? ", RUN CLEARED" : ", RUN OVER");
    gw_log("netrun: STAGE END %s", lb.rd_last);
    if (lb.rd.over != GW_ND_LIVE) gw_log("netrun: the run is over for both - %s", gw_nd_over_name(lb.rd.over));
    lb.seq++;
    return 1;
}
/* gmVsMelee_ExitVs (game), every VS exit: winner -1 none / 0 / 1 (the two humans' ports), s0/s1 their stocks left. Blocks, bounded, until the other side's tuple is in. */
void gw_Netplay_StageEnd(int winner, int s0, int s1) {
    GwNdEnd e;
    char m[GW_NET_LOBBY_MAX];
    DWORD t0;
    int epoch, frame, hf;
    if (!lb.run_on || np.net == NULL || np.phase != NP_RUNNING || lb.rd.over != GW_ND_LIVE) return;
    epoch = gw_rb_epoch();
    frame = gw_rb_current_frame();
    hf = frame - LB_RUN_HASH_BACK;
    memset(&e, 0, sizeof e);
    e.epoch = epoch;
    e.frame = frame;
    e.winner = winner < -1 ? -1 : winner > 1 ? -1 : winner;
    e.s0 = s0;
    e.s1 = s1;
    e.stage = lb.rd.stage;
    e.pool = lb.rd.pool;
    t0 = GetTickCount();
    while ((gw_rb_confirmed_frame() < hf || (e.hash = gw_rb_checksum_e(epoch, hf)) == 0) && GetTickCount() - t0 < LB_RUN_BARRIER_MS && !np.dead) lb_run_pump(frame); /* the remaining inputs arrive and make it final */
    if (gw_rb_confirmed_frame() < hf || e.hash == 0) { lb_run_abort("final stage hash unavailable"); return; }
    if (lb_run_poison() == 3) e.hash ^= 0x5A5A5A5Au; /* TEST-ONLY: this side reports another hash, so the barrier must refuse */
    lb.fz_mine = e;
    gw_nd_end_format(&e, m, sizeof m);
    lb_send(m);
    gw_log("netrun: stage %d is ending - %s", e.stage + 1, m + 2);
    while (!lb.fz_have_peer && GetTickCount() - t0 < LB_RUN_BARRIER_MS && !np.dead) lb_run_pump(frame);
    if (!lb.fz_have_peer) {
        lb_run_abort("the other side never closed the stage (no stage-end message)");
        return;
    }
    lb.fz_have_peer = 0;
    lb_run_close_stage(&e, &lb.fz_peer);
}

/* ---- the interstitial's words (the lobby is the screen between stages; Atlas and the legacy lobby show the same title and instruction line) -------------------------- */
static const char *lb_run_fighter_name(int ck) {
    int i = gw_MexId_FindFighter(ck);
    return i >= 0 ? gw_MexId_Name(i) : "?";
}
static void lb_run_plan_line(char *out, int cap) {
    GwNdPlan pl;
    out[0] = '\0';
    if (lb.rd.kind == GW_ND_KIND_CLASSIC && lb_run_plan(&pl, 0) && lb.rd.stage < pl.n) {
        const GwNdStage *g = &pl.st[lb.rd.stage];
        int k, o = snprintf(out, (size_t) cap, "%s vs", gw_nd_flags_name(g->flags));
        for (k = 0; k < 3 && o < cap - 24; ++k) if (g->ck[k] != GW_ND_ENEMY_NONE) o += snprintf(out + o, (size_t) cap - (size_t) o, "%s %s", k ? "," : "", lb_run_fighter_name(g->ck[k]));
    }
}
/* which: 0 the screen's title, 1 the instruction line. "" when this is not a stage run (the lobby keeps its own words). */
void gw_Netplay_RunText(int which, char *out, int cap) {
    char plan[80];
    int n = lb.rd.stage + 1;
    out[0] = '\0';
    if (!lb.run_on || cap < 8) return;
    lb_run_plan_line(plan, sizeof plan);
    if (which == 0) {
        if (lb.rd.over == GW_ND_CLEARED) snprintf(out, (size_t) cap, "RUN CLEARED");
        else if (lb.rd.over == GW_ND_LOST) snprintf(out, (size_t) cap, "RUN OVER");
        else if (lb.rd.over == GW_ND_ABORTED) snprintf(out, (size_t) cap, "RUN ENDED");
        else if (gw_nd_asking(&lb.rd)) snprintf(out, (size_t) cap, "OUT OF STOCKS");
        else if (lb.rd.len > 0) snprintf(out, (size_t) cap, "RUN - STAGE %d OF %d", n, lb.rd.len);
        else snprintf(out, (size_t) cap, "RUN - STAGE %d", n);
        return;
    }
    if (lb.rd.over == GW_ND_CLEARED) snprintf(out, (size_t) cap, "Cleared all %d stages together with %d shared stocks left. Leave the room to finish.", lb.rd.stage, lb.rd.pool);
    else if (lb.rd.over == GW_ND_LOST) snprintf(out, (size_t) cap, "The pair is out of shared stocks after %d stage%s. The run is over for both.", lb.rd.stage, lb.rd.stage == 1 ? "" : "s");
    else if (lb.rd.over == GW_ND_ABORTED) snprintf(out, (size_t) cap, "The run was ended: %s", lb.run_fail[0] != '\0' ? lb.run_fail : "the two sides did not agree");
    else if (gw_nd_asking(&lb.rd)) snprintf(out, (size_t) cap, "Shared stocks are gone. START spends the continue and plays stage %d again. Leave to end the run.", n);
    else if (lb.phase == LB_READY && lb_run_block()[0] != '\0') snprintf(out, (size_t) cap, "Stage %d: %s.", n, lb.rd_block);
    else if (lb.phase == LB_READY) snprintf(out, (size_t) cap, "Stage %d%s%s. Shared stocks %d. START when you're ready.", n, plan[0] != '\0' ? ": " : "", plan, lb.rd.pool);
}

/* The frontend pushes the fighters this save has unlocked, every frame it is up (bit = fighter id 0..31). */
void gw_Netplay_SetUnlocks(unsigned mask) { np.unlocks = mask; }
int gw_Netplay_RunActive(void) { return lb.run_on; }
