/* Native-only scope for decorative depth. No game struct crosses this boundary.
 * All game fog changes enter through gw_GXSetFog; range adjustment is untouched. */
typedef struct {
    u32 type;
    f32 startz, endz, nearz, farz;
    GXColor color;
} GwGxFogState;
static GwGxFogState gw_gx_fog_state, gw_gx_fog_saved;
static unsigned gw_gx_fog_scope_depth;
void gw_GXSetFog(u32 type, f32 startz, f32 endz, f32 nearz, f32 farz, GXColor color)
{
    gw_gx_fog_state.type=type;
    gw_gx_fog_state.startz=startz; gw_gx_fog_state.endz=endz;
    gw_gx_fog_state.nearz=nearz; gw_gx_fog_state.farz=farz;
    gw_gx_fog_state.color=color;
    GXSetFog((GXFogType)type,startz,endz,nearz,farz,color);
}
void gw_GXBackgroundFogBegin(void)
{
    GXColor clear={0,0,0,0};
    if (gw_gx_fog_scope_depth++ != 0) return;
    gw_gx_fog_saved=gw_gx_fog_state;
    GXSetFog((GXFogType)0,0,0,0,0,clear);
}
void gw_GXBackgroundFogEnd(void)
{
    if (gw_gx_fog_scope_depth==0 || --gw_gx_fog_scope_depth!=0) return;
    gw_gx_fog_state=gw_gx_fog_saved;
    GXSetFog((GXFogType)gw_gx_fog_saved.type,gw_gx_fog_saved.startz,gw_gx_fog_saved.endz,
             gw_gx_fog_saved.nearz,gw_gx_fog_saved.farz,gw_gx_fog_saved.color);
}
