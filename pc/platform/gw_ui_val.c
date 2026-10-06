#include "gw_ui_val.h"

#include <stdio.h>
#include <string.h>

void atv_init(AtvArena *a) { a->nn = a->ne = a->np = a->overflow = 0; }

static int node_new(AtvArena *a, int kind)
{
    AtvNode *n;
    if (a->nn >= ATV_MAX_NODES) { a->overflow = 1; return -1; }
    n = &a->node[a->nn];
    memset(n, 0, sizeof *n);
    n->kind = (unsigned char) kind;
    n->first = n->last = -1;
    n->fn = -1;
    return a->nn++;
}

static int pool_put(AtvArena *a, const char *s)
{
    int len = (int) strlen(s) + 1, at = a->np;
    if (a->np + len > ATV_POOL) { a->overflow = 1; return -1; }
    memcpy(a->pool + at, s, (size_t) len);
    a->np += len;
    return at;
}

int atv_bool(AtvArena *a, int b) { int n = node_new(a, ATV_BOOL); if (n >= 0) a->node[n].b = (unsigned char) (b != 0); return n; }
int atv_num(AtvArena *a, double d) { int n = node_new(a, ATV_NUM); if (n >= 0) a->node[n].num = d; return n; }
int atv_fn(AtvArena *a, int ref) { int n = node_new(a, ATV_FN); if (n >= 0) a->node[n].fn = ref; return n; }
int atv_table(AtvArena *a) { return node_new(a, ATV_TABLE); }
int atv_str(AtvArena *a, const char *s)
{
    int n = node_new(a, ATV_STR), p;
    if (n < 0) return -1;
    p = pool_put(a, s);
    if (p < 0) return -1;
    a->node[n].str = p;
    return n;
}

static int entry_add(AtvArena *a, int t, int key, int v)
{
    AtvEntry *e;
    AtvNode *tn;
    if (t < 0 || t >= a->nn || a->node[t].kind != ATV_TABLE || v < 0) return -1;
    if (a->ne >= ATV_MAX_ENTRIES) { a->overflow = 1; return -1; }
    e = &a->ent[a->ne];
    e->key = key; e->val = v; e->next = -1;
    tn = &a->node[t];
    if (tn->last >= 0) a->ent[tn->last].next = a->ne; else tn->first = a->ne;
    tn->last = a->ne;
    if (key < 0) tn->narr++;
    return a->ne++;
}

int atv_set(AtvArena *a, int t, const char *key, int v)
{
    int k = pool_put(a, key);
    return k < 0 ? -1 : entry_add(a, t, k, v);
}
int atv_push(AtvArena *a, int t, int v) { return entry_add(a, t, -1, v); }

int atv_get(const AtvArena *a, int t, const char *key)
{
    int e;
    if (t < 0 || t >= a->nn || a->node[t].kind != ATV_TABLE) return -1;
    for (e = a->node[t].first; e >= 0; e = a->ent[e].next)
        if (a->ent[e].key >= 0 && strcmp(a->pool + a->ent[e].key, key) == 0) return a->ent[e].val;
    return -1;
}

int atv_at(const AtvArena *a, int t, int i1)
{
    int e, k = 0;
    if (t < 0 || t >= a->nn || a->node[t].kind != ATV_TABLE) return -1;
    for (e = a->node[t].first; e >= 0; e = a->ent[e].next)
        if (a->ent[e].key < 0 && ++k == i1) return a->ent[e].val;
    return -1;
}

int atv_len(const AtvArena *a, int t) { return (t >= 0 && t < a->nn && a->node[t].kind == ATV_TABLE) ? a->node[t].narr : 0; }
int atv_kind(const AtvArena *a, int n) { return (n >= 0 && n < a->nn) ? a->node[n].kind : ATV_NIL; }
double atv_numv(const AtvArena *a, int n, double def) { return atv_kind(a, n) == ATV_NUM ? a->node[n].num : def; }
const char *atv_strv(const AtvArena *a, int n, const char *def) { return atv_kind(a, n) == ATV_STR ? a->pool + a->node[n].str : def; }
int atv_boolv(const AtvArena *a, int n, int def) { return atv_kind(a, n) == ATV_BOOL ? a->node[n].b : def; }
int atv_fnv(const AtvArena *a, int n) { return atv_kind(a, n) == ATV_FN ? a->node[n].fn : -1; }
