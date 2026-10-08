/* Parity test for the native Envoy evaluator (pc/gameworld/script_mods_core.h) against the Lua modifier engine.
 *   script_mods_core_test <fixture.txt>
 * The fixture is written by pc/tests/envoy_stage4_lib.lua (the REAL mod_engine.lua is the oracle): per scenario the program words of each seat, then per
 * frame the sampled players, the events, and the expected statuses / damage / recents / fx / drop counter after begin_frame + drain. This program loads
 * the same words, feeds the same events, ticks once per frame and compares every field. Frames, stacks, expiries and counters must match exactly;
 * percent deltas and amounts to 1e-3 (the engine computes in doubles, the evaluator in floats). Exit 0 = parity.
 *   clang -O1 pc/tests/script_mods_core_test.c -o script_mods_core_test.exe */
#define _CRT_SECURE_NO_WARNINGS 1
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include "../gameworld/script_mods_core.h"

static SmWorld W;
static char line[400000];
static int fails, checked_frames, scenario, frame_no;
static int fired_rules;

#define FAIL(...) do { if (fails < 12) { printf("MISMATCH scenario %d frame %d: ", scenario, frame_no); printf(__VA_ARGS__); printf("\n"); } fails++; } while (0)

typedef struct { int on, stacks, max, expires, next_tick, cause; double amount; } ExpS;
typedef struct { int op, port, atype, frames, direction, count, exits, flags; double value; } ExpFx;

static const char* tok_p;
static long tk_i(void) { char* e; long v = strtol(tok_p, &e, 10); tok_p = e; return v; }
static double tk_f(void) { char* e; double v = strtod(tok_p, &e); tok_p = e; return v; }

static void read_ctx(SmCtx* c) {
    c->has_percent = (int) tk_i(); c->has_grounded = (int) tk_i(); c->has_stocks = (int) tk_i(); c->has_air = (int) tk_i(); c->has_ahit = (int) tk_i();
    c->percent = (float) tk_f(); c->grounded = (int) tk_i(); c->stocks = (int) tk_i(); c->air_frames = (int) tk_i(); c->aerial_hit = (int) tk_i(); (void) tk_i();
}

