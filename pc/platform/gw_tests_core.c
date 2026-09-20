#include <stdlib.h>

#include "gw.h"
#include "gw_test.h"

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
}
