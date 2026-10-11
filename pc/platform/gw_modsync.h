/* gw_modsync.h - "match the host's mods": when two players meet with different mods, say what differs and offer to make the
 * mod sets agree instead of only refusing (OWNER RULES: no peer-to-peer file transfer, ever; mods come from Nucleus by post id;
 * anything else only gets a clear "you need this mod: <name> (<where it came from>)").
 *
 * Implementation: gw_modsync.inc, compiled into gw_mods.c (so no link-list entry). Four parts:
 *   1. MANIFEST  what this install has MOUNTED (id, kind, name, content digest, source + Nucleus post id, whether it changes
 *                the match identity). Costume skins do not ("fine"). Exchanged over gw_net's lobby channel ("MSM" chunks).
 *   2. DIFF/PLAN the rows the room shows (they have / you have / differs) and the actions that would make this install match
 *                (download from Nucleus, enable an installed mod, disable an extra one, or "missing": not obtainable).
 *   3. JOURNAL   <mods dir>/.modsync/: the sync session is TEMPORARY. Before anything changes, the player's own enabled set is
 *                saved; it is put back when the session ends, or at the next boot if the game died first (crash-safe).
 *   4. BOOT      gw_ms_boot runs before enabled.txt is read; it continues a sync session that relaunched the game, or restores.
 * Mods mount at boot, so a sync ends in a RELAUNCH of the game (gw_ms_relaunch) that rejoins the room.
 * Everything here is plain C; the string/int forms are the ones the Lua API and the room screens use. */
#ifndef GW_MODSYNC_H
#define GW_MODSYNC_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define GW_MS_MAX 64
#define GW_MS_ID 64
#define GW_MS_NAME 48

typedef struct gw_ms_entry {
    char id[GW_MS_ID];       /* the mod's folder name */
    char name[GW_MS_NAME];   /* display name ('|' and control characters stripped) */
    char kind[12];           /* mod.json kind: base fighter stage misc script skin */
    char source[12];         /* "nucleus" or "" (unknown origin) */
    int post;                /* the Nucleus post id, 0 when none */
    uint64_t digest;         /* content digest (gw_Mods's per-mod digest; 0 for a mod that is not mounted) */
    int ident;               /* 1 = the mod can change what the match plays (everything except kind "skin") */
    /* LOCAL ONLY, never on the wire: */
    int enabled;             /* in the enabled set for the next boot */
    int mounted;             /* mounted this boot */
} gw_ms_entry;

typedef struct gw_ms_manifest {
    uint64_t global;         /* the global game data hash (np_global_hash): what must match for a match to start */
    int n;
    gw_ms_entry e[GW_MS_MAX];
} gw_ms_manifest;

/* ---- the wire --------------------------------------------------------------------------------------------------
 * Chunks of at most `cap` bytes (cap <= GW_NET_LOBBY_MAX = 200), each starting "MSM ". Greedy packing; the first record is the
 * header. A receiver is "ready" when every chunk of one manifest has arrived and the entry count agrees. */
int gw_ms_wire_encode(const gw_ms_manifest *m, int index, uint8_t *out, int cap); /* chunk `index` (0..): bytes written, 0 when index is past the last */
void gw_ms_wire_reset(gw_ms_manifest *peer_state);     /* receiver: forget (also clears the "ready" mark) */

/* Process-wide pairing, used by gw_netplay.c exactly like gw_MexId_PeerReset / WireNext / WireSent / WireFeed. */
void gw_ms_peer_reset(void);                           /* new connection: snapshot OUR manifest (gw_ms_build_local), restart sending, forget the peer */
int gw_ms_wire_next(uint8_t *out, int cap);            /* next chunk to send, 0 = all sent */
void gw_ms_wire_sent(void);                            /* the chunk wire_next produced was accepted */
int gw_ms_wire_feed(const uint8_t *msg, int len);      /* 1 = it was ours (consumed), 0 = not ours */
void gw_ms_set_global(uint64_t global);                /* the global hash the header carries; stored in the snapshot (set before peer_reset for the first send) */
int gw_ms_peer_ready(void);                            /* 1 = the peer's whole manifest has arrived */
const gw_ms_manifest *gw_ms_peer(void);                /* the peer's manifest (valid when ready) */
const gw_ms_manifest *gw_ms_mine(void);                /* the snapshot of ours that was sent */

