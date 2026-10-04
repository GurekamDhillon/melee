/* Included by ftcoll.c after its headers. Observe the existing cancellation
 * decisions, without changing their ordering, damage, registrations or FX. */
#if defined(TARGET_PC)
extern int Script_ClankWanted(void);
extern void Script_Clank(int,int,int,int,int,int,int,int,int,int,int,int);
static int script_clank_bits(float f) { union { float f; int i; } u;u.f=f;return u.i; }
static void script_clank_observe(Fighter* a,HitCapsule* ha,Fighter* b,
                                 Item* item,HitCapsule* hb)
{
    int ca,cb;
    if(!Script_ClankWanted())return;
    ca=(int) ha->damage-p_ftCommonData->x3CC<(int) hb->damage;
    cb=(int) hb->damage-p_ftCommonData->x3CC<(int) ha->damage;
    if(!ca && !cb)return;
    Script_Clank(a->player_id,b?b->player_id:-1,
        item?(int)item->entity:0,item?(int)item->kind:-1,
        script_clank_bits((ha->hurt_coll_pos.x+hb->hurt_coll_pos.x)*0.5f),
        script_clank_bits((ha->hurt_coll_pos.y+hb->hurt_coll_pos.y)*0.5f),
        script_clank_bits((ha->hurt_coll_pos.z+hb->hurt_coll_pos.z)*0.5f),
        script_clank_bits(ha->damage),script_clank_bits(hb->damage),ca|(cb<<1),
        (int)a->gobj,b?(int)b->gobj:(int)item->entity);
}
#endif
