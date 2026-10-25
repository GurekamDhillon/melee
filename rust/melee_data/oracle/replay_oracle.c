/* Runs the shipping gw_replay.c .slp reader (rp struct .. rp_load, cut out by replay_oracle.sh) over
 * the files named on the command line and prints one summary line per file; tests/replay_oracle.rs
 * prints the same line from the Rust reader. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdarg.h>
#include <windows.h>
#include "gw_rollback.h"
#include "gw_replay_matchrules.h"
#include "gw_replay_controls.h"

static char logbuf[4096];
static void gw_log(const char *fmt, ...) { va_list ap; va_start(ap, fmt); vsnprintf(logbuf, sizeof logbuf, fmt, ap); va_end(ap); }
int gw_Gdlp_CheckFile(const char *path, char *why, size_t cap) { (void)path; snprintf(why, cap, "oracle"); return -1; }
void gw_Replay_ArmLive(int on) { (void)on; }

#include "replay_cut.inc"

static uint64_t fnv(uint64_t h, const void *p, size_t n) {
    const unsigned char *b = p;
    while (n--) { h ^= *b++; h *= 0x100000001b3ull; }
    return h;
}
static uint64_t fnvu(uint64_t h, uint32_t v) { return fnv(h, &v, 4); }
static uint64_t fnvf(uint64_t h, float f) { return fnv(h, &f, 4); }

int main(int argc, char **argv) {
    int a;
    _putenv_s("MELEE_SLP_PARITY", "1");
    _putenv_s("MELEE_GDLP", "");
    for (a = 1; a < argc; ++a) {
        uint64_t hi = 0xcbf29ce484222325ull, hf = hi, hp = hi, hg = hi;
        int n, i, present = 0;
        memset(&rp, 0, sizeof rp);
        rp.frame = GW_RP_UNARMED;
        _putenv_s("MELEE_SLP", argv[a]);
        rp_load();
        if (!rp.active) { printf("%s\tactive=0\n", argv[a]); continue; }
        n = (rp.last - rp.first + 1) * GW_RP_SLOTS;
        for (i = 0; i < n; ++i) {
            const GwRpInput *r = &rp.in[i];
            hi = fnvf(hi, r->lx); hi = fnvf(hi, r->ly); hi = fnvf(hi, r->cx); hi = fnvf(hi, r->cy);
            hi = fnvf(hi, r->trigger); hi = fnvu(hi, r->buttons); hi = fnvu(hi, r->seed);
            hi = fnv(hi, r->raw, 4); hi = fnvu(hi, r->present); hi = fnvu(hi, r->physical_buttons);
            hi = fnvf(hi, r->physical_l); hi = fnvf(hi, r->physical_r); hi = fnvu(hi, r->physical_complete);
            present += r->present;
            if (rp.post) {
                const struct GwRpPost *q = &rp.post[i];
                hp = fnvf(hp, q->x); hp = fnvf(hp, q->y); hp = fnvf(hp, q->facing); hp = fnvf(hp, q->percent);
                hp = fnvu(hp, q->action); hp = fnvu(hp, q->stocks); hp = fnvu(hp, q->has);
            }
        }
        for (i = 0; i < rp.last - rp.first + 1; ++i) { hf = fnvu(hf, rp.fs_seed[i]); hf = fnvu(hf, rp.fs_has[i]); }
        hg = fnv(hg, rp.game_info, GW_RP_GAME_INFO);
        printf("%s	active=1 ver=%u.%u.%u.%u first=%d last=%d seed=%08x turbo=%08x ctl=%08x,%08x,%08x,%08x online=%d frozen=%d trunc=%d gss=%d "
               "ucfd=%u,%u,%u,%u ucfs=%u,%u,%u,%u scene=\"%s\" present=%d expected=%ld gi=%016llx in=%016llx fs=%016llx post=%016llx\n",
               argv[a],
               rp.version[0], rp.version[1], rp.version[2], rp.version[3], rp.first, rp.last, rp.seed, rp.turbo,
               rp.controls[0], rp.controls[1], rp.controls[2], rp.controls[3], rp.online, rp.frozen_ps, rp.truncated, rp.game_start_seen,
               rp.ucf_dashback[0], rp.ucf_dashback[1], rp.ucf_dashback[2], rp.ucf_dashback[3],
               rp.ucf_shield[0], rp.ucf_shield[1], rp.ucf_shield[2], rp.ucf_shield[3],
               rp.scene, present, rp.parity_expected, (unsigned long long) hg, (unsigned long long) hi,
               (unsigned long long) hf, (unsigned long long) hp);
        free(rp.in); free(rp.fs_seed); free(rp.fs_has); free(rp.post);
    }
    return 0;
}
