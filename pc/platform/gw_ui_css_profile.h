/* gw_ui_css_profile.h - what each mode's character select is allowed to do. Pure C, no game types, no libc headers (game-side files include it). */
#ifndef GW_UI_CSS_PROFILE_H
#define GW_UI_CSS_PROFILE_H
#ifdef __cplusplus
extern "C" {
#endif

enum { AT_MT_LOBBY = 0x40 };                 /* not a retail value: the online lobby's pick */
typedef struct {
    int match_type;          /* CSSMatchType, or AT_MT_LOBBY */
    int group;               /* migration group: 1 VS family, Training, LAB; 2 Classic, Adventure, All-Star; 3 Event; 4 Stadium; 5 lobby */
    int max_humans;          /* ports that may be human at once */
    int cpu_cards;           /* 1: a CPU can be added on the cards (VS family) */
    int teams_from_rules;    /* 1: team colours follow css->vs.start.rules.is_teams */
    int min_to_start;        /* fighters in before Start is accepted */
    int entering_port_only;  /* 1: only the port named by css->unk_0x0 plays (one-player modes) */
    int dummy_cpu;           /* 1: Training: the human plus a CPU dummy in the other of slots 0 and 1 */
    int has_sss;             /* 1: the mode has a stage select after the CSS */
    int online;              /* 1: the lobby's pick */
    int roster_filter;       /* 1: the mode restricts the fighters offered (settled for Event in Task 0: it does not) */
} AtCssProfile;

const AtCssProfile *at_css_profile(int match_type);   /* NULL when the value is not known: the caller keeps the retail screen */
int at_css_profile_count(void);
const AtCssProfile *at_css_profile_at(int i);

#ifdef __cplusplus
}
#endif
#endif
