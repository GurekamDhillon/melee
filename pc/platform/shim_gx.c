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
#include <stdio.h>

#include "shim_gx.h"
#include "shim_vi.h"

#include <aurora/gfx.h>
#include <dolphin/gx/GXCpu2Efb.h> /* TEMP DIAG: aurora_get_stats */

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

/* TEMP DIAG (remove with the rest of the black-screen instrumentation): the state the EFB copy
 * inherits, and how much geometry actually reached the frame being copied. A frame that presents
 * with ~0 draws behind it is black because nothing was drawn into it; a frame with draws behind it
 * but colorupd=0 is black because nothing was allowed to write colour. These two cases need
 * opposite fixes, and only a live run separates them. Sampled twice a second so a run can be
 * paused and unpaused and the two regimes compared in one log. */
static uint32_t gw_diag_last_prim;
static uint32_t gw_diag_last_dlist;
static u8 gw_diag_colorupd = 0xFF;
static u8 gw_diag_alphaupd = 0xFF;
static u8 gw_diag_zcmp = 0xFF;
static u8 gw_diag_zupd = 0xFF;
static GXColor gw_diag_clearclr;
static uint32_t gw_diag_seg_prim;
static uint32_t gw_diag_seg_dlist;
static unsigned gw_diag_mtx_slots_this_frame;
static unsigned gw_diag_mtx_calls;
static unsigned gw_diag_mtx_bad;   /* NaN or infinite elements */
static unsigned gw_diag_mtx_huge;  /* |element| > 1e6, i.e. geometry flung out of clip space */
static float gw_diag_mtx_maxabs;
static float gw_diag_mtx_rowlen_min = 1e30f;
static float gw_diag_mtx_rowlen_max;
static uint32_t gw_diag_copytex_calls;
static uint32_t gw_diag_copytex_clears;


/* TEMP DIAG: called from HSD_JObjSetupMatrixSub (TARGET_PC-guarded) to find where NaN first
 * enters a joint matrix. stage 0 = straight out of make_mtx, before any IK; stage 1 = after the
 * joint branch has run. branch is the JOBJ_JOINT selector: 0 JOINT1, 1 JOINT2, 2 EFFECTOR,
 * 3 default. Counting both stages separates "animation produced a NaN transform" from "the IK
 * solver produced one", which is the whole question. */
static uint32_t gw_jobj_seen[2][4];
static uint32_t gw_jobj_nan[2][4];

void gw_diag_jobj_mtx(const float *mtx, int stage, int branch) {
  int i;
  int bad = 0;
  if (mtx == NULL || stage < 0 || stage > 1 || branch < 0 || branch > 3) {
    return;
  }
  for (i = 0; i < 12; ++i) {
    /* jobj->mtx lives in big-endian game memory: read it through gw_rf32, not natively. */
    union { float f; uint32_t u; } v;
    v.f = gw_rf32(&mtx[i]);
    if ((v.u & 0x7F800000u) == 0x7F800000u) {
      bad = 1;
      break;
    }
  }
  ++gw_jobj_seen[stage][branch];
  if (bad) {
    ++gw_jobj_nan[stage][branch];
  }
}

/* TEMP DIAG: the camera's viewing matrix and the three vectors C_MTXLookAt builds it from.
 * The modelview handed to GXLoadPosMtxImm is view x joint, so a NaN view matrix poisons nearly
 * every matrix even when only a few joints are bad -- which is exactly the 4% -> 87% jump the
 * jobj counters show. LookAt goes NaN when eye == interest, or when up is parallel to the view
 * direction: the cross product is zero and normalising it divides by zero. */
static uint32_t gw_cobj_calls;
static uint32_t gw_cobj_nan_mtx;
static uint32_t gw_cobj_nan_in;
static int gw_cobj_reported;

static int gw_diag_is_bad(float f) {
  union { float f; uint32_t u; } v;
  v.f = f;
  return (v.u & 0x7F800000u) == 0x7F800000u;
}

/* TEMP DIAG: the game camera's own source data, sampled inside Camera_8002AF68 -- the
 * CAMERA_STANDARD path that a normal match uses. The existing cobj probe fires for all ~90
 * cobjs per frame without saying which; this one fires only for the main game camera, which is
 * the one whose view matrix multiplies into every model's modelview. If these inputs are
 * already bad, the rot is upstream in game_camera and the camera code is just a courier. */
