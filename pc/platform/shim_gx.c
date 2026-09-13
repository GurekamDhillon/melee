/* GX shims: every GX symbol melee's game objects import, in one translation unit.
 *
 * Aurora owns the GX implementation. These wrappers exist because gwtool renames every external
 * symbol the game calls to gw_<name>, and because game memory is big-endian while Aurora's C
 * parameters are native. Most entry points take scalars or opaque objects (GXTexObj, GXLightObj,
 * GXTlutObj, display lists, vertex data) that Aurora consumes exactly as a GameCube did, so they
 * forward unchanged. The wrappers that do real work are the ones where Aurora reads or writes
 * native values:
 *
 *   - matrices/float arrays passed in (GXLoad*MtxImm, GXSetProjection, GXSetIndTexMtx, GXProject,
 *     GXInitFogAdjTable) are converted from big-endian game memory to native floats;
 *   - float arrays written back (GXGetProjectionv, GXGetViewportv, GXProject's results) are
 *     converted the other way;
 *   - a struct passed by value arrives as a pointer to a big-endian copy: GXColor is four u8, so
 *     its bytes are already correct, but GXColorS10 is four s16 and needs each field swapped;
 *   - GXSetArray's 3-argument GameCube form grows the size/endianness arguments Aurora requires.
 *
 * The three render-mode objects are defined here as big-endian images because game code takes
 * their address and reads the fields itself (see shim_gx.h).
 *
 * GXWaitDrawDone and GXSetMisc/GXSetTevClampMode are not implemented by Aurora. The two draw-done
 * entry points route through the VI shim instead, which owns HSD's XFB state machine; the rest are
 * logged stubs. */
#include "shim_gx.h"
#include "shim_vi.h"

#include <dolphin/gx.h>

#include <math.h>

/* ---- render-mode images ------------------------------------------------------------------- */

unsigned char gw_GXNtsc480IntDf[GW_RENDER_MODE_SIZE];
unsigned char gw_GXNtsc480Int[GW_RENDER_MODE_SIZE];
unsigned char gw_GXNtsc480Prog[GW_RENDER_MODE_SIZE];

static void gw_store_render_mode(unsigned char *dst, u32 tv_mode, u16 fb_width, u16 efb_height,
                                 u16 xfb_height, u16 x_origin, u16 y_origin, u16 width, u16 height,
                                 u32 xfb_mode, u8 field_rendering, u8 aa,
                                 const u8 sample_pattern[12][2], const u8 vfilter[7]) {
  gw_w32(dst + 0x00, tv_mode);
  gw_w16(dst + 0x04, fb_width);
  gw_w16(dst + 0x06, efb_height);
  gw_w16(dst + 0x08, xfb_height);
  gw_w16(dst + 0x0A, x_origin);
  gw_w16(dst + 0x0C, y_origin);
  gw_w16(dst + 0x0E, width);
  gw_w16(dst + 0x10, height);
  gw_w32(dst + 0x14, xfb_mode);
  gw_w8(dst + 0x18, field_rendering);
  gw_w8(dst + 0x19, aa);
  memcpy(dst + 0x1A, sample_pattern, 24);
  memcpy(dst + 0x32, vfilter, 7);
}

void gw_gx_init_render_modes(void) {
  static const u8 pattern[12][2] = {
      {6, 6}, {6, 6}, {6, 6}, {6, 6}, {6, 6}, {6, 6},
      {6, 6}, {6, 6}, {6, 6}, {6, 6}, {6, 6}, {6, 6},
  };
  static const u8 vfilter_df[7] = {8, 8, 10, 12, 10, 8, 8};
  static const u8 vfilter_int[7] = {0, 0, 21, 22, 21, 0, 0};

  /* Values from melee's own SDK source, extern/dolphin/src/dolphin/gx/GXFrameBuf.c. */
  gw_store_render_mode(gw_GXNtsc480IntDf, VI_TVMODE_NTSC_INT, 640, 480, 480, 40, 0, 640, 480,
                       VI_XFBMODE_DF, 0, 0, pattern, vfilter_df);
  gw_store_render_mode(gw_GXNtsc480Int, VI_TVMODE_NTSC_INT, 640, 480, 480, 40, 0, 640, 480,
                       VI_XFBMODE_DF, 0, 0, pattern, vfilter_int);
  gw_store_render_mode(gw_GXNtsc480Prog, VI_TVMODE_NTSC_PROG, 640, 480, 480, 40, 0, 640, 480,
                       VI_XFBMODE_SF, 0, 0, pattern, vfilter_int);
}