int main(int argc, char** argv) {
    FILE* f;
    SmPlayer pl[SM_PORTS];
    ExpS es[SM_PORTS][SM_NSTATUS + 1];
    double ed[SM_PORTS];
    int er[SM_PORTS][SM_NEV];
    ExpFx ex[SM_FX];
    int nex = 0, ticked = 0, edrop = 0, p, i, frames = 0;
    if (argc < 2 || !(f = fopen(argv[1], "r"))) { printf("usage: script_mods_core_test <fixture>\n"); return 2; }
    memset(pl, 0, sizeof pl);
    while (fgets(line, sizeof line, f)) {
        tok_p = line;
        if (!strncmp(line, "SCENARIO", 8)) { sm_reset(&W); sscanf(line, "SCENARIO %d", &scenario); frame_no = 0; }
        else if (!strncmp(line, "PROGRAM", 7)) {
            int port, n;
            sscanf(line, "PROGRAM %d %d", &port, &n);
            if (!fgets(line, sizeof line, f)) return 2;
            tok_p = line;
            for (i = 0; i < n; ++i) sm_prog_word(&W, port, i, (int) tk_i());
            if (!sm_prog_seal(&W, port)) { FAIL("program for port %d refused by sm_prog_seal", port); }
        }
        else if (!strncmp(line, "FRAMES", 6)) { sscanf(line, "FRAMES %d", &frames); }
        else if (line[0] == 'F' && line[1] == ' ') {
            sscanf(line, "F %d", &frame_no); ticked = 0; nex = 0; edrop = 0;
            memset(es, 0, sizeof es); memset(ed, 0, sizeof ed); memset(er, 0, sizeof er);
        }
        else if (line[0] == 'P' && line[1] == ' ') {
            int port; tok_p = line + 2; port = (int) tk_i();
            pl[port].present = (int) tk_i(); pl[port].percent = (float) tk_f(); pl[port].grounded = (int) tk_i(); pl[port].stocks = (int) tk_i();
            pl[port].x = (float) tk_f(); pl[port].y = (float) tk_f(); pl[port].has_pos = 1;
        }
        else if (line[0] == 'E' && line[1] == ' ') {
            SmEvent e; int kind, port, target;
            tok_p = line + 2;
            kind = (int) tk_i(); port = (int) tk_i(); target = (int) tk_i();
            sm_event_init(&e, kind, port, target);
            e.tags = (unsigned) tk_i(); e.status = (int) tk_i(); e.hit = (int) tk_i(); e.aerial = (int) tk_i(); e.direction = (int) tk_i(); e.count = (int) tk_i();
            e.has_damage = (int) tk_i(); e.damage = (float) tk_f(); e.has_strength = (int) tk_i(); e.strength = (float) tk_f();
            e.absorbed = (int) tk_i(); e.broke = (int) tk_i(); e.has_clank = (int) tk_i(); e.damage_a = (float) tk_f(); e.damage_b = (float) tk_f();
            e.has_self = (int) tk_i(); read_ctx(&e.self); e.has_target = (int) tk_i(); read_ctx(&e.target_ctx);
            sm_emit(&W, &e);
        }
        else if (line[0] == 'S' && line[1] == ' ') {
            int port, id; tok_p = line + 2; port = (int) tk_i(); id = (int) tk_i();
            es[port][id].on = 1; es[port][id].stacks = (int) tk_i(); es[port][id].max = (int) tk_i(); es[port][id].expires = (int) tk_i();
            es[port][id].next_tick = (int) tk_i(); es[port][id].amount = tk_f(); es[port][id].cause = (int) tk_i();
        }
        else if (line[0] == 'D' && line[1] == ' ') { int port; tok_p = line + 2; port = (int) tk_i(); ed[port] = tk_f(); }
        else if (line[0] == 'R' && line[1] == ' ') { int port, kind; tok_p = line + 2; port = (int) tk_i(); kind = (int) tk_i(); er[port][kind] = (int) tk_i(); }
        else if (!strncmp(line, "FX ", 3)) {
            ExpFx* x = &ex[nex++]; tok_p = line + 3;
            x->op = (int) tk_i(); x->port = (int) tk_i(); x->atype = (int) tk_i(); x->frames = (int) tk_i(); x->direction = (int) tk_i(); x->count = (int) tk_i();
            x->exits = (int) tk_i(); x->flags = (int) tk_i(); x->value = tk_f();
        }
        else if (line[0] == 'X' && line[1] == ' ') {
            sscanf(line, "X %d", &edrop);
            if (!ticked) { sm_tick(&W, pl); ticked = 1; }
            checked_frames++;
            for (p = 0; p < SM_PORTS; ++p) {
                for (i = 1; i <= SM_NSTATUS; ++i) {
                    const SmStatus* s = &W.st[p][i]; const ExpS* e = &es[p][i];
                    if (s->on != e->on) { FAIL("port %d status %d on: native %d, lua %d", p, i, s->on, e->on); continue; }
                    if (!e->on) continue;
                    if (s->stacks != e->stacks || s->max != e->max || s->expires != e->expires || s->next_tick != e->next_tick || s->cause != e->cause ||
                        fabs((double) s->amount - e->amount) > 1e-3)
                        FAIL("port %d status %d: native stacks %d max %d expires %d next %d cause %d amount %g | lua stacks %d max %d expires %d next %d cause %d amount %g",
                             p, i, s->stacks, s->max, s->expires, s->next_tick, s->cause, (double) s->amount, e->stacks, e->max, e->expires, e->next_tick, e->cause, e->amount);
                }
                if (fabs((double) W.damage[p] - ed[p]) > 1e-3) FAIL("port %d damage: native %g, lua %g", p, (double) W.damage[p], ed[p]);
                for (i = 1; i < SM_NEV; ++i) if (W.recent[p][i] != er[p][i]) FAIL("port %d recent[%d]: native %d, lua %d", p, i, W.recent[p][i], er[p][i]);
            }
            if (W.nfx != nex) FAIL("fx count: native %d, lua %d", W.nfx, nex);
            else for (i = 0; i < nex; ++i) {
                const SmFx* a = &W.fx[i]; const ExpFx* b = &ex[i];
                if (a->op != b->op || a->port != b->port || a->atype != b->atype || a->frames != b->frames || a->direction != b->direction || a->count != b->count ||
                    a->exits != b->exits || a->flags != b->flags || fabs((double) a->value - b->value) > 1e-3)
                    FAIL("fx %d: native op %d port %d type %d frames %d dir %d count %d exits %d flags %d value %g | lua op %d port %d type %d frames %d dir %d count %d exits %d flags %d value %g",
                         i, a->op, a->port, a->atype, a->frames, a->direction, a->count, a->exits, a->flags, (double) a->value, b->op, b->port, b->atype, b->frames, b->direction, b->count, b->exits, b->flags, b->value);
            }
            if (W.dropped != edrop) FAIL("dropped: native %d, lua %d", W.dropped, edrop);
        }
        else if (!strncmp(line, "ENDSCENARIO", 11)) { fired_rules++; }
        if (((line[1] == ' ' && (line[0] == 'S' || line[0] == 'D' || line[0] == 'R')) || !strncmp(line, "FX ", 3)) && !ticked) { sm_tick(&W, pl); ticked = 1; }
    }
    fclose(f);
    printf("script_mods_core_test: %d scenarios, %d frames compared, %d mismatches\n", fired_rules, checked_frames, fails);
    return fails ? 1 : 0;
}
