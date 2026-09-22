/* gw_replay.c - Slippi (.slp) playback: MELEE_SLP=<file>.
 *
 * Plays a Slippi replay through the port's own game code and, optionally, records what the port
 * computes so it can be diffed against what the console computed (tools/replay/replay_compare.py).
 *
 *   MELEE_SLP=C:/path/game.slp               play it
 *   MELEE_STATE_TRACE=trace.csv              also write the port's per-frame post-frame state
 *   MELEE_SLP_RESYNC=1                       Slippi's resync mode: restore the RNG seed each frame
 *
 * WHAT A REPLAY CARRIES, AND WHAT IS DONE WITH IT. This mirrors Slippi's own playback codes
 * (project-slippi/slippi-ssbm-asm, Playback/Core), mapped onto the decomp:
 *
 *   Game Start (0x36) +0x5, 0x138 bytes: the StartMeleeData the console started the match with -
 *     rules plus six PlayerInitData. RestoreGameInfo.asm memcpy's it over the match's own at
 *     0x8016E748, i.e. inside fn_8016E730 (gmvs.c) - gw_Replay_ApplyMatch here. The rules block's
 *     callback pointers are console addresses and are NOT copied; the port's own stay.
 *   Game Start +0x13D: the RNG seed, written to the game's seed at the same point.
 *   Pre-Frame (0x37), per frame per character: the PROCESSED inputs - sticks, C-stick, trigger and
 *     buttons as the fighter holds them after deadzones and the L/R/Z folding. RestoreGameFrame.asm
 *     writes them into the fighter (fp+0x620..0x65C) at 0x8006B0DC, inside
 *     Fighter_Spaghetti_8006AD10 just before pressed/released are derived - the same point here.
 *     Recording and playback both use this point, so pad sampling, the pad queue and controller
 *     calibration are out of the loop entirely.
 *
 * FRAMES. Slippi numbers a match's frames from -123 (the first frame fighters run) and the timer
 * starts at 0. gw_Replay_ApplyMatch arms the counter, and gw_Replay_Tick advances it once per
 * logic frame that actually runs the fighters - the loading hold's frozen frames do not count.
 *
 * The boundary with game code is pull-only and scalar-only (see gmscene.c's overlay comment):
 * game TUs byte-swap their own memory accesses, so values cross by return value, never by a host
 * write through a game pointer. The one exception is the match struct, a byte image in console
 * byte order on both sides - exactly what memcpy is right for.
 */
#include "gw.h"
#include "gw_rollback.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define GW_RP_GAME_INFO 0x138u
#define GW_RP_SLOTS 8 /* 4 ports x (leader, follower) */
#define GW_RP_FIRST_FRAME (-123)
#define GW_RP_UNARMED (-0x7FFFFFFF)

/* the replay's per-frame input and the rollback session's are one type (gw_rollback.h) */
typedef GwRbInput GwRpInput;

/* trace buffering for a rollback session (defined below, before the trace functions' users) */
static FILE *gw_seedf, *gw_randf;
static void gw_tb_printf(int which, FILE *plain, const char *fmt, ...);

static struct {
    int tried;
    int active;
    int live; /* netplay: armed by a live match, no recording behind it (see rp_load) */
    uint8_t game_info[GW_RP_GAME_INFO];
    uint32_t seed;
    uint8_t version[4];
    int first, last; /* frame range with pre-frame data */
    GwRpInput *in;   /* (last - first + 1) * GW_RP_SLOTS */
    uint32_t *fs_seed; /* Frame Start (0x3A) seed per frame, Slippi 2.2+ */
    uint8_t *fs_has;
    int seed_diverged; /* first frame the port's seed left the console's, logged once */
    int frame;       /* current Slippi frame, GW_RP_UNARMED before the match */
    uint32_t ucf_dashback[4]; /* Game Start per-port UCF toggles: 0 off, 1 UCF, 2 arduino */
    uint32_t ucf_shield[4];
    int resync;
    int frozen_ps; /* Game Start Frozen PS (2.0+) */
    int online; /* Game Start major scene 8: Slippi online, which forces the seed every frame */
    FILE *trace;
    FILE *vel; /* <trace>.vel.csv: the velocities Slippi 3.5+ post-frame records */
    char scene[256];
} rp = { .frame = GW_RP_UNARMED };

static const GwRpInput *rp_cur(int port, int follower);

static uint32_t rp_be32(const uint8_t *p) {
    return ((uint32_t) p[0] << 24) | ((uint32_t) p[1] << 16) | ((uint32_t) p[2] << 8) | p[3];
}
static float rp_bef(const uint8_t *p) {
    uint32_t u = rp_be32(p);
    float f;
    memcpy(&f, &u, 4);
    return f;
}

