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

#include <dolphin/os.h>
#include <melee/gm/forward.h>
#include <melee/lb/lbdvd.h>
#include <melee/lb/types.h>
#include <melee/mn/types.h>
#include <melee/pl/forward.h>
#include <melee/gr/forward.h>

#include "gm_1601.h"
#include "gm_1884.h"
#include "gm_1A3F.h"
#include "gmmain_lib.h"

/* Port side (pc/platform/gw_runtime.c). */
int SceneLaunch_Active(void);
int SceneLaunch_BootGameMode(void);
int SceneLaunch_VsModeIndex(void);
int SceneLaunch_EntryStateId(void);
int SceneLaunch_StageExternal(void);
int SceneLaunch_TargetTestCKind(void);
int SceneLaunch_Teams(void);
int SceneLaunch_EnemyTeamColors(void);
int SceneLaunch_TimeLimit(void);
int SceneLaunch_ItemFreq(void);
int SceneLaunch_RuleMatch(void);
int SceneLaunch_RuleStocks(void);
int SceneLaunch_RuleMinutes(void);
int SceneLaunch_RulePause(void);
int SceneLaunch_SkipMemcard(void);
int SceneLaunch_PlayerCKind(int slot);
int SceneLaunch_PreloadCKind(int n);
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

/* Off is a parser sentinel (-2), not frequency index zero. Apply to both the
 * immediate rules and saved preferences from which VS reconstructs the match. */
static void SceneLaunch_ApplyItemFrequency(StartMeleeRules* rules, struct GamePrefs* prefs)
{
    int frequency = SceneLaunch_ItemFreq();
    if (frequency != -1) {
        rules->item_freq = (s8) (frequency == -2 ? -1 : frequency);
        prefs->item_freq = (u8) rules->item_freq;
    }
}

/* Highest occupied slot selects the HUD layout, including a lone P6. */
static void SceneLaunch_HudCount(StartMeleeData* start)
{
    int i;
    for (i=4; i<GM_MAX_PLAYERS; ++i)
        if (start->players[i].slot_type != Gm_PKind_NA)
            start->rules.x0_3 = i+1;
}

/* Same mappings as the CSS. Share costumes without duplicate tinting on the
 * opposing team; the five CPU slots need no fifth (out-of-bounds) tint. */
static int SceneLaunch_UntintedEnemy(StartMeleeData* start, int i)
{
    return SceneLaunch_Active() && SceneLaunch_EnemyTeamColors() &&
           start->rules.is_teams && start->players[i].slot_type != Gm_PKind_NA &&
           start->players[i].team != start->players[0].team;
}

static void SceneLaunch_ApplyTeamCostumes(StartMeleeData* start)
{
    int i;
    for (i=0; i<GM_MAX_PLAYERS; ++i) {
        PlayerInitData* p=&start->players[i];
        if (SceneLaunch_UntintedEnemy(start,i)) {
            p->color = p->team==1 ? gm_801692BC(p->ckind) :
                       p->team==2 ? gm_80169290(p->ckind) : gm_80169264(p->ckind);
            p->sub_color=0;
        }
    }
}

