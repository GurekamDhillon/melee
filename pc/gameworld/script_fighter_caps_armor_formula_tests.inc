/* Real retail formula, with deterministic synthetic common data and no loaded
 * fighter/model/Player tables. The constants are fixture inputs, not disc data. */
#include <melee/ft/fighter.h>
#include <melee/ft/ftcoll.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/kinds/ftCommon/ftCo_Damage.h>
#include "script_fighter_caps_armor_core.h"

static float script_caps_armor_formula_sample(float percent, int damage,
                                              int growth, int base)
{
    static Fighter fp;
    HitCapsule hit;
    memset(&fp, 0, sizeof fp);
    memset(&hit, 0, sizeof hit);
    fp.player_id = 0;
    fp.dmg.x1830_percent = percent;
    fp.dmg.x1838_percentTemp = damage;
    hit.x24 = growth;
    hit.x2C = base;
    return ftColl_80079AB0(&fp, &hit, damage, 1, 1, 1, 100);
}

int ScriptGame_CapsArmorFormulaTest(void)
{
    static struct ftCommonData common;
    struct ftCommonData* old_common = p_ftCommonData;
    ScriptFighterMod old_mod = script_fighter_mod[0];
    ScriptArmorCore core, saved;
    ScriptArmorCoreEvent events[6];
    static Fighter fp;
    HitCapsule hit;
    float low, high, strong, fixed0, fixedhigh;
    int rc = 0, mask;
    memset(&common, 0, sizeof common);
    memset(&core, 0, sizeof core);
    memset(&fp, 0, sizeof fp);
    memset(&hit, 0, sizeof hit);
    memset(&script_fighter_mod[0], 0, sizeof script_fighter_mod[0]);
    common.xF4 = .01f; common.xF8 = 2; common.x108 = 1000;
    common.x110 = .1f; common.x114 = .05f; common.x118 = 10;
    common.x11C = 1.4f; common.x120 = 18;
    common.kb_min = 1; common.metal_armor = 5;
    common.x198 = .5f; common.x19C = 3; common.x1A0 = .5f;
    p_ftCommonData = &common;
    low = script_caps_armor_formula_sample(0, 4, 100, 20);
    high = script_caps_armor_formula_sample(140, 4, 100, 20);
    strong = script_caps_armor_formula_sample(140, 12, 100, 20);
    if (!(low > 39 && low < 40 && high > 98 && high < 99 && strong > high)) rc++;
    /* Fixed-knockback branch ignores accumulated percent. */
    hit.x24 = 100; hit.x28 = 20; hit.x2C = 10;
    fixed0 = ftColl_80079AB0(&fp, &hit, 4, 1, 1, 1, 100);
    fp.dmg.x1830_percent = 140;
    fixedhigh = ftColl_80079AB0(&fp, &hit, 4, 1, 1, 1, 100);
    if (fixed0 != fixedhigh) rc++;
    /* Actual retail subtraction composes max(armor0,armor1)+metal, then floor. */
    fp.x34_scale.y = 1; fp.dmg.kb_applied = high;
    fp.dmg.armor0 = 12; fp.dmg.armor1 = 8; fp.is_metal = 1;
    ftCo_Damage_CalcKnockback(&fp);
    if (fp.dmg.kb_applied != high - 17) rc++;
    fp.dmg.kb_applied = low; fp.dmg.armor0 = 100;
    ftCo_Damage_CalcKnockback(&fp);
    if (fp.dmg.kb_applied != 1) rc++;
    /* No-flinch preserves retail hitlag formula; reaction dispatch owns SDI flags. */
    if (ftCommon_CalcHitlag(4, ftCo_MS_Wait, 1) != 5 ||
        ftCommon_CalcHitlag(12, ftCo_MS_Wait, 1) != 9) rc++;
    if (!ScriptArmorCore_Set(&core, 1, 20, 0, -1, -1, -1, 0) ||
        ScriptArmorCore_Subtract(&core, 14, 0, 1, high) != high - 20 ||
        ScriptArmorCore_Subtract(&core, 14, 0, 1, low) != low - 20) rc++;
    memset(&core, 0, sizeof core);
    ScriptArmorCore_Set(&core, 2, 5, 0, -1, -1, -1, 0);
    mask = ScriptArmorCore_React(&core, 14, 0, 1, 4, high, events);
    if (!(mask & 2) || !events[1].absorbed || events[1].damage != 4) rc++;
    if (ScriptArmorCore_React(&core, 14, 0, 1, 5, low, events) ||
        ScriptArmorCore_React(&core, 14, 0, 1, 12, strong, events)) rc++;
    memset(&core, 0, sizeof core);
    ScriptArmorCore_Set(&core, 3, 70, 0, -1, -1, -1, 0);
    if (!(ScriptArmorCore_React(&core, 14, 0, 1, 4, low, events) & 4) ||
        ScriptArmorCore_React(&core, 14, 0, 1, 4, high, events)) rc++;
    ScriptArmorCore_Set(&core, 3, high, 0, -1, -1, -1, 0);
    if (ScriptArmorCore_React(&core, 14, 0, 1, 4, high, events)) rc++;
    memset(&core, 0, sizeof core);
    ScriptArmorCore_Set(&core, 4, 1, 0, -1, -1, -1, 0);
    if (!(ScriptArmorCore_React(&core, 14, 0, 1, 12, strong, events) & 8) ||
        !(ScriptArmorCore_React(&core, 14, 0, 1, 4, low, events) & 8) ||
        !(ScriptArmorCore_React(&core, 14, 0, 1, 4, high, events) & 8)) rc++;
    memset(&core, 0, sizeof core);
    ScriptArmorCore_Set(&core, 5, 2, 0, -1, -1, -1, 0);
    if (!(ScriptArmorCore_React(&core, 14, 0, 1, 4, low, events) & 16) || events[4].broke) rc++;
    if (!(ScriptArmorCore_React(&core, 14, 0, 1, 12, strong, events) & 16) || !events[4].broke) rc++;
    if (ScriptArmorCore_React(&core, 14, 0, 1, 4, low, events)) rc++;
    memset(&core, 0, sizeof core);
    ScriptArmorCore_Set(&core, 6, 8, 0, -1, -1, -1, 0);
    if (!(ScriptArmorCore_React(&core, 14, 0, 1, 4, low, events) & 32) || events[5].broke) rc++;
    if (ScriptArmorCore_React(&core, 14, 0, 1, 4, low, events) || !events[5].broke) rc++;
    ScriptArmorCore_Set(&core, 6, 8, 0, -1, -1, -1, 0);
    if (ScriptArmorCore_React(&core, 14, 0, 1, 12, strong, events) || !events[5].broke) rc++;
    /* Subtract before threshold; other armour does not hide budget consumption. */
    memset(&core, 0, sizeof core);
    ScriptArmorCore_Set(&core, 1, 40, 0, -1, -1, -1, 0);
    ScriptArmorCore_Set(&core, 3, 70, 0, -1, -1, -1, 0);
    ScriptArmorCore_Set(&core, 4, 1, 0, -1, -1, -1, 0);
    ScriptArmorCore_Set(&core, 5, 1, 0, -1, -1, -1, 0);
    mask = ScriptArmorCore_React(&core, 14, 0, 1, 4,
        ScriptArmorCore_Subtract(&core, 14, 0, 1, high), events);
    if ((mask & 28) != 28 || !events[4].broke) rc++;
    /* Inclusive state/frame/direction gates; window expires after one tick. */
    memset(&core, 0, sizeof core);
    ScriptArmorCore_Set(&core, 4, 1, 1, 14, 2, 4, 1);
    if (ScriptArmorCore_React(&core, 13, 2, 1, 4, low, events) ||
        ScriptArmorCore_React(&core, 14, 1, 1, 4, low, events) ||
        ScriptArmorCore_React(&core, 14, 2, -1, 4, low, events) ||
        !(ScriptArmorCore_React(&core, 14, 2, 1, 4, low, events) & 8) ||
        !(ScriptArmorCore_React(&core, 14, 4, 1, 4, low, events) & 8)) rc++;
    memcpy(&saved, &core, sizeof core);
    if (!(ScriptArmorCore_Tick(&core) & 8) ||
        ScriptArmorCore_React(&core, 14, 2, 1, 4, low, events)) rc++;
    memcpy(&core, &saved, sizeof core);
    if (memcmp(&core, &saved, sizeof core) ||
        !(ScriptArmorCore_React(&core, 14, 2, 1, 4, low, events) & 8)) rc++;
    p_ftCommonData = old_common;
    script_fighter_mod[0] = old_mod;
    return rc;
}
