/* A deterministic space of room views: every screen kind, every lobby phase, host and guest, link states, rules, list sizes. */
#ifndef ATLAS_ROOM_FIXTURE_H
#define ATLAS_ROOM_FIXTURE_H
#include <stdio.h>
#include <string.h>
#include "../platform/gw_ui_room.h"
#include "../platform/gw_ui_render.h"

static const char *const FX_STAGES[] = { "Battlefield", "Final Destination", "Yoshi's Story", "Dream Land", "Fountain of Dreams",
    "Pokemon Stadium", "Princess Peach's Castle", "Stage 147", "Kongo Jungle", "Great Bay", "Brinstar Depths", "Mushroom Kingdom II" };
static const char *const FX_NAMES[] = { "GD", "Opponent", "WWWWWWWWWWWWWWW", "a b c d e f g" };   /* 15 wide characters and spaces */
static const int FX_LISTS[] = { 0, 5, 6, 12, 32 };
static const int FX_PINGS[] = { -1, 20, 95, 260 };
#define FX_COUNT (3 * 9 * 2 * 2 * 4 * 5 * 2)   /* kinds x phases x host x reconnecting x pings x lists x rules */

static void fx_view(int idx, AtRoomView *v)
{
    int kind = 1 + idx % 3, phase = (idx / 3) % 9, host = (idx / 27) % 2, recon = (idx / 54) % 2, ping = (idx / 108) % 4,
        list = (idx / 432) % 5, rules = (idx / 2160) % 2, i;
    char code[8];
    memset(v, 0, sizeof *v);
    v->kind = kind; v->phase = phase; v->host = host; v->me = host ? 0 : 1; v->reconnecting = recon;
    v->ping_ms = FX_PINGS[ping]; v->rules_known = rules ? 1 : host;
    v->link_bars = at_link_bars(v->ping_ms, 0);
    v->turbo = rules; v->envoy = rules; v->stage_all = rules; v->stocks = 4; v->minutes = 8; v->delay = 2;
    snprintf(code, sizeof code, "%s", "ABCD"); memcpy(v->code, code, 5);
    for (i = 0; i < 2; i++) {
        snprintf(v->pl[i].name, sizeof v->pl[i].name, "%s", FX_NAMES[(idx + i) % 4]);
        snprintf(v->pl[i].fighter, sizeof v->pl[i].fighter, "%s", i ? "Captain Falcon" : "Fox");
        v->pl[i].present = i == 0 || kind != AT_ROOM_WAIT || (idx & 1);
        v->pl[i].locked = phase > AT_PH_CHAR_BLIND; v->pl[i].ready = phase == AT_PH_GO || (phase == AT_PH_READY && i == 0);
    }
    v->game = 2; v->pl[0].score = 1; v->pl[1].score = 0; v->turn = 0; v->left = 2;
    v->countdown_s = phase == AT_PH_READY ? 3 : 0; v->coin_on = phase == AT_PH_STRIKE;
    v->my_stage_turn = (phase == AT_PH_STRIKE || phase == AT_PH_BAN || phase == AT_PH_PICK) && v->me == v->turn;
    v->can_pick = phase == AT_PH_CHAR_BLIND || phase == AT_PH_CHAR_WINNER || phase == AT_PH_CHAR_LOSER;
    v->can_ready = phase == AT_PH_READY; v->ready_mine = v->pl[v->me].ready;
    snprintf(v->phase_title, sizeof v->phase_title, "%s", "STAGE STRIKING");
    snprintf(v->instruction, sizeof v->instruction, "%s", "Strike 2 stages: move, then press A. It stays hidden till both lock.");
    snprintf(v->status, sizeof v->status, "%s", "Connected to the matchmaking server, waiting for the other player to join your room.");
    v->code_in.c[0] = 'A'; v->code_in.c[1] = 'B'; v->code_in.slot = idx % 4; v->code_in.invalid = idx % 5 == 0;
    snprintf(v->code_msg, sizeof v->code_msg, "%s", v->code_in.invalid ? "No room with that code." : "");
    v->code_msg_bad = v->code_in.invalid;
    {
        int group[AT_ROOM_STAGES];
        v->n_stages = FX_LISTS[list];
        for (i = 0; i < v->n_stages; i++) {
            snprintf(v->st[i].name, sizeof v->st[i].name, "%s", FX_STAGES[i % 12]);
            group[i] = i >= 5 ? 1 : 0; v->st[i].starter = group[i] == 0; v->st[i].open = i % 4 != 3;
            v->st[i].state = i % 4 == 3 ? AT_STAGE_BANNED : AT_STAGE_FREE;
        }
        at_room_grid(group, v->n_stages, at_room_pick_cols(v->n_stages, v->n_stages > 5, idx & 4), 3, &v->grid);
    }
    v->cursor = v->n_stages > 1 ? 1 : 0;
}
#endif