void gw_read_render_mode(GXRenderModeObj *dst, const void *src_be) {
  const unsigned char *s = (const unsigned char *)src_be;
  dst->viTVmode = (VITVMode)gw_r32(s + 0x00);
  dst->fbWidth = gw_r16(s + 0x04);
  dst->efbHeight = gw_r16(s + 0x06);
  dst->xfbHeight = gw_r16(s + 0x08);
  dst->viXOrigin = gw_r16(s + 0x0A);
  dst->viYOrigin = gw_r16(s + 0x0C);
  dst->viWidth = gw_r16(s + 0x0E);
  dst->viHeight = gw_r16(s + 0x10);
  dst->xFBmode = (VIXFBMode)gw_r32(s + 0x14);
  dst->field_rendering = gw_r8(s + 0x18);
  dst->aa = gw_r8(s + 0x19);
  memcpy(dst->sample_pattern, s + 0x1A, sizeof dst->sample_pattern);
  memcpy(dst->vfilter, s + 0x32, sizeof dst->vfilter);
}

/* ---- draw-done and presentation ----------------------------------------------------------- */

/* HSD hangs its XFB state machine off the draw-done callback and spins on GXWaitDrawDone until it
 * fires, so both entry points go through the VI shim, which calls back synchronously. */
void gw_GXSetDrawDone(void) { gw_gx_set_draw_done(); }
void gw_GXWaitDrawDone(void) { gw_gx_wait_draw_done(); }
void *gw_GXSetDrawDoneCallback(void *cb) { return gw_gx_set_draw_done_callback(cb); }

/* Counters behind gw_gx_get_stats, so the frame driver's heartbeat can say whether the game is
 * actually drawing. A black window with zero copies means the game never finished a frame; zero
 * primitives means it never submitted geometry at all. */
static uint32_t gw_gx_copydisp_count;
static uint32_t gw_gx_prim_count;
static uint32_t gw_gx_dlist_count;

void gw_gx_get_stats(uint32_t *copies, uint32_t *prims, uint32_t *dlists) {
  *copies = gw_gx_copydisp_count;
  *prims = gw_gx_prim_count;
  *dlists = gw_gx_dlist_count;
}

/* The one point where a finished EFB copy means the frame is complete (see shim_vi.h). */
void gw_GXCopyDisp(void *dest, u8 clear) {
  GXCopyDisp(dest, (GXBool)clear);
  ++gw_gx_copydisp_count;
  gw_frame_mark_content();
}

void gw_GXCopyTex(void *dest, u8 clear) { GXCopyTex(dest, (GXBool)clear); }

/* ---- geometry ----------------------------------------------------------------------------- */

void gw_GXBegin(u32 type, u32 vtxfmt, u16 nverts) {
  ++gw_gx_prim_count;
  GXBegin((GXPrimitive)type, (GXVtxFmt)vtxfmt, nverts);
}

void gw_GXCallDisplayList(void *list, u32 nbytes) {
  ++gw_gx_dlist_count;
  GXCallDisplayList(list, nbytes);
}
void gw_GXClearVtxDesc(void) { GXClearVtxDesc(); }
void gw_GXInvalidateVtxCache(void) { GXInvalidateVtxCache(); }

void gw_GXEnableTexOffsets(u32 coord, u8 line_enable, u8 point_enable) {
  GXEnableTexOffsets((GXTexCoordID)coord, (GXBool)line_enable, (GXBool)point_enable);
}

void gw_GXSetLineWidth(u8 width, u32 tex_offsets) {
  GXSetLineWidth(width, (GXTexOffset)tex_offsets);
}

void gw_GXSetPointSize(u8 point_size, u32 tex_offsets) {
  GXSetPointSize(point_size, (GXTexOffset)tex_offsets);
}

void gw_GXSetNumTexGens(u8 n) { GXSetNumTexGens(n); }

