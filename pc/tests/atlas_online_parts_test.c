#include "atlas_lint.h"
#include "../platform/gw_ui_online_parts.h"

static void bars(void)
{
    int i, b, changes;
    CHECK(at_link_bars(-1, 3) == 0);                    /* no link */
    CHECK(at_link_bars(20, 0) == 4 && at_link_bars(50, 0) == 4 && at_link_bars(51, 0) == 3);   /* first reading: the raw bar count */
    CHECK(at_link_bars(90, 0) == 3 && at_link_bars(91, 0) == 2 && at_link_bars(140, 0) == 2 && at_link_bars(141, 0) == 1);
    CHECK(at_link_bars(55, 4) == 4 && at_link_bars(58, 4) == 4 && at_link_bars(59, 4) == 3);   /* worse: 8 ms past the line */
    CHECK(at_link_bars(45, 3) == 3 && at_link_bars(42, 3) == 4);                               /* better: 8 ms inside it */
    CHECK(at_link_bars(500, 4) == 1);                                                          /* several steps in one reading */
    CHECK(at_link_bars(5, 1) == 4);
    /* flicker: a ping alternating across each line changes the count at most once in 100 readings */
    {
        static const int line[3] = { 50, 90, 140 };
        for (i = 0; i < 3; i++) {
            int k;
            b = 0; changes = 0;
            for (k = 0; k < 100; k++) {
                int nb = at_link_bars(line[i] + (k & 1 ? 3 : -3), b);
                if (b != 0 && nb != b) changes++;
                b = nb;
            }
            CHECK(changes <= 1);
        }
    }
}

static void part(void)
{
    AtSink s;
    AtRect box = { 20.0f, 10.0f, 150.0f, 20.0f };
    float adv;
    int b;
    for (b = 0; b <= 4; b++) {
        AtLinkInfo li = at_link_info(b);
        s = rec_sink();
        adv = at_part_linkmeter(&s, &FAKE, box.x, box.y + 16.0f, b, b ? 40 : -1, 1);
        CHECK(li.bars == b && li.word != NULL && find_text(li.word) != NULL);       /* a word for every count */
        CHECK(count_color(b ? li.rgba : AT_C_LINE) >= (b ? b : 4));                  /* the lit bars (or four empty ones) */
        CHECK(adv > 18.0f && adv <= box.w);                                          /* bars are 18 px wide, the rest fits the box */
        CHECK(lint_text_inside(box, 0) == 0 && lint_text_overlaps() == 0);
        CHECK(b == 0 ? find_text("40 ms") == NULL : find_text("40 ms") != NULL);     /* no number with no link */
    }
    s = rec_sink();
    adv = at_part_linkmeter(&s, &FAKE, 0.0f, 16.0f, 3, 61, 0);
    CHECK_NEAR(adv, 18.0f);                                                          /* without the word: just the bars */
    CHECK(REC.nt == 0);
}

static AtCodeView cv(const char *chars, int slot, int invalid)
{
    AtCodeView v; int i;
    memset(&v, 0, sizeof v);
    for (i = 0; i < 4 && chars[i]; i++) v.c[i] = chars[i];
    v.slot = slot; v.invalid = invalid;
    return v;
}

