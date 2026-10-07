#include "gw_ui_toy.h"

#include <stdio.h>
#include <string.h>

/* copy src into dst (cap bytes) clipped to max_bytes whole UTF-8 characters; with an ellipsis (3 bytes, inside max_bytes) when something was cut */
static void clip(char *dst, int cap, const char *src, int max_bytes)
{
    int n = (int) strlen(src), keep;
    if (max_bytes > cap - 1) max_bytes = cap - 1;
    if (n <= max_bytes) { memcpy(dst, src, (size_t) n + 1); return; }
    keep = max_bytes - 3;
    if (keep < 0) keep = 0;
    while (keep > 0 && ((unsigned char) src[keep] & 0xC0) == 0x80) keep--;      /* never split a character */
    while (keep > 0 && src[keep - 1] == ' ') keep--;                           /* no space before the ellipsis */
    memcpy(dst, src, (size_t) keep);
    memcpy(dst + keep, "\xE2\x80\xA6", 4);
}

/* the first sentence: through the first . ! or ? followed by a space or the end; else all of it */
static int first_sentence_len(const char *s)
{
    int i;
    for (i = 0; s[i] != '\0'; i++)
        if ((s[i] == '.' || s[i] == '!' || s[i] == '?') && (s[i + 1] == ' ' || s[i + 1] == '\0' || s[i + 1] == '\n')) return i + 1;
    return i;
}

void at_toy_chrome(AtToyChrome *c, int index, int count, const char *name, const char *desc, const char *series)
{
    char tmp[200];
    int n;
    memset(c, 0, sizeof *c);
    if (name == NULL) name = "";
    if (desc == NULL) desc = "";
    if (series == NULL) series = "";
    if (count <= 0) { snprintf(c->title, sizeof c->title, "NO TROPHIES YET"); return; }
    if (index < 0) index = 0;
    if (index > count - 1) index = count - 1;
    snprintf(c->counter, sizeof c->counter, "%d / %d", index + 1, count);
    if (name[0] != '\0') clip(c->title, sizeof c->title, name, AT_TOY_TITLE - 1);
    else snprintf(c->title, sizeof c->title, "TROPHY %d", index + 1);
    n = first_sentence_len(desc);
    if (n > (int) sizeof tmp - 1) n = (int) sizeof tmp - 1;
    memcpy(tmp, desc, (size_t) n);
    tmp[n] = '\0';
    clip(c->what, sizeof c->what, tmp, AT_TOY_WHAT - 2);                      /* 110 */
    clip(c->from, sizeof c->from, series, AT_TOY_FROM - 1);
}

void at_lot_chrome(AtLotChrome *c, int coins, int bet)
{
    int top;
    memset(c, 0, sizeof *c);
    if (coins < 0) coins = 0;
    if (coins == 0) {
        snprintf(c->title, sizeof c->title, "NO COINS");
        snprintf(c->what, sizeof c->what, "You need coins to play. Coins come from playing matches.");
        return;
    }
    if (coins == 1) snprintf(c->title, sizeof c->title, "1 COIN");
    else snprintf(c->title, sizeof c->title, "%d COINS", coins);
    snprintf(c->what, sizeof c->what, "Bet more coins for a better chance of a new trophy.");
    top = coins < 20 ? coins : 20;                                            /* retail's own ceiling (tyFigupon.c: 0x14) */
    if (bet < 0) bet = 0;
    if (bet > top) bet = top;
    if (bet > 0) snprintf(c->counter, sizeof c->counter, "BET %d / %d", bet, top);
}

void at_coll_chrome(AtCollChrome *c, int total)
{
    memset(c, 0, sizeof *c);
    snprintf(c->title, sizeof c->title, "TROPHY ROOM");
    if (total <= 0) {
        snprintf(c->what, sizeof c->what, "Nothing on the shelves yet.");
        return;
    }
    snprintf(c->what, sizeof c->what, "Look around the room at your trophies.");
    if (total == 1) snprintf(c->counter, sizeof c->counter, "1 TROPHY");
    else snprintf(c->counter, sizeof c->counter, "%d TROPHIES", total);
}
