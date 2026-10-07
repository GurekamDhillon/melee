/* atlas-models: the per-screen scalar models of the data screens (Misc records, events, messages, bonuses, sound test, VS records, name tags).
 * Every input is a number or an invented string: a recorded sample of the game's data would be disc-derived, so the "recorded sample" of the
 * spec is replaced by invented data of the same shape. */
#include "atlas_check.h"
#include "../platform/gw_ui_data_models.h"

static void misc(void)
{
    AtDataRow r; int i;
    CHECK(AT_MISC_ROWS == 30);
    /* every row has an authored label under the row width and a kind; the five time rows and ten fighter rows are where mncount.h puts them */
    for (i = 0; i < AT_MISC_ROWS; i++) { CHECK(at_misc_row(i).label[0] != 0); CHECK(strlen(at_misc_row(i).label) <= 28); }
    for (i = 1; i <= 5; i++) CHECK(at_misc_row(i).kind == AT_MK_TIME);
    CHECK(at_misc_row(0).kind == AT_MK_COUNT && at_misc_row(6).kind == AT_MK_COUNT && at_misc_row(19).kind == AT_MK_COUNT);
    for (i = 20; i < 30; i++) CHECK(at_misc_row(i).kind == AT_MK_FIGHTER);
    CHECK(at_misc_row(-1).label[0] == 0 && at_misc_row(30).label[0] == 0);                     /* out of range: an empty row, no crash */
    for (i = 0; i < AT_MISC_ROWS; i++) { int j; for (j = i + 1; j < AT_MISC_ROWS; j++) CHECK(strcmp(at_misc_row(i).label, at_misc_row(j).label) != 0); }   /* no two rows read alike */

    at_misc_fill(14, 1234567u, "", &r);  CHECK_STR(r.value, "1,234,567");            /* KO total */
    at_misc_fill(14, 0u, "", &r);        CHECK_STR(r.value, "0");
    at_misc_fill(2, 3900u, "", &r);      CHECK_STR(r.value, "1:05");                  /* play time, seconds in, H:MM out */
    at_misc_fill(2, 359940u, "", &r);    CHECK_STR(r.value, "99:59");                 /* the 99 hour 59 minute value the getter returns when it has none */
    at_misc_fill(20, 0, "Fox", &r);      CHECK_STR(r.value, "Fox");
    at_misc_fill(20, 0, "", &r);         CHECK_STR(r.value, "--");                    /* nobody yet: SELKIND_COUNT */
    at_misc_fill(20, 0, NULL, &r);       CHECK_STR(r.value, "--");
    { AtMiscRow m20 = at_misc_row(20); CHECK_STR(r.label, m20.label); }
}

static void events(void)
{
    AtDataRow r;
    /* events: authored fallback label, the record by kind, the cleared mark, names clipped to the row, locked rows say so and never show their name */
    at_event_row(6, 1, 0, 1, 3615u, "", &r);       CHECK_STR(r.label, "EVENT 7");  CHECK_STR(r.value, "01:00 25"); CHECK(r.flags & AT_DR_DONE);
    at_event_row(6, 0, 0, 1, 0u, "", &r);          CHECK_STR(r.value, "--:-- --"); CHECK(!(r.flags & AT_DR_DONE));    /* timed, never cleared */
    at_event_row(0, 1, 0, 0, 12u, "", &r);         CHECK_STR(r.value, "12");                                          /* a count event */
    at_event_row(0, 0, 0, 0, 0u, "", &r);          CHECK_STR(r.value, "--");
    at_event_row(2, 0, 0, 1, 0u, "Invented Name", &r); CHECK_STR(r.label, "Invented Name"); CHECK_STR(r.sub, "EVENT 3");
    at_event_row(2, 0, 0, 1, 0u, "A very very very very long event name that cannot fit a row at all", &r); CHECK(strlen(r.label) <= AT_STR - 1);
    at_event_row(50, 0, 0, 0, 0u, "", &r);         CHECK_STR(r.label, "EVENT 51");
    at_event_row(39, 0, 1, 1, 0u, "Hidden Name", &r); CHECK(r.flags & AT_DR_LOCKED); CHECK_STR(r.label, "EVENT 40"); CHECK(strstr(r.sub, "Hidden") == NULL);
    at_event_row(39, 1, 1, 1, 77u, "", &r);        CHECK(r.flags & AT_DR_LOCKED); CHECK(!(r.flags & AT_DR_DONE)); CHECK_STR(r.value, "--:-- --");   /* locked wins over a stale record */
    CHECK(at_event_last_open(0, 0) == 8 && at_event_last_open(1, 0) == 9);                        /* first_event bound 0 and 1 plus the 9 visible rows */
    CHECK(at_event_last_open(0x2A, 0) == 50 && at_event_last_open(0x29, 0) == 49);
    CHECK(at_event_last_open(0x2A, 1) == 50 && at_event_last_open(0, 1) == 50);                   /* the debug build: everything */
    CHECK(at_event_last_open(60, 0) == 50);                                                       /* never past the 51st */
}

