#ifndef MELEE_GM_GMSCENELAUNCH_H
#define MELEE_GM_GMSCENELAUNCH_H

/*
 * Scene launch: boot straight into any screen, in any configuration, from one environment
 * variable. The grammar, every field and which index space each number is in are documented in
 * _research/scene-launch.md; the parser is pc/platform/gw_runtime.c's SCENE LAUNCH block.
 *
 * This header is deliberately header-only: it adds no translation unit, so it needs no entry in
 * _build/masstest/files.txt or _build/melee_link_objects.rsp. The port-side functions below are
 * native shims - gwtool prefixes every game symbol with `gw_`, so game code calls the unprefixed
 * name and the linker binds it to gw_SceneLaunch_*.
 *
 * WHAT A CORRECT SEED HAS TO DO, and why (this is the part that bites):
 *
 *  - players[1] must never be left at ChKind_None. ftMapping_list is [ChKind_Cap] and a
 *    ChKind_None (0x21) dummy indexes the retail sentinel row, so the match scene builds a
 *    fighter with a garbage id and exhausts the heap in lbMemory_80014FC8.
 *  - all THREE stage seeds have to agree: StartMeleeRules::stkind, gm_80473814.stage_id and the
 *    preload cache's game_cache.stkind. lbDvd_SetupVsPreloadCache reads the cache, and the SSS
 *    exit handler (gm_801B1EEC) sets the other two - jumping straight to the match skips it.
 *  - the preload cache's per-player entries must name the characters actually being loaded, or
 *    the cache holds the wrong files.
 *
 * All of that is what gm_Mode_Training_OnLoad used to do by hand for exactly one configuration.
 */

#if defined(TARGET_PC)

#include <melee/gm/forward.h>
#include <melee/lb/lbdvd.h>
#include <melee/lb/types.h>
#include <melee/mn/types.h>
#include <melee/pl/forward.h>
#include <melee/gr/forward.h>

#include "gm_1884.h"
#include "gm_1A3F.h"

/* Port side (pc/platform/gw_runtime.c). */
int SceneLaunch_Active(void);
int SceneLaunch_BootGameMode(void);
int SceneLaunch_VsModeIndex(void);
int SceneLaunch_EntryStateId(void);
int SceneLaunch_StageExternal(void);
int SceneLaunch_TargetTestCKind(void);
int SceneLaunch_Teams(void);
int SceneLaunch_TimeLimit(void);
int SceneLaunch_ItemFreq(void);
int SceneLaunch_SkipMemcard(void);
int SceneLaunch_PlayerCKind(int slot);
int SceneLaunch_PlayerSlotType(int slot);
int SceneLaunch_PlayerColor(int slot);
int SceneLaunch_PlayerCpuKind(int slot);
int SceneLaunch_PlayerCpuLevel(int slot);
int SceneLaunch_PlayerHandicap(int slot);
int SceneLaunch_PlayerTeam(int slot);
int SceneLaunch_PlayerStocks(int slot);
int SceneLaunch_PlayerNametag(int slot);
int SceneReport_Enabled(void);
void SceneReport_State(int phase, int mode, int state_id, int scene_kind);
void SceneReport_Cursor(const char* what, int a, int b);
void SceneReport_Memcard(int decision, int option);
void SceneReport_Menu(int kind, int hovered, int confirmed);

/**
 * @brief Seeds a VsModeData from the configured scene and jumps the mode's state machine to the
 *        requested screen.
 *
 * @param vs             the mode's own VsModeData row (gmMainLib_804D3EE0->modes.table[...])
 * @param dummy_fallback when true (Training), an unconfigured player 1 is filled with player 0's
 *                       character rather than left empty - see the ChKind_None note above.
 * @return true when a scene was applied, false when nothing was configured.
 *
 * Call it from the mode's on_load, which runs before the first state is entered.
 */
