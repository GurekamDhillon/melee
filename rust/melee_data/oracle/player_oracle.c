/* Drives the shipping gw_replay.c (the whole file, #included so its statics are reachable) through a
 * playback / recording session and prints every observable: log lines, per-frame accessors, seeds,
 * UCF queries, the trace files and the recording (hashed). tests/player_oracle.rs runs the same
 * program through melee_data::player.
 *
 *   player_oracle <slp> <outdir> <flags>
 * flags (comma separated): resync parity trace rec live local=N badseed=F badpost=F codes=X det=X
 *                          frozen noucf073 ucf=X resim skin
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdarg.h>
#include <windows.h>
#include "gw_rollback.h"

/* ---- stubs for everything gw_replay.c reaches outside itself ---- */
static unsigned fcm[4];
static int fck[4] = { -1, -1, -1, -1 };
static char out_trace[600], out_rec[600], out_vel[600], out_seed[600], out_gdlp[600];
static int gdlp_setting;
static char out_gdlp2[600];
static int local_port = -1, resim_flag = 0, skin_flag = 0;

static void gw_log(const char *fmt, ...) {
    char b[2048];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    printf("LOG %s\n", b);
}
unsigned gw_FighterControls_Get(int port, int ck) { return (unsigned) ((port * 0x11 + ck * 3) & 0xff3d); }
void gw_FighterControls_MatchClear(void) { int i; memset(fcm, 0, sizeof fcm); for (i = 0; i < 4; ++i) fck[i] = -1; }
int gw_FighterControls_MatchCK(int port) { return port >= 0 && port < 4 ? fck[port] : -1; }
unsigned gw_FighterControls_MatchGet(int port) { return port >= 0 && port < 4 ? fcm[port] : 0; }
void gw_FighterControls_MatchSet(int port, int ck, unsigned choice) { if (port >= 0 && port < 4) { fcm[port] = choice; fck[port] = ck; } }
unsigned gw_Netplay_FighterControls(int player) { return (unsigned) (player * 5); }
uint32_t gw_Netplay_Handshake(uint8_t *d, int len, int keep_off, int keep_len) {
    int i;
    (void) keep_off; (void) keep_len;
    for (i = 0; i < len; i += 17) d[i] ^= 0x5A;
    return 0xA5A5C3C3u;
}
void gw_HSD_RandRenderBegin(void) {}
void gw_HSD_RandRenderEnd(void) {}
int gw_MatchTurboRules(void) { return 0x5FB; }
const GwRbInput *gw_RB_InputAny(int port, int frame) { (void) port; (void) frame; return NULL; }
const GwRbInput *gw_RB_InputFor(int port, int fol, int frame) { (void) port; (void) fol; (void) frame; return NULL; }
int gw_Settings_Int(const char *key, int dflt) { if (strcmp(key, "gdlp_record") == 0) return gdlp_setting; return dflt; }
int gw_Snap_Resimulating(void) { return resim_flag; }
int gw_StageSkin_Last(int *ext, char *name, int cap, uint64_t *digest) {
    if (!skin_flag) return 0;
    *ext = 3;
    snprintf(name, cap, "Frosty Skin");
    *digest = 0x123456789abcull;
    return 1;
}
int gw_rb_active(void) { return 0; }
int gw_rb_slippi_local_port(void) { return local_port; }

static uint64_t fnv_file(const char *path, long *len) {
    FILE *f = fopen(path, "rb");
    uint64_t h = 0xcbf29ce484222325ull;
    int c;
    *len = -1;
    if (!f) return 0;
    *len = 0;
    while ((c = fgetc(f)) != EOF) { h ^= (unsigned) c; h *= 0x100000001b3ull; ++*len; }
    fclose(f);
    return h;
}
static void dump_files(void) {
    const char *names[6] = { out_trace, out_vel, out_seed, out_rec, out_gdlp, out_gdlp2 };
    const char *tag[6] = { "trace", "vel", "seed", "rec", "gdlp", "gdlp2" };
    int i;
    fflush(NULL);
    for (i = 0; i < 6; ++i) {
        long len;
        uint64_t h = fnv_file(names[i], &len);
        printf("FILE %s %ld %016llx\n", tag[i], len, (unsigned long long) h);
    }
}
void gw_hang_final(const char *msg, unsigned rc) {
    printf("LOG hang_final: %s %u\n", msg, rc);
    dump_files();
}

