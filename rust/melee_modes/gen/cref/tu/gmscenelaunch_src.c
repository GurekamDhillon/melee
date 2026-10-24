/* The scene-launch seeders of src/melee/gm/gmscenelaunch.h are `static` header functions that every
 * including TU compiles in. This wrapper TU is what the oracle compiles: it includes the header
 * verbatim and exports one entry per seeder, so the generator stubs the header's callees. */
#include <melee/gm/gmscenelaunch.h>

int w_SeedPlayers(VsModeData* vs, int dummy_fallback) { return SceneLaunch_SeedPlayers(vs, dummy_fallback); }
int w_SeedVs(VsModeData* vs, int dummy_fallback) { return SceneLaunch_SeedVs(vs, dummy_fallback); }
void w_ApplyItemFrequency(StartMeleeRules* rules, struct GamePrefs* prefs) { SceneLaunch_ApplyItemFrequency(rules, prefs); }
