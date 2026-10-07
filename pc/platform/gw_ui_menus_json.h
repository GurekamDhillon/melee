/* gw_ui_menus_json.h - reads the "menus" array of a mod.json text into AtEntry records (mod filled in). Pure C. */
#ifndef GW_UI_MENUS_JSON_H
#define GW_UI_MENUS_JSON_H
#include "gw_ui_registry.h"
#ifdef __cplusplus
extern "C" {
#endif

/* The number of entries read (at most cap); -1 on a syntax error (err says where). An element that is not valid is skipped and
 * noted in err. */
int at_menus_parse(const char *json, const char *mod, AtEntry *out, int cap, char *err, int errcap);

#ifdef __cplusplus
}
#endif
#endif
