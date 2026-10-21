// The render-test scene, drawn by Aurora (oracle.cpp `render`) and by melee_gx (tests/render.rs,
// `scene` module there is the same code in Rust). Everything is integer-generated or a literal so
// both sides build identical bytes: a 16x16 RGB5A3 noise texture, a big-endian S16 position array,
// and two display lists (textured quads/strip/fan with a 2-stage TEV; a lit, alpha-blended pair of
// triangles). Change one side, change the other.

static uint32_t scene_rng = 0x1234567u;
static uint32_t scene_next() {
  scene_rng ^= scene_rng << 13;
  scene_rng ^= scene_rng >> 17;
  scene_rng ^= scene_rng << 5;
  return scene_rng;
}

alignas(32) static uint8_t scene_tex[16 * 16 * 2];
alignas(32) static uint8_t scene_pos[32 * 6];
alignas(32) static uint8_t scene_dl_a[512];
alignas(32) static uint8_t scene_dl_b[256];
static uint32_t scene_dl_a_size, scene_dl_b_size;
static bool scene_built;
static unsigned scene_mask = 3; // oracle-only debugging aid: which objects to draw

static void put16(uint8_t*& p, uint16_t v) {
  *p++ = static_cast<uint8_t>(v >> 8);
  *p++ = static_cast<uint8_t>(v);
}
static void put32(uint8_t*& p, uint32_t v) {
  put16(p, static_cast<uint16_t>(v >> 16));
  put16(p, static_cast<uint16_t>(v));
}
static void putf(uint8_t*& p, float f) {
  uint32_t u;
  memcpy(&u, &f, 4);
  put32(p, u);
}

// Positions (pixels, S16 frac 2): quad, strip, fan.
static const int16_t kPos[][2] = {
    {40, 40},   {300, 40},  {300, 220}, {40, 220},                // quad
    {340, 40},  {600, 40},  {340, 220}, {600, 220},               // strip
    {170, 350}, {60, 260},  {280, 260}, {280, 440}, {60, 440},    // fan
};
static const uint8_t kClr[][4] = {
    {255, 255, 255, 255}, {255, 64, 64, 255}, {64, 255, 64, 255}, {64, 64, 255, 255},
    {255, 255, 0, 255},   {0, 255, 255, 255}, {255, 0, 255, 255}, {128, 128, 128, 255},
    {255, 255, 255, 255}, {255, 128, 0, 255}, {0, 128, 255, 255}, {128, 255, 0, 255}, {255, 0, 128, 255},
};
static const float kUV[][2] = {
    {0.f, 0.f}, {2.f, 0.f}, {2.f, 1.5f}, {0.f, 1.5f},
    {0.f, 0.f}, {1.f, 0.f}, {0.f, 1.f},  {1.f, 1.f},
    {0.5f, 0.5f}, {0.f, 0.f}, {1.f, 0.f}, {1.f, 1.f}, {0.f, 1.f},
};

static void scene_build() {
  if (scene_built) return;
  scene_built = true;
  scene_rng = 0x1234567u;
  for (auto& b : scene_tex) b = static_cast<uint8_t>(scene_next() >> 24);
  uint8_t* p = scene_pos;
  for (const auto& v : kPos) {
    put16(p, static_cast<uint16_t>(v[0] * 4));
    put16(p, static_cast<uint16_t>(v[1] * 4));
    put16(p, static_cast<uint16_t>(-50 * 4));
  }
  // DL A (VTXFMT0): pos index8, clr0 direct RGBA8, tex0 direct F32 ST.
  p = scene_dl_a;
  const auto vtx = [&](int i) {
    *p++ = static_cast<uint8_t>(i);
    for (int c = 0; c < 4; ++c) *p++ = kClr[i][c];
    putf(p, kUV[i][0]);
    putf(p, kUV[i][1]);
  };
  *p++ = 0x80; put16(p, 4); for (int i = 0; i < 4; ++i) vtx(i);
  *p++ = 0x98; put16(p, 4); for (int i = 4; i < 8; ++i) vtx(i);
  *p++ = 0xA0; put16(p, 5); for (int i = 8; i < 13; ++i) vtx(i);
  scene_dl_a_size = static_cast<uint32_t>(((p - scene_dl_a) + 31) & ~31);
  // DL B (VTXFMT1): pos direct F32 XYZ, nrm direct F32 XYZ; two triangles.
  p = scene_dl_b;
  static const float kB[6][6] = {
      {360, 260, -40, 0.f, 0.f, 1.f},     {600, 260, -40, 0.6f, 0.f, 0.8f},  {600, 440, -40, 0.6f, 0.6f, 0.53f},
      {360, 260, -40, 0.f, 0.f, 1.f},     {600, 440, -40, 0.6f, 0.6f, 0.53f}, {360, 440, -40, -0.7f, 0.2f, 0.69f},
  };
  *p++ = 0x91; put16(p, 6);
  for (const auto& v : kB) for (float f : v) putf(p, f);
  scene_dl_b_size = static_cast<uint32_t>(((p - scene_dl_b) + 31) & ~31);
}

