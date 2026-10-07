/* gw_ui_input.h - Atlas input: pad, keyboard and mouse become one event type; hit rectangles. Pure C. */
#ifndef GW_UI_INPUT_H
#define GW_UI_INPUT_H
#include "gw_ui_focus.h"
#include "gw_ui_layout.h"
#ifdef __cplusplus
extern "C" {
#endif

enum { AT_EV_NONE, AT_EV_MOVE, AT_EV_FOCUS, AT_EV_ACCEPT, AT_EV_BACK, AT_EV_ALT, AT_EV_PAGE, AT_EV_SCROLL, AT_EV_START };
typedef struct { int type, a, b; } AtEvent;   /* MOVE a=dir; FOCUS a=block b=index; ALT a='X'|'Y'|'Z'; PAGE a=-1|+1; SCROLL a=-1|+1 */

enum { AT_HIT_CELL = 1, AT_HIT_KEY = 2, AT_HIT_DIALOG = 3, AT_HIT_TAB = 4, AT_HIT_CARD = 5, AT_HIT_ROOM = 6 };   /* ROOM: the online room's own targets (gw_ui_room.h AT_RH_*); at_mouse_events never sees one, the room's mouse path is at_room_mouse_intents */
typedef struct { AtRect r; int kind, a, b; } AtHit;   /* CELL a=block b=index; KEY and DIALOG a=button char ('A'.., 'S' = START); TAB a=tab index; CARD a=port card index */
#define AT_MAX_HITS 192   /* a 1140-wide character select: 11 columns x 6 rows of tiles, the tabs, the cards and the keys */
typedef struct { AtHit h[AT_MAX_HITS]; int n; } AtHits;

enum { AT_PAD_LEFT = 0x0001, AT_PAD_RIGHT = 0x0002, AT_PAD_DOWN = 0x0004, AT_PAD_UP = 0x0008, AT_PAD_Z = 0x0010, AT_PAD_R = 0x0020,
       AT_PAD_L = 0x0040, AT_PAD_A = 0x0100, AT_PAD_B = 0x0200, AT_PAD_X = 0x0400, AT_PAD_Y = 0x0800, AT_PAD_START = 0x1000 };
enum { AT_KEY_UP = 1, AT_KEY_DOWN = 2, AT_KEY_LEFT = 4, AT_KEY_RIGHT = 8, AT_KEY_ENTER = 16, AT_KEY_ESC = 32, AT_KEY_TAB = 64, AT_KEY_SHIFT = 128 };

typedef struct { unsigned prev; AtRepeat rep; } AtPad;
typedef struct { unsigned prev; AtRepeat rep; } AtKeys;
typedef struct { int valid; float x, y; int buttons; } AtMouse;

/* Each returns how many events it wrote to out (at most cap). */
int at_pad_events(AtPad *p, unsigned buttons, int sx, int sy, double now_ms, AtEvent *out, int cap);
int at_key_events(AtKeys *k, unsigned mask, double now_ms, AtEvent *out, int cap);
int at_mouse_events(AtMouse *m, float x, float y, int buttons, int wheel, const AtHits *hits, AtEvent *out, int cap);
int at_hit_test(const AtHits *hits, float x, float y);   /* the topmost (latest) hit rectangle under the point, or -1 */

#ifdef __cplusplus
}
#endif
#endif
