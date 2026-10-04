/* Shared presentation maths: authored kit coordinates stay centred at x=320.
 * No renderer, host state, or game-memory accesses here. */
#ifndef GW_VIEW_MATH_H
#define GW_VIEW_MATH_H
/* -1 means no window dimensions yet; caller may use the desktop. */
static inline int gw_view_default_wide(int w, int h)
{
    return w > 0 && h > 0 ? (double) w * 3 > (double) h * 4 : -1;
}
#define GW_VIEW_MIN_ASPECT (4.0f / 3.0f)
#define GW_VIEW_MAX_ASPECT (32.0f / 9.0f)
/* GameSceneKind: matches and the kit frontend only; authored native UI stays 4:3. */
static inline int gw_view_scene_wide(int scene)
{ return scene == 2 || scene == 3 || scene == 4 || scene == 44 || scene == 46; }
static inline float gw_view_projection_scale(float aspect, int wide)
{ return wide ? aspect * (60.0f / 73.0f) : 1.0f; }
static inline void gw_view_rectangle(int w, int h, float aspect,
                                    int *x, int *y, int *rw, int *rh)
{
    *rw = w; *rh = h;
    if (w > 0 && h > 0 && aspect > 0.0f) {
        if ((double)w > (double)h * aspect) *rw = (int)(h * aspect + 0.5f);
        else *rh = (int)(w / aspect + 0.5f);
    }
    *x = (w - *rw) / 2; *y = (h - *rh) / 2;
}
static inline float gw_view_aspect(float w, float h, int wide)
{
    float aspect = wide && w > 0.0f && h > 0.0f ? w / h : GW_VIEW_MIN_ASPECT;
    if (aspect < GW_VIEW_MIN_ASPECT) aspect = GW_VIEW_MIN_ASPECT;
    if (aspect > GW_VIEW_MAX_ASPECT) aspect = GW_VIEW_MAX_ASPECT;
    return aspect;
}
static inline float gw_view_left(float width) { return (640.0f - width) * 0.5f; }
static inline void gw_view_map(float w, float h, float width,
                               float *scale, float *ox, float *oy)
{
    float sx = w / width, sy = h / 480.0f;
    *scale = sx < sy ? sx : sy;
    if (*scale <= 0.0f) *scale = 1.0f;
    *ox = (w - width * *scale) * 0.5f;
    *oy = (h - 480.0f * *scale) * 0.5f;
}
#endif
