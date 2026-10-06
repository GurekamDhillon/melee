/* gw_ui_focus.h - Atlas focus movement on blocks of cells, and held-direction repeat. Pure C. */
#ifndef GW_UI_FOCUS_H
#define GW_UI_FOCUS_H
#ifdef __cplusplus
extern "C" {
#endif

typedef struct { int col0, row0, cols, n; const unsigned char *exists; } AtFocusBlock; /* exists NULL = every cell exists */
typedef struct { int block, index; } AtFocusPos;
enum { AT_DIR_LEFT = 1, AT_DIR_RIGHT = 2, AT_DIR_UP = 3, AT_DIR_DOWN = 4 };

AtFocusPos at_focus_first(const AtFocusBlock *b, int nb);
AtFocusPos at_focus_move(const AtFocusBlock *b, int nb, AtFocusPos cur, int dir, int wrap);

typedef struct { int dir; double down_ms, last_ms; } AtRepeat;
int at_repeat_step(AtRepeat *r, int held_dir, double now_ms);

#ifdef __cplusplus
}
#endif
#endif
