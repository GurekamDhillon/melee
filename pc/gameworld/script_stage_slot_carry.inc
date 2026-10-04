#include "script_stage_slot_carry.h"
/* Search the full +/-512 world-unit column, not the old +/-1 contact probe.
 * Repeated downward queries select the closest floor above OR below. This
 * includes moving retail platforms and independent visible script seams. */
static int script_retail_carry_floor(float x,float y,Vec3* best,int* line,
                                     u32* flags,Vec3* normal)
{
    float upper=y+ST_CARRY_FLOOR_RANGE,lower=y-ST_CARRY_FLOOR_RANGE;
    float distance=ST_CARRY_FLOOR_RANGE+1;int found=0,i,skip=-1;
    for(i=0;i<script_stage.map->line_count;++i) {
        Vec3 p,n;int at=-1;u32 f=0;float d;
        if(!mpCheckFloor(x,upper,x,lower,0,&p,&at,&f,&n,skip,-1,-1,NULL,NULL))break;
        if(!st_carry_finite(p.y) || p.y>upper+0.01f || p.y<lower || at<0)break;
        d=p.y-y;if(d<0)d=-d;
        if(d<distance) {distance=d;*best=p;*normal=n;*line=at;*flags=f;found=1;}
        /* Strict progress even for coincident floors and numerical tolerance. */
        upper=p.y-0.01f;skip=at;if(upper<=lower)break;
    }
    return found;
}
static void script_retail_carry(void)
{
    HSD_GObj* g;
    if(!HSD_GObjPLinkHead)return;
    for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER];g;g=g->next) {
        Fighter* fp=GET_FIGHTER(g);struct CpuFighter cpu;Vec3 p,n,self,kb,shield;
        StCarry result;const char* rule;int line=-1,ground;u32 flags=0;float grvel;
        if(!fp)continue;
        ground=fp->ground_or_air==GA_Ground;cpu=fp->cpu;
        self=fp->self_vel;kb=fp->x8c_kb_vel;shield=fp->x98_atk_shield_kb;grvel=fp->gr_vel;
        result=st_carry_point(fp->cur_pos.x,fp->cur_pos.y,fp->cur_pos.z,ground,
            stage_info.blast_zone.left+stage_info.cam_info.cam_x_offset,
            stage_info.blast_zone.right+stage_info.cam_info.cam_x_offset,
            stage_info.blast_zone.bottom+stage_info.cam_info.cam_y_offset,
            stage_info.blast_zone.top+stage_info.cam_info.cam_y_offset);
        if(result.rule==ST_CARRY_INVALID) {
            /* Finite open-air and over-floor positions never take this path.
             * Melee maps are collision surfaces, not solid-volume definitions. */
            if(!Ground_801C2D24(fp->player_id,&p))p=(Vec3){0,0,0};
            fp->cur_pos=p;ftCo_Fall_Enter(g);rule="spawn-nonfinite-or-invalid-bounds";
        } else {
            fp->cur_pos.x=result.x;fp->cur_pos.y=result.y;fp->cur_pos.z=result.z;
            if(ground && script_retail_carry_floor(result.x,result.y,&p,&line,&flags,&n) &&
               p.y>stage_info.blast_zone.bottom+stage_info.cam_info.cam_y_offset &&
               p.y<stage_info.blast_zone.top+stage_info.cam_info.cam_y_offset) {
                fp->cur_pos.y=p.y;result.rule=ST_CARRY_FLOOR;rule="ground-floor";
            } else if(ground) {ftCo_Fall_Enter(g);rule="ground-no-floor-fall";}
            else rule=result.rule==ST_CARRY_CLAMP?"air-blast-clamp":"air-keep";
        }
        script_switch_contacts(fp);
        if(result.rule==ST_CARRY_FLOOR) {
            fp->coll_data.floor.index=line;fp->coll_data.floor.flags=flags;fp->coll_data.floor.normal=n;
            fp->coll_data.env_flags=fp->coll_data.prev_env_flags=Collide_FloorHug;
            fp->coll_data.contact=p;
        }
        fp->self_vel=self;fp->x8c_kb_vel=kb;fp->x98_atk_shield_kb=shield;fp->gr_vel=grvel;
        ftCamera_80076064(fp);fp->cpu=cpu;
        OSReport("stage switch: carry player=%d sub=%d rule=%s clamped=%d pos=(%g,%g,%g)\n",
                 fp->player_id,fp->is_sub_fighter,rule,result.clamped,
                 fp->cur_pos.x,fp->cur_pos.y,fp->cur_pos.z);
    }
}
