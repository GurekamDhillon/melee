/* atlas-data: the data screens' model: a window over a long list, the formatters, a row turned into a list item. All values are invented. */
#include "atlas_check.h"
#include "../platform/gw_ui_data.h"

int main(void)
{
    char b[32]; AtDataView v; AtScreen sc; AtItem it; AtDataRow r; int i;
    /* the window keeps focus in view, slides by one, clamps, and survives a short list */
    CHECK(at_data_first(51, 9, 0, 0) == 0);
    CHECK(at_data_first(51, 9, 8, 0) == 0);        /* the last visible row: no slide */
    CHECK(at_data_first(51, 9, 9, 0) == 1);        /* one past it: slide by one */
    CHECK(at_data_first(51, 9, 50, 0) == 42);      /* the end: the last full page */
    CHECK(at_data_first(51, 9, 3, 42) == 3);       /* a jump back up */
    CHECK(at_data_first(5, 9, 4, 3) == 0);         /* a list shorter than a page never scrolls */
    CHECK(at_data_first(0, 9, 0, 7) == 0);
    CHECK(at_data_first(51, 9, -1, 20) == 0);      /* a bad focus is pulled to the first row, and the window follows it */
    CHECK(at_data_first(51, 9, 99, 0) == 42);      /* past the end: the last row */
    CHECK(at_data_first(51, 0, 5, 0) == 5);        /* a window of nothing never divides or loops: it follows the focus */
    for (i = 0; i < 51; i++) { int f = at_data_first(51, 9, i, (i * 7) % 43); CHECK(f >= 0 && f <= 42 && i >= f && i < f + 9); }
    for (i = 0; i < 120; i++) { int f = at_data_first(120, AT_DATA_ROWS, i, (i * 11) % 80); CHECK(f >= 0 && f <= 120 - AT_DATA_ROWS && i >= f && i < f + AT_DATA_ROWS); }   /* the full tag list */

    at_data_counter(b, sizeof b, 2, 51); CHECK_STR(b, "3 / 51");
    at_data_counter(b, sizeof b, 0, 0);  CHECK_STR(b, "");

    at_fmt_count(b, sizeof b, 0);        CHECK_STR(b, "0");
    at_fmt_count(b, sizeof b, 999);      CHECK_STR(b, "999");
    at_fmt_count(b, sizeof b, 1000);     CHECK_STR(b, "1,000");
    at_fmt_count(b, sizeof b, 1234567);  CHECK_STR(b, "1,234,567");
    at_fmt_count(b, sizeof b, 999999999u); CHECK_STR(b, "999,999,999");
    at_fmt_count(b, 5, 1234567);         CHECK(strlen(b) < 5);                   /* a short buffer is cut, never overrun */
    at_fmt_hm(b, sizeof b, 0);           CHECK_STR(b, "0:00");
    at_fmt_hm(b, sizeof b, 3900);        CHECK_STR(b, "1:05");           /* seconds in, hours:minutes out, as mnCount_CreateRow does */
    at_fmt_hm(b, sizeof b, 359940);      CHECK_STR(b, "99:59");
    at_fmt_hm(b, sizeof b, 3599999940u); CHECK_STR(b, "999999:59");      /* the retail clamp is the getter's: 999999 hours */
    at_fmt_frames(b, sizeof b, 3615);    CHECK_STR(b, "01:00 25");       /* the Event record rule: minutes, seconds, 99*(t%60)/59 */
    at_fmt_frames(b, sizeof b, 0);       CHECK_STR(b, "00:00 00");
    at_fmt_frames(b, sizeof b, 59);      CHECK_STR(b, "00:00 99");
    at_fmt_date(b, sizeof b, 2026, 10, 6); CHECK_STR(b, "2026-10-06");
    at_fmt_date(b, sizeof b, 5, 1, 2);     CHECK_STR(b, "0005-01-02");

    /* a row to an item: the label, the value as text, the sub line; a read-only row that shows its value */
    memset(&r, 0, sizeof r);
    snprintf(r.label, AT_STR, "EVENT 11"); snprintf(r.value, AT_STR, "01:00 25"); snprintf(r.sub, AT_STR, "Fight a foe");
    at_data_fill_item(&it, "r10", &r);
    CHECK_STR(it.id, "r10"); CHECK_STR(it.label, "EVENT 11"); CHECK(it.vkind == AT_VAL_TEXT); CHECK_STR(it.text, "01:00 25"); CHECK_STR(it.sub, "Fight a foe");
    CHECK((it.iflags & AT_ITEM_RO) && !(it.iflags & AT_ITEM_DISABLED) && !(it.flags & AT_CELL_DISABLED) && it.tag[0] == '\0');
    /* done: the jade edge, the tag, and the word on the sub line (colour is never the only signal) */
    r.flags = AT_DR_DONE; at_data_fill_item(&it, "r10", &r);
    CHECK((it.flags & AT_CELL_SELECTED) && strcmp(it.tag, "CLEARED") == 0 && strncmp(it.sub, "Cleared", 7) == 0 && strstr(it.sub, "Fight a foe") != NULL);
    r.sub[0] = '\0'; at_data_fill_item(&it, "r10", &r); CHECK_STR(it.sub, "Cleared");
    /* locked: a disabled row that says why */
    memset(&r, 0, sizeof r); snprintf(r.label, AT_STR, "EVENT 40"); snprintf(r.value, AT_STR, "--"); r.flags = AT_DR_LOCKED;
    at_data_fill_item(&it, "r39", &r);
    CHECK((it.iflags & AT_ITEM_DISABLED) && (it.flags & AT_CELL_DISABLED) && strcmp(it.tag, "LOCKED") == 0 && it.reason[0] != '\0');
    snprintf(r.sub, AT_STR, "Clear more events first."); at_data_fill_item(&it, "r39", &r); CHECK_STR(it.reason, "Clear more events first.");
    memset(&r, 0, sizeof r); snprintf(r.label, AT_STR, "NEW TAG"); r.flags = AT_DR_NEW; at_data_fill_item(&it, "r0", &r); CHECK_STR(it.tag, "NEW");
    at_data_fill_item(NULL, "r0", &r);   /* a NULL item is harmless */

    /* a view of 3 rows in a total of 51, window at 10: the screen has 3 items, the counter says 12 / 51 for focus 11 */
    memset(&v, 0, sizeof v);
    snprintf(v.id, sizeof v.id, "data.test"); snprintf(v.title, sizeof v.title, "TEST");
    v.total = 51; v.first = 10; v.n = 3; v.focus = 11;
    snprintf(v.row[0].label, AT_STR, "EVENT 11"); snprintf(v.row[0].value, AT_STR, "01:00 25"); v.row[0].flags = AT_DR_DONE;
    snprintf(v.row[1].label, AT_STR, "EVENT 12"); snprintf(v.row[1].value, AT_STR, "--");
    snprintf(v.row[2].label, AT_STR, "EVENT 13"); snprintf(v.row[2].sub, AT_STR, "sub");
    CHECK(at_data_screen(&v, &sc) == 1);
    CHECK(sc.primary == AT_PRIMARY_LIST && sc.n_items == 3);
    CHECK_STR(sc.items[0].label, "EVENT 11"); CHECK(sc.items[0].vkind == AT_VAL_TEXT); CHECK_STR(sc.items[0].text, "01:00 25");
    CHECK_STR(sc.items[0].tag, "CLEARED"); CHECK_STR(sc.items[1].tag, ""); CHECK_STR(sc.items[2].sub, "sub");
    CHECK_STR(sc.items[0].id, "r10"); CHECK_STR(sc.items[2].id, "r12");           /* an id is the ABSOLUTE row, so a slide keeps a row's id */
    CHECK_STR(sc.counter, "12 / 51");
    v.n = AT_DATA_ROWS + 1; CHECK(at_data_screen(&v, &sc) == 0);                  /* over the window: refused, never truncated silently */
    v.n = -1; CHECK(at_data_screen(&v, &sc) == 0);
    CHECK(AT_DATA_ROWS <= AT_MAX_ITEMS);
    ATLAS_DONE("atlas data");
}
