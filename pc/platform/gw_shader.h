/* Host visual state only. No game-memory writes or snapshot fields. */
#ifndef GW_SHADER_H
#define GW_SHADER_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct { uint32_t compiles, cache_hits, errors, reloads, draws, passes, resolves, skipped;
                 uint64_t pixels, uniform_bytes, vertex_bytes; double record_ms; } GwShaderPerf;
int gw_Shader_Load(int owner, const char *root, const char *fragment, const char *vertex, int effect,
                   const char *params_json, char *error, int cap);
int gw_Shader_Set(int owner, int shader, const char *params_json, char *error, int cap);
int gw_Shader_Status(int owner, int shader, char *error, int cap);
int gw_Post_Add(int owner, int shader, int order, int stage, int half, int owns_shader, char *error, int cap);
int gw_Post_Set(int owner, int handle, const char *params_json, char *error, int cap);
int gw_Post_Ready(int owner, int handle);
int gw_Post_Remove(int owner, int handle);
/* Director-owned pass: survives script post_clear, until explicit removal. */
void gw_Post_Protect(int owner, int handle);
/* Internal stage director cover, final pass including retail HUD. Owns shader. */
int gw_Post_StageCover(int owner, int flash, char *error, int cap);
void gw_Post_Clear(int owner);
void gw_Shader_Release(int owner);
void gw_Shader_Perf(GwShaderPerf *out);
void gw_Shader_PostDraw(int stage);
void gw_Shader_Lights(int owner, const float *dir, const float *color, const float *ambient);
/* Asset-side opt in: material next to mesh; 0 means legacy, -1 means skip bad art. */
int gw_Shader_ModelLoad(const char *root, const char *mesh_path, const char *atlas_path,
                        const char *glow_path, char *error, int cap);
void gw_Shader_ModelsReset(void);
int gw_Shader_ModelDraw(int material, const void *view_be, const float *local,
                        unsigned tint, int alpha, const unsigned char *mesh,
                        const float *batch, int batch_count);
/* Effect bodies use the existing fx bindings/layout plus a params group. */
int gw_Shader_FxLoad(const char *package_path, const char *fragment, const char *vertex,
                    const char *params_json);
int gw_Shader_FxOverride(int owner, const char *package, const char *emitter, int shader, char *error, int cap);
int gw_Shader_FxSelect(int package, int emitter, int fallback);
void gw_shader_tests_register(void);
#ifdef __cplusplus
}
#endif
#endif
