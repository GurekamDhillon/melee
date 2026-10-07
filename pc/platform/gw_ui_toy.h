/* gw_ui_toy.h - the chrome text of the three framed trophy scenes (Trophy Gallery, Lottery, Collection). Pure C, no game.
 * The game side reads the numbers (and, once the retail text decodes, the strings) and these functions decide what Atlas says: clamped,
 * clipped to the explainer's limits, with authored fallbacks. Names and descriptions come in as UTF-8 and are never stored or logged here. */
#ifndef GW_UI_TOY_H
#define GW_UI_TOY_H
#ifdef __cplusplus
extern "C" {
#endif

#define AT_TOY_TITLE 64     /* the explainer's title limit */
#define AT_TOY_WHAT 112     /* the one rule line: at most 110 bytes */
#define AT_TOY_FROM 64
#define AT_TOY_COUNTER 24

/* Gallery. index is 0-based and clamped into [0, count - 1] (an m-ex trophy past the retail count, a negative index); count <= 0 is an
 * empty gallery: the title says so and the counter is empty. An empty name gives "TROPHY n"; the rule line is the description's first
 * sentence (through the first '.', '!' or '?' that is followed by a space or the end), clipped with an ellipsis; from is the series text. */
typedef struct { char title[AT_TOY_TITLE], what[AT_TOY_WHAT], from[AT_TOY_FROM], counter[AT_TOY_COUNTER]; } AtToyChrome;
void at_toy_chrome(AtToyChrome *c, int index, int count, const char *name, const char *desc, const char *series);

/* Lottery. coins is the balance the machine shows (gm_801623D8() / 10), bet the coins the player has chosen (retail allows 1 to 20, never more
 * than the balance). With no coins the line says where coins come from; otherwise it says what a bet does (retail's chance readout rises with it:
 * tyFigupon.c). No price or odds number is invented. */
typedef struct { char title[AT_TOY_TITLE], what[AT_TOY_WHAT], counter[AT_TOY_COUNTER]; } AtLotChrome;
void at_lot_chrome(AtLotChrome *c, int coins, int bet);

/* Collection (the trophy room). total is Toy_GetTrophyTotal(). The room has no cursor and no selected trophy, so the chrome names the room and counts. */
typedef struct { char title[AT_TOY_TITLE], what[AT_TOY_WHAT], counter[AT_TOY_COUNTER]; } AtCollChrome;
void at_coll_chrome(AtCollChrome *c, int total);

#ifdef __cplusplus
}
#endif
#endif
