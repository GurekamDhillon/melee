/* Shipped preset data. Native SDL indices are SDL_GamepadButton positions;
 * adapter indices follow gc_labels. -1 means keep the original device layout.
 * These are starting points, not claims of a tournament standard. */
static const struct {
    const char *name, *description;
    int count;
    struct { int adapter, sdl, target; } binding[4];
} ctl_presets[] = {
    { "Default", "Your controller's original layout.", 0, {{-1,-1,0}} },
    { "Shoulder Jump", "Left shoulder jumps; X and Y still jump.", 1, {{6,9,2}} },
    { "Face Friendly", "South attacks, east specials, west and north jump.", 4,
        {{-1,0,0},{-1,1,1},{-1,2,2},{-1,3,3}} }
};
