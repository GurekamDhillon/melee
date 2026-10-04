#include "../../extern/aurora/include/aurora/pipeline_warm.hpp"
#include <cassert>
int main() {
  aurora::gfx::PipelineWarmRegistry r;
  auto a = r.begin(); assert(a && !r.begin());
  r.record(7); r.record(7); r.record(8);
  assert(r.pending(a, [](auto){return true;}) == -2);
  r.end(); assert(!r.capturing());
  assert(r.pending(a, [](auto){return false;}) == 2);
  assert(r.pending(a, [](auto x){return x == 7;}) == 1);
  assert(r.pending(a, [](auto){return true;}) == 0);
  r.release(a); assert(r.pending(a, [](auto){return true;}) == -1);
  auto b=r.begin(); assert(b != a); r.clear();
  assert(r.pending(b, [](auto){return true;}) == -1);
}
