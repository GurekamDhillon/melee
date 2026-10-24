/* Include the real TUs so that static helpers are tested without copying their bodies. */
#include "../../../pc/gameworld/gekko_fp.c"
#include "../../../pc/gameworld/mtx_pc.c"

double shard_bits_to_double(u64_t x) { return gekko_bits_to_double(x); }
u64_t shard_double_to_bits(double x) { return gekko_double_to_bits(x); }
double shard_make_quiet(double x) { return gekko_make_quiet(x); }
double shard_force25(double x) { return gekko_force25(x); }
double shard_infinity(int x) { return gekko_infinity(x); }
double shard_nan(void) { return gekko_nan(); }
float shard_fmadds(float a, float c, float b) { return gk_fmadds(a, c, b); }
float shard_madd(float a, float c, float b) { return mtx_madd(a, c, b); }
float shard_reduce(float x, int* q) { return gk_sincos_reduce(x, q); }
float shard_even(float x) { return gk_sincos_even(x); }
float shard_odd(float x) { return gk_sincos_odd_head(x); }
void shard_rot(Mtx m, char axis, float s, float c) { mtx_rot_trig(m, axis, s, c); }
