/* Pure contract check in the real headless suite; no GPU/device required. */
extern "C" {
#include "gw_test.h"
}
#define GW_SHADER_ENGINE_TEST
#include "../tests/shader_contract_test.cpp"
#undef GW_SHADER_ENGINE_TEST
extern "C" void gw_shader_tests_register(void) {
  gw_test_register("shader_contract", shader_contract_test);
}