/* The `raw` UBJSON array of a .slp: `raw` `[` `$` `U` `#` `l` <u32 length> <events>. */
static int rp_parse(const uint8_t *d, size_t n) {
    size_t i, pos, end;
    uint16_t sizes[256];
    int pass;
    memset(sizes, 0, sizeof sizes);
    for (i = 0; i + 12 < n; ++i) {
        if (memcmp(d + i, "raw[$U#l", 8) == 0) {
            break;
        }
    }
    if (i + 12 >= n) {
        gw_log("replay: no raw event block");
        return -1;
    }
    pos = i + 12;
    end = pos + rp_be32(d + i + 8);
    if (end > n || end == pos) {
        end = n; /* a replay still being written has length 0 */
    }
    if (d[pos] != 0x35) {
        gw_log("replay: raw block does not start with the event-size table");
        return -1;
    }
    for (i = 0; i + 3 <= (size_t) d[pos + 1] - 1; i += 3) {
        sizes[d[pos + 2 + i]] = (uint16_t) ((d[pos + 3 + i] << 8) | d[pos + 4 + i]);
    }
    /* two passes: the frame range, then the inputs */
    for (pass = 0; pass < 2; ++pass) {
        size_t c = pos + 1 + d[pos + 1];
        while (c < end) {
            uint8_t cmd = d[c];
            size_t sz = sizes[cmd];
            const uint8_t *b = d + c;
            if (sz == 0 && cmd != 0x35) {
                break;
            }
            if (c + 1 + sz > end) {
                break;
            }
            if (cmd == 0x36 && pass == 0) {
                memcpy(rp.version, b + 1, 4);
                memcpy(rp.game_info, b + 5, GW_RP_GAME_INFO);
                rp.seed = rp_be32(b + 0x13D);
                rp.online = sz >= 0x1A4 && b[0x1A4] == 8; /* major scene, 3.7.0+ */
                rp.frozen_ps = sz >= 0x1A2 && b[0x1A2] != 0; /* 2.0.0+ */
                if (sz >= 0x160) { /* 1.0.0+ */
                    int p;
                    for (p = 0; p < 4; ++p) {
                        rp.ucf_dashback[p] = rp_be32(b + 0x141 + 8 * p);
                        rp.ucf_shield[p] = rp_be32(b + 0x145 + 8 * p);
                    }
                }
            } else if (cmd == 0x3A && pass == 1) {
                int frame = (int) rp_be32(b + 1);
                if (frame >= rp.first && frame <= rp.last) {
                    rp.fs_seed[frame - rp.first] = rp_be32(b + 5);
                    rp.fs_has[frame - rp.first] = 1;
                }
            } else if (cmd == 0x37) {
                int frame = (int) rp_be32(b + 1);
                int port = b[5], fol = b[6] != 0;
                if (pass == 0) {
                    if (rp.first == 0 && rp.last == 0 && rp.in == NULL) {
                        rp.first = rp.last = frame;
                    }
                    if (frame < rp.first) rp.first = frame;
                    if (frame > rp.last) rp.last = frame;
                } else if (port < 4) {
                    GwRpInput *r = &rp.in[(frame - rp.first) * GW_RP_SLOTS + port * 2 + fol];
                    r->seed = rp_be32(b + 0x7);
                    r->lx = rp_bef(b + 0x19);
                    r->ly = rp_bef(b + 0x1D);
                    r->cx = rp_bef(b + 0x21);
                    r->cy = rp_bef(b + 0x25);
                    r->trigger = rp_bef(b + 0x29);
                    r->buttons = rp_be32(b + 0x2D);
                    /* the raw bytes UCF reads, each recorded from a later Slippi version on
                       (x 1.2.0, y 3.15.0, c-stick 3.17.0); absent ones read 0, as in Slippi's
                       own playback of an older replay */
                    r->raw[0] = sz >= 0x3B ? (int8_t) b[0x3B] : 0;
                    r->raw[1] = sz >= 0x40 ? (int8_t) b[0x40] : 0;
                    r->raw[2] = sz >= 0x41 ? (int8_t) b[0x41] : 0;
                    r->raw[3] = sz >= 0x42 ? (int8_t) b[0x42] : 0;
                    r->present = 1;
                }
            }
            c += 1 + sz;
        }
        if (pass == 0) {
            size_t count = (size_t) (rp.last - rp.first + 1) * GW_RP_SLOTS;
            rp.in = (GwRpInput *) calloc(count, sizeof *rp.in);
            rp.fs_seed = (uint32_t *) calloc((size_t) (rp.last - rp.first + 1), 4);
            rp.fs_has = (uint8_t *) calloc((size_t) (rp.last - rp.first + 1), 1);
            if (rp.in == NULL || rp.fs_seed == NULL || rp.fs_has == NULL) {
                return -1;
            }
        }
    }
    return 0;
}

static void rp_load(void) {
    const char *path;
    FILE *f;
    long n;
    uint8_t *d;
    if (rp.tried) {
        return;
    }
    rp.tried = 1;
    path = getenv("MELEE_SLP");
    if (path == NULL || path[0] == '\0') {
        return;
    }
    f = fopen(path, "rb");
    if (f == NULL) {
        gw_log("replay: cannot open MELEE_SLP=\"%s\"", path);
        return;
    }
    fseek(f, 0, SEEK_END);
    n = ftell(f);
    fseek(f, 0, SEEK_SET);
    d = (uint8_t *) malloc((size_t) n);
    if (d == NULL || fread(d, 1, (size_t) n, f) != (size_t) n) {
        fclose(f);
        free(d);
        return;
    }
    fclose(f);
    if (rp_parse(d, (size_t) n) == 0) {
        const uint8_t *gi = rp.game_info;
        int p, len;
        rp.active = 1;
        rp.resync = getenv("MELEE_SLP_RESYNC") != NULL && getenv("MELEE_SLP_RESYNC")[0] == '1';
        /* the scene launcher takes it from here: a VS match straight in, these fighters, this stage */
        len = snprintf(rp.scene, sizeof rp.scene, "mode=vs;at=match;stage=ext:%u",
                       (unsigned) ((gi[0x0E] << 8) | gi[0x0F]));
        for (p = 0; p < 4; ++p) {
            const uint8_t *pl = gi + 0x60 + 0x24 * p;
            if (pl[1] == 3) { /* player type 3 = none */
                continue;
            }
            len += snprintf(rp.scene + len, sizeof rp.scene - (size_t) len, ";p%d=ck:%u/c%u/%s",
                            p + 1, pl[0], pl[3], pl[1] == 1 ? "cpu" : "hu");
        }
        gw_log("replay: %s - Slippi %u.%u.%u, frames %d..%d, seed 0x%08X, scene \"%s\"%s%s", path,
               rp.version[0], rp.version[1], rp.version[2], rp.first, rp.last, rp.seed, rp.scene,
               rp.online ? ", online (per-frame seed)" : "", rp.resync ? ", resync on" : "");
    }
    free(d);
    path = getenv("MELEE_STATE_TRACE");
    if (rp.active && path != NULL && path[0] != '\0') {
        rp.trace = fopen(path, "w");
        if (rp.trace != NULL) {
            char vp[600];
            fprintf(rp.trace, "frame,player,follower,char,action_state,x,y,facing,percent,stocks\n");
            snprintf(vp, sizeof vp, "%s.vel.csv", path);
            rp.vel = fopen(vp, "w");
            if (rp.vel != NULL) {
                fprintf(rp.vel, "frame,player,follower,air_x,air_y,kb_x,kb_y,ground_x\n");
            }
        }
    }
}