void gw_GXSetTexCoordGen2(u32 dst_coord, u32 func, u32 src_param, u32 mtx, u8 normalize,
                          u32 pt_texmtx) {
  GXSetTexCoordGen2((GXTexCoordID)dst_coord, (GXTexGenType)func, (GXTexGenSrc)src_param, mtx,
                    (GXBool)normalize, pt_texmtx);
}

void gw_GXSetVtxDesc(u32 attr, u32 type) { GXSetVtxDesc((GXAttr)attr, (GXAttrType)type); }

void gw_GXSetVtxAttrFmt(u32 vtxfmt, u32 attr, u32 cnt, u32 type, u8 frac) {
  GXSetVtxAttrFmt((GXVtxFmt)vtxfmt, (GXAttr)attr, (GXCompCnt)cnt, (GXCompType)type, frac);
}

/* Melee calls the 3-argument GameCube form. Aurora bounds-checks indexed loads against the byte
 * length of the array, so give it everything from the base to the end of MEM1 when the array lives
 * there, and a value large enough to permit any index otherwise (e.g. arrays in static data). The
 * data is big-endian GC data, so le=false. */
void gw_GXSetArray(u32 attr, const void *base, u8 stride) {
  u32 size = 0xFFFFFFFFu;
  const unsigned char *p = (const unsigned char *)base;
  if (gw_mem1 != NULL && p >= gw_mem1 && p < gw_mem1 + gw_mem1_size) {
    size = (u32)(gw_mem1 + gw_mem1_size - p);
  }
  GXSetArray((GXAttr)attr, base, size, stride, false);
}

/* ---- transform ---------------------------------------------------------------------------- */

void gw_GXSetProjection(const void *mtx, u32 type) {
  f32 native[4][4];
  gw_read_mtx44(native, mtx);
  GXSetProjection(native, (GXProjectionType)type);
}

void gw_GXLoadPosMtxImm(const void *mtx, u32 id) {
  f32 native[3][4];
  gw_read_mtx(native, mtx);
  GXLoadPosMtxImm(native, id);
}

void gw_GXLoadNrmMtxImm(const void *mtx, u32 id) {
  f32 native[3][4];
  gw_read_mtx(native, mtx);
  GXLoadNrmMtxImm(native, id);
}

void gw_GXLoadTexMtxImm(const void *mtx, u32 id, u32 type) {
  f32 native[12];
  const int count = ((GXTexMtxType)type == GX_MTX2x4) ? 8 : 12;
  gw_read_f32v(native, mtx, count);
  GXLoadTexMtxImm(native, id, (GXTexMtxType)type);
}

void gw_GXSetCurrentMtx(u32 id) { GXSetCurrentMtx(id); }

void gw_GXSetViewport(f32 left, f32 top, f32 wd, f32 ht, f32 nearz, f32 farz) {
  GXSetViewport(left, top, wd, ht, nearz, farz);
}

void gw_GXSetViewportJitter(f32 left, f32 top, f32 wd, f32 ht, f32 nearz, f32 farz, u32 field) {
  GXSetViewportJitter(left, top, wd, ht, nearz, farz, field);
}

void gw_GXProject(f32 x, f32 y, f32 z, const void *mtx, const void *pm, const void *vp, f32 *sx,
                  f32 *sy, f32 *sz) {
  f32 native_mtx[3][4];
  f32 native_pm[GX_PROJECTION_SZ];
  f32 native_vp[6];
  f32 nx, ny, nz;
  gw_read_mtx(native_mtx, mtx);
  gw_read_f32v(native_pm, pm, GX_PROJECTION_SZ);
  gw_read_f32v(native_vp, vp, 6);
  GXProject(x, y, z, native_mtx, native_pm, native_vp, &nx, &ny, &nz);
  gw_wf32(sx, nx);
  gw_wf32(sy, ny);
  gw_wf32(sz, nz);
}

void gw_GXGetProjectionv(void *ptr) {
  f32 native[GX_PROJECTION_SZ];
  GXGetProjectionv(native);
  gw_write_f32v(ptr, native, GX_PROJECTION_SZ);
}

void gw_GXGetViewportv(void *vp) {
  f32 native[6];
  GXGetViewportv(native);
  gw_write_f32v(vp, native, 6);
}

