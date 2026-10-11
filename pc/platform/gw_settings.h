/* Native persisted settings operations; strings contain bytes, not guest
 * multibyte fields. Prefix removal compacts the bounded table and saves once. */
#ifndef GW_SETTINGS_H
#define GW_SETTINGS_H
#ifdef __cplusplus
extern "C" {
#endif
int gw_Settings_RemovePrefix(const char* prefix);
#ifdef __cplusplus
}
#endif
#endif
