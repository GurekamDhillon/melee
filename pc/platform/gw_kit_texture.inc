/* One GPU owner per kit slot. Disc-art reuse changes the generation, so overwrite that
 * owner's pixels (Aurora releases the old texture/view when dimensions change). */
ImTextureID g_kit_tex[GW_KIT_TEX_MAX];
bool g_kit_tex_up[GW_KIT_TEX_MAX];
int g_kit_tex_gen[GW_KIT_TEX_MAX];
unsigned g_kit_tex_updates;

ImTextureID kit_texture(int tex) {
  if (tex < 0 || tex >= GW_KIT_TEX_MAX) {
    return ImTextureID{};
  }
  const int gen = gw_Kit_TexGeneration(tex);
  if (!g_kit_tex_up[tex] || g_kit_tex_gen[tex] != gen) {
    int w = 0, h = 0;
    const uint8_t *px = gw_Kit_TexPixels(tex);
    if (px == nullptr || !gw_Kit_TexInfo(tex, &w, &h, nullptr, nullptr, nullptr, nullptr))
      return ImTextureID{}; /* Keep ownership across a pool drop, but never draw stale art. */
    ImTextureID id = g_kit_tex[tex]
        ? aurora_imgui_update_texture(g_kit_tex[tex], (uint32_t)w, (uint32_t)h, px)
        : aurora_imgui_add_texture((uint32_t)w, (uint32_t)h, px);
    if (!id) return ImTextureID{}; /* Retry next draw; a failed upload cannot bless this generation. */
    if (g_kit_tex[tex] && (++g_kit_tex_updates == 1 || (g_kit_tex_updates & 63) == 0))
      gw_log("ui: disc art: updated texture slot %d generation %d (%dx%d), updates %u", tex, gen, w, h, g_kit_tex_updates);
    g_kit_tex[tex] = id;
    g_kit_tex_gen[tex] = gen;
    g_kit_tex_up[tex] = true;
  }
  return g_kit_tex[tex];
}
