#include <stdlib.h>
#include <string.h>

#include "gw.h"
#include "gw_test.h"

int gw_Mex_CssIconCount(void);
void *gw_Mex_CssIconTable(void);
int gw_Mex_InternalCount(void);
int gw_Mex_InternalForExt(int ext);
int gw_Mex_FtCostumeCount(int internal);

static int test_u32_roundtrip(void) {
  unsigned char buf[8];
  gw_w32(buf, 0x12345678u);
  if (gw_r32(buf) != 0x12345678u) {
    gw_test_fail("gw_r32(gw_w32(12345678)) = %08X", (unsigned)gw_r32(buf));
    return 1;
  }
  return 0;
}

static int test_w32_is_big_endian(void) {
  unsigned char buf[4];
  gw_w32(buf, 0x01020304u);
  if (buf[0] != 0x01 || buf[1] != 0x02 || buf[2] != 0x03 || buf[3] != 0x04) {
    gw_test_fail("gw_w32 wrote %02X %02X %02X %02X", buf[0], buf[1], buf[2], buf[3]);
    return 1;
  }
  return 0;
}

static int test_u16_roundtrip(void) {
  unsigned char buf[4];
  gw_w16(buf, 0xABCDu);
  if (buf[0] != 0xAB || buf[1] != 0xCD || gw_r16(buf) != 0xABCDu) {
    gw_test_fail("gw_w16(ABCD) -> %02X %02X", buf[0], buf[1]);
    return 1;
  }
  return 0;
}

static int test_f32_roundtrip(void) {
  unsigned char buf[8];
  const float f = 3.5f;
  gw_wf32(buf, f);
  if (gw_rf32(buf) != f) {
    gw_test_fail("gw_rf32(gw_wf32(3.5)) = %f", (double)gw_rf32(buf));
    return 1;
  }
  return 0;
}

static int test_wf32_is_big_endian(void) {
  unsigned char buf[4];
  gw_wf32(buf, 1.0f);
  if (buf[0] != 0x3F || buf[1] != 0x80 || buf[2] != 0x00 || buf[3] != 0x00) {
    gw_test_fail("gw_wf32(1.0) wrote %02X %02X %02X %02X", buf[0], buf[1], buf[2], buf[3]);
    return 1;
  }
  return 0;
}

static int test_mem1_at_guest_base(void) {
  if (gw_mem1 == NULL) {
    gw_test_fail("gw_mem1 is NULL");
    return 1;
  }
  if ((uintptr_t)gw_mem1 != (uintptr_t)0x80000000u) {
    gw_test_fail("gw_mem1 = %p, expected 0x80000000", (void *)gw_mem1);
    return 1;
  }
  return 0;
}

static int test_mem1_aram_distinct(void) {
  if (gw_mem1 == NULL || gw_aram == NULL) {
    gw_test_fail("gw_mem1=%p gw_aram=%p", (void *)gw_mem1, (void *)gw_aram);
    return 1;
  }
  if (gw_mem1_size == 0 || gw_aram_size == 0) {
    gw_test_fail("sizes mem1=%u aram=%u", (unsigned)gw_mem1_size, (unsigned)gw_aram_size);
    return 1;
  }
  if ((unsigned char *)gw_mem1 == (unsigned char *)gw_aram) {
    gw_test_fail("MEM1 and ARAM alias the same buffer");
    return 1;
  }
  return 0;
}

extern int gw_Mex_Enabled(const char *name);

/* The whole m-ex port rests on "off unless explicitly requested". If this ever reported true for
 * an unrequested feature, every ported behaviour would be on by default and the promise that a
 * stock build is unchanged would be silently broken. */
static int test_mex_flag_unknown_is_off(void) {
  if (gw_Mex_Enabled("zzz_not_a_real_feature")) {
    gw_test_fail("unknown feature reported enabled");
    return 1;
  }
  return 0;
}