/* LIVE MODE (netplay, gw_netplay.c). The rollback session runs on this module's machinery - the
 * Slippi frame counter, the per-frame seed, the input accessors - so a live match arms it the way
 * a replay does, only with no recorded inputs and no last frame. The match itself comes from the
 * scene launcher; the seed from the host, in gw_Replay_ApplyMatch. It plays as Slippi online does:
 * the online codeset, UCF 0.84 on every port, and a seed forced every frame. Off again when the
 * match is over. */
void gw_Replay_ArmLive(int on) {
    int p;
    rp_load();
    if (rp.active && !rp.live) {
        return; /* a replay is playing: leave it */
    }
    if (!on) {
        rp.active = 0;
        rp.live = 0;
        rp.frame = GW_RP_UNARMED;
        return;
    }
    rp.active = 1;
    rp.live = 1;
    rp.first = GW_RP_FIRST_FRAME;
    rp.last = 0x3FFFFFFF;
    rp.online = 1;
    rp.version[0] = 3;
    rp.version[1] = 19;
    rp.version[2] = 1;
    for (p = 0; p < 4; ++p) {
        rp.ucf_dashback[p] = 1;
        rp.ucf_shield[p] = 1;
    }
    rp.scene[0] = ' ';
    rp.frame = GW_RP_UNARMED;
    gw_log("replay: live mode (netplay) - frames counted from %d, online codes, UCF 0.84", rp.first);
}

/* For gw_sl_load (gw_runtime.c): the scene a replay implies, or NULL. */
const char *gw_replay_scene(void) {
    rp_load();
    return rp.active && !rp.live ? rp.scene : NULL;
}

int gw_Replay_Active(void) {
    rp_load();
    return rp.active;
}

/* fn_8016E730 (gmvs.c), at the point RestoreGameInfo.asm hooks: overwrite the match struct with
 * the replay's, keeping the port's own callback pointers (rules +0x38..+0x5B), and arm the frame
 * counter. Returns the seed for the caller to store. */
uint32_t gw_Replay_ApplyMatch(void *start_melee_data) {
    uint8_t *d = (uint8_t *) start_melee_data;
    uint8_t keep[0x5C - 0x38];
    if (!gw_Replay_Active()) {
        return 0;
    }
    if (rp.live) {
        /* netplay: connect here, before the first frame. The host sends its match struct and a
           seed; the guest takes both (keeping its own callback pointers, as for a replay). */
        extern uint32_t gw_Netplay_Handshake(uint8_t *start_melee_data, int len, int keep_off,
                                             int keep_len);
        {
            /* Netplay needs port p to be player p on controller p: PlayerInitData +4 (`slot`,
               player id + 1: the P1..P4 tag) and +7 (`sub_color`, really the controller index).
               The scene launcher sets both now; a match set up any other way is corrected. */
            int p;
            for (p = 0; p < 4; ++p) {
                uint8_t *pl = d + 0x60 + 0x24 * p;
                if (pl[1] != 3 && (pl[4] != (uint8_t) (p + 1) || pl[7] != (uint8_t) p)) {
                    gw_log("replay: live match - player %d had player slot %d / controller %d, now "
                           "%d / %d", p + 1, pl[4], pl[7], p + 1, p);
                    pl[4] = (uint8_t) (p + 1);
                    pl[7] = (uint8_t) p;
                }
            }
        }
        rp.seed = gw_Netplay_Handshake(d, GW_RP_GAME_INFO, 0x38, (int) sizeof keep);
        memcpy(rp.game_info, d, GW_RP_GAME_INFO);
        rp.frame = GW_RP_FIRST_FRAME - 1;
        gw_log("replay: live match armed, seed 0x%08X", rp.seed);
        return rp.seed;
    }
    memcpy(keep, d + 0x38, sizeof keep);
    memcpy(d, rp.game_info, GW_RP_GAME_INFO);
    memcpy(d + 0x38, keep, sizeof keep);
    rp.frame = GW_RP_FIRST_FRAME - 1;
    gw_log("replay: match struct restored, seed 0x%08X", rp.seed);
    return rp.seed;
}


/* ---- recording: MELEE_SLP_RECORD=<file> ----------------------------------------------------
 * Writes the port's own match as a Slippi replay, from the same points playback reads: the
 * StartMeleeData and seed at fn_8016E730, each frame's start seed, each character's processed
 * inputs at the pre-frame point, and the post-frame state at Slippi's post-frame point. Event sizes
 * and framing are Slippi 3.19.1's, so the file is also readable by Slippi's tools; fields the port
 * has no source for (raw pad bytes, shield size, combo counters...) are zero.
 *
 * A port recording played back on the same build must reproduce every frame bit for bit - that is
 * the self-consistency test rollback rests on (tools/replay/first_div.py against the recording).
 * The raw-block length is written as 0 up front (Slippi's own convention for a replay still being
 * written) and patched at exit, so a run that dies mid-match still leaves a playable file. */
#define GW_RP_SZ_START 0x2F8
#define GW_RP_SZ_PRE 0x42
#define GW_RP_SZ_POST 0x54
#define GW_RP_SZ_END 0x6
#define GW_RP_SZ_FSTART 0xC

static struct {
    int tried;
    FILE *f;
    long raw_len_at;  /* file offset of the raw block's u32 length */
    long raw_start;   /* file offset of the first event byte */
    int frame;        /* the recording's own frame counter, GW_RP_UNARMED before the match */
    uint32_t seed;    /* this frame's start seed, repeated in each pre-frame */
} rec = { .frame = GW_RP_UNARMED };

static void rec_be32(uint8_t *p, uint32_t v) {
    p[0] = (uint8_t) (v >> 24); p[1] = (uint8_t) (v >> 16); p[2] = (uint8_t) (v >> 8);
    p[3] = (uint8_t) v;
}
static void rec_bef(uint8_t *p, float f) {
    uint32_t u;
    memcpy(&u, &f, 4);
    rec_be32(p, u);
}

static void rec_close(void) {
    long end;
    uint8_t ev[1 + GW_RP_SZ_END], len[4];
    static const char meta[] = "U\x08metadata{U\x08playedOnSU\x08melee-pc}}";
    if (rec.f == NULL) {
        return;
    }
    memset(ev, 0, sizeof ev);
    ev[0] = 0x39;
    ev[1] = 7; /* game end method: no contest - the port does not know how the run ended */
    fwrite(ev, 1, sizeof ev, rec.f);
    end = ftell(rec.f);
    fwrite(meta, 1, sizeof meta - 1, rec.f);
    rec_be32(len, (uint32_t) (end - rec.raw_start));
    fseek(rec.f, rec.raw_len_at, SEEK_SET);
    fwrite(len, 1, 4, rec.f);
    fclose(rec.f);
    rec.f = NULL;
    gw_log("replay: recording closed (%ld event bytes)", end - rec.raw_start);
}

