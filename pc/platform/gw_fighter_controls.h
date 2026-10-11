/* Per-player fighter choices. Scalars cross the game/native boundary; the
 * game's imports use unprefixed names (gwtool supplies gw_). */
#ifndef GW_FIGHTER_CONTROLS_H
#define GW_FIGHTER_CONTROLS_H
#define GW_FC_JUMP_MASK 3u
#define GW_FC_LAYOUT_MASK 0x3cu
#define GW_FC_EXTRA_MASK 0xff00u
#define GW_FC_CHOICE_MASK (GW_FC_JUMP_MASK | GW_FC_LAYOUT_MASK | GW_FC_EXTRA_MASK)
#define GW_FC_JUMP_BOTH 0u
#define GW_FC_JUMP_X 1u
#define GW_FC_JUMP_Y 2u
#define GW_FC_LAYOUTS 0
#define GW_FC_EXTRAS 1

#ifdef __cplusplus
extern "C" {
#endif
#if defined(__powerpc__)
#define GW_FC_API(name) name
#else
#define GW_FC_API(name) gw_##name
#endif
unsigned GW_FC_API(FighterControls_Get)(int port, int ck);
int GW_FC_API(FighterControls_Set)(int port, int ck, unsigned choice);
int GW_FC_API(FighterControls_Count)(int ck, int field);
void GW_FC_API(FighterControls_Name)(int ck, int field, int index, char* out, int cap);
unsigned GW_FC_API(FighterControls_Normalize)(int ck, unsigned choice);
void GW_FC_API(FighterControls_MatchSet)(int port, int ck, unsigned choice);
unsigned GW_FC_API(FighterControls_MatchGet)(int port);
int GW_FC_API(FighterControls_MatchCK)(int port);
void GW_FC_API(FighterControls_MatchClear)(void);
int GW_FC_API(FighterControls_Apply)(int port, int buttons);
#undef GW_FC_API
#if !defined(__powerpc__)
/* Native snapshots copy the COMPLETE frozen mapping, never reload preferences.
 * Save/Load return 1 on success; bad size/version/contents leave state intact. */
int gw_FighterControls_StateSize(void);
int gw_FighterControls_StateSave(void* out, int size);
int gw_FighterControls_StateLoad(const void* in, int size);
#endif
#ifdef __cplusplus
}
#endif
#endif