static uint64_t id2 = 0x99990000aaaa1111ull;
static const char *mods_text = "";
/* ---- the LAB recorder container (gw_script_rec.inc up to its recording section) ---- */
static int geno_version = 9;
#define GENO_VERSION geno_version
static struct { char data_dir[MAX_PATH]; int match_active; } gs;
static int gs_players_present(int s) { (void) s; return 0; }
static int gw_ScriptGame_FighterI(int s, int what) { (void) s; (void) what; return 0; }
enum { SI_CHAR, SI_COSTUME, SI_SLOT_TYPE };
static const char *gs_char_name(int c) { (void) c; return "?"; }
static void gs_identity(uint64_t out[4]) { out[0] = 0x1111222233334444ull; out[1] = 0x5555666677778888ull; out[2] = id2; out[3] = 0xdeadbeefcafef00dull; }
const char *gw_Mods_Describe(void) { return mods_text; }
static int gw_ScriptGame_StageKind(void) { return 0; }
static long long fake_time(void *p) { (void) p; return 1700000000ll; }
#define time(x) fake_time(x)
#include "script_rec_cut.inc"
#undef time

#include "gw_replay.c"

static uint64_t line_hash = 0xcbf29ce484222325ull;
static long line_count;
static int verbose;
/* one per-frame observation: hashed (and printed with the "v" flag) */
static void ln(const char *fmt, ...) {
    char b[1024];
    const char *c;
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    for (c = b; *c; ++c) { line_hash ^= (unsigned char) *c; line_hash *= 0x100000001b3ull; }
    line_hash ^= 10; line_hash *= 0x100000001b3ull;
    ++line_count;
    if (verbose) printf("%s\n", b);
}

