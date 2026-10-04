/* Background models have an explicit effect bucket. Ordinary geometry is drawn
 * ONCE per frame, in the first main-camera link-3 walk (camera mode 2), before
 * fighters, items and effects. fn_800301D0 (camera.c) walks link 3 four times:
 * modes 2, 1, 0 and then mode 3 after fighters/items/link 7. Native stage
 * pieces filter on their own mode (grDisplay_801C5DB0: b012 == mode); the old
 * "always true" drew opaque kit parts again in mode 3, after the effects.
 * Shield bubbles, lasers, dust and glow quads do not write depth, so a part
 * behind them overwrote them. Drawn first, every effect depth-tests against
 * the kit exactly as against native stage geometry. */
static int script_stage_render_pass(int camera_mode, int code)
{
    return (camera_mode==2 && (code==0 || code==2)) || (camera_mode==3 && code==2);
}
/* The ordinary fighter pass traverses links4,5,6 after refraction. Background
 * must precede item link6: transparent laser materials need not write depth. */
enum { SCRIPT_BACKGROUND_GX_LINK = 4 };
static int script_background_render_pass(int code, int main_camera, int fighter_phase)
{
    return code==0 && main_camera && fighter_phase==0;
}
