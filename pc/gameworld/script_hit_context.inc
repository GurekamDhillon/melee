/* Observer payload only. Capture the collision participants before damage reaction changes motion. */
static int script_hit_move_tag(int motion)
{
    if (motion>=ftCo_MS_Attack11 && motion<=ftCo_MS_Attack100End) return 1;
    if (motion==ftCo_MS_AttackDash) return 2;
    if (motion>=ftCo_MS_AttackS3Hi && motion<=ftCo_MS_AttackLw3) return 3;
    if (motion>=ftCo_MS_AttackS4Hi && motion<=ftCo_MS_AttackLw4) return 4;
    if (motion>=ftCo_MS_AttackAirN && motion<=ftCo_MS_AttackAirLw) return 5;
    if (motion>=ftCo_MS_Catch && motion<=ftCo_MS_CatchCut) return 6;
    if (motion>=ftCo_MS_ThrowF && motion<=ftCo_MS_ThrowLw) return 7;
    return 0; /* family-specific states are not all specials */
}
/* Inclusive Special ranges audited from each ft family forward.h enum.
 * Non-special extension states (taunts, extra jumps/smashes, zair, float) stay unknown.
 * DrMario/Falco/Pichu/Ganon/Emblem/CLink share their base-family tables. */
static int script_hit_family_move(Fighter* fp)
{
 static const unsigned short range[33][2]={
 {343,350},{341,369},{347,363},{369,386},{353,543},{341,363},{344,359},{341,364},
 {348,376},{352,368},{341,366},{341,366},{341,366},{341,356},{346,368},{346,372},
 {341,360},{341,358},{341,372},{341,358},{344,359},{343,350},{341,369},{341,366},
 {353,380},{347,363},{341,372},{0,0},{0,0},{0,0},{0,0},{341,363},{0,0}};
 int tag=script_hit_move_tag(fp->motion_id);
 extern int Geno_ProfileForKind(int kind);
 if(tag)return tag;
 if(fp->kind<0 || fp->kind>=33 || Geno_ProfileForKind(fp->kind)>=0)return 0;
 return range[fp->kind][0] && fp->motion_id>=range[fp->kind][0] && fp->motion_id<=range[fp->kind][1] ? 8 : 0;
}
void ScriptGame_ReportHitContext(Fighter* attacker,Fighter* victim,int element,int item)
{
    extern void Script_HitContext(int,int,int,int,int,int,int,int,int);
    union {float f; int i;} a,v;
    Fighter* owner=attacker;
    if (!victim) return;
    if (owner && !item && owner->x1064_thrownHitbox.owner)
        owner=GET_FIGHTER(owner->x1064_thrownHitbox.owner);
    a.f=owner ? owner->dmg.x1830_percent : 0;
    v.f=victim->dmg.x1830_percent;
    Script_HitContext(owner ? owner->player_id : -1,victim->player_id,element,
        /* An item or article declares itself: its hits are projectile hits, whatever its owner was doing
           (the hit rules already record item hits as move 9). */
        owner ? owner->motion_id : -1,item ? 9 : owner ? script_hit_family_move(owner) : 0,
        owner ? owner->ground_or_air==GA_Ground : -1,victim->ground_or_air==GA_Ground,a.i,v.i);
}

/* Transition signals: successful common grabs are Pull, never initial grab attempts. */
int ScriptGame_ActionCategory(int motion)
{
    if (motion==ftCo_MS_JumpF || motion==ftCo_MS_JumpB) return 1;
    if (motion==ftCo_MS_JumpAerialF || motion==ftCo_MS_JumpAerialB) return 2;
    if (motion==ftCo_MS_CliffCatch) return 3;
    if (motion==ftCo_MS_CatchPull || motion==ftCo_MS_CatchDashPull) return 4;
    if (motion>=ftCo_MS_ThrowF && motion<=ftCo_MS_ThrowLw) return 5;
    if (motion==ftCo_MS_AppealSR || motion==ftCo_MS_AppealSL) return 6;
    return 0;
}
