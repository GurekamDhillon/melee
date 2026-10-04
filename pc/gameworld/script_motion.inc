/* Read-only motion anchors. No HSD_JObjSetupMatrix, game allocation, callback
 * replacement, game globals, or gameplay writes. Matrices are sampled after the
 * live render updates them. Sword tip reads the retail trail's latest sample;
 * it never enables or replaces that trail (ftCo_800C2FD8). */
#include <melee/ft/ftparts.h>
static Fighter* script_motion_fighter(int slot,int sub)
{
    HSD_GObj* g;
    if(slot<0||slot>=6||sub<0||sub>1)return NULL;
    g=Player_GetEntityAtIndex(slot,sub);
    return script_gobj_live(g)?GET_FIGHTER(g):NULL;
}
/* Called only for warm-up or a retain-only FIFO scope. Reuse the actual item
 * draw callback so visibility, custom parts and render offsets match live draws;
 * never advance item animation, physics, hitboxes or ownership here. */
int ScriptGame_MotionHeldDraw(int slot,int sub)
{
    Fighter* fp=script_motion_fighter(slot,sub);
    HSD_GObj* item;
    if(!fp || !(item=fp->item_gobj) || !script_gobj_live(item) || !item->render_cb)return 0;
    HSD_StateInvalidate(-1);
    { extern HSD_CObj* Camera_PCMainCObj(void);
      HSD_CObj* main=Camera_PCMainCObj();
      if(main && HSD_CObjGetCurrent()==main){int pass;for(pass=0;pass<3;++pass)item->render_cb(item,pass);}
      else if(main && item->hsd_obj){
          /* Warm-up outside the camera walk: callbacks may dereference Current.
           * Discover root model materials with an explicit main-camera view. */
          HSD_JObjDispAll(item->hsd_obj,main->view_mtx,HSD_GObj_80390EB8(0)|HSD_GObj_80390EB8(1)|HSD_GObj_80390EB8(2),0);
      }else return 0;
    }
    HSD_StateInvalidate(-1);
    return 1;
}
int ScriptGame_MotionAnchorI(int slot,int sub,int kind,int index,int item,int field)
{
    Fighter* fp=script_motion_fighter(slot,sub);
    HSD_GObj* g;
    if(kind==4){
        for(g=HSD_GObjPLinkHead?HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]:NULL;g;g=g->next)
            if(GET_ITEM(g)->x1C==item)return field==0?1:field==2?item:0;
        return 0;
    }
    if(!fp)return 0;
    if(field==2)return (int)fp->gobj;
    if(field==3)return Player_GetFalls(slot);
    if(field==4)return fp->x619_costume_id;
    if(field==1 && kind==1){HitCapsule* h=lab_hit(fp,index);return h?(int)h->element:0;}
    if(field==1)return 0;
    if(kind==1){HitCapsule* h=lab_hit(fp,index);return h && h->state!=HitCapsule_Disabled && (index!=4||h->owner);}
    if(kind==2)return fp->item_gobj!=NULL;
    if(kind==3)return fp->x2100>0 && fp->x2100<=3 && fp->x2101_bits_0to6<3;
    if(kind==0){
        static const int common[]={FtPart_LHandN,FtPart_RHandN,FtPart_LFootJ,FtPart_RFootJ,FtPart_HeadN};
        if(index<=-1&&index>=-5)index=ftParts_GetBoneIndex(fp,common[-index-1]);
        return index>=0&&index<lab_joint_count(fp)&&fp->parts[index].joint!=NULL;
    }
    return 0;
}
float ScriptGame_MotionAnchorF(int slot,int sub,int kind,int index,int item,int field)
{
    Fighter* fp=script_motion_fighter(slot,sub);
    HSD_GObj* g;
    if(field<0||field>2)return 0;
    if(kind==4){
        for(g=HSD_GObjPLinkHead?HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]:NULL;g;g=g->next)
            if(GET_ITEM(g)->x1C==item){Vec3 p=GET_ITEM(g)->pos;return field==0?p.x:field==1?p.y:p.z;}
        return 0;
    }
    if(!fp)return 0;
    if(kind==1){HitCapsule* h=lab_hit(fp,index);return h?(field==0?h->x4C.x:field==1?h->x4C.y:h->x4C.z):0;}
    if(kind==2 && fp->item_gobj){Vec3 p=GET_ITEM(fp->item_gobj)->pos;return field==0?p.x:field==1?p.y:p.z;}
    if(kind==3 && fp->x2100>0 && fp->x2100<=3 && fp->x2101_bits_0to6<3){
        int n=(fp->x2101_bits_0to6+2)%3;
        struct Fighter_x20B0_t* e=&fp->x20B0[n];
        return field==0?e->x0.x+e->xC.x*fp->x20FC:field==1?e->x0.y+e->xC.y*fp->x20FC:e->x0.z+e->xC.z*fp->x20FC;
    }
    if(kind==0){
        static const int common[]={FtPart_LHandN,FtPart_RHandN,FtPart_LFootJ,FtPart_RFootJ,FtPart_HeadN};
        if(index<=-1&&index>=-5)index=ftParts_GetBoneIndex(fp,common[-index-1]);
        if(index>=0&&index<lab_joint_count(fp)&&fp->parts[index].joint)return fp->parts[index].joint->mtx[field][3];
    }
    return 0;
}
