/* gw_ui_online_parts.h - Atlas parts the online screens need: the link meter and the room-code field. Pure C. */
#ifndef GW_UI_ONLINE_PARTS_H
#define GW_UI_ONLINE_PARTS_H
#include "gw_ui_parts.h"
#ifdef __cplusplus
extern "C" {
#endif

typedef struct { int bars; const char *word; unsigned rgba; } AtLinkInfo;
int at_link_bars(int ms, int prev_bars);   /* 0..4; ms < 0 = no link; prev_bars (0 = none yet) gives the 8 ms dead band */
AtLinkInfo at_link_info(int bars);
/* The connection meter: four bars, then (with_word) the word and the round trip. Not at_part_link, which is step 3's segment quad. */
float at_part_linkmeter(const AtSink *s, const AtTextOps *o, float x, float base, int bars, int ms, int with_word);

typedef struct { char c[4]; int slot; int invalid; } AtCodeView;   /* c[i] == 0: an empty slot */
#define AT_CODE_BAND 18.0f                                          /* the strip above and below the active slot: step up / down */
AtRect at_code_slot_rect(AtRect field, int i);                      /* four slots, centred; at most 64 x 84 */
/* what is under the point: 0 nothing, 1 a slot (*arg = its index), 2 the active slot's up strip, 3 its down strip */
int at_code_hit(AtRect field, int active, float px, float py, int *arg);
void at_part_code(const AtSink *s, const AtTextOps *o, AtRect field, const AtCodeView *v);

#ifdef __cplusplus
}
#endif
#endif
