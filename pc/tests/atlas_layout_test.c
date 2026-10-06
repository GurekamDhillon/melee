#include "atlas_check.h"
#include "atlas_fake.h"

static void roles(void)
{
    int r;
    for (r = 0; r < AT_R_COUNT; r++) {
        const AtRoleInfo *i = at_role(r);
        CHECK(i->size >= 12);                                   /* the floor at 640x480 */
        if (i->smaller >= 0) {
            CHECK(at_role(i->smaller)->face == i->face);        /* the fit rule never changes face */
            CHECK(at_role(i->smaller)->size < i->size);
        }
    }
    CHECK(at_role_by_name("a_cap12") == AT_R_CAP12);
    CHECK(at_role_by_name("a_num16") == AT_R_NUM16);
    CHECK(at_role_by_name("body") == -1);                       /* legacy names are not Atlas roles */
    CHECK(at_role(AT_R_CAP12)->tracked == 1 && at_role(AT_R_TITLE)->tracked == 0);
    CHECK(at_role(AT_R_HERO)->caps_only == 1 && at_role(AT_R_DISPLAY)->caps_only == 1);
}

static void fit(void)
{
    char out[160];
    int r;
    r = at_fit(&FAKE, AT_R_ROW16, "Hello world", 200.0f, out, sizeof out);
    CHECK(r == AT_R_ROW16); CHECK_STR(out, "Hello world");
    r = at_fit(&FAKE, AT_R_ROW16, "Hello world", 80.0f, out, sizeof out);   /* 88 wide at 16: one role down */
    CHECK(r == AT_R_BODY14); CHECK_STR(out, "Hello world");
    r = at_fit(&FAKE, AT_R_ROW16, "Hello world", 60.0f, out, sizeof out);   /* 66 at 12 is still too wide: truncate */
    CHECK(r == AT_R_BODY12); CHECK_STR(out, "Hello wor\xE2\x80\xA6");
    r = at_fit(&FAKE, AT_R_BODY12, "Hello", 0.0f, out, sizeof out);         /* no limit: unchanged */
    CHECK(r == AT_R_BODY12); CHECK_STR(out, "Hello");
    r = at_fit(&FAKE, AT_R_BODY12, "Wide", 1.0f, out, sizeof out);          /* never below one character */
    CHECK_STR(out, "W\xE2\x80\xA6");
}

static void wrap(void)
{
    char lines[6][96];
    int clamped = 0, n;
    n = at_wrap(&FAKE, AT_R_BODY14, "Your hits set the target Burning for 3 s.", 136.0f, 4, lines, &clamped);
    CHECK(n == 3 && !clamped);
    CHECK_STR(lines[0], "Your hits set the"); CHECK_STR(lines[1], "target Burning for"); CHECK_STR(lines[2], "3 s.");
    n = at_wrap(&FAKE, AT_R_BODY14, "one\ntwo", 136.0f, 4, lines, &clamped);        /* hard breaks */
    CHECK(n == 2); CHECK_STR(lines[0], "one"); CHECK_STR(lines[1], "two");
    n = at_wrap(&FAKE, AT_R_BODY14, "aaa bbb ccc ddd eee fff ggg hhh iii jjj kkk lll mmm nnn ooo ppp", 60.0f, 2, lines, &clamped);
    CHECK(n == 2 && clamped);                                                        /* clamped lines end in an ellipsis */
    CHECK(strstr(lines[1], "\xE2\x80\xA6") != NULL);
    CHECK(fake_width(0, AT_R_BODY14, lines[1]) <= 60.0f);
    n = at_wrap(&FAKE, AT_R_BODY14, "Supercalifragilisticexpialidocious", 60.0f, 3, lines, &clamped);  /* one overlong word */
    CHECK(n == 1 && fake_width(0, AT_R_BODY14, lines[0]) <= 60.0f);
}

static void layout(void)
{
    AtLayout L;
    at_layout(640.0f, AT_PRESET_NORMAL, &L);
    CHECK(!L.wide);
    CHECK_NEAR(L.primary.x, 32); CHECK_NEAR(L.primary.y, 66); CHECK_NEAR(L.primary.w, 368); CHECK_NEAR(L.primary.h, 362);
    CHECK_NEAR(L.explainer.x, 412); CHECK_NEAR(L.explainer.w, 196);
    CHECK_NEAR(L.header.y, 22); CHECK_NEAR(L.header.h, 30); CHECK_NEAR(L.rule.y, 56);
    CHECK_NEAR(L.keys.y, 434); CHECK_NEAR(L.keys.h, 26);
    CHECK_NEAR(L.chapter.x, 608 - 112); CHECK_NEAR(L.chapter.w, 112); CHECK_NEAR(L.rail.w, 0);
    at_layout(640.0f, AT_PRESET_NARROW, &L);                                          /* the bag: 160 explainer */
    CHECK_NEAR(L.explainer.x, 448); CHECK_NEAR(L.explainer.w, 160); CHECK_NEAR(L.primary.w, 404);
    at_layout(640.0f, AT_PRESET_WIDE, &L);
    CHECK_NEAR(L.explainer.w, 256); CHECK_NEAR(L.primary.w, 640 - 64 - 256 - 12);
    at_layout(640.0f, AT_PRESET_NONE, &L);
    CHECK_NEAR(L.explainer.w, 0); CHECK_NEAR(L.primary.w, 576);
    at_layout(500.0f, AT_PRESET_NORMAL, &L);                                          /* never under 640 */
    CHECK_NEAR(L.canvas.w, 640);
    at_layout(853.3333f, AT_PRESET_NORMAL, &L);                                       /* 16:9: the wide arrangement */
    CHECK(L.wide);
    CHECK_NEAR(L.rail.x, 32); CHECK_NEAR(L.rail.w, 104); CHECK_NEAR(L.chapter.w, 0);
    CHECK_NEAR(L.primary.x, 32 + 104 + 12);
    CHECK_NEAR(L.explainer.w, 196 + 75);                                              /* + round((853.33 - 640) * 0.35) */
    CHECK_NEAR(L.explainer.x + L.explainer.w, 853.3333f - 32);
    at_layout(759.0f, AT_PRESET_NORMAL, &L); CHECK(!L.wide);
    at_layout(760.0f, AT_PRESET_NORMAL, &L); CHECK(L.wide);
    at_layout(1706.6667f, AT_PRESET_NORMAL, &L);                                      /* ultrawide: content capped and centred */
    CHECK_NEAR(L.content_w, 1140); CHECK_NEAR(L.content_x, (1706.6667f - 1140) / 2);
    CHECK_NEAR(L.rail.x, L.content_x + 32);
    CHECK_NEAR(L.explainer.x + L.explainer.w, L.content_x + 1140 - 32);
}

static void scroll(void)
{
    CHECK(at_list_scroll(0, 0, 5, 12) == 0);
    CHECK(at_list_scroll(5, 0, 5, 12) == 1);      /* the focus row stays inside the window */
    CHECK(at_list_scroll(11, 0, 5, 12) == 7);
    CHECK(at_list_scroll(2, 7, 5, 12) == 2);
    CHECK(at_list_scroll(3, 9, 5, 4) == 0);       /* never scrolls past the end */
}

int main(void)
{
    roles(); fit(); wrap(); layout(); scroll();
    ATLAS_DONE("atlas layout");
}