static void messages(void)
{
    AtDataRow r; char d[16];
    /* ascending by date (mnInfo_80251AFC puts the oldest first), locked ids skipped, equal dates keep the lower id first */
    { int un[6] = { 0, 1, 0, 1, 1, 0 }; unsigned dt[6] = { 0, 30, 0, 10, 20, 0 }; int ord[6]; int n = at_msg_order(un, dt, 6, ord);
      CHECK(n == 3 && ord[0] == 3 && ord[1] == 4 && ord[2] == 1);
      un[2] = 1; dt[2] = 20; n = at_msg_order(un, dt, 6, ord); CHECK(n == 4 && ord[0] == 3 && ord[1] == 2 && ord[2] == 4 && ord[3] == 1); }
    { int un0[3] = { 0, 0, 0 }; unsigned d0[3] = { 0, 0, 0 }; int o0[3]; CHECK(at_msg_order(un0, d0, 3, o0) == 0); }
    { int un1[66], i, ord[66]; unsigned d1[66]; for (i = 0; i < 66; i++) { un1[i] = 1; d1[i] = (unsigned) (66 - i); } CHECK(at_msg_order(un1, d1, 66, ord) == 66 && ord[0] == 65 && ord[65] == 0); }
    at_msg_row(2, 1, 2026, 10, 6, &r);   CHECK_STR(r.label, "MESSAGE 3"); CHECK_STR(r.value, "2026-10-06");
    at_msg_row(0, 0, 0, 0, 0, &r);       CHECK_STR(r.value, "");                                       /* no date to show: none, never a made-up one */
    (void) d;
    at_bonus_row(4, &r);                    CHECK_STR(r.label, "BONUS 5"); CHECK(r.flags == 0);
}

static void sound(void)
{
    AtDataRow r;
    CHECK(at_sound_toggle(-1, 7) == AT_SND_PLAY);                                                 /* nothing playing: play the row */
    CHECK(at_sound_toggle(7, 7) == AT_SND_STOP);                                                  /* A on the playing row stops it */
    CHECK(at_sound_toggle(3, 7) == AT_SND_SWITCH);                                                /* another row plays: switch to this one */
    CHECK(at_sound_toggle(-1, 0) == AT_SND_PLAY && at_sound_toggle(0, 0) == AT_SND_STOP);
    CHECK(at_sound_toggle(79, 0) == AT_SND_SWITCH && at_sound_toggle(5, -1) == AT_SND_NONE);      /* no row: nothing */
    at_sound_row(0, "", 0, &r);              CHECK_STR(r.label, "TRACK 1"); CHECK_STR(r.value, "");
    at_sound_row(79, "Invented Song", 1, &r); CHECK_STR(r.label, "Invented Song"); CHECK_STR(r.value, "PLAYING"); CHECK_STR(r.sub, "TRACK 80");
    at_sfx_row(2, 3, 5, &r);                 CHECK_STR(r.label, "SOUND GROUP 3"); CHECK_STR(r.value, "4 / 5");
    at_sfx_row(0, 0, 0, &r);                 CHECK_STR(r.value, "--");                           /* an empty group */
    at_sfx_row(0, 9, 3, &r);                 CHECK_STR(r.value, "3 / 3");                        /* a stale index is pulled in */
}