static uint32_t fb(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
static int has_flag(const char *flags, const char *name) {
    size_t n = strlen(name);
    const char *p = flags;
    while ((p = strstr(p, name)) != NULL) {
        if ((p == flags || p[-1] == ',') && (p[n] == ',' || p[n] == '\0' || p[n] == '=')) return 1;
        p += n;
    }
    return 0;
}
static const char *flag_val(const char *flags, const char *name) {
    size_t n = strlen(name);
    const char *p = flags;
    while ((p = strstr(p, name)) != NULL) {
        if ((p == flags || p[-1] == ',') && p[n] == '=') return p + n + 1;
        p += n;
    }
    return NULL;
}
static long flag_int(const char *flags, const char *name, long dflt) {
    const char *v = flag_val(flags, name);
    return v ? strtol(v, NULL, 10) : dflt;
}

int main(int argc, char **argv) {
    const char *slp, *dir, *flags;
    unsigned char mbuf[0x138 + 16];
    int i, f, p, fol, w, back, probe = 0;
    uint32_t seed;
    if (argc < 4) return 2;
    verbose = argc > 4;
    slp = argv[1]; dir = argv[2]; flags = argv[3];
    snprintf(out_trace, sizeof out_trace, "%s/trace.csv", dir);
    snprintf(out_vel, sizeof out_vel, "%s/trace.csv.vel.csv", dir);
    snprintf(out_seed, sizeof out_seed, "%s/trace.csv.seed.csv", dir);
    snprintf(out_rec, sizeof out_rec, "%s/rec.slp", dir);
    snprintf(out_gdlp, sizeof out_gdlp, "%s/m.gdlp", dir);
    snprintf(out_gdlp2, sizeof out_gdlp2, "%s/geno-lab_lab/recordings/match_1700000000_1.gdlp", dir);
    remove(out_trace); remove(out_vel); remove(out_seed); remove(out_rec); remove(out_gdlp); remove(out_gdlp2);
    local_port = (int) flag_int(flags, "local", -1);
    snprintf(gs.data_dir, sizeof gs.data_dir, ".");
    resim_flag = has_flag(flags, "resim");
    skin_flag = has_flag(flags, "skin");
    gdlp_setting = has_flag(flags, "gset");
    _putenv_s("MELEE_GDLP_RECORD", has_flag(flags, "gdlp") ? out_gdlp : (has_flag(flags, "gdlp0") ? "0" : ""));
    { const char *v = flag_val(flags, "idx"); if (v) { int k = atoi(v); if (k == 2) id2 ^= 1; } }
    { const char *v = flag_val(flags, "geno"); if (v) geno_version = atoi(v); }
    { const char *v = flag_val(flags, "mods"); if (v) { static char mt[64]; size_t n = strcspn(v, ","); snprintf(mt, sizeof mt, "%.*s", (int) n, v); mods_text = mt; } }
    if (has_flag(flags, "play_gdlp")) { _putenv_s("MELEE_GDLP", slp); _putenv_s("MELEE_SLP", ""); }
    else { _putenv_s("MELEE_GDLP", ""); _putenv_s("MELEE_SLP", slp); }
    _putenv_s("MELEE_SLP_RESYNC", has_flag(flags, "resync") ? "1" : "");
    _putenv_s("MELEE_SLP_PARITY", has_flag(flags, "parity") ? "1" : "");
    _putenv_s("MELEE_SLP_NO_UCF073", has_flag(flags, "noucf073") ? "1" : "");
    _putenv_s("MELEE_STATE_TRACE", has_flag(flags, "trace") ? out_trace : "");
    _putenv_s("MELEE_SLP_RECORD", has_flag(flags, "rec") ? out_rec : "");
    _putenv_s("MELEE_SLIPPI_FROZEN_PS", has_flag(flags, "frozen") ? "1" : "");
    { const char *v = flag_val(flags, "codes"); _putenv_s("MELEE_SLIPPI_CODES", v ? "" : ""); if (v) { char b[8]; snprintf(b, sizeof b, "%c", v[0]); _putenv_s("MELEE_SLIPPI_CODES", b); } }
    { const char *v = flag_val(flags, "det"); char b[8] = ""; if (v) snprintf(b, sizeof b, "%c", v[0]); _putenv_s("MELEE_DETERMINISTIC", b); }
    { const char *v = flag_val(flags, "ucf"); char b[16] = ""; if (v) { size_t n = strcspn(v, ","); snprintf(b, sizeof b, "%.*s", (int) n, v); } _putenv_s("MELEE_UCF", b); }
    _putenv_s("MELEE_RB_LIVETEST", "");

    if (has_flag(flags, "live")) gw_Replay_ArmLive(1);
    printf("ACTIVE %d\n", gw_Replay_Active());
    printf("SCENE %s\n", gw_replay_scene() ? gw_replay_scene() : "(null)");
    if (has_flag(flags, "rec")) printf("RECORDING %d\n", gw_Replay_Recording());
    printf("ENABLED %d\n", gw_Replay_Enabled());
    {
        int c_codes = gw_Slippi_Codes(), c_ver = gw_Slippi_Version(), c_frozen = gw_Slippi_FrozenStadium(), c_det = gw_Det_Enabled();
        int c_last = gw_Replay_LastFrame(), c_live = gw_Replay_Live(), c_raw = gw_Replay_ControlsRaw();
        unsigned c_turbo = (unsigned) gw_Replay_TurboRules();
        printf("CODES %d VER %d FROZEN %d DET %d LAST %d LIVE %d CTRLRAW %d TURBO %08x\n", c_codes, c_ver, c_frozen, c_det, c_last, c_live, c_raw, c_turbo);
    }
    for (p = 0; p < 4; ++p) printf("HUMAN %d %d\n", p, gw_Replay_PortHuman(p));
    for (i = 0; i < (int) sizeof mbuf; ++i) mbuf[i] = (unsigned char) (i * 7 + 3);
    for (p = 0; p < 4; ++p) { mbuf[0x60 + 0x24 * p + 1] = (unsigned char) (p == 2 ? 3 : p == 3 ? 1 : 0); mbuf[0x60 + 0x24 * p + 7] = (unsigned char) p; mbuf[0x60 + 0x24 * p] = (unsigned char) (p + 8); }
    seed = gw_Replay_ApplyMatch(mbuf);
    {
        uint64_t h = 0xcbf29ce484222325ull;
        for (i = 0; i < (int) sizeof mbuf; ++i) { h ^= mbuf[i]; h *= 0x100000001b3ull; }
        printf("APPLY seed=%08x buf=%016llx frame=%d\n", seed, (unsigned long long) h, gw_Replay_Frame());
    }
    gw_FighterControls_MatchBegin(mbuf);
    printf("MC %08x %08x %08x %08x\n", fcm[0], fcm[1], fcm[2], fcm[3]);
    gw_Replay_RecordMatch(mbuf, seed);
    {
        GwSlippiFixtureInfo fi;
        int ok = gw_Replay_SlippiFixtureInfo(&fi);
        printf("FIXTURE %d %s", ok, gw_Replay_SlippiFixtureReason());
        if (ok) printf(" %d.%d.%d.%d %d..%d seed=%08x", fi.version[0], fi.version[1], fi.version[2], fi.version[3], fi.first_frame, fi.last_frame, fi.seed);
        printf("\n");
    }
    /* offline UCF history */
    gw_Ucf_PadLatch(1, 10, -20, 30, -40);
    gw_Ucf_PadLatch(1, 11, -21, 31, -41);
    for (w = 0; w < 4; ++w) printf("OFFRAW %d %d %d\n", w, gw_Replay_RawStick(1, w), gw_Replay_RawStickBack(1, w, 1));
    {
        long limit = rp.active ? (long) rp.last - rp.first + 3 : 6;
        long badseed = flag_int(flags, "badseed", -100000), badpost = flag_int(flags, "badpost", -100000);
        long maxframes = flag_int(flags, "frames", 100000);
        if (limit > maxframes) limit = maxframes;
        for (probe = 0; probe < limit; ++probe) {
            uint32_t rs, arrived, cs;
            f = gw_Replay_Tick();
            ln("T %d", f);
            if (f == GW_RP_UNARMED) break;
            rs = gw_Replay_ResyncSeed();
            cs = gw_Replay_ChecksumSeed(0x1234u, f);
            ln("RS %08x CS %08x", rs, cs);
            arrived = rs ? rs : (0x80000000u + (uint32_t) f * 0x9E3779B1u);
            if (f == badseed) arrived ^= 1;
            gw_Replay_NoteSeed(arrived);
            gw_Replay_CheckSeed(arrived);
            for (p = 0; p < 4; ++p) {
                for (fol = 0; fol < 2; ++fol) {
                    if (gw_Replay_HasInput(p, fol)) {
                        float lx = gw_Replay_StickX(p, fol), ly = gw_Replay_StickY(p, fol), cx = gw_Replay_CStickX(p, fol),
                              cy = gw_Replay_CStickY(p, fol), tr = gw_Replay_Trigger(p, fol);
                        uint32_t bt = gw_Replay_Buttons(p, fol);
                        ln("IN %d %d %08x %08x %08x %08x %08x %08x", p, fol, fb(lx), fb(ly), fb(cx), fb(cy), fb(tr), bt);
                        gw_Replay_RecordInput(p, fol, lx, ly, cx, cy, tr, bt, p * 1000 + (f & 0xfff), (float) f * 0.25f + p, (float) p - f * 0.125f,
                                              (fol ? -1.0f : 1.0f), (float) (f & 0xff) * 0.5f);
                    }
                }
            }
            if (rp.live) {
                for (p = 0; p < 2; ++p)
                    gw_Replay_RecordInput(p, 0, 0.5f * p, -0.25f, 0.125f, 1.0f, 0.75f, 0x100u + (unsigned) p, 0x20 + p, (float) f, (float) -p, 1.0f, (float) (f & 0x3f));
            }
            for (p = 0; p < 4; ++p) {
                char ub[512];
                int n = snprintf(ub, sizeof ub, "U %d card=%d ver=%d raw=", p, gw_Replay_UcfCardinals(p), gw_Replay_UcfVersion(p));
                for (w = 0; w < 4; ++w) {
                    n += snprintf(ub + n, sizeof ub - n, "%d", gw_Replay_RawStick(p, w));
                    for (back = 0; back < 3; ++back) n += snprintf(ub + n, sizeof ub - n, "/%d", gw_Replay_RawStickBack(p, w, back));
                    n += snprintf(ub + n, sizeof ub - n, " ");
                }
                ln("%s", ub);
            }
            for (i = 0; i < 8; ++i) {
                GwRbInput o;
                memset(&o, 0x77, sizeof o);
                if (gw_Replay_PeekInput(i, f, &o))
                    ln("PK %d %08x %08x %08x %d %d %d %d conf=%d", i, o.buttons, o.seed, fb(o.lx), o.raw[0], o.raw[1], o.raw[2], o.raw[3], o.confirmed);
                else ln("PK %d none", i);
            }
            if (rp.post != NULL && f >= rp.first && f <= rp.last) {
                for (i = 0; i < GW_RP_SLOTS; ++i) {
                    const struct GwRpPost *q = &rp.post[(f - rp.first) * GW_RP_SLOTS + i];
                    if (q->has) {
                        float x = q->x;
                        if (f == badpost && i == 0) x += 1.0f;
                        gw_Replay_TraceFighter(i / 2, i & 1, i / 2 + 1, q->action, x, q->y, q->facing, q->percent, q->stocks,
                                               q->x * 0.5f, q->y * 0.25f, -q->x, q->y + 1.0f, 3.0f);
                    }
                }
            } else if (rp.active) {
                gw_Replay_TraceFighter(0, 0, 1, 0x20, 1.5f, -2.25f, 1.0f, 12.0f, 4, 0.1f, 0.2f, 0.3f, 0.4f, 5.0f);
            }
            ln("M tracing=%d procmap=%d frame=%d", gw_Replay_Tracing(), gw_Replay_TraceAtProcMap(), gw_Replay_Frame());
            {
                int c[4];
                gw_Replay_GetCursor(c);
                ln("CUR %d %d %d %08x", c[0], c[1], c[2], (unsigned) c[3]);
                gw_Replay_SetCursor(c);
            }
        }
    }
    gw_Replay_MatchRecordEnd();
    for (p = 0; p < 2; ++p) {
        GwSlippiPad pad;
        int fr = rp.first + 5;
        if (gw_Replay_SlippiPad(p, fr, &pad)) {
            printf("SPAD %d %02x%02x%02x%02x%02x%02x%02x%02x\n", p, pad.bytes[0], pad.bytes[1], pad.bytes[2], pad.bytes[3], pad.bytes[4], pad.bytes[5], pad.bytes[6], pad.bytes[7]);
        } else printf("SPAD %d none\n", p);
    }
    printf("FINISH %d\n", gw_Replay_FinishRecording());
    printf("LINES %ld %016llx\n", line_count, (unsigned long long) line_hash);
    dump_files();
    return 0;
}
