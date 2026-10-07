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

int main(void)
{
    bars();
    part();
    ATLAS_DONE("atlas online parts");
}