static uint32_t gw_gcam_calls;
static uint32_t gw_gcam_bad_interest;
static uint32_t gw_gcam_bad_position;
static uint32_t gw_gcam_bad_translation;
static int gw_gcam_reported;

/* TEMP DIAG: the camera tuning constants from cm_803BCCA0, printed once. x64 is the smoothing
 * coefficient in interest += (target - interest) * x64 and must be a small fraction; anything
 * else makes the update diverge. These live in the original binary's static data. */
/* TEMP DIAG: the tracked player's position, the subject extent and the camera yaw -- the only
 * inputs to target_interest. Counts bad values per frame and prints the first few offenders. */
static uint32_t gw_subj_calls;
static uint32_t gw_subj_bad;
static int gw_subj_reported;

void gw_diag_camera_subject(float px, float py, float pz, float ext_z, float yaw) {
  const int bad = gw_diag_is_bad(px) || gw_diag_is_bad(py) || gw_diag_is_bad(pz) ||
                  gw_diag_is_bad(ext_z) || gw_diag_is_bad(yaw);
  ++gw_subj_calls;
  if (bad) {
    ++gw_subj_bad;
  }
  if ((bad || px > 1e6f || px < -1e6f || py > 1e6f || py < -1e6f) && gw_subj_reported < 6) {
    ++gw_subj_reported;
    gw_log("gw: DIAG   subject BAD pos=(%g,%g,%g) ext_z=%g yaw=%g", (double)px, (double)py,
           (double)pz, (double)ext_z, (double)yaw);
  }
}

void gw_diag_camera_subject_report(void) {
  if (gw_subj_calls != 0u) {
    gw_log("gw: DIAG   subject: calls=%u bad=%u", gw_subj_calls, gw_subj_bad);
  }
  gw_subj_calls = 0u;
  gw_subj_bad = 0u;
}

/* TEMP DIAG: catch the MOMENT game_camera's transform goes bad, rather than guessing which
 * branch writes it. Keeps the previous value of each component and logs the transition from good
 * to bad with both values. The shape of that transition is the diagnosis:
 *   a sudden jump from a sane number straight to 1e33 or NaN  -> something assigns garbage
 *   a value roughly doubling or growing each frame            -> a feedback loop diverging
 * Also logs a bad -> good recovery, since the camera evidently does recover between episodes. */
static float gw_camtrk_prev[6];
static int gw_camtrk_prev_bad[6];
static int gw_camtrk_have;
static int gw_camtrk_events;

static int gw_diag_suspect(float f) {
  return gw_diag_is_bad(f) || f > 1e6f || f < -1e6f;
}

/* TEMP DIAG: read back C-source static initialisers whose values are known exactly from the
 * source, to test whether statics defined in game C source are read with the wrong endianness.
 *   cm_803BCB3C.pos = { 0.0f, 40.241425f, 300.241f }   (eyepos)
 *   cm_803BCB50.pos = { 0.0f, 10.0f, 0.0f }            (interest)
 *   fov 30.0f, near 0.1f, far 16384.0f, aspect 1.2173333f
 * cm_803BCCA0, which reads correctly, is extern data from the ORIGINAL big-endian binary, so a
 * byte-swapping load is right for it. Statics emitted by clang are native little-endian and the
 * same load would read them backwards. If the values below come back wrong, that is the bug. */
/* TEMP DIAG: the inputs to the only writer of game_camera.translation that is not a zeroing
 * call (Camera_80030DE4 at camera.c:978). translation.x/.y are NaN in game, and the computation
 * divides by the viewport width and height, so a zero viewport extent yields infinity and then
 * NaN. Expect viewport 0..640 x 0..480, aspect ~1.217, fov ~30. */
void gw_diag_cam_translate(int xmin, int xmax, int ymin, int ymax, float aspect, float fov,
                           float z_pos, float half_h, float out_x, float out_y) {
  static int n;
  if (n >= 4) {
    return;
  }
  ++n;
  gw_log("gw: DIAG   camtrans viewport=[%d..%d]x[%d..%d] aspect=%g fov=%g z_pos=%g half_h=%g",
         xmin, xmax, ymin, ymax, (double)aspect, (double)fov, (double)z_pos, (double)half_h);
  gw_log("gw: DIAG   camtrans -> translation=(%g, %g)", (double)out_x, (double)out_y);
}

