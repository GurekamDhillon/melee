#pragma once
// GD motion v1, presentation-only. All functions except callback() run on the GX recording thread.
#include <cstdint>
#include <memory>
#include <array>
#include <vector>
#define AURORA_GD_MOTION_VERSION 2
#define GX_AURORA_MOTION_CALLBACK 0x0047
extern "C" void GXAuroraMotionCallback(void (*fn)(const void*,uint32_t),const void*,uint32_t);
namespace aurora::gx::motion {
struct Snapshot;
using Pose=std::shared_ptr<const Snapshot>;
// A failed/over-budget capture returns null, never a partial fighter.
void begin(uint32_t surface,int additive,bool warming,bool textureless=false);
Pose end();
bool active();
size_t bytes(const Pose&);
size_t draws(const Pose&);
// Delta maps captured eye coordinates into current eye coordinates. Params are the
// existing surface system's 20 floats; callers reserve params[4].a for copy fade.
bool replay(const Pose&,const std::array<float,12>& delta,const float* params20);
bool replay_group(const std::vector<Pose>&,const std::array<float,12>& delta,const float* params20);
// GX-only scope for an extra descriptor traversal: retain but do not draw live.
void capture_only(bool);
bool suppress_live();
enum class Failure : unsigned { Pipeline,VertexArena,IndexArena,UniformArena,
    CopyTexture,IndexArray,Palette,Fog,PoseBytes,DrawCount,FrameArena,
    ResidentBytes,EmptyPose,PipelinePending,ReplayArena,Geometry,Count };
enum class Fallback : unsigned { NoFog,Textureless,CopySilhouette,PaletteWarm,HeldDraws,HeldGeometry,Count };
inline constexpr size_t FailureCount=static_cast<size_t>(Failure::Count);
inline constexpr size_t FallbackCount=static_cast<size_t>(Fallback::Count);
const char* failure_name(Failure);
void diagnostics(uint64_t* failures,uint64_t* fallbacks,uint64_t* warm_variants);
Failure last_failure();
void rejected(Failure);
void held_draws();
bool contract_test();
void clear_warm();
bool replaying();
void projection(float* out16);
bool arena_room(size_t vertices,size_t indices,size_t uniforms,size_t storage);
}