/* ---- texture ------------------------------------------------------------------------------ */

void gw_GXInitTexObj(GXTexObj *obj, const void *image, u16 width, u16 height, u32 format,
                     u32 wrap_s, u32 wrap_t, u8 mipmap) {
  GXInitTexObj(obj, image, width, height, (GXTexFmt)format, (GXTexWrapMode)wrap_s,
               (GXTexWrapMode)wrap_t, (GXBool)mipmap);
}

void gw_GXInitTexObjCI(GXTexObj *obj, const void *image, u16 width, u16 height, u32 format,
                       u32 wrap_s, u32 wrap_t, u8 mipmap, u32 tlut_name) {
  GXInitTexObjCI(obj, image, width, height, (GXCITexFmt)format, (GXTexWrapMode)wrap_s,
                 (GXTexWrapMode)wrap_t, (GXBool)mipmap, tlut_name);
}

void gw_GXInitTexObjLOD(GXTexObj *obj, u32 min_filt, u32 mag_filt, f32 min_lod, f32 max_lod,
                        f32 lod_bias, u8 bias_clamp, u8 do_edge_lod, u32 max_aniso) {
  GXInitTexObjLOD(obj, (GXTexFilter)min_filt, (GXTexFilter)mag_filt, min_lod, max_lod, lod_bias,
                  (GXBool)bias_clamp, (GXBool)do_edge_lod, (GXAnisotropy)max_aniso);
}

void gw_GXInitTlutObj(GXTlutObj *obj, const void *lut, u32 format, u16 n_entries) {
  GXInitTlutObj(obj, lut, (GXTlutFmt)format, n_entries);
}

void gw_GXLoadTexObj(GXTexObj *obj, u32 id) { GXLoadTexObj(obj, (GXTexMapID)id); }

void gw_GXLoadTlut(GXTlutObj *obj, u32 id) { GXLoadTlut(obj, id); }

void gw_GXInvalidateTexAll(void) { GXInvalidateTexAll(); }

u32 gw_GXGetTexObjFmt(const void *obj) { return (u32)GXGetTexObjFmt((GXTexObj *)obj); }

u16 gw_GXGetTexObjWidth(const void *obj) { return GXGetTexObjWidth((GXTexObj *)obj); }

u16 gw_GXGetTexObjHeight(const void *obj) { return GXGetTexObjHeight((GXTexObj *)obj); }

u32 gw_GXGetTexBufferSize(u16 width, u16 height, u32 format, u8 mipmap, u8 max_lod) {
  return GXGetTexBufferSize(width, height, format, (GXBool)mipmap, max_lod);
}

/* ---- lighting ----------------------------------------------------------------------------- */

void gw_GXInitLightAttn(GXLightObj *obj, f32 a0, f32 a1, f32 a2, f32 k0, f32 k1, f32 k2) {
  GXInitLightAttn(obj, a0, a1, a2, k0, k1, k2);
}

void gw_GXInitLightPos(GXLightObj *obj, f32 x, f32 y, f32 z) { GXInitLightPos(obj, x, y, z); }

void gw_GXInitLightDir(GXLightObj *obj, f32 nx, f32 ny, f32 nz) {
  GXInitLightDir(obj, nx, ny, nz);
}

void gw_GXInitLightSpot(GXLightObj *obj, f32 cutoff, u32 spot_func) {
  GXInitLightSpot(obj, cutoff, (GXSpotFn)spot_func);
}

void gw_GXInitLightDistAttn(GXLightObj *obj, f32 ref_dist, f32 ref_br, u32 dist_func) {
  GXInitLightDistAttn(obj, ref_dist, ref_br, (GXDistAttnFn)dist_func);
}

/* By-value struct arguments arrive as their raw big-endian bytes (the caller's materialisation is
 * itself byte-swapped), so GXColor's single-byte fields pass straight through while GXColorS10's
 * s16 fields are swapped here. */
void gw_GXInitLightColor(GXLightObj *obj, GXColor color) {
  GXInitLightColor(obj, color);
}

void gw_GXLoadLightObjImm(GXLightObj *obj, u32 light) {
  GXLoadLightObjImm(obj, (GXLightID)light);
}