static void code(void)
{
    AtRect f = { 100.0f, 120.0f, 300.0f, 120.0f };
    AtSink s;
    AtCodeView v;
    int i, arg = -1;
    /* geometry: four slots inside the field, centred, never wider than 64 x 84, no overlap */
    for (i = 0; i < 4; i++) {
        AtRect r = at_code_slot_rect(f, i);
        CHECK(r.w <= 64.0f && r.h <= 84.0f && r.x >= f.x && r.x + r.w <= f.x + f.w && r.y >= f.y + AT_CODE_BAND - 0.1f);
        if (i > 0) { AtRect p = at_code_slot_rect(f, i - 1); CHECK(r.x >= p.x + p.w + 11.0f); }
    }
    /* the active slot: three cues (lift, ember edge, brackets) and chevrons */
    v = cv("AB", 2, 0);
    s = rec_sink(); at_part_code(&s, &FAKE, f, &v);
    CHECK(lint_focus_cell(at_code_slot_rect(f, 2), AT_C_EMBER) == 0);
    CHECK(find_text("A") != NULL && find_text("B") != NULL && REC.nt == 4);          /* A, B and two dashes */
    CHECK(count_color(AT_C_MUTED) >= 2);                                             /* the chevrons */
    /* the other slots keep the chamfer rule (probe one on a fresh sink: the active slot's brackets are outside it) */
    v = cv("ABCD", 0, 0);
    s = rec_sink(); at_part_code(&s, &FAKE, f, &v);
    CHECK(find_text("D") != NULL);
    {
        AtSink s2 = rec_sink(); AtCodeView solo = cv("ABCD", 9, 0);                  /* slot 9: nothing active */
        at_part_code(&s2, &FAKE, f, &solo);
        for (i = 0; i < 4; i++) CHECK(lint_plate_corners(at_code_slot_rect(f, i), 3.0f) == 0);
        CHECK(count_color(AT_C_EMBER) == 0);                                         /* no focus anywhere: no ember */
    }
    /* lower case is shown in capitals (the role is caps only) */
    v = cv("abcd", 0, 0);
    s = rec_sink(); at_part_code(&s, &FAKE, f, &v);
    CHECK(find_text("A") != NULL && find_text("a") == NULL);
    /* invalid: rose edges on the three non-active slots, the active one stays ember */
    v = cv("ABCD", 1, 1);
    s = rec_sink(); at_part_code(&s, &FAKE, f, &v);
    CHECK(count_color(AT_C_ROSE) == 3 && count_color(AT_C_EMBER) >= 1);
    /* the widest glyph in a normal field, a narrow one and a very narrow one: every text inside its slot, 12 px or more */
    {
        static const float widths[3] = { 300.0f, 200.0f, 150.0f };
        int k;
        for (k = 0; k < 3; k++) {
            AtRect nf = { 50.0f, 100.0f, widths[k], 120.0f };
            v = cv("WWWW", 0, 0);
            s = rec_sink(); at_part_code(&s, &FAKE, nf, &v);
            CHECK(texts_legible());
            for (i = 0; i < 4; i++) {                                                /* glyph i belongs to slot i (drawn in order) */
                RecText t = REC.t[i];
                AtRect slot = at_code_slot_rect(nf, i);
                float w = FAKE.width(FAKE.user, t.role, t.s);
                CHECK(t.x - w * 0.5f >= slot.x - 0.5f && t.x + w * 0.5f <= slot.x + slot.w + 0.5f);
            }
        }
    }
    /* hit testing: a slot, the active slot's strips, nothing elsewhere, nothing outside the field */
    {
        AtRect r1 = at_code_slot_rect(f, 1);
        CHECK(at_code_hit(f, 1, r1.x + 5.0f, r1.y + 5.0f, &arg) == 1 && arg == 1);
        CHECK(at_code_hit(f, 1, r1.x + 5.0f, r1.y - 5.0f, &arg) == 2);
        CHECK(at_code_hit(f, 1, r1.x + 5.0f, r1.y + r1.h + 5.0f, &arg) == 3);
        CHECK(at_code_hit(f, 0, r1.x + 5.0f, r1.y - 5.0f, &arg) == 0);               /* a strip of a slot that is not active */
        CHECK(at_code_hit(f, 1, -1000.0f, -1000.0f, &arg) == 0);                     /* a pointer off the picture */
        CHECK(at_code_hit(f, 1, r1.x + r1.w + 4.0f, r1.y + 5.0f, &arg) == 0);       /* in the gap between slots */
    }
}

int main(void)
{
    bars();
    part();
    code();
    ATLAS_DONE("atlas online parts");
}
