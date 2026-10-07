#include "gw_ui_data_models.h"
#include "gw_ui_retailtext.h"
#include <stdio.h>
#include <string.h>

/* ---- Misc. records ---- */

/* OUR words, in mnCount_row order (mncount.h:28-59). Rows 20 to 29 rank the fighters; what each ranks was read from the getters (mncount.c):
 * play time (mnCount_GetMatchTime), wins, losses, damage dealt, damage taken, KOs, falls (mnDiagram_GetFighterTotalFalls), self-destructs. */
static const char *const MISC_LABEL[AT_MISC_ROWS] = {
    "Times switched on", "Time switched on", "Time played", "Single-player time", "VS time", "VS time, all fighters",
    "VS matches", "Time matches", "Stock matches", "Coin matches", "Bonus matches", "VS contestants",
    "Match resets", "Damage dealt", "KOs", "Self-destructs", "Fighters available", "Stages available",
    "Trophies", "Name tags",
    "Most played fighter", "Second most played", "Least played fighter", "Most wins", "Most losses",
    "Most damage dealt", "Most damage taken", "Most KOs", "Most falls", "Most self-destructs"
};

AtMiscRow at_misc_row(int row)
{
    AtMiscRow r;
    memset(&r, 0, sizeof r);
    if (row < 0 || row >= AT_MISC_ROWS) return r;
    snprintf(r.label, sizeof r.label, "%s", MISC_LABEL[row]);
    r.kind = (row >= 1 && row <= 5) ? AT_MK_TIME : (row >= 20) ? AT_MK_FIGHTER : AT_MK_COUNT;
    return r;
}

void at_misc_fill(int row, unsigned value, const char *fighter, AtDataRow *out)
{
    AtMiscRow m = at_misc_row(row);
    memset(out, 0, sizeof *out);
    snprintf(out->label, AT_STR, "%s", m.label);
    if (m.kind == AT_MK_TIME) at_fmt_hm(out->value, AT_STR, value);
    else if (m.kind == AT_MK_FIGHTER) snprintf(out->value, AT_STR, "%s", (fighter != NULL && fighter[0] != '\0') ? fighter : "--");
    else at_fmt_count(out->value, AT_STR, value);
}

/* ---- Event Match ---- */

int at_event_last_open(int first_bound, int debug)
{
    int last = first_bound + 8;
    if (debug || last > AT_EVENTS - 1) last = AT_EVENTS - 1;
    if (last < 0) last = 0;
    return last;
}

void at_event_row(int slot, int cleared, int locked, int timed, unsigned best, const char *name, AtDataRow *out)
{
    int named = !locked && name != NULL && name[0] != '\0';
    memset(out, 0, sizeof *out);
    if (named) {
        snprintf(out->label, AT_STR, "%s", name);
        snprintf(out->sub, AT_STR, "EVENT %d", slot + 1);
    } else {
        snprintf(out->label, AT_STR, "EVENT %d", slot + 1);
    }
    if (locked) {
        out->flags |= AT_DR_LOCKED;
        snprintf(out->sub, AT_STR, "%s", "Clear more events to unlock it.");
        snprintf(out->value, AT_STR, "%s", timed ? "--:-- --" : "--");
    } else if (cleared) {
        out->flags |= AT_DR_DONE;
        if (timed) at_fmt_frames(out->value, AT_STR, best);
        else snprintf(out->value, AT_STR, "%u", best);
    } else {
        snprintf(out->value, AT_STR, "%s", timed ? "--:-- --" : "--");
    }
}

/* ---- Special Messages and Bonus Records ---- */

int at_msg_order(const int *unlocked, const unsigned *date, int n, int *out)
{
    int i, k = 0;
    if (unlocked == NULL || date == NULL || out == NULL) return 0;
    for (i = 0; i < n; i++) {
        int j;
        if (!unlocked[i]) continue;
        j = k++;
        while (j > 0 && date[out[j - 1]] > date[i]) { out[j] = out[j - 1]; j--; }   /* stable: an equal date stays behind its lower id */
        out[j] = i;
    }
    return k;
}

void at_msg_row(int k, int has_date, int year, int month, int day, AtDataRow *out)
{
    memset(out, 0, sizeof *out);
    snprintf(out->label, AT_STR, "MESSAGE %d", k + 1);
    if (has_date) at_fmt_date(out->value, AT_STR, year, month, day);
}

void at_bonus_row(int k, AtDataRow *out)
{
    memset(out, 0, sizeof *out);
    snprintf(out->label, AT_STR, "BONUS %d", k + 1);
}

/* ---- Sound Test ---- */

int at_sound_toggle(int playing, int row)
{
    if (row < 0) return AT_SND_NONE;
    if (playing < 0) return AT_SND_PLAY;
    return playing == row ? AT_SND_STOP : AT_SND_SWITCH;
}

void at_sound_row(int i, const char *name, int playing, AtDataRow *out)
{
    memset(out, 0, sizeof *out);
    if (name != NULL && name[0] != '\0') {
        snprintf(out->label, AT_STR, "%s", name);
        snprintf(out->sub, AT_STR, "TRACK %d", i + 1);
    } else {
        snprintf(out->label, AT_STR, "TRACK %d", i + 1);
    }
    if (playing) snprintf(out->value, AT_STR, "%s", "PLAYING");
}

void at_sfx_row(int group, int sub, int count, AtDataRow *out)
{
    memset(out, 0, sizeof *out);
    snprintf(out->label, AT_STR, "SOUND GROUP %d", group + 1);
    if (count < 1) { snprintf(out->value, AT_STR, "%s", "--"); return; }
    if (sub < 0) sub = 0;
    if (sub >= count) sub = count - 1;
    snprintf(out->value, AT_STR, "%d / %d", sub + 1, count);
}