void gw_GXSetNumChans(u8 n) { GXSetNumChans(n); }

void gw_GXSetChanCtrl(u32 chan, u8 enable, u32 amb_src, u32 mat_src, u32 light_mask,
                      u32 diff_fn, u32 attn_fn) {
  GXSetChanCtrl((GXChannelID)chan, (GXBool)enable, (GXColorSrc)amb_src, (GXColorSrc)mat_src,
                light_mask, (GXDiffuseFn)diff_fn, (GXAttnFn)attn_fn);
}

void gw_GXSetChanAmbColor(u32 chan, GXColor color) {
  GXSetChanAmbColor((GXChannelID)chan, color);
}

void gw_GXSetChanMatColor(u32 chan, GXColor color) {
  GXSetChanMatColor((GXChannelID)chan, color);
}

/* ---- TEV ---------------------------------------------------------------------------------- */

void gw_GXSetNumTevStages(u8 n) { GXSetNumTevStages(n); }

void gw_GXSetTevOp(u32 id, u32 mode) { GXSetTevOp((GXTevStageID)id, (GXTevMode)mode); }

void gw_GXSetTevColorIn(u32 stage, u32 a, u32 b, u32 c, u32 d) {
  GXSetTevColorIn((GXTevStageID)stage, (GXTevColorArg)a, (GXTevColorArg)b, (GXTevColorArg)c,
                  (GXTevColorArg)d);
}

void gw_GXSetTevAlphaIn(u32 stage, u32 a, u32 b, u32 c, u32 d) {
  GXSetTevAlphaIn((GXTevStageID)stage, (GXTevAlphaArg)a, (GXTevAlphaArg)b, (GXTevAlphaArg)c,
                  (GXTevAlphaArg)d);
}

void gw_GXSetTevColorOp(u32 stage, u32 op, u32 bias, u32 scale, u8 clamp, u32 out_reg) {
  GXSetTevColorOp((GXTevStageID)stage, (GXTevOp)op, (GXTevBias)bias, (GXTevScale)scale,
                  (GXBool)clamp, (GXTevRegID)out_reg);
}

void gw_GXSetTevAlphaOp(u32 stage, u32 op, u32 bias, u32 scale, u8 clamp, u32 out_reg) {
  GXSetTevAlphaOp((GXTevStageID)stage, (GXTevOp)op, (GXTevBias)bias, (GXTevScale)scale,
                  (GXBool)clamp, (GXTevRegID)out_reg);
}

void gw_GXSetTevColor(u32 id, GXColor color) {
  GXSetTevColor((GXTevRegID)id, color);
}

void gw_GXSetTevColorS10(u32 id, GXColorS10 color) {
  const GXColorS10 native = {
      (s16)gw_bswap16((u16)color.r),
      (s16)gw_bswap16((u16)color.g),
      (s16)gw_bswap16((u16)color.b),
      (s16)gw_bswap16((u16)color.a),
  };
  GXSetTevColorS10((GXTevRegID)id, native);
}

void gw_GXSetTevKColor(u32 id, GXColor color) {
  GXSetTevKColor((GXTevKColorID)id, color);
}

void gw_GXSetTevKColorSel(u32 stage, u32 sel) {
  GXSetTevKColorSel((GXTevStageID)stage, (GXTevKColorSel)sel);
}

void gw_GXSetTevKAlphaSel(u32 stage, u32 sel) {
  GXSetTevKAlphaSel((GXTevStageID)stage, (GXTevKAlphaSel)sel);
}

void gw_GXSetTevSwapMode(u32 stage, u32 ras_sel, u32 tex_sel) {
  GXSetTevSwapMode((GXTevStageID)stage, (GXTevSwapSel)ras_sel, (GXTevSwapSel)tex_sel);
}

void gw_GXSetTevSwapModeTable(u32 table, u32 red, u32 green, u32 blue, u32 alpha) {
  GXSetTevSwapModeTable((GXTevSwapSel)table, (GXTevColorChan)red, (GXTevColorChan)green,
                        (GXTevColorChan)blue, (GXTevColorChan)alpha);
}

