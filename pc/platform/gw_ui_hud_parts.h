/* gw_ui_hud_parts.h - three HUD parts for read-only data: a readout (label and value rows), a track (a move's timeline) and a chip strip (a mode and its
 * toggles). They are drawn by the zones of gw_ui_hud.c; this file holds the parts and their value-tree conversions. Pure C: only AtSink and AtTextOps. */
#ifndef GW_UI_HUD_PARTS_H
#define GW_UI_HUD_PARTS_H
#include "gw_ui_parts.h"
#include "gw_ui_val.h"
#ifdef __cplusplus
extern "C" {
#endif

#define AT_READ_ROWS 16
typedef struct { char label[24]; char value[40]; int tone; } AtReadRow;     /* tone: 0 plain, 1 ok (jade), 2 warn (sun), 3 bad (rose) */
typedef struct { char title[AT_STR]; int cols, n; AtReadRow row[AT_READ_ROWS]; } AtReadout;
float at_readout_height(const AtReadout *d);
void at_part_readout(const AtSink *s, const AtTextOps *o, AtRect r, const AtReadout *d);
int at_readout_from_val(const AtvArena *a, int t, AtReadout *out, char *err, int errcap);

#define AT_TRACK_SPANS 16
#define AT_TRACK_MARKS 24
enum { AT_MK_IASA, AT_MK_INVINC, AT_MK_GFX, AT_MK_SFX, AT_MK_VIS };
typedef struct { int from, to, id; } AtTrackSpan;
typedef struct { int frame, kind; } AtTrackMark;
typedef struct { char title[AT_STR], right[AT_STR], note[AT_STR]; int len, now, n_spans, n_marks; AtTrackSpan span[AT_TRACK_SPANS]; AtTrackMark mark[AT_TRACK_MARKS]; } AtTrack;
float at_track_height(void);
void at_part_track(const AtSink *s, const AtTextOps *o, AtRect r, const AtTrack *t);
int at_track_from_val(const AtvArena *a, int t, AtTrack *out, char *err, int errcap);

/* a strip of chips: the first names the mode (a word, whatever its colour), the rest are toggles drawn on (a filled square and bright text) or off (an outline and dim text) */
#define AT_CHIPS_MAX 12
typedef struct { char text[24]; int tone; int on; } AtChip;                  /* tone 0 plain, 1 ok, 2 warn; on: -1 not a toggle, 0 off, 1 on */
typedef struct { int n; AtChip c[AT_CHIPS_MAX]; char right[AT_STR]; } AtChips;
float at_chips_height(void);
void at_part_chips(const AtSink *s, const AtTextOps *o, AtRect r, const AtChips *d);
int at_chips_from_val(const AtvArena *a, int t, AtChips *out, char *err, int errcap);

#ifdef __cplusplus
}
#endif
#endif