/* TEMP DIAG: envelope (skinned) blending. SetupEnvelopeModelMtx accumulates
 *     mtx += joint_mtx * weight
 * over the envelope list, so the weights for one matrix slot must sum to 1.0 or the blended
 * transform is scaled wrong and the limb stretches. Mario's head (a single weight-1 joint) takes
 * the other branch and renders correctly, which is consistent with the blend being at fault. */
static uint32_t gw_env_draws;
static uint32_t gw_env_badsum;
static int gw_env_reported;

void gw_diag_envelope(int count, float weight_sum) {
  ++gw_env_draws;
  if (weight_sum < 0.99f || weight_sum > 1.01f) {
    ++gw_env_badsum;
    if (gw_env_reported < 8) {
      ++gw_env_reported;
      gw_log("gw: DIAG   envelope BAD joints=%d weight_sum=%g", count, (double)weight_sum);
    }
  }
}

void gw_diag_envelope_report(void) {
  if (gw_env_draws != 0u) {
    gw_log("gw: DIAG   envelope: blends=%u bad_weight_sum=%u", gw_env_draws, gw_env_badsum);
  }
  gw_env_draws = 0u;
  gw_env_badsum = 0u;
}

void gw_diag_camera_desc(const float *eyepos, const float *interest, float fov, float nearz,
                         float farz, float aspect) {
  static int done;
  int i;
  if (done || eyepos == NULL || interest == NULL) {
    return;
  }
  done = 1;
  /* Game memory is big-endian by design (gwtool pre-swaps scalar initialisers), so a native
   * shim MUST read game-visible scalars through gw_rf32. Reading them natively byte-swaps them
   * and manufactures exactly the huge/tiny garbage this probe was built to look for. */
  gw_log("gw: DIAG   camdesc eyepos=(%g,%g,%g) expect (0, 40.241425, 300.241)",
         (double)gw_rf32(&eyepos[0]), (double)gw_rf32(&eyepos[1]), (double)gw_rf32(&eyepos[2]));
  gw_log("gw: DIAG   camdesc interest=(%g,%g,%g) expect (0, 10, 0)",
         (double)gw_rf32(&interest[0]), (double)gw_rf32(&interest[1]),
         (double)gw_rf32(&interest[2]));
  gw_log("gw: DIAG   camdesc fov=%g near=%g far=%g aspect=%g  expect 30 / 0.1 / 16384 / 1.2173333",
         (double)fov, (double)nearz, (double)farz, (double)aspect);
  for (i = 0; i < 3; ++i) {
    union { float f; uint32_t u; } v, sw;
    v.f = eyepos[i];
    sw.u = ((v.u & 0x000000FFu) << 24) | ((v.u & 0x0000FF00u) << 8) |
           ((v.u & 0x00FF0000u) >> 8) | ((v.u & 0xFF000000u) >> 24);
    gw_log("gw: DIAG   camdesc eyepos[%d] bits=%08X byteswapped=%g", i, v.u, (double)sw.f);
  }
}

void gw_diag_camera_track(const float *interest, const float *position) {
  static const char *const names[6] = {"int.x", "int.y", "int.z", "pos.x", "pos.y", "pos.z"};
  float cur[6];
  int i;
  if (interest == NULL || position == NULL) {
    return;
  }
  for (i = 0; i < 3; ++i) {
    cur[i] = gw_rf32(&interest[i]);      /* big-endian game memory */
    cur[i + 3] = gw_rf32(&position[i]);
  }
  for (i = 0; i < 6; ++i) {
    const int bad = gw_diag_suspect(cur[i]);
    if (gw_camtrk_have && bad != gw_camtrk_prev_bad[i] && gw_camtrk_events < 24) {
      ++gw_camtrk_events;
      union { float f; uint32_t u; } v, sw;
      v.f = cur[i];
      sw.u = ((v.u & 0x000000FFu) << 24) | ((v.u & 0x0000FF00u) << 8) |
             ((v.u & 0x00FF0000u) >> 8) | ((v.u & 0xFF000000u) >> 24);
      /* If this value is a big-endian float being read natively, the byte-swapped
       * interpretation is the real one and should look like a plausible camera coordinate. */
      gw_log("gw: DIAG   camtrk %s %s: %g -> %g   [bits=%08X  byteswapped=%g]", names[i],
             bad ? "WENT BAD" : "recovered", (double)gw_camtrk_prev[i], (double)cur[i], v.u,
             (double)sw.f);
    }
    gw_camtrk_prev[i] = cur[i];
    gw_camtrk_prev_bad[i] = bad;
  }
  gw_camtrk_have = 1;
}

