#ifndef GW_ROSTER_RUNTIME_H
#define GW_ROSTER_RUNTIME_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Boot catalogue only. Existing gw_Mex_* / UI_Fighter* APIs remain byte-id APIs.
 * Integration callers must explicitly retain wide identities through selection,
 * prepare resident tables and save/restore the versioned match map.
 */
int gw_Roster_NativeCount(void);
int gw_Roster_IdAt(int index);
const char* gw_Roster_Name(int id);
int gw_Roster_SourceCK(int id);
int gw_Roster_SourceInternal(int id);
uint64_t gw_Roster_Hash(void);
/* Read-only full-roster enumeration: legacy icons first, native rows appended.
 * SourceIconAt reports the legacy icon to reuse; it never wraps a wide id.
 */
int gw_UI_RosterCount(void);
int gw_UI_RosterIdAt(int index);
int gw_UI_RosterSourceIconAt(int index);
#ifdef __cplusplus
}
#endif
#endif
