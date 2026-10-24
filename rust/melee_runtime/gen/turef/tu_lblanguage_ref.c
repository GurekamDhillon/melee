/* C reference for rust/melee_runtime/src/tu/lblanguage.rs: the game's lblanguage.c verbatim over a synthetic
 * gmMainLib state. Run: python rust/melee_runtime/gen/turef/tu_shard833.py tu_lblanguage */
#include "../../../../src/melee/lb/lblanguage.c"
#include "tu_gm_common.h"

static struct gmm_x0 G0;
static struct GamePrefs GP;
struct gmm_x0* gmMainLib_804D3EE0 = &G0;
struct GamePrefs* gmMainLib_GetGamePrefs(void) { return &GP; }

int main(void)
{
    int i;
    for (i = 0; i < 400; i++) {
        int a = (int) (rnd() % 9) - 3;
        G0.language = (u8) (rnd() % 5);
        GP.saved_language = (u8) (rnd() % 5);
        printf("%d lang=%u saved=%u\n", i, (unsigned) G0.language, (unsigned) GP.saved_language);
        printf("  get=%d jp=%d us=%d sget=%d sjp=%d sus=%d\n", (int) lbLang_GetLanguageSetting(), (int) lbLang_IsSettingJP(),
               (int) lbLang_IsSettingUS(), (int) lbLang_GetSavedLanguage(), (int) lbLang_IsSavedLanguageJP(),
               (int) lbLang_IsSavedLanguageUS());
        printf("  set(%d)=%d", a, (int) lbLang_SetLanguageSetting(a));
        lbLang_SetSavedLanguage(a);
        printf(" -> lang=%u saved=%u\n", (unsigned) G0.language, (unsigned) GP.saved_language);
    }
    return 0;
}
