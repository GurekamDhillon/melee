/* Item visuals are reconstructed from the current snapshot-owned item. Assets
 * are immutable scene pins; there is no per-item host instance to rewind. */
static void gs_item_draw_primitive(const void* guest_view,float local[3][4],unsigned tint,float height) {
 static const float cube[8][3]={{-2,0,-2},{2,0,-2},{2,6,-2},{-2,6,-2},{-2,0,2},{2,0,2},{2,6,2},{-2,6,2}};
 static const unsigned char faces[24]={0,1,2,3,4,7,6,5,0,4,5,1,3,2,6,7,0,3,7,4,1,5,6,2};
 /* Keep the legacy six-unit fallback unless the model height was authored.
  * Scaling its local Y coordinates preserves the same tilt/scale transform. */
 float yscale=height>0?height/6.0f:1.0f;
 float mv[3][4];int r,c,i;
 for(r=0;r<3;++r)for(c=0;c<4;++c) {
  const unsigned char* v=(const unsigned char*)guest_view+r*16;
  mv[r][c]=gw_rf32(v)*local[0][c]+gw_rf32(v+4)*local[1][c]+gw_rf32(v+8)*local[2][c]+(c==3?gw_rf32(v+12):0);
 }
 GXClearVtxDesc();GXSetVtxDesc(GX_VA_POS,GX_DIRECT);GXSetVtxDesc(GX_VA_CLR0,GX_DIRECT);
 GXSetVtxAttrFmt(GX_VTXFMT0,GX_VA_POS,GX_POS_XYZ,GX_F32,0);GXSetVtxAttrFmt(GX_VTXFMT0,GX_VA_CLR0,GX_CLR_RGBA,GX_RGBA8,0);
 GXSetNumTexGens(0);GXSetNumChans(1);GXSetNumTevStages(1);GXSetNumIndStages(0);
 GXSetChanCtrl(GX_COLOR0A0,GX_FALSE,GX_SRC_REG,GX_SRC_VTX,GX_LIGHT_NULL,GX_DF_NONE,GX_AF_NONE);
 GXSetTevOrder(GX_TEVSTAGE0,GX_TEXCOORD_NULL,GX_TEXMAP_NULL,GX_COLOR0A0);GXSetTevOp(GX_TEVSTAGE0,GX_PASSCLR);
 GXSetCullMode(GX_CULL_NONE);GXSetZMode(GX_TRUE,GX_LEQUAL,GX_TRUE);GXSetBlendMode(GX_BM_NONE,GX_BL_ONE,GX_BL_ZERO,GX_LO_COPY);
 GXSetAlphaCompare(GX_ALWAYS,0,GX_AOP_AND,GX_ALWAYS,0);
 GXLoadPosMtxImm(mv,GX_PNMTX0);GXSetCurrentMtx(GX_PNMTX0);
 GXBegin(GX_QUADS,GX_VTXFMT0,24);
 for(i=0;i<24;++i) {
  const float* p=cube[faces[i]];unsigned shade=180+(i/4%2)*55;
  GXPosition3f32(p[0],p[1]*yscale,p[2]);GXColor4u8((u8)((tint>>24)*shade/255),(u8)(((tint>>16)&255)*shade/255),(u8)(((tint>>8)&255)*shade/255),255);
 }GXEnd();gs_model_bound=NULL;
}
void gw_ItemDraw(int index,const void* view,int code) {
 static const unsigned colours[5]={0xEE514AFFu,0x57D376FFu,0x539DF0FFu,0xF6D74FFFu,0xF4F2EBFFu};
 float local[3][4]={{0}},rot[3][3],scale;int d,c,r,k,part,model;
 unsigned tint;
 if(!view||(code!=0&&code!=2)||gw_Geno_ItemI(index,GENO_II_VISIBLE)<=0) return;
 d=gw_Geno_ItemI(index,GENO_II_DEF);c=gw_Geno_ItemI(index,GENO_II_COLOUR);
 if(d<0||d>=GENO_ITEM_MAX_DEFS||c<0||c>=5) return;
 /* Ry(spin)*Rx(tilt): rotate the tilted long axis so even symmetric
  * crystals visibly orbit. Rx after Ry would leave the long axis stationary. */
 script_model_rotation(gs_camera_float(gw_Geno_ItemVisual(index,GENO_IV_TILT)),
  gs_camera_float(gw_Geno_ItemVisual(index,GENO_IV_ROT_Y)),0,rot);
 scale=gs_camera_float(gw_Geno_ItemVisual(index,GENO_IV_SCALE));
 for(r=0;r<3;++r)for(k=0;k<3;++k)local[r][k]=rot[r][k]*scale;
 local[0][3]=gs_camera_float(gw_Geno_ItemF(index,GENO_IF_X));local[1][3]=gs_camera_float(gw_Geno_ItemVisual(index,GENO_IV_Y));local[2][3]=gs_camera_float(gw_Geno_ItemF(index,GENO_IF_Z));
 tint=gw_Geno_ItemTint(d);
 gs_model_batch_flush(view);
 if(!gs_item_models[d][c][0]) {
  if(code==0)gs_item_draw_primitive(view,local,tint==0xFFFFFFFFu?colours[c]:tint,
   gs_camera_float(gw_Geno_ItemParam(d,GENO_IP_HEIGHT)));
  return;
 }
 for(part=0;part<2;++part) {
  int alpha;
  model=gs_item_models[d][c][part];if(model<1||model>gs_stage_nmodels)continue;
  alpha=part||gs_stage_models[model-1].collision.alpha||(tint&255)!=255;
  if(alpha!=(code==2))continue;
  gs_model_bound=NULL;gs_model_draw(model,view,local,part?0xFFFFFFFFu:tint,alpha);gs_model_bound=NULL;
 }
}
