/* Exercise the real pool and overlay cache with a GPU-free Aurora stand-in. */
#include "../platform/gw_kit.h"
#include <cassert>
#include <cstdio>
#include <cstdint>
#include <vector>

using ImTextureID = uintptr_t;
static int adds, updates;
static int resizes;
static ImTextureID next_id;
struct Owner { ImTextureID id; uint32_t w, h; };
static std::vector<Owner> owners;
static bool fail_update;
static std::vector<uint8_t> uploaded;
static ImTextureID aurora_imgui_add_texture(uint32_t w, uint32_t h, const void *px) {
  ++adds;
  uploaded.assign((const uint8_t*)px, (const uint8_t*)px + w * h * 4);
  owners.push_back({++next_id, w, h});
  return next_id;
}
static ImTextureID aurora_imgui_update_texture(ImTextureID id, uint32_t w, uint32_t h, const void *px) {
  assert(id);
  ++updates;
  if (fail_update) return 0;
  uploaded.assign((const uint8_t*)px, (const uint8_t*)px + w * h * 4);
  for (auto& owner : owners) {
    if (owner.id == id) {
      if (owner.w != w || owner.h != h) {
        ++resizes;
        owner = {++next_id, w, h};
      }
      return owner.id;
    }
  }
  assert(false && "overlay retained an obsolete texture ID");
  return 0;
}
extern "C" void gw_log(const char*, ...);
extern "C" void kit_native_tests(void);
#include "../platform/gw_kit_texture.inc"

int main() {
  kit_native_tests();
  gw_Kit_TexDropHsd();
  uint8_t image[128] = {};
  char key[32];
  // More than 384 reassignments; slots and uploads must stay bounded at 192.
  for (int n = 0; n < 1200; ++n) {
    gw_Kit_TexHsdFrame(2000 + n * 2);
    std::snprintf(key, sizeof key, "reuse:%d", n);
    image[0] = uint8_t(n);
    int slot = gw_Kit_TexAddHsd(key, 1, image, sizeof image, n % 5 ? 8 : 16, 4, 0, nullptr, 0);
    assert(slot >= 0);
    int before = adds + updates;
    assert(kit_texture(slot));
    assert(adds + updates == before + 1);
    assert(uploaded[3] == image[0]);
    assert(kit_texture(slot));
    assert(adds + updates == before + 1); // Same generation uploads once.
    assert(gw_Kit_TexHsdCount() <= 192);
  }
  assert(adds == 192 && updates == 1008);
  assert(owners.size() == 192 && resizes > 0);
  // A dropped slot keeps its GPU owner, hides missing pixels, then updates on refill.
  gw_Kit_TexDropHsd();
  assert(!kit_texture(384));
  int slot = gw_Kit_TexAddHsd("after-drop", 1, image, sizeof image, 8, 4, 0, nullptr, 0);
  fail_update = true;
  assert(!kit_texture(slot)); // Failed update must not display the previous picture.
  fail_update = false;
  assert(kit_texture(slot)); // Nor poison the generation cache: retry succeeds.
  assert(adds == 192 && updates == 1010);
  assert(!kit_texture(-1) && !kit_texture(GW_KIT_TEX_MAX));
  gw_Kit_TexDropHsd();
  puts("PASS kit_texture_pool_reuse (192 adds, 1010 updates including failure/retry)");
}