static int test_mex_flags_default_off(void) {
  static const char *const known[] = {"no_title_demo", "no_special_messages", "no_trophy_messages"};
  const char *env = getenv("MELEE_MEX");
  unsigned i;
  if (env != NULL && env[0] != '\0') {
    /* Something was requested, so "off" is the wrong expectation here; the enabling direction is
     * covered by mex_env_enables. The two tests are written to cover the two configurations. */
    return 0;
  }
  if (gw_Mex_Enabled(NULL)) {
    gw_test_fail("NULL feature name reported enabled");
    return 1;
  }
  for (i = 0; i < sizeof known / sizeof known[0]; ++i) {
    if (gw_Mex_Enabled(known[i])) {
      gw_test_fail("feature %s enabled without MELEE_MEX", known[i]);
      return 1;
    }
  }
  return 0;
}

/* The default-off tests above cannot prove the framework ever turns anything ON, so this asserts
 * the enabling direction. It only has teeth when MELEE_MEX is set at process start (see
 * run_tests_mex.bat); the flag list is read once and cached, so it cannot be toggled mid-run.
 * Without MELEE_MEX this deliberately returns pass, which is why it must be run both ways. */
static int test_mex_env_enables(void) {
  const char *env = getenv("MELEE_MEX");
  if (env == NULL || env[0] == '\0') {
    return 0;
  }
  if (!gw_Mex_Enabled(env)) {
    gw_test_fail("MELEE_MEX=%s was set but the flag reports disabled", env);
    return 1;
  }
  return 0;
}


/* ------------------------------------------------------------------ m-ex CSS portraits -----
 *
 * The character-select screen cannot be reached headless, so what CAN be checked here is the
 * DATA the portrait code indexes: mexSelectChr's single CSP material animation inside
 * MnSlChr, read straight off the mounted disc.
 *
 * What it pins, and why it is worth pinning: the port used to compute the portrait frame as
 * `external_id + costume * mexSelectChr.csp_stride`. Both m-ex discs say that is wrong. The
 * animation is laid out in m-ex INTERNAL kind order with each kind's costumes CONSECUTIVE, so
 * the frame is the cumulative costume index. Decoding individual frames out of ACE's MnSlChr
 * confirms it by eye - frame 0 is Mario, 30 is Kirby's third costume, 201 is Sonic, 324 is
 * Knuckles - and this test states the same thing in a form that fails if it ever drifts:
 * the costume counts in mexData, summed over the leading internal kinds, land exactly on the
 * animation's own image count (221 on Akaneia, 388 on ACE).
 *
 * It also records the number that made the bug LOOK like corruption rather than a mix-up:
 * ACE's animation has 388 CI8 frames with 388 separate palettes, and HSD_TObj::tlut_no was a
 * u8, so every frame past 255 drew the right pixels through some other character's palette.
 */

/* An HSD archive header, big-endian: file size, data size, reloc count, public count, extern
 * count, then 12 reserved bytes; data at 0x20. */
#define GW_HSD_HDR 0x20

static const unsigned char *gw_hsd_public(const unsigned char *ar, uint32_t size,
                                          const char *want, uint32_t *out_off) {
  uint32_t data_size, nb_reloc, nb_public, nb_extern, o_public, o_symbols, i;
  if (ar == NULL || size < GW_HSD_HDR) {
    return NULL;
  }
  data_size = gw_r32(ar + 0x04);
  nb_reloc = gw_r32(ar + 0x08);
  nb_public = gw_r32(ar + 0x0C);
  nb_extern = gw_r32(ar + 0x10);
  o_public = GW_HSD_HDR + data_size + nb_reloc * 4u;
  o_symbols = o_public + (nb_public + nb_extern) * 8u;
  if (o_symbols > size) {
    return NULL;
  }
  for (i = 0; i < nb_public; ++i) {
    const uint32_t off = gw_r32(ar + o_public + i * 8u);
    const uint32_t sym = gw_r32(ar + o_public + i * 8u + 4u);
    if (o_symbols + sym < size && strcmp((const char *)ar + o_symbols + sym, want) == 0) {
      if (out_off != NULL) {
        *out_off = off;
      }
      return ar + GW_HSD_HDR;
    }
  }
  return NULL;
}

