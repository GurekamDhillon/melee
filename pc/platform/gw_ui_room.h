/* gw_ui_room.h - the online room screens (code entry, waiting room, lobby): a plain-data view, the stage-grid model,
 * the screen builder, the composite render and the input mapping. Pure C. It never includes a netplay header and never
 * calls a netplay function: the game-side adapter (gmfrontend_atlas_online.inc) fills AtRoomView from the readbacks the
 * legacy screens already used and applies the AtRoomIntents it gets back. */
#ifndef GW_UI_ROOM_H
#define GW_UI_ROOM_H
#include "gw_ui_input.h"
#include "gw_ui_online_parts.h"
#include "gw_ui_screen.h"
#ifdef __cplusplus
extern "C" {
#endif

#define AT_ROOM_STAGES 32      /* the lobby's stage list cap (LB_MAX_UI_STAGES in gmfrontend.c; the adapter asserts they match) */
#define AT_ROOM_NAME 40        /* the legacy buffers: stage names 40, fighter names 24, player names 18 */
#define AT_ROOM_TEXT 96
enum { AT_ROOM_CODE = 1, AT_ROOM_WAIT = 2, AT_ROOM_LOBBY = 3 };
enum { AT_PH_OFF, AT_PH_CHAR_BLIND, AT_PH_STRIKE, AT_PH_BAN, AT_PH_PICK, AT_PH_CHAR_WINNER, AT_PH_CHAR_LOSER, AT_PH_READY, AT_PH_GO };
enum { AT_STAGE_FREE, AT_STAGE_STRUCK_P1, AT_STAGE_STRUCK_P2, AT_STAGE_BANNED, AT_STAGE_PICKED };

/* ---- the stage grid: one model for drawing AND for the legacy cursor code ---- */
typedef struct { int col, row, page; } AtGridCell;
typedef struct { int page, row, group, count; } AtGridHead;     /* a group's label strip sits above `row` of `page` */
typedef struct { int cols, rows, pages, n, nhead; AtGridCell cell[AT_ROOM_STAGES]; AtGridHead head[16]; } AtGrid;
/* group[i] is 0 for a starter, 1 for a counterpick (starters first, as the netplay list is). Rows of `cols`; a group starts a
 * new row; `rows` rows to a page; when the list is grouped each group's header is drawn again at the top of a page it
 * continues on. */
void at_room_grid(const int *group, int n, int cols, int rows, AtGrid *g);
int at_room_pick_cols(int n, int grouped, int wide);            /* 3 for a short ungrouped list; else 4 compact, 6 wide */

typedef struct { char name[AT_ROOM_NAME]; int state, starter, open; } AtRoomStage;
typedef struct { char name[AT_ROOM_NAME], fighter[AT_ROOM_NAME]; int present, locked, ready, score; } AtRoomPlayer;

typedef struct AtRoomView {
    int kind;                                         /* AT_ROOM_* */
    /* the room */
    char code[8];
    int host, random_search, random_secs, found;      /* found: the opponent is in (the short beat before the lobby opens) */
    int rules_known, turbo, envoy, stage_all, stocks, minutes, delay;
    char status[AT_ROOM_TEXT];
    int ping_ms, link_bars;                           /* ping -1: no link; link_bars is the meter's memory, kept by the host shim */
    int copied;                                       /* the code was just copied */
    AtRoomPlayer pl[2]; int me;                       /* 0 host, 1 guest */
    /* code entry */
    AtCodeView code_in; char code_msg[AT_ROOM_TEXT]; int code_msg_bad;
    /* lobby */
    int phase, game, turn, left, first, countdown_s, coin_on, reward_open, reward_left_s;
    int reconnecting, leave_armed, my_stage_turn, can_pick, can_ready, ready_mine;
    char phase_title[24], instruction[AT_ROOM_TEXT];
    int n_stages, cursor; AtRoomStage st[AT_ROOM_STAGES]; AtGrid grid;
    char toast[AT_ROOM_TEXT]; int toast_bad;
    int fade_out;                                     /* 0..1000: the leave fade (the game's own fade does not cover host quads); drawn by the renderer's ground quad */
} AtRoomView;

/* The cursor to show: the view's own when it is a real stage, else the first open stage, else -1 (no list, nothing open). */
int at_room_cursor_fix(const AtRoomView *v);
/* Fill the screen record and the dynamic part of the view (keys, counter, explainer, the toast as a corner note) from a room view. 1 ok; 0 for an unknown kind. */
int at_room_fill(const AtRoomView *rv, AtScreen *sc, AtView *vw, double now_ms);

/* hit codes carried in AtHit.a (kind AT_HIT_ROOM); AtHit.b is the stage index or code slot */
enum { AT_RH_STAGE = 1, AT_RH_ACTION, AT_RH_LEAVE, AT_RH_COPY, AT_RH_CODE_SLOT, AT_RH_CODE_UP, AT_RH_CODE_DOWN };
/* Draws the primary of a room screen into L->primary and records hit rectangles (hits may be NULL: nothing is recorded).
 * A NULL view draws nothing. Returns how many hit rectangles did not fit in AtHits. */
int at_room_render(const AtRoomView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, AtHits *hits, double now_ms);

/* Input for the room screens. The PAD is not here: the game side reads its own pad bits, exactly as before. Keyboard and
 * mouse become intents, and the adapter turns each intent into the menu bit the equivalent pad press has. */
enum { AT_RI_NONE, AT_RI_UP, AT_RI_DOWN, AT_RI_LEFT, AT_RI_RIGHT, AT_RI_ACCEPT, AT_RI_BACK, AT_RI_START, AT_RI_COPY, AT_RI_PASTE,
       AT_RI_PAGE_L, AT_RI_PAGE_R, AT_RI_STAGE_AT, AT_RI_CODE_SLOT, AT_RI_STAGE_CLICK };   /* STAGE_CLICK: arg = stage; cursor there AND confirm, as one (applied only if both can be) */
typedef struct { int kind, arg; } AtRoomIntent;
int at_room_key_intents(AtKeys *k, unsigned mask, double now_ms, AtRoomIntent *out, int cap);
int at_room_mouse_intents(const AtRoomView *v, AtMouse *m, float x, float y, int buttons, int wheel, const AtHits *hits, AtRoomIntent *out, int cap);
const char *at_room_intent_name(int kind);
/* 0 on the code screen: the netplay layer polls the keyboard there itself (letters, arrows, Enter, Escape), so turning
 * the same keys into intents would act twice. The mouse is unaffected. */
int at_room_takes_keys(const AtRoomView *v);

#ifdef __cplusplus
}
#endif
#endif
