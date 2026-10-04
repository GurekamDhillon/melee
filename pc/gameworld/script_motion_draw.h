#ifndef SCRIPT_MOTION_DRAW_H
#define SCRIPT_MOTION_DRAW_H
#include <sysdolphin/baselib/cobj.h>
#include <melee/cm/camera.h>
#include <melee/pl/player.h>
extern int MotionFighterBegin(int,int,int,int,int,int,int,int,int,int);
extern void MotionFighterEnd(void);
extern HSD_CObj* Camera_PCMainCObj(void);
static int script_motion_draw_begin(HSD_GObj* gobj)
{
    Fighter* fp=gobj->user_data;
    HSD_CObj* camera=HSD_CObjGetCurrent();
    union {float f;int i;} speed;
    /* Exclude reflection/refraction and fighter diagnostic cameras. */
    /* Compare with the camera gobj's own cobj: cm_804D6464 is a second copy that is never current here,
     * which made this test always fail and no pose was ever captured. */
    if(Camera_80031060()!=0 || !camera || camera!=Camera_PCMainCObj())return 0;
    speed.f=fp->self_vel.x*fp->self_vel.x+fp->self_vel.y*fp->self_vel.y;
    return MotionFighterBegin(fp->player_id+1,fp->is_sub_fighter,(int)gobj,
        (int)&camera->view_mtx,(int)&fp->cur_pos,fp->x2219_b5,
        fp->item_gobj!=NULL,fp->x619_costume_id,Player_GetFalls(fp->player_id),speed.i);
}
#endif
