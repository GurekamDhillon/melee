/* atlas_settings_stub.h - stand-ins for the Ui_* shims the table walker (gmfrontend_atlas_table.h) calls, so the walker and the pages run with no game and no host.
 * It records every row the walker submits into REC_ROWS (the game calls the shims unprefixed: Ui_SetRows, not gw_Ui_SetRows) and applies a change through the
 * real rule (at_item_apply, so the test links gw_ui_item.c). */
#ifndef ATLAS_SETTINGS_STUB_H
#define ATLAS_SETTINGS_STUB_H
#include <stdio.h>
#include <string.h>
#include "../platform/gw_ui_item.h"

enum { STUB_NONE = 0, STUB_TOGGLE = 1, STUB_CHOICE = 2, STUB_SLIDER = 3, STUB_TEXT = 4, STUB_COUNTER = 5 };   /* = AT_VAL_* */
enum { STUB_A_STEPS = 1, STUB_RO = 2, STUB_DANGER = 4, STUB_DISABLED = 8 };                                  /* = AT_ITEM_* */
#define STUB_ROWS 64

typedef struct {
    char id[24], label[64], help[160];
    int vkind, vmin, vmax, vstep, value, n_opts;
    char text[96], reason[48], group[32], opt[8][24];
    unsigned flags;
    int set;                      /* the three calls that fill a row all arrived */
} StubRow;
static struct { int n, rows_calls, row_calls, h; StubRow r[STUB_ROWS]; } REC_ROWS;

static void stub_reset(void) { memset(&REC_ROWS, 0, sizeof REC_ROWS); }
static StubRow *stub_row(int slot) { return (slot >= 0 && slot < REC_ROWS.n && slot < STUB_ROWS) ? &REC_ROWS.r[slot] : NULL; }
static void stub_cut(char *dst, int cap, const char *s) { snprintf(dst, (size_t) cap, "%s", s != NULL ? s : ""); }

void Ui_SetRows(int h, int n)
{
    REC_ROWS.h = h; REC_ROWS.rows_calls++;
    memset(REC_ROWS.r, 0, sizeof REC_ROWS.r);
    REC_ROWS.n = n < 0 ? 0 : (n > STUB_ROWS ? STUB_ROWS : n);                          /* clamped to 64 */
}
void Ui_SetRow(int h, int slot, const char *id, const char *label, const char *help, int vkind)
{
    StubRow *r = stub_row(slot);
    (void) h; REC_ROWS.row_calls++;
    if (r == NULL) return;                                                              /* a slot past SetRows' count is ignored */
    stub_cut(r->id, sizeof r->id, id); stub_cut(r->label, sizeof r->label, label); stub_cut(r->help, sizeof r->help, help); r->vkind = vkind;
}
void Ui_SetRowVal(int h, int slot, int vmin, int vmax, int vstep, int value, unsigned flags)
{
    StubRow *r = stub_row(slot);
    (void) h;
    if (r == NULL) return;
    r->vmin = vmin; r->vmax = vmax; r->vstep = vstep; r->value = value; r->flags = flags;
}
void Ui_SetRowText(int h, int slot, const char *text, const char *reason, const char *group)
{
    StubRow *r = stub_row(slot);
    (void) h;
    if (r == NULL) return;
    stub_cut(r->text, sizeof r->text, text); stub_cut(r->reason, sizeof r->reason, reason); stub_cut(r->group, sizeof r->group, group); r->set = 1;
}
void Ui_SetRowOpt(int h, int slot, int k, const char *text)
{
    StubRow *r = stub_row(slot);
    (void) h;
    if (r == NULL || k < 0 || k >= 8) return;
    stub_cut(r->opt[k], sizeof r->opt[k], text);
    if (k + 1 > r->n_opts) r->n_opts = k + 1;
}
/* the new value (cur when nothing changed): the walker compares them */
int Ui_ItemStep(int vkind, int vmin, int vmax, int vstep, int cur, int dir)
{
    int v = cur;
    return at_item_apply(vkind, vmin, vmax, vstep, cur, dir, &v) ? v : cur;
}
#endif