void gw_GXSetTevOrder(u32 stage, u32 coord, u32 map, u32 color) {
  GXSetTevOrder((GXTevStageID)stage, (GXTexCoordID)coord, (GXTexMapID)map, (GXChannelID)color);
}

void gw_GXSetAlphaCompare(u32 comp0, u8 ref0, u32 op, u32 comp1, u8 ref1) {
  GXSetAlphaCompare((GXCompare)comp0, ref0, (GXAlphaOp)op, (GXCompare)comp1, ref1);
}

void gw_GXSetZTexture(u32 op, u32 format, u32 bias) {
  GXSetZTexture((GXZTexOp)op, (GXTexFmt)format, bias);
}

void gw_GXSetTevDirect(u32 stage) { GXSetTevDirect((GXTevStageID)stage); }

/* Aurora does not implement GXSetTevClampMode (a GC-only register); nothing in the WebGPU pipeline
 * depends on it. */
void gw_GXSetTevClampMode(u32 stage, u32 mode) {
  (void)stage;
  (void)mode;
  GW_STUB();
}

/* ---- indirect stages ---------------------------------------------------------------------- */

void gw_GXSetNumIndStages(u8 n) { GXSetNumIndStages(n); }

void gw_GXSetIndTexOrder(u32 ind_stage, u32 tex_coord, u32 tex_map) {
  GXSetIndTexOrder((GXIndTexStageID)ind_stage, (GXTexCoordID)tex_coord, (GXTexMapID)tex_map);
}

void gw_GXSetIndTexCoordScale(u32 ind_stage, u32 scale_s, u32 scale_t) {
  GXSetIndTexCoordScale((GXIndTexStageID)ind_stage, (GXIndTexScale)scale_s, (GXIndTexScale)scale_t);
}

void gw_GXSetIndTexMtx(u32 mtx_id, const void *offset, s8 scale_exp) {
  f32 native[2][3];
  gw_read_f32v(&native[0][0], offset, 6);
  GXSetIndTexMtx((GXIndTexMtxID)mtx_id, native, scale_exp);
}

void gw_GXSetTevIndirect(u32 tev_stage, u32 ind_stage, u32 format, u32 bias_sel, u32 matrix_sel,
                         u32 wrap_s, u32 wrap_t, u8 add_prev, u8 utc_lod, u32 alpha_sel) {
  GXSetTevIndirect((GXTevStageID)tev_stage, (GXIndTexStageID)ind_stage, (GXIndTexFormat)format,
                   (GXIndTexBiasSel)bias_sel, (GXIndTexMtxID)matrix_sel, (GXIndTexWrap)wrap_s,
                   (GXIndTexWrap)wrap_t, (GXBool)add_prev, (GXBool)utc_lod,
                   (GXIndTexAlphaSel)alpha_sel);
}

/* ---- pixel / frame buffer ----------------------------------------------------------------- */

void gw_GXSetFog(u32 type, f32 startz, f32 endz, f32 nearz, f32 farz, GXColor color) {
  GXSetFog((GXFogType)type, startz, endz, nearz, farz, color);
}

/* The SDK computes this table on the CPU; Aurora has no equivalent, so port the reference body
 * from melee's own GXPixel.c and store the result as big-endian game data. */
void gw_GXInitFogAdjTable(void *table, u16 width, const void *projmtx) {
  f32 mtx[4][4];
  f32 near_z;
  f32 side_x;

  gw_read_mtx44(mtx, projmtx);
  if (0.0f == mtx[3][3]) {
    near_z = mtx[2][3] / (mtx[2][2] - 1.0f);
    side_x = (near_z * (1.0f + mtx[0][2])) / mtx[0][0];
  } else {
    near_z = (1.0f + mtx[2][3]) / mtx[2][2];
    side_x = -(mtx[0][3] - 1.0f) / mtx[0][0];
  }

  for (u32 i = 0; i < 10; ++i) {
    f32 xi = (f32)((i + 1) << 5);
    xi *= 2.0f / (f32)width;
    xi *= side_x;
    const f32 range = sqrtf(1.0f + (xi * xi) / (near_z * near_z));
    gw_w16((unsigned char *)table + 2 * i, (u16)(((u32)(256.0f * range)) & 0xFFF));
  }
}