static void ranks(void)
{
    char b[32]; int i, k;
    /* ranking: a stable descending sort of (index, value) pairs, ties by the lower index, zero-value entries dropped */
    { unsigned val[6] = { 5, 0, 9, 5, 0, 7 }; int ord[6]; int n = at_rank(val, 6, ord);
      CHECK(n == 4 && ord[0] == 2 && ord[1] == 5 && ord[2] == 0 && ord[3] == 3); }
    { unsigned z[3] = { 0, 0, 0 }; int o[3]; CHECK(at_rank(z, 3, o) == 0); }
    { unsigned one[1] = { 4 }; int o[1]; CHECK(at_rank(one, 1, o) == 1 && o[0] == 0); }
    { unsigned big[120]; int o[120]; for (i = 0; i < 120; i++) big[i] = (unsigned) (i % 7); CHECK(at_rank(big, 120, o) == 120 - 18 && big[o[0]] == 6); }   /* a full tag list */
    CHECK(at_rank(NULL, 0, NULL) == 0);
    /* a stat's text by kind: count, time (seconds: M:SS), percent in hundredths, a decimal, damage, distance with the retail overflow mark */
    at_stat_text(AT_STAT_COUNT, 1500u, 0, b, sizeof b);       CHECK_STR(b, "1,500");
    at_stat_text(AT_STAT_TIME, 3615u, 0, b, sizeof b);        CHECK_STR(b, "60:15");
    at_stat_text(AT_STAT_TIME, 59u, 0, b, sizeof b);          CHECK_STR(b, "0:59");
    at_stat_text(AT_STAT_TIME, 999999u, 0, b, sizeof b);      CHECK_STR(b, "9999:59");                            /* the display clamp of 599,999 s, as retail's */
    at_stat_text(AT_STAT_PERCENT, 8743u, 0, b, sizeof b);     CHECK_STR(b, "87.43%");
    at_stat_text(AT_STAT_PERCENT, 5u, 0, b, sizeof b);        CHECK_STR(b, "0.05%");
    at_stat_text(AT_STAT_DECIMAL, 237u, 0, b, sizeof b);      CHECK_STR(b, "2.37");
    at_stat_text(AT_STAT_DAMAGE, 12345u, 0, b, sizeof b);     CHECK_STR(b, "12,345%");
    at_stat_text(AT_STAT_DISTANCE, 1234u, AT_SF_US, b, sizeof b);                     CHECK_STR(b, "1,234 ft");
    at_stat_text(AT_STAT_DISTANCE, 12u, AT_SF_US | AT_SF_OVER, b, sizeof b);          CHECK_STR(b, "12 mi");       /* overflow: a bigger unit instead of a wrong number */
    at_stat_text(AT_STAT_DISTANCE, 1234u, 0, b, sizeof b);                            CHECK_STR(b, "1,234 m");
    at_stat_text(AT_STAT_DISTANCE, 12u, AT_SF_OVER, b, sizeof b);                     CHECK_STR(b, "12 km");
    at_stat_text(AT_STAT_COUNT, 999999999u + 5u, 0, b, sizeof b);                     CHECK_STR(b, "9,999,999");     /* the retail clamp (0x98967F) */
    at_stat_text(AT_STAT_PERCENT, 5000000u, 0, b, sizeof b);                          CHECK_STR(b, "9999.99%");      /* and its percent clamp (0xF423F) */
    CHECK(AT_STATS == 21);
    for (k = 0; k < AT_STATS; k++) { CHECK(at_stat_info(k).label[0] != 0 && strlen(at_stat_info(k).label) <= 28); }
    CHECK(at_stat_info(3).kind == AT_STAT_PERCENT && at_stat_info(11).kind == AT_STAT_TIME && at_stat_info(13).kind == AT_STAT_DECIMAL && at_stat_info(16).kind == AT_STAT_DISTANCE);
    CHECK(at_stat_info(0).kind == AT_STAT_COUNT && at_stat_info(4).kind == AT_STAT_DAMAGE && at_stat_info(18).kind == AT_STAT_COUNT);
    CHECK(at_stat_info(21).label[0] == 0 && at_stat_info(-1).label[0] == 0);
    { AtDataRow r;
      at_rank_row(0, "Fox", "1,500", &r);  CHECK_STR(r.label, "1. Fox"); CHECK_STR(r.value, "1,500");
      at_rank_row(11, "", "7", &r);        CHECK_STR(r.label, "12."); }
}

static void tags(void)
{
    char t[16]; AtDataRow r;
    /* a tag is up to four full-width SJIS letters in the save; Latin ones decode, anything else is "TAG n" */
    { const unsigned char ok[9] = { 0x82, 0x60, 0x82, 0x81, 0x82, 0x4F, 0x81, 0x40, 0 };           /* A a 0 (space) */
      CHECK(at_tag_text(ok, 8, t, sizeof t) == 1); CHECK_STR(t, "Aa0"); }                                 /* the trailing space is trimmed */
    { const unsigned char kana[9] = { 0x82, 0x60, 0x83, 0x41, 0, 0, 0, 0, 0 };                    /* A, then katakana */
      CHECK(at_tag_text(kana, 8, t, sizeof t) == 0); CHECK_STR(t, ""); }
    { const unsigned char none[9] = { 0 }; CHECK(at_tag_text(none, 8, t, sizeof t) == 0); CHECK_STR(t, ""); }
    { const unsigned char sp[9] = { 0x81, 0x40, 0x81, 0x40, 0, 0, 0, 0, 0 }; CHECK(at_tag_text(sp, 8, t, sizeof t) == 0); }   /* all spaces: no name */
    { const unsigned char odd[9] = { 0x82, 0x60, 0x82 }; CHECK(at_tag_text(odd, 3, t, sizeof t) == 0); }     /* a cut pair never reads past n */
    { const unsigned char ok[9] = { 0x82, 0x60, 0x82, 0x61, 0x82, 0x62, 0x82, 0x63, 0 }; char small[3]; CHECK(at_tag_text(ok, 8, small, sizeof small) == 1); CHECK_STR(small, "AB"); }   /* a short buffer is cut */
    at_tag_row(4, "Invented", &r);  CHECK_STR(r.label, "Invented");
    at_tag_row(4, "", &r);          CHECK_STR(r.label, "TAG 5");
    at_tag_row(-1, NULL, &r);       CHECK_STR(r.label, "NEW TAG"); CHECK(r.flags & AT_DR_NEW);
}

int main(void)
{
    misc(); events(); messages(); sound(); ranks(); tags();
    ATLAS_DONE("atlas models");
}