void gw_diag_camera_consts(float smooth, float target_fov, float fov_rate, float scale) {
  static int done;
  if (done) {
    return;
  }
  done = 1;
  gw_log("gw: DIAG   camera consts: smooth(x64)=%g target_fov(x6C)=%g fov_rate(x70)=%g scale(x3C)=%g",
         (double)smooth, (double)target_fov, (double)fov_rate, (double)scale);
}

void gw_diag_game_camera(const float *interest, const float *position, const float *translation) {
  int i;
  int bi = 0, bp = 0, bt = 0;
  if (interest == NULL || position == NULL || translation == NULL) {
    return;
  }
  ++gw_gcam_calls;
  for (i = 0; i < 3; ++i) {
    if (gw_diag_is_bad(gw_rf32(&interest[i]))) {
      bi = 1;
    }
    if (gw_diag_is_bad(gw_rf32(&position[i]))) {
      bp = 1;
    }
    if (gw_diag_is_bad(gw_rf32(&translation[i]))) {
      bt = 1;
    }
  }
  gw_gcam_bad_interest += (uint32_t)bi;
  gw_gcam_bad_position += (uint32_t)bp;
  gw_gcam_bad_translation += (uint32_t)bt;
  if ((bi || bp || bt) && gw_gcam_reported < 4) {
    ++gw_gcam_reported;
    gw_log("gw: DIAG   gamecam BAD interest=(%.3f,%.3f,%.3f) position=(%.3f,%.3f,%.3f) "
           "translation=(%.3f,%.3f,%.3f)",
           interest[0], interest[1], interest[2], position[0], position[1], position[2],
           translation[0], translation[1], translation[2]);
  }
}

void gw_diag_cobj_view(const float *mtx, const float *eye, const float *up, const float *interest) {
  int i;
  int badIn = 0;
  int badMtx = 0;
  if (mtx == NULL || eye == NULL || up == NULL || interest == NULL) {
    return;
  }
  ++gw_cobj_calls;
  for (i = 0; i < 3; ++i) {
    if (gw_diag_is_bad(gw_rf32(&eye[i])) || gw_diag_is_bad(gw_rf32(&up[i])) ||
        gw_diag_is_bad(gw_rf32(&interest[i]))) {
      badIn = 1;
    }
  }
  for (i = 0; i < 12; ++i) {
    if (gw_diag_is_bad(gw_rf32(&mtx[i]))) {
      badMtx = 1;
    }
  }
  if (badIn) {
    ++gw_cobj_nan_in;
  }
  if (badMtx) {
    ++gw_cobj_nan_mtx;
  }
  if ((badIn || badMtx) && gw_cobj_reported < 4) {
    ++gw_cobj_reported;
    /* Raw bits as well as values: %.3f prints a denormal as 0.000, which is exactly how a
     * misread up-vector would masquerade as a genuinely zero one. 0x3F800000 is 1.0f. */
    gw_log("gw: DIAG   cobj BAD eye=(%g,%g,%g) up=(%g,%g,%g) interest=(%g,%g,%g)",
           (double)gw_rf32(&eye[0]), (double)gw_rf32(&eye[1]), (double)gw_rf32(&eye[2]),
           (double)gw_rf32(&up[0]), (double)gw_rf32(&up[1]), (double)gw_rf32(&up[2]),
           (double)gw_rf32(&interest[0]), (double)gw_rf32(&interest[1]),
           (double)gw_rf32(&interest[2]));
    gw_log("gw: DIAG   cobj BAD up bits (be-read)=%08X %08X %08X  raw native=%08X %08X %08X",
           gw_r32(&up[0]), gw_r32(&up[1]), gw_r32(&up[2]),
           ((const uint32_t *)up)[0], ((const uint32_t *)up)[1], ((const uint32_t *)up)[2]);
    gw_log("gw: DIAG   cobj view row0 bits=%08X %08X %08X %08X", gw_r32(&mtx[0]), gw_r32(&mtx[1]),
           gw_r32(&mtx[2]), gw_r32(&mtx[3]));
  }
}