void gw_GXSetFogRangeAdj(u8 enable, u16 center, const void *table) {
  GXFogAdjTable native;
  GXFogAdjTable *native_ptr = NULL;
  if (enable != 0 && table != NULL) {
    for (int i = 0; i < 10; ++i) {
      native.r[i] = gw_r16((const unsigned char *)table + 2 * i);
    }
    native_ptr = &native;
  }
  GXSetFogRangeAdj((GXBool)enable, center, native_ptr);
}

void gw_GXSetBlendMode(u32 type, u32 src_factor, u32 dst_factor, u32 op) {
  GXSetBlendMode((GXBlendMode)type, (GXBlendFactor)src_factor, (GXBlendFactor)dst_factor,
                 (GXLogicOp)op);
}

void gw_GXSetColorUpdate(u8 enable) { GXSetColorUpdate((GXBool)enable); }
void gw_GXSetAlphaUpdate(u8 enable) { GXSetAlphaUpdate((GXBool)enable); }

void gw_GXSetZMode(u8 compare_enable, u32 func, u8 update_enable) {
  GXSetZMode((GXBool)compare_enable, (GXCompare)func, (GXBool)update_enable);
}

void gw_GXSetZCompLoc(u8 before_tex) { GXSetZCompLoc((GXBool)before_tex); }

void gw_GXSetPixelFmt(u32 pix_fmt, u32 z_fmt) {
  GXSetPixelFmt((GXPixelFmt)pix_fmt, (GXZFmt16)z_fmt);
}

void gw_GXSetDither(u8 dither) { GXSetDither((GXBool)dither); }

void gw_GXSetDstAlpha(u8 enable, u8 alpha) { GXSetDstAlpha((GXBool)enable, alpha); }

void gw_GXSetFieldMode(u8 field_mode, u8 half_aspect_ratio) {
  GXSetFieldMode((GXBool)field_mode, (GXBool)half_aspect_ratio);
}

void gw_GXSetCopyClear(GXColor clear_clr, u32 clear_z) {
  GXSetCopyClear(clear_clr, clear_z);
}

/* Aurora declares GXSetCopyClamp but does not define it; the EFB copy path it would configure is
 * a no-op in Aurora's model, so nothing is lost. */
void gw_GXSetCopyClamp(u32 clamp) {
  (void)clamp;
  GW_STUB();
}

void gw_GXSetCopyFilter(u8 aa, const void *sample_pattern, u8 vf, const void *vfilter) {
  GXSetCopyFilter((GXBool)aa, (u8(*)[2])sample_pattern, (GXBool)vf, (u8 *)vfilter);
}

void gw_GXSetDispCopySrc(u16 left, u16 top, u16 wd, u16 ht) {
  GXSetDispCopySrc(left, top, wd, ht);
}

void gw_GXSetDispCopyDst(u16 wd, u16 ht) { GXSetDispCopyDst(wd, ht); }

u32 gw_GXSetDispCopyYScale(f32 vscale) { return GXSetDispCopyYScale(vscale); }

void gw_GXSetDispCopyGamma(u32 gamma) { GXSetDispCopyGamma((GXGamma)gamma); }

void gw_GXSetTexCopySrc(u16 left, u16 top, u16 wd, u16 ht) {
  GXSetTexCopySrc(left, top, wd, ht);
}

void gw_GXSetTexCopyDst(u16 wd, u16 ht, u32 format, u8 mipmap) {
  GXSetTexCopyDst(wd, ht, (GXTexFmt)format, (GXBool)mipmap);
}

/* ---- culling / scissor -------------------------------------------------------------------- */

void gw_GXSetScissor(u32 left, u32 top, u32 wd, u32 ht) { GXSetScissor(left, top, wd, ht); }

void gw_GXSetCullMode(u32 mode) { GXSetCullMode((GXCullMode)mode); }

/* ---- manage ------------------------------------------------------------------------------- */

GXFifoObj *gw_GXInit(void *base, u32 size) { return GXInit(base, size); }

void gw_GXPixModeSync(void) { GXPixModeSync(); }

/* GXSetMisc only tweaks GC-internal flush behaviour; Aurora needs none of it. */
void gw_GXSetMisc(u32 token, u32 val) {
  (void)token;
  (void)val;
  GW_STUB();
}