/* This install's manifest: every INSTALLED mod (e = enabled/mounted flags say which are on) built from gw_Mods_*; `global` is
 * set by the caller. The wire carries only the mounted ones. */
void gw_ms_build_local(gw_ms_manifest *out, uint64_t global);

/* ---- diff and plan ------------------------------------------------------------------------------------------- */
enum { GW_MS_SAME = 0, GW_MS_MINE_ONLY = 1, GW_MS_THEIRS_ONLY = 2, GW_MS_DIFFERS = 3 };
typedef struct gw_ms_row {
    char id[GW_MS_ID];
    char name[GW_MS_NAME];
    int state;               /* GW_MS_MINE_ONLY: you have it, they do not; THEIRS_ONLY: they have it; DIFFERS: same id, other content */
    int ident;               /* 0 = cosmetic (a costume skin): fine either way */
    char source[12];
    int post;
} gw_ms_row;
/* Compare what is MOUNTED here (mine entries with .mounted) with the peer's list. Writes the rows that are not SAME, in id order;
 * returns how many (also when more than cap: the count is complete, rows stop at cap). */
int gw_ms_diff(const gw_ms_manifest *mine, const gw_ms_manifest *theirs, gw_ms_row *rows, int cap);
/* How many of the rows change the match identity (ident != 0). 0 = the mod sets agree where it matters. */
int gw_ms_ident_diffs(const gw_ms_row *rows, int n);

enum { GW_MS_ACT_ENABLE = 1, GW_MS_ACT_DISABLE = 2, GW_MS_ACT_DOWNLOAD = 3, GW_MS_ACT_MISSING = 4 };
typedef struct gw_ms_action {
    int kind;
    char id[GW_MS_ID];
    char name[GW_MS_NAME];
    int post;                /* DOWNLOAD: the Nucleus post id */
    char why[112];           /* the player's words: "you need this mod: Name (from SSBM Nucleus)" ... */
} gw_ms_action;
typedef struct gw_ms_plan {
    int n;
    gw_ms_action a[GW_MS_MAX * 2];
    int ndownload, nenable, ndisable, nmissing;
    int can_complete;        /* 1 = nothing is MISSING */
    char summary[200];       /* one line for the button: "Download 1, turn on 0, turn off 1" or "Cannot match: 1 mod is not on Nucleus" */
} gw_ms_plan;
/* "Match the peer's mods": `mine` = every installed mod (enabled/mounted flags valid), `theirs` = the peer's mounted list. Only
 * identity-affecting mods (ident != 0) are touched; costume skins stay as they are. Rules, per peer mod T with ident:
 *   installed here, enabled, digest equal                 nothing
 *   installed here, off for next boot                     ENABLE (re-checked after the relaunch)
 *   installed here, enabled+mounted, digest differs       DOWNLOAD when T.source is "nucleus" and T.post > 0, else MISSING
 *   enabled but not mounted yet (a restart is pending)    nothing
 *   not installed                                         DOWNLOAD (nucleus + post) else MISSING
 * and for each mounted ident mod of ours that the peer lacks: DISABLE. */
void gw_ms_plan_build(const gw_ms_manifest *mine, const gw_ms_manifest *theirs, gw_ms_plan *out);
/* The target enabled set after the plan: ids of `mine`'s enabled mods + ENABLE/DOWNLOAD ids - DISABLE ids. Returns the count. */
int gw_ms_target_ids(const gw_ms_manifest *mine, const gw_ms_plan *plan, char ids[][GW_MS_ID], int cap);