int gw_Replay_Recording(void) {
    if (!rec.tried) {
        const char *path = getenv("MELEE_SLP_RECORD");
        rec.tried = 1;
        if (path != NULL && path[0] != '\0') {
            rec.f = fopen(path, "wb");
            if (rec.f == NULL) {
                gw_log("replay: cannot open MELEE_SLP_RECORD=\"%s\"", path);
            } else {
                static const uint8_t head[] = { '{', 'U', 3, 'r', 'a', 'w', '[', '$', 'U', '#', 'l' };
                static const uint8_t sizes[] = { 0x35, 16,
                    0x36, GW_RP_SZ_START >> 8, GW_RP_SZ_START & 0xFF,
                    0x37, 0, GW_RP_SZ_PRE, 0x38, 0, GW_RP_SZ_POST,
                    0x39, 0, GW_RP_SZ_END, 0x3A, 0, GW_RP_SZ_FSTART };
                uint8_t zero[4] = { 0, 0, 0, 0 };
                fwrite(head, 1, sizeof head, rec.f);
                rec.raw_len_at = ftell(rec.f);
                fwrite(zero, 1, 4, rec.f);
                rec.raw_start = ftell(rec.f);
                fwrite(sizes, 1, sizeof sizes, rec.f);
                atexit(rec_close);
                gw_log("replay: recording to %s", path);
            }
        }
    }
    return rec.f != NULL;
}

/* fn_8016E730: the match as it starts, and its seed. Arms the recording's frame counter. */
void gw_Replay_RecordMatch(void *start_melee_data, uint32_t seed) {
    uint8_t ev[1 + GW_RP_SZ_START];
    if (!gw_Replay_Recording()) {
        return;
    }
    memset(ev, 0, sizeof ev);
    ev[0] = 0x36;
    ev[1] = 3; ev[2] = 19; ev[3] = 1; ev[4] = 0;
    memcpy(ev + 5, start_melee_data, GW_RP_GAME_INFO);
    rec_be32(ev + 0x13D, seed);
    ev[0x1A3] = 2;    /* minor scene: in-game */
    ev[0x1A4] = 2;    /* major scene: VS */
    if (rp.active) {
        /* recording while playing a replay back: carry what playback itself consumes, so the
           port's recording replays exactly as the original did - its Slippi version (UCF gating),
           per-port UCF toggles, and online-ness (per-frame seeds, carried by Frame Start) */
        int p;
        memcpy(ev + 1, rp.version, 4);
        for (p = 0; p < 4; ++p) {
            rec_be32(ev + 0x141 + 8 * p, rp.ucf_dashback[p]);
            rec_be32(ev + 0x145 + 8 * p, rp.ucf_shield[p]);
        }
        ev[0x1A4] = rp.online ? 8 : 2;
    }
    fwrite(ev, 1, sizeof ev, rec.f);
    rec.frame = GW_RP_FIRST_FRAME - 1;
    gw_log("replay: recording the match (seed 0x%08X)", seed);
}

static void rec_tick(uint32_t seed) {
    uint8_t ev[1 + GW_RP_SZ_FSTART];
    if (rec.f == NULL || rec.frame == GW_RP_UNARMED) {
        return;
    }
    fflush(rec.f); /* a killed run loses at most the frame in flight */
    ++rec.frame;
    rec.seed = seed;
    memset(ev, 0, sizeof ev);
    ev[0] = 0x3A;
    rec_be32(ev + 1, (uint32_t) rec.frame);
    rec_be32(ev + 5, seed);
    fwrite(ev, 1, sizeof ev, rec.f);
}

void gw_Replay_RecordInput(int port, int follower, float lx, float ly, float cx, float cy,
                           float trigger, uint32_t buttons, int action, float x, float y,
                           float facing, float percent) {
    uint8_t ev[1 + GW_RP_SZ_PRE];
    if (rec.f == NULL || rec.frame == GW_RP_UNARMED) {
        return;
    }
    memset(ev, 0, sizeof ev);
    ev[0] = 0x37;
    rec_be32(ev + 1, (uint32_t) rec.frame);
    ev[5] = (uint8_t) port;
    ev[6] = (uint8_t) (follower != 0);
    rec_be32(ev + 0x7, rec.seed);
    ev[0xB] = (uint8_t) (action >> 8); ev[0xC] = (uint8_t) action;
    rec_bef(ev + 0xD, x); rec_bef(ev + 0x11, y); rec_bef(ev + 0x15, facing);
    rec_bef(ev + 0x19, lx); rec_bef(ev + 0x1D, ly);
    rec_bef(ev + 0x21, cx); rec_bef(ev + 0x25, cy);
    rec_bef(ev + 0x29, trigger);
    rec_be32(ev + 0x2D, buttons);
    rec_bef(ev + 0x3C, percent);
    {
        const GwRpInput *r = rp_cur(port, follower); /* the raw bytes UCF read, when playing back */
        if (r != NULL) {
            ev[0x3B] = (uint8_t) r->raw[0];
            ev[0x40] = (uint8_t) r->raw[1];
            ev[0x41] = (uint8_t) r->raw[2];
            ev[0x42] = (uint8_t) r->raw[3];
        }
    }
    fwrite(ev, 1, sizeof ev, rec.f);
}

