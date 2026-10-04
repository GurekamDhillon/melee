/* Fake fighters: byte-exact attrs, reload replacement, respawn and Geno edits. */
#include <melee/ft/ftcommon.h>
int ScriptGame_FighterModTest(void)
{
    static Fighter fp, respawn;
    static unsigned char original[sizeof fp.co_attrs];
    ScriptFighterMod saved = script_fighter_mod[0];
    ScriptFighterMod saved1 = script_fighter_mod[1];
    HSD_GObj** old_heads=HSD_GObjPLinkHead;
    static HSD_GObj* heads[64];
    static HSD_GObj thrower;
    int rc = 0;
    memset(&fp, 0, sizeof fp); memset(&respawn, 0, sizeof respawn);
    memset(&script_fighter_mod[0], 0, sizeof script_fighter_mod[0]);
    fp.co_attrs.ground_max_horizontal_velocity = 2;
    fp.co_attrs.dash_max_velocity = 3;
    fp.co_attrs.air_max_horizontal_velocity = 1;
    memcpy(original, &fp.co_attrs, sizeof original);
    if (!ScriptGame_FighterMod(0,1,1,script_mesh_bits(2),script_mesh_bits(0.5f),
        script_mesh_bits(1.5f),script_mesh_bits(2),script_mesh_bits(1.25f))) rc++;
    if (ScriptGame_ModValue(0,0,3)!=6 || ScriptGame_ModValue(0,1,2)!=1 ||
        FT_SCRIPT_VALUE(&fp,2,fp.co_attrs.ground_max_horizontal_velocity)!=3 ||
        FT_SCRIPT_VALUE(&fp,2,fp.co_attrs.dash_max_velocity)!=4.5f) rc++;
    memset(heads,0,sizeof heads); memset(&thrower,0,sizeof thrower);
    thrower.user_data=&fp; heads[HSD_GOBJ_PLINK_FIGHTER]=&thrower;
    HSD_GObjPLinkHead=heads;
    respawn.player_id=1; respawn.x1064_thrownHitbox.owner=&thrower;
    memset(&script_fighter_mod[1],0,sizeof script_fighter_mod[1]);
    if (ScriptGame_FighterDealtDamage(&respawn,10)!=20 || ScriptGame_ModDamage(0,1,10)!=5) rc++;
    respawn.x1064_thrownHitbox.owner=NULL;
    if (ScriptGame_FighterDealtDamage(&respawn,10)!=10) rc++;
    HSD_GObjPLinkHead=old_heads;
    if (!ScriptGame_FighterMod(0,1,1,script_mesh_bits(2),script_mesh_bits(0.5f),
        script_mesh_bits(1.5f),script_mesh_bits(2),script_mesh_bits(1.25f))) rc++;
    if (ScriptGame_FighterModRead(0,0)!=script_mesh_bits(2) ||
        ScriptGame_FighterModRead(0,1)!=script_mesh_bits(0.5f)) rc++;
    if (ScriptGame_ModValue(0,0,3)!=6) rc++; /* replacement, not multiplication */
    respawn.player_id=0; respawn.co_attrs.air_max_horizontal_velocity=3;
    if (FT_SCRIPT_VALUE(&respawn,3,respawn.co_attrs.air_max_horizontal_velocity)!=6) rc++;
    {
        static struct ftCommonData common;
        struct ftCommonData* old=p_ftCommonData;
        memset(&common,0,sizeof common); common.aerial_friction_oob=0.01f;
        p_ftCommonData=&common;
        script_fighter_mod[0].value[3]=0.5f;
        respawn.co_attrs.air_drift_max=0.86f; respawn.self_vel.x=1;
        if (!ftCommon_CalcSelfAccel_DeaccelQuickAir(&respawn) ||
            respawn.self_vel.x!=1 || respawn.x74_self_accel.x!=-0.01f) rc++;
        respawn.self_vel.x+=respawn.x74_self_accel.x;
        if (respawn.self_vel.x<=0.43f) rc++; /* retail deceleration, not instant clamp */
        respawn.self_vel.x=-1;
        if (!ftCommon_CalcSelfAccel_DeaccelQuickAir(&respawn) ||
            respawn.x74_self_accel.x!=0.01f) rc++;
        script_fighter_mod[0].value[3]=2; p_ftCommonData=old;
    }
    if (ScriptGame_FighterMod(0,2,0,0,0,0,0,0)) rc++; /* another owner */
    ScriptGame_FighterModsRelease(2);
    if (ScriptGame_ModValue(0,0,3)!=6) rc++;
    if (!ScriptGame_FighterMod(0,1,0,0,0,0,0,0)) rc++;
    if (memcmp(original,&fp.co_attrs,sizeof original) || ScriptGame_ModValue(0,0,3)!=3) rc++;
    if (ScriptGame_FighterModRead(0,0)!=0 || ScriptGame_FighterModOwner(0)!=0) rc++;
    fp.shield_health=120;
    script_mod_shield_reconcile(&fp,60);
    if (fp.shield_health!=60) rc++;
    script_fighter_mod[0].owner=1; script_fighter_mod[0].value[4]=0.5f;
    script_mod_shield_reconcile(&fp,60);
    if (fp.shield_health!=30) rc++;
    ScriptGame_FighterModsRelease(1);
    fp.co_attrs.air_max_horizontal_velocity=4; /* latest Geno state is preserved */
    if (FT_SCRIPT_VALUE(&fp,3,fp.co_attrs.air_max_horizontal_velocity)!=4) rc++;
    if (ScriptGame_FighterMod(-1,1,1,0,0,0,0,0) ||
        ScriptGame_FighterMod(6,1,1,0,0,0,0,0)) rc++;
    script_fighter_mod[0]=saved;
    script_fighter_mod[1]=saved1;
    return rc;
}
