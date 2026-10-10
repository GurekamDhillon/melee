/* Draw-only workbench controls. Capture state lives in gw_script.c, never in
 * the game snapshot. Only scalar values cross the PPC/native boundary. */
#ifndef PC_WORKBENCH_RENDER_H
#define PC_WORKBENCH_RENDER_H
enum {
    WB_RENDER_MODEL, WB_RENDER_HITBOXES, WB_RENDER_HURTBOXES, WB_RENDER_ECB,
    WB_RENDER_BONES, WB_RENDER_FLOOR, WB_RENDER_HUD, WB_RENDER_STAGE,
    WB_RENDER_OPTIONS
};
#if defined(TARGET_PC)
int Wb_RenderActive(void);
int Wb_RenderOption(int option, int fallback);
int Wb_RenderCameraBits(int field);
void Wb_RenderDrawn(void);
#endif
#endif