void gw_diag_aobj_bad(void *aobj, uint32_t flags, void *caller) {
  static int reported;
  if (reported < 8) {
    const uint32_t c = (uint32_t)(uintptr_t)caller;
    const uint32_t csw = (c >> 24) | ((c >> 8) & 0x0000FF00u) | ((c << 8) & 0x00FF0000u) | (c << 24);
    ++reported;
    gw_log("gw: DIAG   BADAOBJ aobj=%08X flags=%08X caller=%08X caller_bswap=%08X",
           (uint32_t)(uintptr_t)aobj, flags, c, csw);
  }
}

void gw_diag_card_engine_read(int buf, int len, int off) {
  static int logged;
  if (logged < 30) {
    ++logged;
    gw_log("gw: DIAG   CARDEREAD buf=%08X len=%d off=%d", (unsigned)buf, len, off);
  }
}

void gw_diag_card_read(int type, int state, int dst, int size) {
  static int logged;
  if (logged < 40) {
    ++logged;
    gw_log("gw: DIAG   CARDREAD type=%d state=%08X dst=%08X size=%d", type, state, dst, size);
  }
}

void gw_diag_card_dequeue(int type, int arg) {
  static int logged;
  if (logged < 40) {
    ++logged;
    gw_log("gw: DIAG   CARDDEQ type=%d arg=%08X", type, (unsigned)arg);
  }
}

void gw_diag_card_engine(int busy, int idx, int type, int read_idx, int write_idx, int tail) {
  static int logged;
  if (logged < 80) {
    ++logged;
    gw_log("gw: DIAG   CARDENG busy=%d head=%d type=%d tail=%d r=%d w=%d", busy, idx, type, tail,
           read_idx, write_idx);
  }
}

void gw_diag_card_pending(int pending, int state) {
  static int logged;
  if (logged < 60) {
    ++logged;
    gw_log("gw: DIAG   CARDPEND pending=%d state=%d", pending, state);
  }
}

void gw_diag_card_state(void *x5c, int enable, void *x64) {
  static int reported;
  if (reported < 8) {
    ++reported;
    gw_log("gw: DIAG   CARDSTATE x5C=%08X enable=%d x64=%08X", (uint32_t)(uintptr_t)x5c, enable,
           (uint32_t)(uintptr_t)x64);
  }
}

void gw_diag_jobj_report(void) {
  static const char *const names[4] = {"JOINT1", "JOINT2", "EFFECTOR", "default"};
  int b;
  gw_diag_camera_subject_report();
  gw_diag_envelope_report();
  if (gw_gcam_calls != 0u) {
    gw_log("gw: DIAG   gamecam: calls=%u bad_interest=%u bad_position=%u bad_translation=%u",
           gw_gcam_calls, gw_gcam_bad_interest, gw_gcam_bad_position, gw_gcam_bad_translation);
  }
  gw_gcam_calls = 0u;
  gw_gcam_bad_interest = 0u;
  gw_gcam_bad_position = 0u;
  gw_gcam_bad_translation = 0u;
  if (gw_cobj_calls != 0u) {
    gw_log("gw: DIAG   cobj lookat: calls=%u nan_inputs=%u nan_viewmtx=%u", gw_cobj_calls,
           gw_cobj_nan_in, gw_cobj_nan_mtx);
  }
  gw_cobj_calls = 0u;
  gw_cobj_nan_in = 0u;
  gw_cobj_nan_mtx = 0u;
  for (b = 0; b < 4; ++b) {
    if (gw_jobj_seen[0][b] != 0u || gw_jobj_seen[1][b] != 0u) {
      gw_log("gw: DIAG   jobj %-8s premake nan=%u/%u   postjoint nan=%u/%u", names[b],
             gw_jobj_nan[0][b], gw_jobj_seen[0][b], gw_jobj_nan[1][b], gw_jobj_seen[1][b]);
    }
    gw_jobj_seen[0][b] = gw_jobj_seen[1][b] = 0u;
    gw_jobj_nan[0][b] = gw_jobj_nan[1][b] = 0u;
  }
}

