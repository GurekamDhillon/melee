/* Derived collision reads only. No mpCheck* call, collision-cache mutation or retained observer state. */
#include "script_contacts.h"
static int script_contact_owner(int line, int* part)
{
    int i, j, h = 0;
    *part = -1;
    if (script_stage.map && line >= script_stage.base_l &&
        line < script_stage.base_l + script_stage.cap) {
        ScriptStageLine* l = &script_stage.line[line-script_stage.base_l];
        if (l->active) h = l->handle;
    }
    if (!h) return 0;
    for (i=0;i<SCRIPT_MESH_INSTANCES;++i) {
        ScriptMeshInstance* m=&script_stage.instance[i];
        if (!m->handle) continue;
        for (j=0;j<m->count;++j) if (m->line[j].handle==h) {
            *part=j; return m->handle;
        }
    }
    return h; /* standalone gd.stage_add_* line owns itself */
}
static int script_contact_line(Fighter* fp, int surface)
{
    CollData* cd=&fp->coll_data;
    switch(surface) {
    case SC_FLOOR: return script_lab_mission_i(fp,LAB_I_ON_FLOOR) ? cd->floor.index : -1;
    /* Retail names are wall normals: a right-facing wall is on the fighter's left. */
    case SC_LEFT: return cd->env_flags & Collide_RightWallMask ? cd->right_facing_wall.index : -1;
    case SC_RIGHT: return cd->env_flags & Collide_LeftWallMask ? cd->left_facing_wall.index : -1;
    case SC_CEILING: return cd->env_flags & Collide_CeilingMask ? cd->ceiling.index : -1;
    case SC_LEDGE:
        if (!script_lab_mission_i(fp,LAB_I_LEDGE)) return -1;
        return fp->mv.co.cliff.ledge_id; /* latched by ftcliffcommon, unlike transient collision candidates */
    default: return -1;
    }
}
static int script_contact_valid(int line)
{
    MapCollData* map=mpLib_8004D164();
    CollLine* lines=mpGetGroundCollLine();
    return map && lines && mpGetGroundCollVtx() && line>=0 && line<map->line_count &&
        (lines[line].flags & LINE_FLAG_ENABLED) && !(lines[line].flags & LINE_FLAG_HIDDEN) &&
        lines[line].x0 && lines[line].x0->v0_idx<map->vert_count &&
        lines[line].x0->v1_idx<map->vert_count;
}
static int script_contact_sweep(float ax,float ay,float bx,float by,
    float px,float py,float x,float y,float* outx,float* outy)
{
    float dx,dy,old_side,new_side,u,cx,tmp;
    if(ax>bx){tmp=ax;ax=bx;bx=tmp;tmp=ay;ay=by;by=tmp;}
    dx=bx-ax;dy=by-ay;
    if(dx<=0 || y<=py)return 0;
    old_side=(py-ay)*dx-(px-ax)*dy;new_side=(y-ay)*dx-(x-ax)*dy;
    if(old_side>=0 || new_side<0)return 0;
    u=-old_side/(new_side-old_side);cx=px+(x-px)*u;
    if(cx<ax || cx>bx)return 0;
    *outx=cx;*outy=py+(y-py)*u;return 1;
}
int ScriptGame_ContactField(int slot, int surface, int field, int explicit_line)
{
    Fighter* fp=script_fighter(slot);
    Vec3 a,b;
    SurfaceData* data;
    int line, part, owner;
    if (!fp) return field==SC_LINE ? -1 : 0;
    line=explicit_line>=0 ? explicit_line : script_contact_line(fp,surface);
    if (!script_contact_valid(line)) return field==SC_LINE ? -1 : 0;
    if(field==SC_LINE) return line;
    if(field==SC_OWNER || field==SC_PART) {
        owner=script_contact_owner(line,&part);
        return field==SC_OWNER ? owner : part;
    }
    if(field==SC_KIND) return surface;
    if(field==SC_PASS) return (mpGetGroundCollLine()[line].x0->lo_flags & LINE_FLAG_PLATFORM)!=0;
    mpLineGetV0Pos(line,&a); mpLineGetV1Pos(line,&b);
    if(explicit_line>=0 && (field==SC_X || field==SC_Y)) {
        float cx,cy;
        CollData* cd=&fp->coll_data;
        if(script_contact_sweep(a.x,a.y,b.x,b.y,cd->prev_pos.x+cd->prev_ecb.bottom.x,
            cd->prev_pos.y+cd->prev_ecb.bottom.y,cd->cur_pos.x+cd->ecb.bottom.x,
            cd->cur_pos.y+cd->ecb.bottom.y,&cx,&cy))return script_mesh_bits(field==SC_X ? cx:cy);
    }
    switch(field) {
    case SC_X0: return script_mesh_bits(a.x); case SC_Y0: return script_mesh_bits(a.y);
    case SC_X1: return script_mesh_bits(b.x); case SC_Y1: return script_mesh_bits(b.y);
    case SC_X: return script_mesh_bits(fp->coll_data.cur_pos.x +
        (surface==SC_LEFT ? fp->coll_data.ecb.left.x : surface==SC_RIGHT ? fp->coll_data.ecb.right.x : fp->coll_data.ecb.bottom.x));
    case SC_Y: return script_mesh_bits(fp->coll_data.cur_pos.y +
        (surface==SC_CEILING ? fp->coll_data.ecb.top.y : surface==SC_LEFT ? fp->coll_data.ecb.left.y :
         surface==SC_RIGHT ? fp->coll_data.ecb.right.y : fp->coll_data.ecb.bottom.y));
    }
    data=surface==SC_LEFT ? &fp->coll_data.right_facing_wall : surface==SC_RIGHT ?
        &fp->coll_data.left_facing_wall : surface==SC_CEILING ? &fp->coll_data.ceiling : &fp->coll_data.floor;
    if(field==SC_NX) return script_mesh_bits(data->normal.x);
    if(field==SC_NY) return script_mesh_bits(data->normal.y);
    return 0;
}
static int script_contact_flags(Fighter* fp)
{
    int s;
    if(!fp) return 0;
    s=fp->motion_id;
    return ((s>=ftCo_MS_JumpF && s<=ftCo_MS_JumpAerialB) ? 1:0) |
        (s==ftCo_MS_Pass ? 2:0) |
        ((s==ftCo_MS_DamageFall || (s>=ftCo_MS_DamageHi1 && s<=ftCo_MS_DamageFlyRoll)) ? 4:0) |
        ((s>=ftCo_MS_DeadDown && s<=ftCo_MS_DeadUpFallHitCameraIce) ? 8:0) |
        ((s==ftCo_MS_Rebirth || s==ftCo_MS_RebirthWait) ? 16:0);
}
int ScriptGame_ContactFlags(int slot) { return script_contact_flags(script_fighter(slot)); }
/* Sweep the ECB bottom through pass-through segments, including host-stage lines.
 * Positive bottom motion, airborne, and a strict previous-side test suppress duplicates. */
