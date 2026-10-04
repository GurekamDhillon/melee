#ifndef MELEE_IF_HUD_MATH_H
#define MELEE_IF_HUD_MATH_H
#if defined(TARGET_PC)
/* Fit six authored group centres into the four-player centre span.
 * Percent digits already use the engine's 0.65 scale for >=5 players. */
static inline float ifHUD_FitCentre(float x, float source_left,
                                   float source_right, float target_left,
                                   float target_right)
{
    if (source_right <= source_left) return x;
    return target_left + (x - source_left) *
           (target_right - target_left) / (source_right - source_left);
}
#endif
#endif
