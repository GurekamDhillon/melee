/* Synthetic flat-floor / multi-platform fixtures, not disc-derived stage data.
 * Exercises the actual read-only query without a game build or game memory. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
typedef struct { float x, y; } Vec2;
typedef struct { int v0_idx, v1_idx, prev_id0, next_id0, lo_flags; } MapLine;
typedef struct { MapLine* x0; unsigned flags; } CollLine;
typedef struct { Vec2 pos; } CollVtx;
typedef struct { int line_count; } MapCollData;
typedef struct { float left, right, top, bottom; } StageBlastZone;
enum { CollLine_Floor=1, LINE_FLAG_EMPTY=128, LINE_FLAG_PLATFORM=256,
       LINE_FLAG_ENABLED=65536, LINE_FLAG_HIDDEN=262144 };
static struct { StageBlastZone blast_zone; struct { StageBlastZone cam_bounds; } cam_info; } stage_info;
static struct { MapCollData* map; int base_l; } script_stage;
static MapCollData map;
static MapLine authored[6];
static CollLine lines[6];
static CollVtx verts[12];
static MapCollData* mpLib_8004D164(void) { return &map; }
static CollLine* mpGetGroundCollLine(void) { return lines; }
static CollVtx* mpGetGroundCollVtx(void) { return verts; }
static int script_mesh_bits(float f) { union { int i; float f; } v; v.f=f; return v.i; }
#include "../gameworld/script_bounds.inc"
static void floor_line(int i, float x0, float y0, float x1, float y1, int platform)
{
    authored[i].v0_idx=i*2; authored[i].v1_idx=i*2+1;
    authored[i].prev_id0=authored[i].next_id0=-1;
    authored[i].lo_flags=platform ? LINE_FLAG_PLATFORM : 0;
    lines[i].x0=&authored[i]; lines[i].flags=CollLine_Floor|LINE_FLAG_ENABLED;
    verts[i*2].pos.x=x0; verts[i*2].pos.y=y0;
    verts[i*2+1].pos.x=x1; verts[i*2+1].pos.y=y1;
}
int main(void)
{
    float out[5];
    CollLine before_lines[6]; CollVtx before_verts[12];
    map.line_count=1;
    floor_line(0,-50,0,50,0,0);
    assert(script_stage_floor_bounds(out) && out[0]==-50 && out[1]==50 && out[2]==0);
    /* A segmented, sloped main floor with three pass-through platforms. */
    map.line_count=6; script_stage.map=&map; script_stage.base_l=5;
    floor_line(0,-50,0,0,4,0); floor_line(1,0,4,50,0,0);
    authored[0].next_id0=1; authored[1].prev_id0=0;
    floor_line(2,-40,20,-10,20,1); floor_line(3,10,20,40,20,1);
    floor_line(4,-15,40,15,40,1);
    floor_line(5,-500,90,500,90,0); /* scripted floor must not change placement */
    memcpy(before_lines,lines,sizeof lines); memcpy(before_verts,verts,sizeof verts);
    assert(script_stage_floor_bounds(out));
    assert(out[0]==-50 && out[1]==50 && out[2]==4 && out[3]==0 && out[4]==40);
    assert(!memcmp(before_lines,lines,sizeof lines) && !memcmp(before_verts,verts,sizeof verts));
    stage_info.blast_zone.left=-200; stage_info.cam_info.cam_bounds.right=120;
    assert(ScriptGame_StageBounds(0)==script_mesh_bits(-200));
    assert(ScriptGame_StageBounds(5)==script_mesh_bits(120));
    assert(ScriptGame_StageBounds(12)==1 && ScriptGame_StageBounds(13)==script_mesh_bits(40));
    /* Disconnected floor islands choose the widest, rather than spanning a gap. */
    authored[0].next_id0=authored[1].prev_id0=-1;
    verts[3].pos.x=80;
    assert(script_stage_floor_bounds(out) && out[0]==0 && out[1]==80);
    lines[0].flags|=LINE_FLAG_HIDDEN; lines[1].flags|=LINE_FLAG_EMPTY;
    assert(!script_stage_floor_bounds(out) && !ScriptGame_StageBounds(12));
    puts("stage bounds live geometry selection passed");
    return 0;
}
