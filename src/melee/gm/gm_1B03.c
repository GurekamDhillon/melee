#include "gm_1B03.h"

#include <melee/mn/forward.h>
#include <melee/pl/forward.h>

#include "gm_unsplit.h"
#include "types.h"
#include "gmscenelaunch.h"
#include <string.h>
#include <dolphin/types.h>
#include <melee/mn/types.h>

/**
 * Update character tints
 *
 * If any two players are the same character, team, and subcolor (tint),
 * increment the tint of one of them.
 */
void gm_SetupSubColors(StartMeleeData* start)
{
    ssize_t i;
    ssize_t j;

    if (start->rules.is_teams != true) {
        return;
    }

    for (i = 0; i < GM_MAX_PLAYERS; i++) {
#if defined(TARGET_PC)
        if (SceneLaunch_UntintedEnemy(start,i)) { start->players[i].sub_color=0; continue; }
#endif
        for (j = 0; j < GM_MAX_PLAYERS; j++) {
#if defined(TARGET_PC)
            if (SceneLaunch_UntintedEnemy(start,j)) continue;
#endif
            if (i == j) {
                continue;
            }
            if (start->players[i].team == start->players[j].team &&
                start->players[i].ckind == start->players[j].ckind &&
                start->players[i].sub_color == start->players[j].sub_color)
            {
                start->players[j].sub_color++;
            }
        }
    }
}

#if defined(TARGET_PC)
/* Native fixture supplies teams=1;enemy_team_colors=1. Five identical enemies
 * retain the untinted base costume; holes still select P6's HUD anchor. */
int SceneLaunch_SixPresentationTest(int force)
{
    StartMeleeData start;
    int i;
    memset(&start,0,sizeof start);
    start.rules.x0_3=4;
    start.rules.is_teams=true;
    for (i=0; i<GM_MAX_PLAYERS; ++i) {
        start.players[i].slot_type=Gm_PKind_Cpu;
        start.players[i].ckind=CKind_Fox;
        start.players[i].team=i ? 1 : 0;
    }
    SceneLaunch_HudCount(&start);
    if (start.rules.x0_3!=6) return 1;
    SceneLaunch_ApplyTeamCostumes(&start);
    gm_SetupSubColors(&start);
    for (i=1; i<GM_MAX_PLAYERS; ++i) {
        int j;
        if (force) {
            if (start.players[i].sub_color ||
                start.players[i].color!=gm_801692BC(CKind_Fox)) return 1;
        } else {
            if (start.players[i].sub_color>4) return 1;
            for (j=1; j<i; ++j)
                if (start.players[i].sub_color==start.players[j].sub_color) return 1;
        }
    }
    start.players[4].slot_type=Gm_PKind_NA;
    start.rules.x0_3=4;
    SceneLaunch_HudCount(&start);
    if (start.rules.x0_3!=6) return 1;
    start.players[5].slot_type=Gm_PKind_NA;
    start.rules.x0_3=4;
    SceneLaunch_HudCount(&start);
    return start.rules.x0_3!=4;
}
#endif

static inline void player_standings_inline(StartMeleeData* arg0,
                                           MatchEnd* arg1, int i,
                                           u32 is_big_loser, int var_r7)
{
    if (is_big_loser == 0 && var_r7 > 0) {
        s8 var_r6 = arg1->player_standings[i].ckind;
        if (var_r6 == 0x12 || var_r6 == 0x13) {
            if (arg1->player_standings[i].ftkind == 7) {
                var_r6 = 0x13;
            } else {
                var_r6 = 0x12;
            }
        }
        arg0->players[i].ckind = var_r6;
        arg0->players[i].stocks = 1;
        arg0->players[i].damage1 = 300;
    } else {
        arg0->players[i].slot_type = Gm_PKind_NA;
    }
}

static inline int gm_801B0474_inline(MatchEnd* arg1, int i)
{
    if (arg1->match_kind == 1) {
        if (arg1->outcome == OUTCOME_TIMEOUT) {
            return arg1->player_standings[i].stocks;
        } else {
            u8 var_r7 = arg1->player_standings[i].stocks;
            if (arg1->player_standings[i].x28 < arg1->frame_count ||
                var_r7 != 0)
            {
                return var_r7;
            }
        }
    }
    return 1;
}