static int test_mex_csp_frame_map(void) {
  static const char *const paths[] = { "/MnSlChr.usd", "/MnSlChr.dat" };
  unsigned char *ar = NULL;
  uint32_t size = 0, data_size = 0, sel = 0;
  const unsigned char *data;
  uint32_t matanim, texanim, imagetbl, tluttbl, stride;
  int n_img, n_lut, i, internal, total, kinds, n_internal, rc = 0;

  if (gw_Mex_CssIconCount() == 0) {
    return 0; /* a retail disc: no mexData, no m-ex CSS */
  }
  for (i = 0; i < 2 && ar == NULL; ++i) {
    ar = gw_DVDReadFileAlloc(paths[i], &size);
  }
  if (ar == NULL) {
    gw_test_fail("no MnSlChr on this disc");
    return 1;
  }
  data_size = gw_r32(ar + 0x04);
  data = gw_hsd_public(ar, size, "mexSelectChr", &sel);
  if (data == NULL) {
    /* An m-ex disc whose MnSlChr has no mexSelectChr runs the retail CSS; nothing to check. */
    gw_log("test mex_csp_frame_map: MnSlChr has no mexSelectChr - retail CSS path");
    free(ar);
    return 0;
  }
  matanim = gw_r32(data + sel + 0x0C);
  stride = gw_r32(data + sel + 0x10);
  if (matanim == 0u || matanim + 0x10u > data_size) {
    gw_test_fail("mexSelectChr CSP matanim offset %u is outside the data section", matanim);
    free(ar);
    return 1;
  }
  texanim = gw_r32(data + matanim + 0x08);
  if (texanim == 0u || texanim + 0x18u > data_size) {
    gw_test_fail("CSP matanim has no texanim (offset %u)", texanim);
    free(ar);
    return 1;
  }
  imagetbl = gw_r32(data + texanim + 0x0C);
  tluttbl = gw_r32(data + texanim + 0x10);
  n_img = (int)gw_r16(data + texanim + 0x14);
  n_lut = (int)gw_r16(data + texanim + 0x16);
  gw_log("test mex_csp_frame_map: %d CSP frames, %d TLUTs, csp_stride field %u",
         n_img, n_lut, stride);
  if (n_img <= 0 || n_img != n_lut) {
    gw_test_fail("CSP animation has %d images and %d TLUTs - the portrait code assumes one "
                 "palette per frame", n_img, n_lut);
    rc = 1;
  }

  /* Every frame is a CI8 image of one size with its own palette. That is the premise of the
   * whole layout: costumes of one fighter SHARE a pixel buffer and differ only by TLUT. */
  for (i = 0; rc == 0 && i < n_img; ++i) {
    uint32_t desc = gw_r32(data + imagetbl + (uint32_t)i * 4u);
    uint32_t lut = gw_r32(data + tluttbl + (uint32_t)i * 4u);
    if (desc == 0u || desc + 0x18u > data_size || lut == 0u || lut + 0x10u > data_size) {
      gw_test_fail("CSP frame %d has no image or no TLUT descriptor", i);
      rc = 1;
      break;
    }
    if (gw_r32(data + desc + 0x08) != 9u) { /* GX_TF_C8 */
      gw_test_fail("CSP frame %d is format %u, not CI8 - a non-paletted frame would not need "
                   "tlut_no at all", i, gw_r32(data + desc + 0x08));
      rc = 1;
    }
  }

  /* The mapping itself: cumulative costume counts over INTERNAL kinds must land exactly on
   * the frame count. `kinds` is how many leading kinds have portraits; the handful of
   * non-selectable kinds after them (Master Hand, the wireframes, Sandbag) have none. */
  n_internal = gw_Mex_InternalCount();
  total = 0;
  kinds = -1;
  for (i = 0; i < n_internal; ++i) {
    if (total == n_img) {
      kinds = i;
      break;
    }
    total += gw_Mex_FtCostumeCount(i);
  }
  if (kinds < 0 && total == n_img) {
    kinds = n_internal;
  }
  if (kinds < 0) {
    gw_test_fail("no prefix of the %d internal kinds' costume counts sums to the %d CSP "
                 "frames (total %d) - the portrait frame is not the cumulative costume index",
                 n_internal, n_img, total);
    rc = 1;
  } else {
    gw_log("test mex_csp_frame_map: internal kinds 0..%d carry all %d frames", kinds - 1, n_img);
  }

  /* Every fighter the CSS can actually select must fall inside that range, at every costume. */
  for (i = 0; rc == 0 && kinds >= 0 && i < gw_Mex_CssIconCount(); ++i) {
    int base = 0, j, count;
    int ext;
    const unsigned char *icons = (const unsigned char *)gw_Mex_CssIconTable();
    if (icons == NULL) {
      break;
    }
    ext = (int)icons[(uint32_t)i * 0x1Cu + 1u]; /* CSSIcon +0x01 is the m-ex external id */
    internal = gw_Mex_InternalForExt(ext);
    if (internal < 0 || internal >= kinds) {
      gw_test_fail("CSS icon %d (external %d) is internal kind %d, outside the %d kinds that "
                   "have portraits", i, ext, internal, kinds);
      rc = 1;
      break;
    }
    for (j = 0; j < internal; ++j) {
      base += gw_Mex_FtCostumeCount(j);
    }
    count = gw_Mex_FtCostumeCount(internal);
    if (count <= 0 || base + count > n_img) {
      gw_test_fail("internal kind %d has costumes %d..%d, past the %d CSP frames",
                   internal, base, base + count - 1, n_img);
      rc = 1;
      break;
    }
  }

  if (rc == 0 && n_img > 256) {
    gw_log("test mex_csp_frame_map: %d frames - frames 256..%d need a tlut_no wider than the "
           "retail u8 (see sysdolphin/baselib/tobj.h)", n_img, n_img - 1);
  }
  free(ar);
  return rc;
}


