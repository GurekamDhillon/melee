/* gw_ui_val.h - a tiny value tree, so the Lua-to-screen conversion is testable without Lua. Pure C. */
#ifndef GW_UI_VAL_H
#define GW_UI_VAL_H
#ifdef __cplusplus
extern "C" {
#endif

#define ATV_MAX_NODES 2048
#define ATV_MAX_ENTRIES 4096
#define ATV_POOL 49152

typedef enum { ATV_NIL, ATV_BOOL, ATV_NUM, ATV_STR, ATV_TABLE, ATV_FN } AtvKind;
typedef struct { unsigned char kind, b; int fn; double num; int str; int first, last, narr; } AtvNode;
typedef struct { int key, val, next; } AtvEntry;   /* key: pool offset of the name, -1 for an array entry */
typedef struct { AtvNode node[ATV_MAX_NODES]; AtvEntry ent[ATV_MAX_ENTRIES]; char pool[ATV_POOL]; int nn, ne, np, overflow; } AtvArena;

void atv_init(AtvArena *a);
/* constructors return the node index, or -1 when the arena is full (then a->overflow is set) */
int atv_bool(AtvArena *a, int b);
int atv_num(AtvArena *a, double d);
int atv_str(AtvArena *a, const char *s);
int atv_fn(AtvArena *a, int ref);
int atv_table(AtvArena *a);
int atv_set(AtvArena *a, int t, const char *key, int v);   /* t[key] = v; returns the entry or -1 */
int atv_push(AtvArena *a, int t, int v);                   /* append to t's array part */
/* readers: -1 / defaults for anything that is not there or has another kind */
int atv_get(const AtvArena *a, int t, const char *key);
int atv_at(const AtvArena *a, int t, int i1);              /* the i-th array entry, 1-based */
int atv_len(const AtvArena *a, int t);                     /* the number of array entries */
int atv_kind(const AtvArena *a, int n);
double atv_numv(const AtvArena *a, int n, double def);
const char *atv_strv(const AtvArena *a, int n, const char *def);
int atv_boolv(const AtvArena *a, int n, int def);
int atv_fnv(const AtvArena *a, int n);                     /* the registry reference, or -1 */

#ifdef __cplusplus
}
#endif
#endif