static void rec_post(int port, int follower, int ckind, int action, float x, float y, float facing,
                     float percent, int stocks, float air_x, float air_y, float kb_x, float kb_y,
                     float ground_x) {
    uint8_t ev[1 + GW_RP_SZ_POST];
    if (rec.f == NULL || rec.frame == GW_RP_UNARMED) {
        return;
    }
    memset(ev, 0, sizeof ev);
    ev[0] = 0x38;
    rec_be32(ev + 1, (uint32_t) rec.frame);
    ev[5] = (uint8_t) port;
    ev[6] = (uint8_t) (follower != 0);
    ev[7] = (uint8_t) ckind;
    ev[8] = (uint8_t) (action >> 8); ev[9] = (uint8_t) action;
    rec_bef(ev + 0xA, x); rec_bef(ev + 0xE, y); rec_bef(ev + 0x12, facing);
    rec_bef(ev + 0x16, percent);
    ev[0x21] = (uint8_t) stocks;
    rec_bef(ev + 0x35, air_x); rec_bef(ev + 0x39, air_y);
    rec_bef(ev + 0x3D, kb_x); rec_bef(ev + 0x41, kb_y);
    rec_bef(ev + 0x45, ground_x);
    fwrite(ev, 1, sizeof ev, rec.f);
}

/* Playback or recording: the scene loop's per-frame hooks run for either. */
int gw_Replay_Enabled(void) {
    return gw_Replay_Active() || gw_Replay_Recording();
}

/* Once per logic frame that runs the fighters. Returns the frame now running. */
int gw_Replay_Tick(void) {
    if (!rp.active || rp.frame == GW_RP_UNARMED) {
        return GW_RP_UNARMED;
    }
    ++rp.frame;
    if ((rp.frame & 63) == 0) {
        /* the harness ends a run by killing it: without this a trace loses its buffered tail,
           and a shorter trace than the other run reads like a divergence */
        fflush(NULL);
    }
    if (rp.frame == rp.last + 1) {
        gw_log("replay: past the replay's last frame (%d)", rp.last);
        if (rp.trace != NULL) {
            fflush(rp.trace);
        }
    }
    return rp.frame;
}

static const GwRpInput *rp_cur(int port, int follower) {
    const GwRpInput *r;
    if (gw_rb_active()) {
        /* a rollback session decides what each fighter reads: confirmed or predicted inputs */
        return gw_RB_InputFor(port, follower, rp.frame);
    }
    if (!rp.active || rp.live || rp.frame < rp.first || rp.frame > rp.last || port < 0 || port > 3) {
        return NULL;
    }
    r = &rp.in[(rp.frame - rp.first) * GW_RP_SLOTS + port * 2 + (follower != 0)];
    return r->present ? r : NULL;
}

int gw_Replay_HasInput(int port, int follower) { return rp_cur(port, follower) != NULL; }
float gw_Replay_StickX(int port, int follower) { return rp_cur(port, follower)->lx; }
float gw_Replay_StickY(int port, int follower) { return rp_cur(port, follower)->ly; }
float gw_Replay_CStickX(int port, int follower) { return rp_cur(port, follower)->cx; }
float gw_Replay_CStickY(int port, int follower) { return rp_cur(port, follower)->cy; }
float gw_Replay_Trigger(int port, int follower) { return rp_cur(port, follower)->trigger; }
uint32_t gw_Replay_Buttons(int port, int follower) { return rp_cur(port, follower)->buttons; }

/* The seed the console had at the start of the frame now running (Frame Start, 0x3A), and whether
 * the replay has one. */
static int rp_frame_seed(uint32_t *out) {
    if (!rp.active || rp.live || rp.frame < rp.first || rp.frame > rp.last ||
        !rp.fs_has[rp.frame - rp.first]) {
        return 0;
    }
    *out = rp.fs_seed[rp.frame - rp.first];
    return 1;
}

/* The seed to force at the start of this frame, or 0 to leave it.
 *
 * Frame -123 always gets the Game Start seed: it is recorded AFTER match setup's own draws (on
 * Fountain of Dreams, grIzumi_801CC358 x2, ftCo_800A101C x2 and ftCo_800B9704 x2 - six), so the
 * early restore in fn_8016E730 alone leaves the port six draws past the console by the first frame.
 * Slippi's playback restores it twice for exactly this reason: RestoreGameInfo.asm at 0x8016E748,
 * then RestoreInitialRNG.s from a proc created at match start, before anything animates.
 * Under MELEE_SLP_RESYNC every later frame gets the console's own Frame Start seed too. */
uint32_t gw_Replay_ResyncSeed(void) {
    uint32_t s;
    if (rp.live) {
        /* Slippi online's rule: every frame starts from seed + ((frame + 123) << 16). Both peers
           force it, so an RNG draw that only one of them makes (render-side effects) cannot carry
           into the next frame. */
        if (rp.frame == GW_RP_UNARMED) {
            return 0;
        }
        s = rp.seed + ((uint32_t) (rp.frame - GW_RP_FIRST_FRAME) << 16);
        return s != 0 ? s : 1;
    }
    if (rp.active && rp.frame == GW_RP_FIRST_FRAME) {
        /* A replay without Frame Start events (Slippi < 2.2) records its seed in each PRE-frame
         * instead. The first one is the seed the console actually entered frame -123 with: on the
         * 2019 corpus it is the Game Start seed advanced 11 times (setup draws the port makes only
         * 5-6 of), so the Game Start seed would leave the port's RNG behind from frame one and every
         * later random pick (DamageFlyTop, tech direction...) would go the other way. */
        const GwRpInput *r0 = rp_cur(0, 0);
        int p;
        for (p = 0; p < 4 && r0 == NULL; ++p) {
            r0 = rp_cur(p, 0);
        }
        if (r0 != NULL && !rp.fs_has[0] && r0->seed != 0) {
            return r0->seed;
        }
        return rp.seed;
    }
    /* Slippi online does not let the RNG run on: every frame starts from
     * seed + ((frame + 123) << 16) (Frame Start, 0x3A, carries it - 0x0000F8BC, 0x0001F8BC, ...).
     * So for an online replay the console itself forced this seed each frame, and restoring it is
     * reproduction, not resync. */
    if ((rp.online || rp.resync) && rp_frame_seed(&s)) {
        return s;
    }
    /* No Frame Start events (Slippi < 2.2): each frame's PRE-frame seed carries the same
     * information, so resync from it - the way an online replay's forced per-frame seed is
     * reproduced above. The port's RNG stream cannot be kept in step by construction here: the
     * particle system draws tens of values per frame for effects, and a difference of a few draws
     * per frame (measured: 21 console vs 29 port at frame -106 of a Battlefield replay, from
     * hsd_8039DAD4/hsd_8039EE24 spawn effects) shifts every later random pick - DamageFlyTop, techs,
     * CPU choices - for the rest of the game, starting a chain of unrelated divergences. Resyncing
     * once per frame keeps those picks aligned; the first frame the un-resynced stream left the
     * console's is logged (gw_Replay_CheckSeed), so a real cause stays visible. */
    if (rp.active && !rp.live && !rp.fs_has[0] && rp.frame > GW_RP_FIRST_FRAME) {
        int p;
        for (p = 0; p < 4; ++p) {
            const GwRpInput *r = rp_cur(p, 0);
            if (r != NULL && r->seed != 0) {
                return r->seed;
            }
        }
    }
    return 0;
}