static void scene_draw() {
  scene_build();
  GXSetCopyClear(GXColor{32, 48, 64, 255}, GX_MAX_Z24);
  GXSetViewport(0.f, 0.f, 640.f, 480.f, 0.f, 1.f);
  GXSetScissor(0, 0, 640, 480);
  float proj[4][4] = {};
  proj[0][0] = 2.0f / 640.0f;
  proj[0][3] = -1.0f;
  proj[1][1] = -2.0f / 480.0f;
  proj[1][3] = 1.0f;
  proj[2][2] = -1.0f / 100.0f;
  proj[2][3] = -1.0f;
  proj[3][3] = 1.0f;
  GXSetProjection(proj, GX_ORTHOGRAPHIC);
  float ident[3][4] = {{1, 0, 0, 0}, {0, 1, 0, 0}, {0, 0, 1, 0}};
  GXLoadPosMtxImm(ident, GX_PNMTX0);
  GXLoadNrmMtxImm(ident, GX_PNMTX0);
  GXSetCurrentMtx(GX_PNMTX0);
  GXSetCullMode(GX_CULL_NONE);
  GXSetZMode(GX_TRUE, GX_LEQUAL, GX_TRUE);
  GXSetAlphaCompare(GX_ALWAYS, 0, GX_AOP_AND, GX_ALWAYS, 0);
  GXSetFog(GX_FOG_NONE, 0.f, 1.f, 0.1f, 1.f, GXColor{0, 0, 0, 0});
  GXSetNumIndStages(0);

  // Object A: textured, vertex colour, 2 TEV stages.
  static GXTexObj tex;
  GXInitTexObj(&tex, scene_tex, 16, 16, GX_TF_RGB5A3, GX_REPEAT, GX_REPEAT, GX_FALSE);
  GXInitTexObjLOD(&tex, GX_NEAR, GX_NEAR, 0.f, 0.f, 0.f, GX_FALSE, GX_FALSE, GX_ANISO_1);
  GXLoadTexObj(&tex, GX_TEXMAP0);
  GXClearVtxDesc();
  GXSetVtxDesc(GX_VA_POS, GX_INDEX8);
  GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
  GXSetVtxDesc(GX_VA_TEX0, GX_DIRECT);
  GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_S16, 2);
  GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
  GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_F32, 0);
  GXSetArray(GX_VA_POS, scene_pos, sizeof(scene_pos), 6, false);
  GXSetNumChans(1);
  GXSetChanCtrl(GX_COLOR0A0, GX_FALSE, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_NONE, GX_AF_NONE);
  GXSetNumTexGens(1);
  GXSetTexCoordGen(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY);
  GXSetNumTevStages(2);
  GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
  GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_TEXC, GX_CC_RASC, GX_CC_ZERO);
  GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_TEXA, GX_CA_RASA, GX_CA_ZERO);
  GXSetTevColorOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_TRUE, GX_TEVPREV);
  GXSetTevAlphaOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_TRUE, GX_TEVPREV);
  GXSetTevOrder(GX_TEVSTAGE1, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR_NULL);
  GXSetTevKColor(GX_KCOLOR0, GXColor{60, 20, 0, 255});
  GXSetTevKColorSel(GX_TEVSTAGE1, GX_TEV_KCSEL_K0);
  GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_KONST, GX_CC_HALF, GX_CC_CPREV);
  GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_APREV);
  GXSetTevColorOp(GX_TEVSTAGE1, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_TRUE, GX_TEVPREV);
  GXSetTevAlphaOp(GX_TEVSTAGE1, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_TRUE, GX_TEVPREV);
  GXSetBlendMode(GX_BM_NONE, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
  if (scene_mask & 1) GXCallDisplayList(scene_dl_a, scene_dl_a_size);

  // Object B: per-vertex lighting (one point light), register colours, alpha blend.
  static GXLightObj light;
  GXInitLightPos(&light, 480.f, 240.f, 60.f);
  GXInitLightColor(&light, GXColor{255, 240, 200, 255});
  GXInitLightAttn(&light, 1.f, 0.f, 0.f, 1.f, 0.f, 0.f);
  GXLoadLightObjImm(&light, GX_LIGHT0);
  GXClearVtxDesc();
  GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
  GXSetVtxDesc(GX_VA_NRM, GX_DIRECT);
  GXSetVtxAttrFmt(GX_VTXFMT1, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
  GXSetVtxAttrFmt(GX_VTXFMT1, GX_VA_NRM, GX_NRM_XYZ, GX_F32, 0);
  GXSetNumChans(1);
  GXSetChanCtrl(GX_COLOR0A0, GX_TRUE, GX_SRC_REG, GX_SRC_REG, GX_LIGHT0, GX_DF_CLAMP, GX_AF_NONE);
  GXSetChanAmbColor(GX_COLOR0A0, GXColor{40, 40, 60, 255});
  GXSetChanMatColor(GX_COLOR0A0, GXColor{200, 100, 50, 160});
  GXSetNumTexGens(0);
  GXSetNumTevStages(1);
  GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR0A0);
  GXSetTevOp(GX_TEVSTAGE0, GX_PASSCLR);
  GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
  if (scene_mask & 2) GXCallDisplayList(scene_dl_b, scene_dl_b_size);
}
