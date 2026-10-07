#include "gw_ui_data.h"
#include <stdio.h>
#include <string.h>

int at_data_first(int total, int per, int focus, int first)
{
    int max_first;
    if (per < 1) per = 1;
    max_first = total > per ? total - per : 0;
    if (focus < 0) focus = 0;
    if (focus >= total) focus = total > 0 ? total - 1 : 0;
    if (focus < first) first = focus;
    else if (focus >= first + per) first = focus - per + 1;
    if (first > max_first) first = max_first;
    if (first < 0) first = 0;
    return first;
}

void at_data_counter(char *out, int cap, int focus, int total)
{
    if (cap <= 0) return;
    if (total <= 0) { snprintf(out, (size_t) cap, "%s", ""); return; }
    snprintf(out, (size_t) cap, "%d / %d", focus + 1, total);
}

void at_fmt_count(char *out, int cap, unsigned v)
{
    char d[16]; int n, i, o = 0;
    if (cap <= 0) return;
    n = snprintf(d, sizeof d, "%u", v);
    for (i = 0; i < n && o + 2 < cap; i++) {
        if (i > 0 && (n - i) % 3 == 0) out[o++] = ',';
        out[o++] = d[i];
    }
    out[o] = '\0';
}

void at_fmt_hm(char *out, int cap, unsigned seconds)
{
    if (cap <= 0) return;
    snprintf(out, (size_t) cap, "%u:%02u", seconds / 3600u, seconds / 60u % 60u);
}

void at_fmt_frames(char *out, int cap, unsigned t)
{
    if (cap <= 0) return;
    snprintf(out, (size_t) cap, "%02u:%02u %02u", t / 3600u % 60u, t / 60u % 60u, (unsigned) (99.0 * (double) (t % 60u) / 59.0));
}

void at_fmt_date(char *out, int cap, int year, int month, int day)
{
    if (cap <= 0) return;
    snprintf(out, (size_t) cap, "%04d-%02d-%02d", year, month, day);
}

void at_data_fill_item(AtItem *it, const char *id, const AtDataRow *r)
{
    if (it == NULL || r == NULL) return;
    memset(it, 0, sizeof *it);
    snprintf(it->id, sizeof it->id, "%s", id != NULL ? id : "");
    snprintf(it->label, sizeof it->label, "%s", r->label);
    it->vkind = AT_VAL_TEXT;
    snprintf(it->text, sizeof it->text, "%s", r->value);
    it->iflags = AT_ITEM_RO;
    if (r->flags & AT_DR_LOCKED) {
        it->iflags |= AT_ITEM_DISABLED;
        it->flags |= AT_CELL_DISABLED;
        snprintf(it->reason, sizeof it->reason, "%s", r->sub[0] != '\0' ? r->sub : "Locked.");
        snprintf(it->sub, sizeof it->sub, "%s", r->sub);
        snprintf(it->tag, sizeof it->tag, "%s", "LOCKED");
    } else if (r->flags & AT_DR_DONE) {
        it->flags |= AT_CELL_SELECTED;
        snprintf(it->tag, sizeof it->tag, "%s", "CLEARED");
        if (r->sub[0] != '\0') snprintf(it->sub, sizeof it->sub, "Cleared  %s", r->sub);
        else snprintf(it->sub, sizeof it->sub, "%s", "Cleared");
    } else {
        snprintf(it->sub, sizeof it->sub, "%s", r->sub);
        if (r->flags & AT_DR_NEW) snprintf(it->tag, sizeof it->tag, "%s", "NEW");
    }
}

int at_data_screen(const AtDataView *v, AtScreen *sc)
{
    int i;
    if (v->n < 0 || v->n > AT_DATA_ROWS) return 0;
    memset(sc, 0, sizeof *sc);
    snprintf(sc->id, sizeof sc->id, "%s", v->id);
    snprintf(sc->title, sizeof sc->title, "%s", v->title);
    sc->primary = AT_PRIMARY_LIST;
    sc->n_items = v->n;
    for (i = 0; i < v->n; i++) {
        char id[AT_ID];
        snprintf(id, sizeof id, "r%d", v->first + i);
        at_data_fill_item(&sc->items[i], id, &v->row[i]);
    }
    at_data_counter(sc->counter, (int) sizeof sc->counter, v->focus, v->total);
    return 1;
}
