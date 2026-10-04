/* Continuous carry policy. Shared verbatim with standalone fixtures. */
#ifndef SCRIPT_STAGE_SLOT_CARRY_H
#define SCRIPT_STAGE_SLOT_CARRY_H
#define ST_CARRY_FLOOR_RANGE 512.0f
#define ST_CARRY_BLAST_MARGIN 1.0f
enum { ST_CARRY_AIR, ST_CARRY_CLAMP, ST_CARRY_FLOOR, ST_CARRY_FALL,
       ST_CARRY_INVALID };
typedef struct { float x,y,z; int rule,clamped; } StCarry;
static int st_carry_finite(float x)
{ return x==x && x<=3.402823466e38f && x>=-3.402823466e38f; }
static StCarry st_carry_point(float x,float y,float z,int ground,
                              float left,float right,float bottom,float top)
{
    StCarry r={x,y,z,ground?ST_CARRY_FALL:ST_CARRY_AIR,0};
    if(!st_carry_finite(x) || !st_carry_finite(y) || !st_carry_finite(z) ||
       !st_carry_finite(left) || !st_carry_finite(right) ||
       !st_carry_finite(bottom) || !st_carry_finite(top) ||
       right-left<=2*ST_CARRY_BLAST_MARGIN || top-bottom<=2*ST_CARRY_BLAST_MARGIN) {
        r.rule=ST_CARRY_INVALID;return r;
    }
    if(r.x<left) {r.x=left+ST_CARRY_BLAST_MARGIN;r.clamped=1;}
    if(r.x>right) {r.x=right-ST_CARRY_BLAST_MARGIN;r.clamped=1;}
    if(r.y<bottom) {r.y=bottom+ST_CARRY_BLAST_MARGIN;r.clamped=1;}
    if(r.y>top) {r.y=top-ST_CARRY_BLAST_MARGIN;r.clamped=1;}
    if(!ground && r.clamped)r.rule=ST_CARRY_CLAMP;
    return r;
}
#endif
