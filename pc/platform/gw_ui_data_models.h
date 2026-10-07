/* gw_ui_data_models.h - the per-screen scalar models of the data screens (Atlas step 8). Pure C; every input is a number or a string, never a
 * callback (game-side C cannot hand the host a function pointer). The game side reads the retail getters and passes what they returned; these
 * functions say what the row shows. Labels here are OUR words (authored); words that live on the disc come in as strings from the run-time decoder
 * (gw_ui_retailtext.h) and are never stored. */
#ifndef GW_UI_DATA_MODELS_H
#define GW_UI_DATA_MODELS_H
#include <string.h>
#include "gw_ui_data.h"
#ifdef __cplusplus
extern "C" {
#endif

/* ---- Misc. records: the 30 rows of mnCount_row (mncount.h) ---- */
#define AT_MISC_ROWS 30
enum { AT_MK_COUNT, AT_MK_TIME, AT_MK_FIGHTER };
typedef struct { char label[32]; int kind; } AtMiscRow;
AtMiscRow at_misc_row(int row);
void at_misc_fill(int row, unsigned value, const char *fighter, AtDataRow *out);   /* fighter: "" or NULL when the getter says nobody */

/* ---- Event Match ---- */
#define AT_EVENTS 51   /* 0x33: gm_803DF918's length */
/* the last display index the retail list can reach: the page start is bounded by mnEvent_8024CE74 (first_bound) and nine rows show from it; a
 * debug build (debug != 0) reaches all. Never past the 51st. */
int  at_event_last_open(int first_bound, int debug);
/* slot: the display index (what gm_801BEB74 stores); cleared/timed/best from gmMainLib_8015CEFC, gm_801BEB8C, gmMainLib_8015CF5C at that index; name
 * "" (or a locked row) shows the authored "EVENT n". */
void at_event_row(int slot, int cleared, int locked, int timed, unsigned best, const char *name, AtDataRow *out);

/* ---- Special Messages and Bonus Records ---- */
/* the ids of the unlocked messages in the order mnInfo_80251AFC sorts them: oldest first, equal dates by id. Returns the count. */
int  at_msg_order(const int *unlocked, const unsigned *date, int n, int *out);
void at_msg_row(int k, int has_date, int year, int month, int day, AtDataRow *out);   /* k: the position in the order */
void at_bonus_row(int k, AtDataRow *out);

/* ---- Sound Test ---- */
enum { AT_SND_NONE, AT_SND_PLAY, AT_SND_STOP, AT_SND_SWITCH };
int  at_sound_toggle(int playing, int row);                                   /* playing: the row that plays, -1 none */
void at_sound_row(int i, const char *name, int playing, AtDataRow *out);      /* a music track */
void at_sfx_row(int group, int sub, int count, AtDataRow *out);               /* a sound group: "SOUND GROUP 3", "4 / 5" (sub is 0-based) */

/* ---- VS Records ---- */
#define AT_STATS 21   /* VSSTAT_COUNT_FIGHTER: the stats both modes have (the three icon-only name stats are not ranked) */
enum { AT_STAT_COUNT, AT_STAT_TIME, AT_STAT_PERCENT, AT_STAT_DECIMAL, AT_STAT_DISTANCE, AT_STAT_DAMAGE };
enum { AT_SF_US = 1, AT_SF_OVER = 2 };   /* distance: US units (ft, mi) rather than metric; the value is already converted and is past the overflow line */
typedef struct { char label[32]; int kind; } AtStatInfo;
AtStatInfo at_stat_info(int stat);
void at_stat_text(int kind, unsigned value, int flags, char *out, int cap);
/* a stable descending ranking of the indices with a non-zero value; returns the count */
int  at_rank(const unsigned *val, int n, int *out);
void at_rank_row(int rank, const char *name, const char *value, AtDataRow *out);

/* ---- Name tags: up to four full-width SJIS letters, two bytes each ---- */
/* 1 when every glyph is Latin (letters, digits, punctuation): out is the text without trailing spaces. 0 (out "") when empty, all spaces, cut
 * or not Latin: the row says "TAG n". */
int  at_tag_text(const unsigned char *b, int n, char *out, int cap);
void at_tag_row(int slot, const char *text, AtDataRow *out);                  /* slot < 0: the "NEW TAG" row */

#ifdef __cplusplus
}
#endif
#endif