/* The one point where a finished EFB copy means the frame is complete (see shim_vi.h). */
void gw_GXCopyDisp(void *dest, u8 clear) {
  if ((gw_gx_copydisp_count % 30u) == 0u) {
    gw_log("gw: DIAG copy #%u  since-last prim=%u dlist=%u  clear=%u colorupd=%u alphaupd=%u "
           "zcmp=%u zupd=%u clearclr=%u,%u,%u,%u",
           gw_gx_copydisp_count, gw_gx_prim_count - gw_diag_last_prim,
           gw_gx_dlist_count - gw_diag_last_dlist, (unsigned)clear, (unsigned)gw_diag_colorupd,
           (unsigned)gw_diag_alphaupd, (unsigned)gw_diag_zcmp, (unsigned)gw_diag_zupd,
           (unsigned)gw_diag_clearclr.r, (unsigned)gw_diag_clearclr.g, (unsigned)gw_diag_clearclr.b,
           (unsigned)gw_diag_clearclr.a);
    {
      const AuroraStats *st = aurora_get_stats();
      gw_log("gw: DIAG   aurora drawcalls=%u merged=%u verts=%u", st->drawCallCount,
             st->mergedDrawCallCount, st->lastVertSize);
    }
    gw_log("gw: DIAG   copytex since-last calls=%u with-clear=%u; tail-seg prim=%u dlist=%u",
           gw_diag_copytex_calls, gw_diag_copytex_clears, gw_gx_prim_count - gw_diag_seg_prim,
           gw_gx_dlist_count - gw_diag_seg_dlist);
  }
  gw_diag_seg_prim = gw_gx_prim_count;
  gw_diag_seg_dlist = gw_gx_dlist_count;
  /* TEMP DIAG: sample the depth buffer across the frame. This distinguishes the two remaining
   * possibilities without needing anyone to look at the screen. Depth writes are enabled
   * (zupd=1 every frame), so if triangles are rasterising they must leave varying depth behind.
   *   all samples identical -> nothing rasterises at all (culled, degenerate, or off-screen)
   *   samples vary          -> geometry IS rasterising and the fault is in colour output
   * GXPeekZ returns the previous frame's snapshot and requests a new one, so a whole grid read
   * in one frame comes from one consistent snapshot. */
  if ((gw_gx_copydisp_count % 30u) == 0u) {
    unsigned distinct = 0;
    u32 seen[8];
    u32 zmin = 0xFFFFFFFFu;
    u32 zmax = 0;
    for (int gy = 0; gy < 5; ++gy) {
      for (int gx = 0; gx < 7; ++gx) {
        u32 z = 0;
        unsigned k;
        GXPeekZ((u16)(45 + gx * 92), (u16)(48 + gy * 96), &z);
        if (z < zmin) {
          zmin = z;
        }
        if (z > zmax) {
          zmax = z;
        }
        for (k = 0; k < distinct; ++k) {
          if (seen[k] == z) {
            break;
          }
        }
        if (k == distinct && distinct < 8u) {
          seen[distinct++] = z;
        }
      }
    }
    gw_log("gw: DIAG   depth grid: distinct=%u min=0x%06X max=0x%06X", distinct, zmin, zmax);
    gw_diag_jobj_report();
    gw_log("gw: DIAG   posmtx health: loads=%u nan/inf=%u huge=%u maxabs=%.3f rowlen=[%.4f..%.4f]",
           gw_diag_mtx_calls, gw_diag_mtx_bad, gw_diag_mtx_huge, gw_diag_mtx_maxabs,
           gw_diag_mtx_rowlen_min > 1e29f ? 0.0f : gw_diag_mtx_rowlen_min, gw_diag_mtx_rowlen_max);
  }
  gw_diag_mtx_calls = 0;
  gw_diag_mtx_bad = 0;
  gw_diag_mtx_huge = 0;
  gw_diag_mtx_maxabs = 0.0f;
  gw_diag_mtx_rowlen_min = 1e30f;
  gw_diag_mtx_rowlen_max = 0.0f;

  gw_diag_copytex_calls = 0;
  gw_diag_copytex_clears = 0;
  gw_diag_mtx_slots_this_frame = 0;
  gw_diag_last_prim = gw_gx_prim_count;
  gw_diag_last_dlist = gw_gx_dlist_count;

  GXCopyDisp(dest, (GXBool)clear);
  ++gw_gx_copydisp_count;
  gw_frame_mark_content();
}