/* ---- the journal (files under <mods dir>/.modsync/) ------------------------------------------------------------------ */
enum { GW_MS_J_NONE = 0, GW_MS_J_DOWNLOADING = 1, GW_MS_J_APPLIED = 2, GW_MS_J_LAUNCHING = 3, GW_MS_J_ACTIVE = 4 };
typedef struct gw_ms_journal {
    int state;
    char room[8];            /* the room code to rejoin ("" none) */
    int64_t t;               /* unix seconds of the last state change */
    uint32_t pid;            /* ACTIVE: the process that owns the session */
} gw_ms_journal;
int gw_ms_j_read(const char *mods_dir, gw_ms_journal *j);   /* 0 = read (state NONE when there is no journal), -1 = unreadable (treated as NONE) */
/* Start a session: save the player's own enabled set (orig_ids, always an explicit list: with no enabled.txt the caller passes every
 * installed mod) to .modsync/enabled.orig, state DOWNLOADING. Fails (-1) when a session already exists. */
int gw_ms_j_begin(const char *mods_dir, const char *room, const char *const *orig_ids, int n, int64_t now);
int gw_ms_j_set(const char *mods_dir, int state, uint32_t pid, int64_t now);  /* change the state atomically */
int gw_ms_write_enabled(const char *mods_dir, const char *const *ids, int n);  /* enabled.txt, atomically (tmp + replace) */
/* End a session: put the saved enabled set back into enabled.txt, then remove the journal. Idempotent and crash-safe in that order
 * (a crash between the two steps repeats the same restore at the next boot). Returns 0 when nothing is left behind. */
int gw_ms_j_restore(const char *mods_dir);
int gw_ms_j_keep(const char *mods_dir);             /* "keep these mods": drop the journal, leave enabled.txt as the sync set it */
/* The boot decision, before enabled.txt is read. DOWNLOADING -> drop the journal (nothing was applied). APPLIED -> restore (the
 * game ended before it relaunched). LAUNCHING -> continue when it is no older than 180 s (state ACTIVE, owner my_pid, room filled),
 * else restore. ACTIVE -> restore when the owning process is gone (pid_alive==0), else leave it alone (another running game owns it).
 * Returns 0 nothing to do / left alone, 1 continued (an active session), 2 restored. pid_alive may be NULL (= not alive). */
int gw_ms_boot(const char *mods_dir, int64_t now, uint32_t my_pid, int (*pid_alive)(uint32_t), char *room_out, int room_cap);

/* ---- the live session (what the room screens and the Lua API read and drive) ----------------------------------------------- */
enum { GW_MS_PH_IDLE = 0, GW_MS_PH_DOWNLOADING = 1, GW_MS_PH_APPLYING = 2, GW_MS_PH_RELAUNCH = 3, GW_MS_PH_FAILED = 4 };
int gw_ms_phase(void);
const char *gw_ms_status(void);                      /* one line for the room: "Downloading 1 of 2 (40%)", "Cannot match: ...", ... */
int gw_ms_session_active(void);                      /* 1 = this process runs a sync session (the mounted set is the host's, temporary) */
const char *gw_ms_session_room(void);                /* the room code recorded for the rejoin, "" none */
int gw_ms_start(const char *room_code);              /* "Match the host's mods": build the plan from the live manifests, refuse (-1) when it cannot complete or no peer list; else begin the journal and start the downloads; 0 = started */
void gw_ms_tick(void);                               /* once a frame: advances DOWNLOADING -> APPLYING -> RELAUNCH */
void gw_ms_cancel(void);                             /* abandon a sync that has not relaunched: restore, forget */
int gw_ms_keep(void);                                /* the player keeps the synced set: drop the journal */
void gw_ms_end_session(void);                        /* the room was left: restore the player's own enabled set (the mounted set stays until the next boot) */
int gw_ms_plan_view(const gw_ms_plan **plan, const gw_ms_row **rows, int *nrows);  /* the last computed plan/rows for the peer, 1 when there is one */

void gw_modsync_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_MODSYNC_H */