/* The RNG is the earliest thing to diverge: any difference in what consumed it shows here frames
 * before a position does. Called at the start of each running frame with the port's seed. */
/* The seed the port's stream reached on its own, before any per-frame resync: compared with the
 * console's for this frame, the first difference is where the port's RNG consumption first leaves
 * the console's - logged once, whether or not a resync then hides it from the rest of the run. */
void gw_Replay_NoteSeed(uint32_t arrived) {
    static int noted;
    uint32_t want = 0;
    if (!noted) {
        const GwRpInput *r = NULL;
        int p;
        for (p = 0; p < 4 && r == NULL; ++p) {
            const GwRpInput *c = rp_cur(p, 0);
            if (c != NULL && c->seed != 0) {
                r = c;
            }
        }
        want = rp_frame_seed(&want) ? want : (r != NULL ? r->seed : 0);
        if (want != 0 && rp.frame > GW_RP_FIRST_FRAME + 1 && arrived != want) {
            noted = 1;
            gw_log("replay: port RNG stream first left the console's at frame %d (port 0x%08X, "
                   "console 0x%08X)%s", rp.frame, arrived, want,
                   rp.online || !rp.fs_has[0] ? " - resynced per frame from here on" : "");
        }
    }
}

void gw_Replay_CheckSeed(uint32_t port_seed) {
    uint32_t want;
    extern int gw_Snap_Resimulating(void);
    static int seed_tried;
    if (!gw_rb_active() && gw_Snap_Resimulating()) {
        return;
    }
    rec_tick(port_seed);
    /* <trace>.seed.csv: the port's seed at the start of every frame, so two port runs of the same
       replay can be diffed for the first frame they part ways (port-vs-port determinism) */
    if (!seed_tried) {
        const char *tp = getenv("MELEE_STATE_TRACE");
        seed_tried = 1;
        if (tp != NULL && tp[0] != '\0') {
            char sp[600];
            snprintf(sp, sizeof sp, "%s.seed.csv", tp);
            gw_seedf = fopen(sp, "w");
        }
    }
    if (gw_seedf != NULL && rp.frame != GW_RP_UNARMED) {
        gw_tb_printf(2, gw_seedf, "%d,%08X\n", rp.frame, port_seed);
    }
    if (rp.seed_diverged || !rp_frame_seed(&want)) {
        return;
    }
    if (port_seed != want) {
        rp.seed_diverged = 1;
        gw_log("replay: RNG seed diverged at frame %d: port 0x%08X, console 0x%08X", rp.frame,
               port_seed, want);
    } else if (rp.frame == rp.last) {
        gw_log("replay: RNG seed matched the console on every frame through %d", rp.last);
    }
}

/* ---- trace buffering for a rollback session --------------------------------------------------
 * Without a session every trace row is written as it is produced. With one, a frame can be
 * simulated more than once (predicted, then corrected), so rows are buffered PER LOGIC ITERATION -
 * the session calls gw_Replay_TraceBeginIter(next) at the top of each - and only iterations whose
 * inputs are all confirmed are written (gw_Replay_TraceFlushUpTo), in order. A resimulated
 * iteration replaces its buffer. The file then holds exactly what a plain playback of the same
 * replay would have written, which is the session's acceptance test. Rows keep the frame number
 * they were produced under (rows drawn before the frame counter ticks carry the previous one, as
 * in the plain run). Four files: 0 trace, 1 vel, 2 seed, 3 rand. */
#define GW_TB_FILES 4
#define GW_TB_RING 64
typedef struct {
    int iter;
    char *buf[GW_TB_FILES];
    size_t len[GW_TB_FILES], cap[GW_TB_FILES];
} GwTbSlot;
static GwTbSlot gw_tb[GW_TB_RING];
static int gw_tb_cur = -0x7FFFFFFF, gw_tb_next = -0x7FFFFFFF;

static FILE *gw_tb_file(int which) {
    switch (which) {
    case 0: return rp.trace;
    case 1: return rp.vel;
    case 2: return gw_seedf;
    default: return gw_randf;
    }
}

static GwTbSlot *gw_tb_slot(int iter) {
    GwTbSlot *t = &gw_tb[(unsigned) iter % GW_TB_RING];
    if (t->iter != iter) {
        int k;
        for (k = 0; k < GW_TB_FILES; ++k) {
            t->len[k] = 0;
        }
        t->iter = iter;
    }
    return t;
}

void gw_Replay_TraceBeginIter(int iter) {
    GwTbSlot *t = gw_tb_slot(iter);
    int k;
    for (k = 0; k < GW_TB_FILES; ++k) {
        t->len[k] = 0; /* a resimulated iteration replaces its rows */
    }
    gw_tb_cur = iter;
    if (gw_tb_next == -0x7FFFFFFF) {
        gw_tb_next = iter;
    }
}

/* Write iterations gw_tb_next..iter (those still buffered), in order. */
void gw_Replay_TraceFlushUpTo(int iter) {
    int wrote = gw_tb_next <= iter && gw_tb_next != -0x7FFFFFFF;
    while (gw_tb_next <= iter && gw_tb_next != -0x7FFFFFFF) {
        GwTbSlot *t = &gw_tb[(unsigned) gw_tb_next % GW_TB_RING];
        if (t->iter == gw_tb_next) {
            int k;
            for (k = 0; k < GW_TB_FILES; ++k) {
                FILE *f = gw_tb_file(k);
                if (f != NULL && t->len[k] != 0) {
                    fwrite(t->buf[k], 1, t->len[k], f);
                }
                t->len[k] = 0;
            }
        }
        ++gw_tb_next;
    }
    if (wrote) {
        int k;
        for (k = 0; k < GW_TB_FILES; ++k) {
            FILE *f = gw_tb_file(k);
            if (f != NULL) {
                fflush(f); /* the harness ends a run by killing it */
            }
        }
    }
}