/* ---- VS Records ---- */

/* the 21 stats both modes have, in VSRecordsStatType order (mn/types.h), with the kind each is shown as (mnDiagram2_Is*Stat) */
static const AtStatInfo STAT[AT_STATS] = {
    { "KOs", AT_STAT_COUNT }, { "Falls", AT_STAT_COUNT }, { "Self-destructs", AT_STAT_COUNT }, { "Hit rate", AT_STAT_PERCENT },
    { "Damage dealt", AT_STAT_DAMAGE }, { "Damage taken", AT_STAT_DAMAGE }, { "Damage recovered", AT_STAT_DAMAGE }, { "Highest damage survived", AT_STAT_DAMAGE },
    { "Matches", AT_STAT_COUNT }, { "Wins", AT_STAT_COUNT }, { "Losses", AT_STAT_COUNT }, { "Time played", AT_STAT_TIME },
    { "Share of VS time", AT_STAT_PERCENT }, { "Average players", AT_STAT_DECIMAL }, { "Distance walked", AT_STAT_DISTANCE }, { "Distance run", AT_STAT_DISTANCE },
    { "Distance fallen", AT_STAT_DISTANCE }, { "Highest point", AT_STAT_DISTANCE }, { "Coins collected", AT_STAT_COUNT }, { "Coins taken", AT_STAT_COUNT },
    { "Coins lost", AT_STAT_COUNT }
};

AtStatInfo at_stat_info(int stat)
{
    AtStatInfo z;
    memset(&z, 0, sizeof z);
    return (stat >= 0 && stat < AT_STATS) ? STAT[stat] : z;
}

void at_stat_text(int kind, unsigned value, int flags, char *out, int cap)
{
    char n[32];
    if (cap <= 0) return;
    switch (kind) {
    case AT_STAT_TIME:                                  /* seconds, M:SS with the minutes not capped at 59 (mnDiagram_FormatTime); retail's clamp is 0x927BF */
        if (value > 0x927BFu) value = 0x927BFu;
        snprintf(out, (size_t) cap, "%u:%02u", value / 60u, value % 60u);
        break;
    case AT_STAT_PERCENT:                               /* hundredths (mnDiagram_FormatDecimalNumber, 2 places); clamp 0xF423F */
        if (value > 0xF423Fu) value = 0xF423Fu;
        snprintf(out, (size_t) cap, "%u.%02u%%", value / 100u, value % 100u);
        break;
    case AT_STAT_DECIMAL:
        if (value > 0xF423Fu) value = 0xF423Fu;
        snprintf(out, (size_t) cap, "%u.%02u", value / 100u, value % 100u);
        break;
    case AT_STAT_DISTANCE:                              /* already converted by mnDiagram_ConvertDistanceForDisplay; the overflow mark is a bigger unit */
        if (value > 0x98967Fu) value = 0x98967Fu;
        at_fmt_count(n, (int) sizeof n, value);
        snprintf(out, (size_t) cap, "%s %s", n, (flags & AT_SF_US) ? ((flags & AT_SF_OVER) ? "mi" : "ft") : ((flags & AT_SF_OVER) ? "km" : "m"));
        break;
    case AT_STAT_DAMAGE:
        if (value > 0x98967Fu) value = 0x98967Fu;
        at_fmt_count(n, (int) sizeof n, value);
        snprintf(out, (size_t) cap, "%s%%", n);
        break;
    default:
        if (value > 0x98967Fu) value = 0x98967Fu;
        at_fmt_count(out, cap, value);
        break;
    }
}

int at_rank(const unsigned *val, int n, int *out)
{
    int i, k = 0;
    if (val == NULL || out == NULL) return 0;
    for (i = 0; i < n; i++) {
        int j;
        if (val[i] == 0u) continue;
        j = k++;
        while (j > 0 && val[out[j - 1]] < val[i]) { out[j] = out[j - 1]; j--; }
        out[j] = i;
    }
    return k;
}

void at_rank_row(int rank, const char *name, const char *value, AtDataRow *out)
{
    memset(out, 0, sizeof *out);
    if (name != NULL && name[0] != '\0') snprintf(out->label, AT_STR, "%d. %s", rank + 1, name);
    else snprintf(out->label, AT_STR, "%d.", rank + 1);
    snprintf(out->value, AT_STR, "%s", value != NULL ? value : "");
}

/* ---- Name tags ---- */

int at_tag_text(const unsigned char *b, int n, char *out, int cap)
{
    int i = 0, len = 0, ok = 1;
    char tmp[16];
    if (cap <= 0) return 0;
    out[0] = '\0';
    if (b == NULL) return 0;
    while (i < n && b[i] != 0 && len < (int) sizeof tmp - 1) {
        char u[5];
        if (i + 1 >= n || b[i + 1] == 0 || !at_sjis_to_utf8(b[i], b[i + 1], u)) { ok = 0; break; }
        tmp[len++] = u[0];
        i += 2;
    }
    tmp[len] = '\0';
    while (len > 0 && tmp[len - 1] == ' ') tmp[--len] = '\0';
    if (!ok || len == 0) return 0;
    snprintf(out, (size_t) cap, "%s", tmp);
    return 1;
}

void at_tag_row(int slot, const char *text, AtDataRow *out)
{
    memset(out, 0, sizeof *out);
    if (slot < 0) {
        snprintf(out->label, AT_STR, "%s", "NEW TAG");
        out->flags |= AT_DR_NEW;
        return;
    }
    if (text != NULL && text[0] != '\0') snprintf(out->label, AT_STR, "%s", text);
    else snprintf(out->label, AT_STR, "TAG %d", slot + 1);
}