int ScriptGame_ContactCross(int slot, int after)
{
    Fighter* fp=script_fighter(slot);
    MapCollData* map=mpLib_8004D164();
    int i;
    float px,py,x,y;
    if(!fp || !map || fp->ground_or_air!=GA_Air) return -1;
    px=fp->coll_data.prev_pos.x+fp->coll_data.prev_ecb.bottom.x;
    py=fp->coll_data.prev_pos.y+fp->coll_data.prev_ecb.bottom.y;
    x=fp->coll_data.cur_pos.x+fp->coll_data.ecb.bottom.x;
    y=fp->coll_data.cur_pos.y+fp->coll_data.ecb.bottom.y;
    if(y<=py) return -1;
    for(i=after+1;i<map->line_count;++i) {
        Vec3 a,b; float cx,cy;
        if(!script_contact_valid(i) || !(mpGetGroundCollLine()[i].flags & CollLine_Floor) ||
           !(mpGetGroundCollLine()[i].x0->lo_flags & LINE_FLAG_PLATFORM)) continue;
        mpLineGetV0Pos(i,&a); mpLineGetV1Pos(i,&b);
        if(script_contact_sweep(a.x,a.y,b.x,b.y,px,py,x,y,&cx,&cy))return i;
    }
    return -1;
}
#include "script_contacts_tests.inc"