static void gw_tb_printf(int which, FILE *plain, const char *fmt, ...) {
    va_list ap;
    char line[512];
    int n;
    va_start(ap, fmt);
    if (!gw_rb_active()) {
        if (plain != NULL) {
            vfprintf(plain, fmt, ap);
        }
        va_end(ap);
        return;
    }
    n = vsnprintf(line, sizeof line, fmt, ap);
    va_end(ap);
    if (n > 0 && gw_tb_cur != -0x7FFFFFFF) {
        GwTbSlot *t = gw_tb_slot(gw_tb_cur);
        if (t->len[which] + (size_t) n + 1 > t->cap[which]) {
            size_t nc = t->cap[which] ? t->cap[which] * 2 : 4096;
            while (nc < t->len[which] + (size_t) n + 1) {
                nc *= 2;
            }
            t->buf[which] = (char *) realloc(t->buf[which], nc);
            t->cap[which] = nc;
        }
        memcpy(t->buf[which] + t->len[which], line, (size_t) n);
        t->len[which] += (size_t) n;
    }
}

/* Where this replay's Slippi recorded post-frame state: 1.x/2.x at Fighter_procMap's epilogue
 * (0x8006C5D8), 3.x at Fighter_UnkCallCameraCallback_8006D9EC's (0x8006DA34, after hits). */
int gw_Replay_TraceAtProcMap(void) {
    if (rp.active) {
        return rp.version[0] < 3;
    }
    return 0;
}

int gw_Replay_Tracing(void) {
    return (rp.trace != NULL && rp.frame != GW_RP_UNARMED) ||
           (rec.f != NULL && rec.frame != GW_RP_UNARMED);
}

void gw_Replay_TraceFighter(int port, int follower, int ckind, int action, float x, float y,
                            float facing, float percent, int stocks, float air_x, float air_y,
                            float kb_x, float kb_y, float ground_x) {
    extern int gw_Snap_Resimulating(void);
    if (!gw_rb_active() && gw_Snap_Resimulating()) {
        return; /* SyncTest re-running a frame already traced */
    }
    rec_post(port, follower, ckind, action, x, y, facing, percent, stocks, air_x, air_y, kb_x,
             kb_y, ground_x);
    if (rp.trace == NULL || rp.frame == GW_RP_UNARMED) {
        return;
    }
    gw_tb_printf(0, rp.trace, "%d,%d,%d,%d,%d,%.17g,%.17g,%.17g,%.17g,%d\n", rp.frame, port,
                 follower != 0, ckind, action, x, y, facing, percent, stocks);
    if (rp.vel != NULL) {
        gw_tb_printf(1, rp.vel, "%d,%d,%d,%.17g,%.17g,%.17g,%.17g,%.17g\n", rp.frame, port,
                     follower != 0, air_x, air_y, kb_x, kb_y, ground_x);
    }
    if (rp.frame > rp.last) {
        fflush(rp.trace);
    }
}

/* The replay's cursor, for savestates (gw_snap.c): a rollback must rewind which inputs the next
 * frame gets along with the game state. */
void gw_Replay_GetCursor(int out[4]) {
    out[0] = rp.frame;
    out[1] = rp.seed_diverged;
    out[2] = 0;
    out[3] = 0;
}

void gw_Replay_SetCursor(const int in[4]) {
    rp.frame = in[0];
    rp.seed_diverged = in[1];
}

/* The Slippi frame now running, or a large negative number before the match. */
int gw_Replay_Frame(void) { return rp.frame; }

/* UCF, as the replay's Game Start set it for this port (dashback option 1 = UCF), and whether this
 * replay's Slippi carried UCF 0.84's 1.0 cardinals. 0.84 is what Slippi 3.x installs; the 2019-era
 * 1.x recordings ran an older UCF without them. */
int gw_Replay_UcfCardinals(int port) {
    if (!rp.active || port < 0 || port > 3 || rp.ucf_dashback[port] != 1) {
        return 0;
    }
    return rp.version[0] >= 3;
}

/* This frame's raw stick byte (0 x, 1 y, 2 c-x, 3 c-y) for the port. */
int gw_Replay_RawStick(int port, int which) {
    const GwRpInput *r = gw_rb_active() ? gw_RB_InputAny(port, rp.frame) : rp_cur(port, 0);
    return r != NULL && which >= 0 && which < 4 ? r->raw[which] : 0;
}

/* Which UCF the recording console ran for this port: 0 off, 73, or 84 (Slippi's per-port Game
 * Start dashback toggle, 1 = UCF). Slippi's console codes shipped UCF 0.73 "Check for Toggle" from
 * Jan 2019 until 0.74 in Sep 2019 (slippi-ssbm-asm acc6f71, b89e160), 0.8 from Mar 2021 (bb86519)
 * and 0.84 with 3.x - so 1.x replays are 0.73. 2.x (0.74/0.8) is approximated as 0.73. */
int gw_Replay_UcfVersion(int port) {
    if (!rp.active || port < 0 || port > 3 || rp.ucf_dashback[port] != 1) {
        return 0;
    }
    if (rp.version[0] >= 3) {
        return 84;
    }
    {   /* MELEE_SLP_NO_UCF073=1: leave 0.73 out, for before/after measurements */
        static int off = -1;
        if (off < 0) {
            const char *v = getenv("MELEE_SLP_NO_UCF073");
            off = v != NULL && v[0] == '1';
        }
        return off ? 0 : 73;
    }
}

/* The port's raw stick byte `back` frames before the one running (0 = this frame), as the pad
 * queue held it - UCF's dashback compares this frame's raw X with the one two frames earlier. */
int gw_Replay_RawStickBack(int port, int which, int back) {
    const GwRpInput *r;
    int f = rp.frame - back;
    if (gw_rb_active()) {
        /* what the session USED for that frame - a predicted input's raw bytes, not the truth */
        r = gw_RB_InputAny(port, f);
        return r != NULL && which >= 0 && which < 4 ? r->raw[which] : 0;
    }
    if (!rp.active || rp.live || f < rp.first || f > rp.last || port < 0 || port > 3 || which < 0 ||
        which > 3) {
        return 0;
    }
    r = &rp.in[(f - rp.first) * GW_RP_SLOTS + port * 2];
    return r->present ? r->raw[which] : 0;
}