void gm_SetupSuddenDeath(StartMeleeData* start, MatchEnd* end)
{
    int var_r7;
    int i;

    start->rules.match_kind = MatchKind_Stock;
    start->rules.timer_enabled = false;
    start->rules.x2_5 = false;

    for (i = 0; i < GM_MAX_PLAYERS; i++) {
        if (start->players[i].slot_type != Gm_PKind_NA) {
            var_r7 = gm_801B0474_inline(end, i);
            if (end->is_teams == 1) {
                player_standings_inline(
                    start, end, i,
                    end->team_standings[end->player_standings[i].team]
                        .is_big_loser,
                    var_r7);
            } else {
                player_standings_inline(start, end, i,
                                        end->player_standings[i].is_big_loser,
                                        var_r7);
            }
        }
    }
}

void gm_801B05F4(PlayerInitData* player, int slot)
{
    player->slot = slot + 1;
    if (slot == 2) {
        slot = 3;
    } else if (slot == 3) {
        slot = 2;
    }
    player->team = slot;
}

void gm_SetupHumanPlayer(PlayerInitData* player, u8 ckind, u8 color, u8 stocks,
                         u8 slot)
{
    player->slot_type = Gm_PKind_Human;
    player->ckind = ckind;
    player->color = color;
    player->stocks = stocks;
    gm_801B05F4(player, slot);
}

void gm_SetupCpuPlayer(PlayerInitData* arg0, u8 ckind, u8 color, u8 stocks,
                       u8 slot)
{
    arg0->slot_type = Gm_PKind_Cpu;
    arg0->ckind = ckind;
    arg0->color = color;
    arg0->stocks = stocks;
    gm_801B05F4(arg0, slot);
    arg0->team = 4;
}

void gm_801B06B0(CSSData* css_data, u8 type, s8 c_kind, s8 stocks, s8 color,
                 u8 arg5, u8 level, u8 slot)
{
    gm_InitVsMode(&css_data->vs);
    css_data->match_type = type;
    css_data->unk_0x0 = slot + 1;
    css_data->vs.start.players[slot].ckind = c_kind;
    css_data->vs.start.players[slot].stocks = stocks;
    css_data->vs.start.players[slot].color = color;
    css_data->vs.start.players[slot].cpu_level = level;
    css_data->vs.start.players[slot].nametag = arg5;
    css_data->vs.start.players[0].cpu_level = level;
    css_data->vs.start.players[0].stocks = stocks;
}

void gm_801B0730(CSSData* css_data, s8* c_kind, u8* stocks, u8* color,
                 u8* nametag, u8* level)
{
    s32 slot;

    slot = css_data->unk_0x0 - 1;
    if (c_kind != NULL) {
        *c_kind = css_data->vs.start.players[slot].ckind;
    }
    if (stocks != NULL) {
        *stocks = css_data->vs.start.players[slot].stocks;
    }
    if (color != NULL) {
        *color = css_data->vs.start.players[slot].color;
    }
    if (level != NULL) {
        *level = css_data->vs.start.players[slot].cpu_level;
    }
    if (nametag != NULL) {
        *nametag = css_data->vs.start.players[slot].nametag;
    }
}

void gm_801B07B4(CSSData* css_data, s8 c_kind, s8 stocks, s8 color, u8 arg4,
                 u8 level, u8 arg6)
{
    s32 var_r0;

    if (arg6 == 0) {
        var_r0 = 1;
    } else {
        var_r0 = 0;
    }
    css_data->vs.start.players[var_r0].ckind = c_kind;
    css_data->vs.start.players[var_r0].stocks = stocks;
    css_data->vs.start.players[var_r0].color = color;
    css_data->vs.start.players[var_r0].cpu_level = level;
    css_data->vs.start.players[var_r0].nametag = arg4;
}

void gm_801B07E8(CSSData* css_data, s8* c_kind, s8* stocks, s8* color,
                 s8* arg4, u8* level)
{
    s32 slot;

    if ((css_data->unk_0x0 - 1) == 0) {
        slot = 1;
    } else {
        slot = 0;
    }
    if (c_kind != NULL) {
        *c_kind = css_data->vs.start.players[slot].ckind;
    }
    if (stocks != NULL) {
        *stocks = css_data->vs.start.players[slot].stocks;
    }
    if (color != NULL) {
        *color = css_data->vs.start.players[slot].color;
    }
    if (level != NULL) {
        *level = css_data->vs.start.players[slot].cpu_level;
    }
    if (arg4 != NULL) {
        *arg4 = css_data->vs.start.players[slot].nametag;
    }
}
