/* gw_ui_registry.h - the Atlas entry registry (U6): the entries of the menu tree (built in, and a mod's from its mod.json
 * "menus"), by parent, with ordering, caps, visibility and the online rule. Pure C: no game, no Lua. */
#ifndef GW_UI_REGISTRY_H
#define GW_UI_REGISTRY_H
#include "gw_ui_screen.h"
#ifdef __cplusplus
extern "C" {
#endif

#define AT_REG_MAX 96
#define AT_REG_PER_MOD_PARENT 6
#define AT_REG_VISIBLE_PER_PARENT 12
#define AT_REG_LABEL_MAX 18
enum { AT_ENTRY_OPENS = 1, AT_ENTRY_SCRIPT = 2, AT_ENTRY_NATIVE = 3 };
typedef struct {
    char id[AT_ID * 2], parent[AT_ID], mod[AT_ID], label[AT_REG_LABEL_MAX + 1], blurb[AT_TEXT], icon[AT_ID], after[AT_ID * 2],
         opens[AT_ID * 2], badge[8];
    int action, online, visible, builtin, native_arg;  /* native_arg: what the adapter runs for AT_ENTRY_NATIVE */
} AtEntry;
typedef struct { AtEntry e[AT_REG_MAX]; int n; char log[4][96]; int nlog; } AtRegistry;

void at_reg_init(AtRegistry *r);
int  at_reg_is_parent(const char *id);
const char *at_reg_owner(const char *parent);                       /* the mod id whose script may read and activate this parent's entries ("" none): lab.pause is geno-lab's, mods.<id> is <id>'s */
int  at_reg_parent_rendered(const char *id);                        /* the parents a menu draws today; a mod entry under another is refused */                              /* the built-in nodes, spec section 8.1 */
/* 1 added; 0 refused, with one line in r->log (the first four are kept). mod "" = a built-in entry. */
int  at_reg_add(AtRegistry *r, const AtEntry *e);
/* gd.ui.entry: only the owning mod may change an entry. visible < 0 leaves it; badge NULL leaves it. 1 done, 0 refused. */
int  at_reg_set(AtRegistry *r, const char *mod, const char *id, int visible, const char *badge);
void at_reg_drop_mod(AtRegistry *r, const char *mod);                /* a mod switched off or unloaded */
/* the visible children of parent in order (built-ins first in their own order, then mods by `after`, then by id);
 * netplay != 0 hides entries without `online` under versus and online. Returns the count (<= cap and <= 12). */
int  at_reg_children(const AtRegistry *r, const char *parent, int netplay, const AtEntry **out, int cap);

#ifdef __cplusplus
}
#endif
#endif