/* TEMP DIAG: GXCopyTex resolves the EFB into a texture and, when clear is set, CLEARS the EFB
 * afterwards (GXFrameBuffer.cpp copy_tex -> resolve_pass_into with clearColor/clearAlpha/
 * clearDepth). A mid-frame copy-with-clear therefore wipes everything drawn so far. If Melee
 * issues one between the 3D scene and the HUD, the scene is discarded and only the HUD survives
 * the frame -- which is exactly the reported symptom. Count them per frame to find out. */
void gw_GXCopyTex(void *dest, u8 clear) {
  ++gw_diag_copytex_calls;
  if (clear) {
    ++gw_diag_copytex_clears;
  }
  /* TEMP DIAG: how much geometry this copy is about to wipe. Sampled on the same frames as the
   * copy-disp line so one frame's segments read together. */
  if ((gw_gx_copydisp_count % 30u) == 0u) {
    gw_log("gw: DIAG   copytex seg prim=%u dlist=%u clear=%u", gw_gx_prim_count - gw_diag_seg_prim,
           gw_gx_dlist_count - gw_diag_seg_dlist, (unsigned)clear);
  }
  gw_diag_seg_prim = gw_gx_prim_count;
  gw_diag_seg_dlist = gw_gx_dlist_count;
  GXCopyTex(dest, (GXBool)clear);
}

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

/* TEMP DIAG (remove before ship): log matrix-index attribute setup so a live run can confirm
 * whether PNMTXIDX/TEXnMTXIDX are used as INDEX8/16 (the suspect path) vs DIRECT/NONE. */
static void gw_diag_mtxidx(const char *where, u32 attr, u32 a, u32 b) {
  static int count;
  if (attr <= GX_VA_TEX7MTXIDX) {
    if (count < 40) {
      gw_log("gw: DIAG %s mtxidx attr=%u a=%u b=%u", where, attr, a, b);
      ++count;
    }
  }
}

void gw_GXSetVtxDesc(u32 attr, u32 type) {
  gw_diag_mtxidx("vtxdesc", attr, type, 0);
  GXSetVtxDesc((GXAttr)attr, (GXAttrType)type);
}

void gw_GXSetVtxAttrFmt(u32 vtxfmt, u32 attr, u32 cnt, u32 type, u8 frac) {
  static int fmtcount;
  if (attr == GX_VA_POS && fmtcount < 20) {
    gw_log("gw: DIAG vtxattrfmt POS vtxfmt=%u cnt=%u comptype=%u frac=%u", vtxfmt, cnt, type, frac);
    ++fmtcount;
  }
  GXSetVtxAttrFmt((GXVtxFmt)vtxfmt, (GXAttr)attr, (GXCompCnt)cnt, (GXCompType)type, frac);
}

/* Melee calls the 3-argument GameCube form. Aurora bounds-checks indexed loads against the byte
 * length of the array, so give it everything from the base to the end of MEM1 when the array lives
 * there, and a value large enough to permit any index otherwise (e.g. arrays in static data). The
 * data is big-endian GC data, so le=false. */
void gw_GXSetArray(u32 attr, const void *base, u8 stride) {
  u32 size = 0xFFFFFFFFu;
  const unsigned char *p = (const unsigned char *)base;
  gw_diag_mtxidx("setarray", attr, (u32)(uintptr_t)base, stride);
  if (gw_mem1 != NULL && p >= gw_mem1 && p < gw_mem1 + gw_mem1_size) {
    size = (u32)(gw_mem1 + gw_mem1_size - p);
  }
  GXSetArray((GXAttr)attr, base, size, stride, false);
}

