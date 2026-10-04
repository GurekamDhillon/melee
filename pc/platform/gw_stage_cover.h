#ifndef GW_STAGE_COVER_H
#define GW_STAGE_COVER_H
/* Wipe enters left-to-right, then exits left-to-right. Both halves meet
 * at full coverage, where the atomic switch occurs. Flash peaks at white. */
static float gw_stage_cover_amount(int flash, float progress, float x)
{
    if(progress<0)progress=0;if(progress>1)progress=1;
    if(flash)return progress<0.5f?progress*2:(1-progress)*2;
    return (progress<0.5f?(x<progress*2):(x>=progress*2-1))?1.0f:0.0f;
}
#ifdef __cplusplus
static const char* gw_stage_cover_wgsl=R"WGSL(
let c = previous_color(in.uv);
let t = clamp(params.progress.x, 0.0, 1.0);
var a = 1.0 - abs(2.0 * t - 1.0);
var cover = vec3f(1.0);
if (params.flash.x < 0.5) {
  cover = vec3f(0.063, 0.094, 0.153);
  a = select(select(0.0, 1.0, in.uv.x >= 2.0*t-1.0),
             select(0.0, 1.0, in.uv.x < 2.0*t), t < 0.5);
}
return vec4f(mix(c.rgb, cover, a), c.a);
)WGSL";
#endif
#endif
