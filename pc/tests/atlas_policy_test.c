/* atlas-policy: the scene policy table. No game. */
#include "atlas_check.h"
#include "../platform/gw_ui_policy.h"
#include "../platform/gw_ui_retail_ids.h"
#define GS_TITLE_K 0      /* GS_TITLE: the first value of enum GameSceneKind (src/melee/gm/forward.h:89, "+00") */
int main(void)
{
    CHECK(at_policy_for(GS_TITLE_K, 1, NULL) == AT_POLICY_OVERLAY);
    CHECK(at_policy_for(GS_TITLE_K, 0, NULL) == AT_POLICY_RETAIL);           /* MELEE_ATLAS=0: everything retail */
    CHECK(at_policy_for(5, 1, NULL) == AT_POLICY_RETAIL);                    /* any other scene: retail by default */
    CHECK(at_policy_for(GS_TITLE_K, 1, "0:retail") == AT_POLICY_RETAIL);     /* the override wins */
    CHECK(at_policy_for(40, 1, "0:retail,40:replace") == AT_POLICY_REPLACE);
    CHECK(at_policy_for(GS_TITLE_K, 1, "0:retail,40:replace") == AT_POLICY_RETAIL);
    CHECK(at_policy_for(40, 1, "40:bogus") == AT_POLICY_RETAIL);             /* a bad word is ignored */
    CHECK(at_policy_for(GS_TITLE_K, 1, "0:bogus") == AT_POLICY_OVERLAY);     /* ... and leaves the default alone */
    CHECK(at_policy_for(40, 1, "nonsense") == AT_POLICY_RETAIL);
    CHECK(at_policy_for(2, 1, "1:overlay,2:replace,2:retail") == AT_POLICY_RETAIL);   /* the last row naming a kind wins */
    CHECK(at_policy_for(2, 0, "2:replace") == AT_POLICY_RETAIL);             /* MELEE_ATLAS=0 beats an override */
    CHECK(at_policy_for(3, 1, "") == AT_POLICY_RETAIL);
    CHECK(at_policy_screen(GS_TITLE_K) != NULL && strcmp(at_policy_screen(GS_TITLE_K), "title") == 0);
    CHECK(at_policy_screen(5) == NULL);
    /* step 10: every bespoke scene is RETAIL by default; an env override turns one on; the mask follows only an OVERLAY */
    CHECK(at_policy_for(11, 1, NULL) == AT_POLICY_RETAIL);                   /* GS_TOY_GALLERY */
    CHECK(at_policy_for(11, 1, "11:overlay") == AT_POLICY_OVERLAY);
    CHECK(at_policy_mask(11, 1, NULL, 1) == 0);                              /* retail: no mask */
    CHECK(at_policy_mask(11, 0, "11:overlay", 1) == 0);                      /* MELEE_ATLAS=0 beats everything */
    CHECK(at_policy_mask(11, 1, "11:overlay", 1) == ((1u << AT_RE_TOY_PANEL) | (1u << AT_RE_TOY_INFO) | (1u << AT_RE_TOY_TEXT)));
    CHECK(at_policy_mask(11, 1, "11:overlay", 0) == ((1u << AT_RE_TOY_PANEL) | (1u << AT_RE_TOY_INFO)));   /* text not decodable: the retail text stays */
    CHECK(at_policy_mask(11, 1, "11:retail", 1) == 0 && at_policy_mask(11, 1, "11:replace", 1) == 0);      /* only an OVERLAY masks */
    /* the Lottery's machine and the Collection's room are retail 3D with no 2D piece to hide (read from source): chrome only */
    CHECK(at_policy_mask(12, 1, "12:overlay", 1) == 0);                      /* GS_TOY_LOTTERY */
    CHECK(at_policy_mask(13, 1, "13:overlay", 1) == 0);                      /* GS_TOY_COLLECTION */
    CHECK(at_policy_mask(0, 1, NULL, 1) == 0);                               /* the title: an overlay with no mask */
    CHECK_STR(at_policy_screen_for(11), "toy.gallery"); CHECK_STR(at_policy_screen_for(12), "toy.lottery"); CHECK_STR(at_policy_screen_for(13), "toy.collection");
    CHECK_STR(at_policy_screen_for(0), "title");
    CHECK(at_policy_screen_for(5) == NULL);                                  /* results: step 8's REPLACE, no chrome id */
    /* the owner-skipped scenes never get a mask or a chrome id, whatever the override says about their policy */
    { static const int skipped[] = { 0x1C /* the opening */, 0x1D, 0x1E, 0x1F, 0x2B /* staff roll */, 0x2A /* memory card */ }; size_t i;
      for (i = 0; i < sizeof skipped / sizeof skipped[0]; i++) {
          CHECK(at_policy_mask(skipped[i], 1, NULL, 1) == 0); CHECK(at_policy_screen_for(skipped[i]) == NULL);
          CHECK(at_policy_for(skipped[i], 1, NULL) == AT_POLICY_RETAIL); } }
    ATLAS_DONE("atlas-policy");
}