/* The replay's recorded input for (slot, frame): the "player" side of the session's fake network.
 * Returns 0 when the replay has none. */
int gw_Replay_PeekInput(int slot, int frame, GwRbInput *out) {
    const GwRpInput *r;
    if (!rp.active || rp.live || frame < rp.first || frame > rp.last || slot < 0 ||
        slot >= GW_RP_SLOTS) {
        return 0;
    }
    r = &rp.in[(frame - rp.first) * GW_RP_SLOTS + slot];
    if (!r->present) {
        return 0;
    }
    *out = *r;
    out->confirmed = 1;
    return 1;
}

/* The replay's frame range and whether it is armed - the session's fake network needs both. */
int gw_Replay_LastFrame(void) { return rp.active ? rp.last : 0; }

/* Live mode (netplay): whether it is on, and whether port p is a HUMAN player in the armed match
 * (StartMeleeData player type 0) - the session's slot list when there is no recording. */
int gw_Replay_Live(void) { return rp.live; }
int gw_Replay_PortHuman(int p) {
    if (!rp.active || p < 0 || p > 3) {
        return 0;
    }
    return rp.game_info[0x60 + 0x24 * p + 1] == 0;
}

/* <trace>.rand.csv: every RNG draw while a replay is armed - frame, caller (native return address),
 * seed after, and whether it drew the global seed or a redirected one (HSD_RandSeedPtr). */
/* Set by gmscene.c around the render section of the scene loop. */
static int gw_det_in_render;
void gw_Det_SetInRender(int on) { gw_det_in_render = on; }

void gw_Replay_RandTrace(uint32_t caller, uint32_t seed_after, int32_t global) {
    static int tried;
    extern int gw_Snap_Resimulating(void);
    if (!gw_rb_active() && gw_Snap_Resimulating()) {
        return;
    }
    if (gw_det_in_render && global) {
        /* render code drawing from the GLOBAL seed: simulation state consumed by rendering, so a
           frame that renders and one that does not (catch-up, rollback resimulation) diverge.
           Logged once per caller; resolve the address against melee-pc.map. */
        static uint32_t seen[64];
        static int nseen;
        int k;
        for (k = 0; k < nseen && seen[k] != caller; ++k) {
        }
        if (k == nseen && nseen < 64) {
            seen[nseen++] = caller;
            gw_log("det: RNG draw during render from 0x%08X (frame %d)", caller, rp.frame);
        }
    }
    if (!rp.active || rp.frame == GW_RP_UNARMED) {
        return;
    }
    if (!tried) {
        const char *tp = getenv("MELEE_STATE_TRACE");
        tried = 1;
        if (tp != NULL && tp[0] != '\0') {
            char p[600];
            snprintf(p, sizeof p, "%s.rand.csv", tp);
            gw_randf = fopen(p, "w");
        }
    }
    if (gw_randf != NULL) {
        gw_tb_printf(3, gw_randf, "%d,%08X,%08X,%d\n", rp.frame, caller, seed_after, (int) global);
    }
}

/* MELEE_DETERMINISTIC: game-visible state must depend only on the logic frame count, never on
 * wall-clock timing. On by default during .slp playback (MELEE_DETERMINISTIC=0 turns it off), and
 * on for any run with MELEE_DETERMINISTIC=1. What it changes is listed where each change is made;
 * gw_Det_OneLogicPerRender is the first. */
int gw_Det_Enabled(void) {
    static int cached = -1;
    if (rp.live) {
        return 1; /* netplay: both peers must advance one logic frame per render */
    }
    if (cached < 0) {
        const char *v = getenv("MELEE_DETERMINISTIC");
        if (v != NULL && v[0] != '\0') {
            cached = v[0] == '1';
        } else {
            cached = gw_Replay_Active();
        }
        gw_log("det: deterministic mode %s", cached ? "ON" : "off");
    }
    return cached;
}

/* ---- Slippi's gameplay codes -----------------------------------------------------------------
 * Which of Slippi's gameplay-affecting codes are in force (_research/slippi-gameplay-codes.md):
 *   0 none, 1 the console/tournament codeset, 2 the online codeset.
 * During .slp playback it is the recording's own: major scene 8 (online) -> 2, any other replay
 * -> 1 (each code gates itself further by the replay's Slippi version, gw_Slippi_Version).
 * Outside playback MELEE_SLIPPI_CODES=tournament|online|off picks it (default off, i.e. vanilla).
 */
int gw_Slippi_Codes(void) {
    static int cached = -1;
    if (rp.live) {
        return 2; /* netplay plays the online codeset on both sides */
    }
    if (cached < 0) {
        const char *v = getenv("MELEE_SLIPPI_CODES");
        if (v != NULL && v[0] != '\0') {
            cached = (v[0] == 'o' || v[0] == 'O') ? 2 : (v[0] == 't' || v[0] == 'T' || v[0] == 'c') ? 1 : 0;
        } else if (gw_Replay_Active()) {
            cached = rp.online ? 2 : 1;
        } else {
            cached = 0;
        }
        gw_log("slippi: gameplay codes %s", cached == 2 ? "online" : cached == 1 ? "tournament" : "off");
    }
    return cached;
}

/* The replay's Slippi version as major*10000 + minor*100 + build (3.19.1 -> 31901), or a large
 * number outside playback (live play gets the current codes). */
int gw_Slippi_Version(void) {
    if (!gw_Replay_Active()) {
        return 999999;
    }
    return rp.version[0] * 10000 + rp.version[1] * 100 + rp.version[2];
}

/* Slippi's Frozen PS: the recording says so in its Game Start (Frozen PS, 2.0+); outside playback
 * MELEE_SLIPPI_FROZEN_PS=1 enables it (the console codeset g_stages_stadium is opt-in too). */
int gw_Slippi_FrozenStadium(void) {
    static int live = -1;
    if (gw_Replay_Active()) {
        return rp.frozen_ps;
    }
    if (live < 0) {
        const char *v = getenv("MELEE_SLIPPI_FROZEN_PS");
        live = v != NULL && v[0] == '1';
    }
    return live;
}