static bool SceneLaunch_SeedVs(VsModeData* vs, bool dummy_fallback)
{
    PreloadedGameModeState* cache;
    int entry;
    int stkind;
    int i;
    int seeded = 0;

    if (!SceneLaunch_Active() || SceneLaunch_VsModeIndex() < 0) {
        return false;
    }

    for (i = 0; i < 4; i++) {
        int ckind = SceneLaunch_PlayerCKind(i);
        int v;
        if (ckind < 0) {
            continue;
        }
        vs->start.players[i].ckind = (s8) ckind;
        /* A named slot is a human by default; the seeder only overrides what was asked for. */
        v = SceneLaunch_PlayerSlotType(i);
        vs->start.players[i].slot_type = (u8) (v >= 0 ? v : Gm_PKind_Human);
        v = SceneLaunch_PlayerColor(i);
        vs->start.players[i].color = (u8) (v >= 0 ? v : i);
        v = SceneLaunch_PlayerCpuKind(i);
        if (v >= 0) {
            vs->start.players[i].cpu_kind = (u8) v;
        } else if (vs->start.players[i].slot_type == Gm_PKind_Cpu) {
            vs->start.players[i].cpu_kind = 0;
        }
        v = SceneLaunch_PlayerCpuLevel(i);
        if (v >= 0) {
            vs->start.players[i].cpu_level = (u8) v;
        }
        v = SceneLaunch_PlayerHandicap(i);
        if (v >= 0) {
            vs->start.players[i].handicap = (s8) v;
        }
        v = SceneLaunch_PlayerTeam(i);
        if (v >= 0) {
            vs->start.players[i].team = (u8) v;
        }
        v = SceneLaunch_PlayerStocks(i);
        if (v >= 0) {
            vs->start.players[i].stocks = (s8) v;
        }
        v = SceneLaunch_PlayerNametag(i);
        if (v >= 0) {
            vs->start.players[i].nametag = (u8) v;
        }
        vs->start.players[i].slot = (u8) i;
        seeded++;
    }

    /* Training's dummy. Leaving it at ChKind_None walks off the end of ftMapping_list's retail
     * rows and exhausts the heap; see the header comment. */
    if (dummy_fallback && vs->start.players[0].ckind >= 0 &&
        SceneLaunch_PlayerCKind(1) < 0)
    {
        vs->start.players[1] = vs->start.players[0];
        vs->start.players[1].slot_type = Gm_PKind_Cpu;
        vs->start.players[1].cpu_kind = 0;
        vs->start.players[1].cpu_level = 0;
        vs->start.players[1].color = 1;
        vs->start.players[1].slot = 1;
    }

    if (SceneLaunch_Teams() >= 0) {
        vs->start.rules.is_teams = (u8) SceneLaunch_Teams();
    }
    if (SceneLaunch_TimeLimit() >= 0) {
        vs->start.rules.timer_enabled = SceneLaunch_TimeLimit() != 0;
        vs->start.rules.time_limit = (u32) SceneLaunch_TimeLimit();
    }
    if (SceneLaunch_ItemFreq() >= 0) {
        vs->start.rules.item_freq = (s8) SceneLaunch_ItemFreq();
    }

    /* The three stage seeds, which must agree. */
    stkind = SceneLaunch_StageExternal();
    if (stkind < 0) {
        stkind = St_Kind_Izumi;
    }
    vs->start.rules.stkind = (StKind) stkind;
    gm_80473814.stage_id = (s16) stkind;

    cache = lbDvd_GetPreloadCacheScene();
    cache->game_cache.stkind = (StKind) stkind;
    for (i = 0; i < 4; i++) {
        cache->game_cache.entries[i].char_id = vs->start.players[i].ckind;
        cache->game_cache.entries[i].color = vs->start.players[i].color;
    }
    lbDvd_SetupVsPreloadCache();

    entry = SceneLaunch_EntryStateId();
    if (entry >= 0) {
        /* CSS is 0, SSS is 1 and the playable state is 2 in both GM_VS and GM_TRAINING. Entering
         * at the CSS or the SSS keeps the seed as that screen's starting selection. */
        gm_SetGameModeStateId((u8) entry);
    }
    (void) seeded;
    return true;
}

#endif /* TARGET_PC */
#endif