/* ---- transform ---------------------------------------------------------------------------- */

/* TEMP DIAG (remove before ship): dump the first projection + position matrices so a live run
 * can tell a sane transform from garbage. Gated to a few calls to avoid log spam. */
static void gw_diag_dump_mtx(const char *tag, const f32 *m, int n) {
  if ((gw_gx_copydisp_count % 30u) != 0u) {
    return;
  }
  if (n == 16) {
    gw_log("gw: DIAG %s %.3f %.3f %.3f %.3f | %.3f %.3f %.3f %.3f | %.3f %.3f %.3f %.3f | %.3f %.3f %.3f %.3f",
           tag, m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8], m[9], m[10], m[11], m[12], m[13], m[14],
           m[15]);
  } else {
    gw_log("gw: DIAG %s %.3f %.3f %.3f %.3f | %.3f %.3f %.3f %.3f | %.3f %.3f %.3f %.3f", tag, m[0], m[1], m[2],
           m[3], m[4], m[5], m[6], m[7], m[8], m[9], m[10], m[11]);
  }
}

void gw_GXSetProjection(const void *mtx, u32 type) {
  f32 native[4][4];
  gw_read_mtx44(native, mtx);
  gw_diag_dump_mtx("proj", &native[0][0], 16);
  GXSetProjection(native, (GXProjectionType)type);
}

void gw_GXLoadPosMtxImm(const void *mtx, u32 id) {
  f32 native[3][4];
  gw_read_mtx(native, mtx);
  /* TEMP DIAG: log every matrix slot, not just PNMTX0. In-game HSD loads its object matrices
   * into PNMTX1..9 and selects between them with per-vertex PNMTXIDX, so gating on id==0 saw
   * only the boot-time menu matrix and missed the gameplay transform path entirely. Cap at 4
   * slots per sampled frame to bound the volume. */
  /* TEMP DIAG: health of every position matrix this frame, not just the logged ones. If
   * animation is producing NaN or exploded transforms, every vertex leaves clip space and
   * nothing rasterises -- and pausing, which freezes animation, would restore the last good
   * values. That is exactly the pause/unpause split the depth grid measures. */
  {
    int e;
    ++gw_diag_mtx_calls;
    for (e = 0; e < 12; ++e) {
      const float v = native[e / 4][e % 4];
      if (isnan(v) || isinf(v)) {
        ++gw_diag_mtx_bad;
        continue;
      }
      if (fabsf(v) > gw_diag_mtx_maxabs) {
        gw_diag_mtx_maxabs = fabsf(v);
      }
      if (fabsf(v) > 1e6f) {
        ++gw_diag_mtx_huge;
      }
    }
    for (e = 0; e < 3; ++e) {
      const float a = native[e][0], b = native[e][1], c = native[e][2];
      const float len = sqrtf(a * a + b * b + c * c);
      if (!isnan(len)) {
        if (len < gw_diag_mtx_rowlen_min) {
          gw_diag_mtx_rowlen_min = len;
        }
        if (len > gw_diag_mtx_rowlen_max) {
          gw_diag_mtx_rowlen_max = len;
        }
      }
    }
  }

  if (gw_diag_mtx_slots_this_frame < 4u) {
    char tag[16];
    snprintf(tag, sizeof tag, "posmtx%u", (unsigned)id);
    gw_diag_dump_mtx(tag, &native[0][0], 12);
    ++gw_diag_mtx_slots_this_frame;
  }
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

void gw_GXSetColorUpdate(u8 enable) {
  gw_diag_colorupd = enable; /* TEMP DIAG */
  GXSetColorUpdate((GXBool)enable);
}
void gw_GXSetAlphaUpdate(u8 enable) {
  gw_diag_alphaupd = enable; /* TEMP DIAG */
  GXSetAlphaUpdate((GXBool)enable);
}

void gw_GXSetZMode(u8 compare_enable, u32 func, u8 update_enable) {
  gw_diag_zcmp = compare_enable; /* TEMP DIAG */
  gw_diag_zupd = update_enable;  /* TEMP DIAG */
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
  gw_diag_clearclr = clear_clr; /* TEMP DIAG */
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