/* Separate player seeding from preload/scene side effects for suite fixtures. */
static int SceneLaunch_SeedPlayers(VsModeData* vs, bool dummy_fallback)
{
    int i, seeded = 0;
    for (i = 0; i < GM_MAX_PLAYERS; i++) {
        int ckind = SceneLaunch_PlayerCKind(i);
        int v;
        if (i>=4) gm_SetupPlayerDefaults(&vs->start.players[i]);
        if (ckind < 0) {
            /* Extra slots must not inherit Multi-Man/previous-match state. */
            if (i >= 4) {
                vs->start.players[i].ckind = ChKind_None;
                vs->start.players[i].slot_type = Gm_PKind_NA;
            }
            continue;
        }
        vs->start.players[i].ckind = (s8) ckind;
        /* A named slot is a human by default; the seeder only overrides what was asked for. */
        v = SceneLaunch_PlayerSlotType(i);
        /* `none` (ChKind_None) is an empty slot, as on the CSS - never a fighter: its
         * ftMapping_list row is zeros, so a Human/CPU "none" would build Mario with nothing
         * preloaded (see the extra-CPU note below). */
        vs->start.players[i].slot_type =
            (u8) (v >= 0 ? v : ckind == ChKind_None ? Gm_PKind_NA : Gm_PKind_Human);
        v = SceneLaunch_PlayerColor(i);
        {
            int costumes = ckind == ChKind_None ? 1 : gm_GetNumCostumesForCKind(ckind);
            if (costumes <= 0) costumes = 1;
            vs->start.players[i].color = (u8) ((v >= 0 ? v : i) % costumes);
        }
        v = SceneLaunch_PlayerCpuKind(i);
        if (v >= 0) {
            vs->start.players[i].cpu_kind = (u8) v;
        } else if (vs->start.players[i].slot_type == Gm_PKind_Cpu) {
            /* VS mode's own CPUs are kind 4, the fighting AI (gmvsmode.c); 0 is Training's
               stand-still dummy, which is what every scene-launched VS CPU used to get - they
               never moved (GD). Training keeps 0. */
            vs->start.players[i].cpu_kind = dummy_fallback ? 0 : 4;
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
        } else if (i>=4 && SceneLaunch_Teams()==1) {
            vs->start.players[i].team = 1;
        }
        v = SceneLaunch_PlayerStocks(i);
        if (v >= 0) {
            vs->start.players[i].stocks = (s8) v;
        }
        v = SceneLaunch_PlayerNametag(i);
        if (v >= 0) {
            vs->start.players[i].nametag = (u8) v;
        }
        /* What the CSS would have written (gmvs.c's player setup reads both):
         *  - slot = the player id + 1 (0 = "its own index"), i.e. the P1..P4 tag. Writing the bare
         *    index made port 2 player id 0: two "P1"s.
         *  - sub_color is the fighter's SHADE, not a controller: Fighter_UnkInitLoad_80068914
         *    (fighter.c, "fighter sub color num over!") tints the model with
         *    ftCommon x6DC_colorsByPlayer[sub_color - 1] when it is non-zero, the light/dark
         *    variants the game gives same-team duplicates (gm_1B03.c bumps it itself). The pad
         *    a player reads comes from the player id (slot, above). Writing the slot index
         *    here lightened or darkened every fighter after slot 1. */
        vs->start.players[i].slot = (u8) (i + 1);
        vs->start.players[i].sub_color = 0;
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
        vs->start.players[1].slot = 2;
        vs->start.players[1].sub_color = 0;
    }

    /* Training's extra CPUs (the pause menu's CPU count, 2 and 3 - gm_1884.c fn_80188550). The
     * CSS exit (gm_801B1C24) fills players[2..3] with copies of the CPU in the next free costumes
     * and names them in the preload cache; jumping straight to the match skips it, so they stayed
     * ChKind_None. ChKind_None's ftMapping_list row is all zeros in the port (retail reads past
     * the table), so the first extra CPU built Mario: none of his files were preloaded, and
     * lbFile_800168A0 put PlMrAJ.dat (1.2 MB) in the ARAM heap 1, which only has ~155 KB -
     * ALLOC_FAIL heap 1 and a crash. Do what the CSS exit does; a slot the scene names keeps it. */
    if (dummy_fallback && vs->start.players[1].ckind >= 0 &&
        vs->start.players[1].ckind != ChKind_None)
    {
        for (i = 2; i < 4; i++) {
            PlayerInitData* p = &vs->start.players[i];
            u8 ncost;
            if (SceneLaunch_PlayerCKind(i) >= 0) {
                continue;
            }
            *p = vs->start.players[1];
            ncost = gm_GetNumCostumesForCKind(p->ckind);
            if (ncost == 0) {
                ncost = 1;
            }
            p->color = (vs->start.players[i - 1].color + 1) % ncost;
            if (p->color == vs->start.players[0].color) {
                p->color = (p->color + 1) % ncost;
            }
            p->slot_type = Gm_PKind_NA; /* the menu turns them on (fn_80188550) */
            p->slot = (u8) (i + 1);
            p->sub_color = 0;
        }
    }

    return seeded;
}

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

    seeded = SceneLaunch_SeedPlayers(vs, dummy_fallback);

    if (SceneLaunch_Teams() >= 0) {
        vs->start.rules.is_teams = (u8) SceneLaunch_Teams();
    }
    SceneLaunch_HudCount(&vs->start);
    SceneLaunch_ApplyTeamCostumes(&vs->start);
    if (SceneLaunch_TimeLimit() >= 0) {
        vs->start.rules.timer_enabled = SceneLaunch_TimeLimit() != 0;
        vs->start.rules.time_limit = (u32) SceneLaunch_TimeLimit();
    }

    /* The saved rules. VS mode rebuilds every match's rules from GameRules/GamePrefs when it
     * starts the match (gm_80167BC8), so a time or stock count set only in vs->start is lost -
     * the "time=480 but the clock says 2:00" bug. Anything configured is written there too,
     * which also makes two netplay peers agree whatever their memory cards hold. */
    {
        GameRules* gr = gmMainLib_GetGameRules();
        struct GamePrefs* gp = gmMainLib_GetGamePrefs();
        bool any = false;
        int player_stocks = -1, k;
        /* per-player /stocksN: GameRules has one stock count, so the largest one named */
        for (k = 0; k < GM_MAX_PLAYERS; k++) {
            if (SceneLaunch_PlayerStocks(k) > player_stocks) {
                player_stocks = SceneLaunch_PlayerStocks(k);
            }
        }
        if (SceneLaunch_RuleMatch() >= 0) {
            gr->mode = (u8) SceneLaunch_RuleMatch();
            any = true;
        } else if (SceneLaunch_RuleStocks() >= 0 || player_stocks >= 0) {
            gr->mode = 1; /* a stock count with no match= means a stock match (it ends at 0) */
            any = true;
        } else if (SceneLaunch_TimeLimit() > 0 || SceneLaunch_RuleMinutes() > 0) {
            gr->mode = 0; /* a time with no match= means a timed match */
            any = true;
        }
        if (SceneLaunch_RuleStocks() >= 0) {
            gr->stock_count = (u8) SceneLaunch_RuleStocks();
            any = true;
        } else if (player_stocks >= 0) {
            gr->stock_count = (u8) player_stocks;
            any = true;
        }
        if (SceneLaunch_RuleMinutes() >= 0) {
            gr->time_limit = (u8) SceneLaunch_RuleMinutes();
            gr->stock_time_limit = (u8) SceneLaunch_RuleMinutes();
            any = true;
        } else if (SceneLaunch_TimeLimit() >= 0) {
            /* GameRules counts whole minutes and 0 means no limit: round seconds up, so time=1..60
             * is a one-minute match (it truncated to 0 before - a match with no end) */
            int minutes = (SceneLaunch_TimeLimit() + 59) / 60;
            gr->time_limit = (u8) minutes;
            gr->stock_time_limit = (u8) minutes;
            any = true;
        }
        if (SceneLaunch_RulePause() >= 0) {
            gr->pause = (u8) SceneLaunch_RulePause();
        }
        SceneLaunch_ApplyItemFrequency(&vs->start.rules, gp);
        if (any) {
            /* the rest of a clean competitive rule set: no handicap, 1.0x damage */
            gr->handicap = 0;
            gr->damage_ratio = 10;
        }
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
    for (i = 0; i < GM_MAX_PLAYERS; i++) {
        cache->game_cache.entries[i].char_id = vs->start.players[i].ckind;
        cache->game_cache.entries[i].color = vs->start.players[i].color;
        if (vs->start.players[4].slot_type != Gm_PKind_NA ||
            vs->start.players[5].slot_type != Gm_PKind_NA || SceneLaunch_PreloadCKind(0) >= 0)
            cache->game_cache.entries[i].x5 = 1;
    }
    for (i = 0; i < 2; ++i) {
        int ck = SceneLaunch_PreloadCKind(i);
        cache->game_cache.entries[i + GM_MAX_PLAYERS].char_id = ck >= 0 ? ck : ChKind_None;
        cache->game_cache.entries[i + GM_MAX_PLAYERS].color = 0;
        cache->game_cache.entries[i + GM_MAX_PLAYERS].x5 = 1;
    }
    /* Six-slot file admission is checked before starting asynchronous loads. */
    if ((vs->start.players[4].slot_type != Gm_PKind_NA ||
         vs->start.players[5].slot_type != Gm_PKind_NA || SceneLaunch_PreloadCKind(0) >= 0) && !lbDvd_TrySetupSixSlotCache()) {
        gm_SetupAllPlayerDefaults(vs->start.players);
        for (i = 0; i < 8; ++i) cache->game_cache.entries[i].char_id = ChKind_None;
        OSReport("gw: scene: six-slot memory admission refused; returning to selection without loading fighters\n");
        gm_SetGameModeStateId(0);
        return false;
    }
    if (vs->start.players[4].slot_type == Gm_PKind_NA &&
        vs->start.players[5].slot_type == Gm_PKind_NA && SceneLaunch_PreloadCKind(0) < 0) lbDvd_SetupVsPreloadCache();

    entry = SceneLaunch_EntryStateId();
    if (dummy_fallback && entry == 2 && (vs->start.players[0].ckind < 0 ||
                       vs->start.players[0].ckind == ChKind_None))
    {
        /* Training with no player 1 fighter (the field was refused or never given): the match
         * would build ChKind_None - Mario, unpreloaded. Open the CSS instead. */
        OSReport("gw: scene: player 1 has no fighter - opening the CSS instead of the match\n");
        entry = 0;
    }
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
