/* atlas-policy: the scene policy table. No game. */
#include "atlas_check.h"
#include "../platform/gw_ui_policy.h"
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
    /* Atlas proof D12: the scenes outside a run (GameSceneKind: TITLE 0, MENU 1, GAMEOVER 0x22, STAFFROLL 0x2B, the movies) end it; the run's own scenes do not */
    CHECK(at_policy_ends_run(0) && at_policy_ends_run(1) && at_policy_ends_run(0x22) && at_policy_ends_run(0x2B) && at_policy_ends_run(0x1C) && at_policy_ends_run(0x1D));
    CHECK(!at_policy_ends_run(2) && !at_policy_ends_run(3) && !at_policy_ends_run(5) && !at_policy_ends_run(8) && !at_policy_ends_run(9) && !at_policy_ends_run(0x20));   /* VS, sudden death, results, CSS, SSS, 1P intro */
    CHECK(!at_policy_ends_run(-1));
    ATLAS_DONE("atlas-policy");
}
