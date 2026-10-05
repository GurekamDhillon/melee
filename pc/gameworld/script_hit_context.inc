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
/* Which fighters have just started a special by input. Geno_SpecialEnter runs at all eight special
 * dispatch sites (ftCo_Attack100.c, ftCo_SpecialAir.c, ftCo_SpecialS.c) for every fighter, before the
 * fighter's own special; so a fighter-specific state reached after it is that input family's move,
 * even when the engine has no table for the fighter (m-ex) or a Geno state carries no declaration.
 * Cleared when the fighter returns to a common state (Fighter_ChangeMotionState). Game BSS. */
static unsigned char script_special_open[12];
static int script_special_slot(Fighter* fp) {
 return fp && fp->player_id<6 ? fp->player_id*2+(fp->is_sub_fighter ? 1 : 0) : -1;
}
void ScriptGame_NoteSpecialInput(Fighter* fp) {int e=script_special_slot(fp);if(e>=0)script_special_open[e]=1;}
void ScriptGame_NoteMotion(Fighter* fp,int motion) {
 int e=script_special_slot(fp);
 if(e>=0 && motion>=0 && motion<341)script_special_open[e]=0; /* 341: the first fighter-specific state */
}
/* Inclusive Special ranges audited from each ft family forward.h enum.
 * Non-special extension states (taunts, extra jumps/smashes, zair, float) stay unknown.
 * DrMario/Falco/Pichu/Ganon/Emblem/CLink share their base-family tables. */
/* The tag a Geno profile declares for the state or animation `fp` is in (0 = none declared). */
static int script_hit_declared(Fighter* fp,int profile)
{
 extern int Geno_StateMoveTag(int profile,int state);
 extern int Geno_OverlayMoveTag(int profile,int index);
 extern int Geno_OverlayCount(int profile);
 extern int Geno_OverlayAnim(int profile,int index);
 extern int Geno_CommonMoveTag(int profile,int motion);
 int i,declared=0;
 /* Declared in the fighter's data first: a Geno state, a define common_states row, or a subaction
  * overlay on the animation now playing. */
 if(fp->motion_id>=0x400)declared=Geno_StateMoveTag(profile,fp->motion_id-0x400);
 else declared=Geno_CommonMoveTag(profile,fp->motion_id);
 for(i=0;!declared && i<Geno_OverlayCount(profile);++i)
  if(Geno_OverlayAnim(profile,i)==(int)fp->anim_id)declared=Geno_OverlayMoveTag(profile,i);
 return declared;
}
static int script_hit_family_move(Fighter* fp)
{
 static const unsigned short range[33][2]={
 {343,350},{341,369},{347,363},{369,386},{353,543},{341,363},{344,359},{341,364},
 {348,376},{352,368},{341,366},{341,366},{341,366},{341,356},{346,368},{346,372},
 {341,360},{341,358},{341,372},{341,358},{344,359},{343,350},{341,369},{341,366},
 {353,380},{347,363},{341,372},{0,0},{0,0},{0,0},{0,0},{341,363},{0,0}};
 int tag=script_hit_move_tag(fp->motion_id),profile,base,e=script_special_slot(fp);
 extern int Geno_ProfileForKind(int kind);
 extern int Geno_DefineBaseKind(int kind);
 profile=Geno_ProfileForKind(fp->kind);
 /* A DEFINE's own declaration wins over the tag its motion id would derive (slice 2, D6): a define may
  * bind a tilt's row to a smash-like script and say so. Retail, m-ex and attach fighters keep the old
  * order (the motion-id tag first), byte for byte. */
 if(profile>=0 && Geno_DefineBaseKind(fp->kind)>=0) {
  int declared=script_hit_declared(fp,profile);
  if(declared)return declared;
 }
 if(tag)return tag;
 if(profile>=0) {
  int declared=script_hit_declared(fp,profile);
  if(declared)return declared;
  /* Undeclared: a state a special input started is that special; a state in the base fighter's own
   * special range (define = native Mario, attach = the vanilla kind) keeps the vanilla answer. */
  if(fp->motion_id>=0x400)return e>=0 && script_special_open[e] ? 8 : 0;
  base=Geno_DefineBaseKind(fp->kind)>=0 ? 0 : fp->kind;
  if(base>=0 && base<33 && range[base][0] && fp->motion_id>=range[base][0] && fp->motion_id<=range[base][1])return 8;
  return e>=0 && script_special_open[e] && fp->motion_id>=341 ? 8 : 0;
 }
 if(fp->kind>=0 && fp->kind<33)
  return range[fp->kind][0] && fp->motion_id>=range[fp->kind][0] && fp->motion_id<=range[fp->kind][1] ? 8 : 0;
 /* m-ex fighters: no table of their own, so the input family is the honest answer. */
 return e>=0 && script_special_open[e] && fp->motion_id>=341 ? 8 : 0;
}
/* The move tag the hit context reports for `fp`'s current action (suite tests; the reports use the static). */
int ScriptGame_HitFamilyMove(Fighter* fp) {return script_hit_family_move(fp);}
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