void gw_tests_register_all(void) {
  extern void gw_MexTestRegisterAll(void);
  extern void gw_ppc_tests_register(void);
  extern void gw_ftfunction_tests_register(void);
  extern void gw_mex_ftfunction_runtime_tests_register(void);
  extern void gw_mex_grfunction_tests_register(void);
  extern void gw_mex_graudio_tests_register(void);
  extern void gw_mex_sss_tests_register(void);
  extern void gw_scene_tests_register(void);
  gw_MexTestRegisterAll();
  gw_ppc_tests_register();
  gw_ftfunction_tests_register();
  gw_mex_ftfunction_runtime_tests_register();
  gw_mex_grfunction_tests_register();
  gw_mex_graudio_tests_register();
  gw_mex_sss_tests_register();
  gw_scene_tests_register();
  gw_test_register("mex_flag_unknown_is_off", test_mex_flag_unknown_is_off);
  gw_test_register("mex_flags_default_off", test_mex_flags_default_off);
  gw_test_register("mex_env_enables", test_mex_env_enables);
  gw_test_register("endian_u32_roundtrip", test_u32_roundtrip);
  gw_test_register("endian_w32_is_big_endian", test_w32_is_big_endian);
  gw_test_register("endian_u16_roundtrip", test_u16_roundtrip);
  gw_test_register("endian_f32_roundtrip", test_f32_roundtrip);
  gw_test_register("endian_wf32_is_big_endian", test_wf32_is_big_endian);
  gw_test_register("mem1_at_guest_base", test_mem1_at_guest_base);
  gw_test_register("mem1_aram_distinct", test_mem1_aram_distinct);
  gw_test_register("mex_csp_frame_map", test_mex_csp_frame_map);
}
