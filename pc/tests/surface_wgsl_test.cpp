#include "../../extern/aurora/lib/gx/gx.hpp"
#include "../../extern/aurora/lib/gx/surface.hpp"
#include <cassert>
#include <iostream>
#include <fstream>
namespace aurora {
AuroraConfig g_config{};
void log_internal(AuroraLogLevel, const char*, const char*, unsigned) noexcept {}
namespace gfx { uint32_t align_uniform(uint32_t n) { return (n + 255) & ~255u; } }
namespace gx { std::string baseline_source(const ShaderConfig&, uint32_t) noexcept; }
}
int main() {
    using namespace aurora;
    using namespace aurora::gx;
    ShaderConfig config{};
    config.tevStageCount = 1; // Real GX draws always have at least one TEV stage.
    config.attrs[GX_VA_POS].attrType = GX_DIRECT;
    config.attrs[GX_VA_POS].cnt = 3;
    config.attrs[GX_VA_POS].compType = GX_F32;
    for (bool palette : {false, true}) {
        config.pnPalette = palette;
        for (uint32_t normal : {UINT32_MAX, 1u}) {
            assert(build_shader_source(config, normal) == baseline_source(config, normal));
        }
    }
    config.pnPalette = false;
    const auto vanilla_key = xxh3_hash(config);
    config.surfaceProgram = surface::registry.add(
        "fn gd_surface(c: vec4f, s: GdSurfaceInput) -> vec4f { return c; }", "test");
    assert(config.surfaceProgram);
    assert(xxh3_hash(config) != vanilla_key);
    auto source = build_shader_source(config, UINT32_MAX);
    assert(source.find("gd_data: array<vec4f, 5>") != std::string::npos);
    assert(source.find("prev = gd_surface(prev,") != std::string::npos);
    assert(source.find("gd_normal") != std::string::npos);
    assert(source.find("struct GdSurfaceInput") < source.find("@fragment\nfn fs_main("));
    auto id1_key = xxh3_hash(config);
    config.surfaceProgram = surface::registry.add(
        "fn gd_surface(c: vec4f, s: GdSurfaceInput) -> vec4f { return c * 0.5; }", "test2");
    assert(xxh3_hash(config) != id1_key);
    source = build_shader_source(config, 1);
    std::ofstream("_build/tmp/surface-generated.wgsl") << source;
    assert(source.find("prev = gd_surface(prev,") < source.find("out.color = prev;"));
    config.attrs[GX_VA_NRM] = config.attrs[GX_VA_POS];
    config.attrs[GX_VA_NRM].offset = 12;
    for (const char* name : {"cel-outline", "rim-light", "dissolve"}) {
        std::ifstream f(std::string("melee/pc/scripts/examples/surface-shaders/shaders/") + name + ".wgsl");
        std::string body((std::istreambuf_iterator<char>(f)), std::istreambuf_iterator<char>());
        assert(!body.empty());
        config.surfaceProgram = surface::registry.add(body, name);
        std::ofstream(std::string("_build/tmp/generated-") + name + ".wgsl") << build_shader_source(config, 1);
    }
    std::cout << "actual WGSL: 4 baseline byte comparisons, surface splice, cache keys: PASS\n";
    return 0;
}
