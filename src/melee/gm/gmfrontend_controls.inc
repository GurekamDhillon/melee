/* Controller-only remapping; included by gmfrontend_settings.inc. Reuses kit
 * rows, the existing fitted description/toast strip and scroll cues. */
extern void Controls_Menu(int editor);
extern int Controls_Port(void);
extern void Controls_Select(int port);
extern int Controls_Connected(int port);
extern int Controls_Profile(void);
extern void Controls_SetProfile(int profile);
extern void Controls_ProfileName(char* out, int cap);
extern void Controls_ProfileAction(int action);
extern int Controls_Option(int what);
extern void Controls_SetOption(int what, int value);
extern void Controls_Preset(int preset);
extern void Controls_Capture(int target, int also);
extern void Controls_Cancel(void);
extern int Controls_Capturing(void);
extern int Controls_Result(void);
extern int Controls_Held(void);
extern void Controls_Binding(int target, char* out, int cap);
extern void Controls_Tester(char* out, int cap);
extern int Controls_Assign(int port);
static const FrontendScreen fe_screen_remap;
static int fcr_port, fcr_target, fcr_also, fcr_preset, fcr_assign;
static int fcr_result, fcr_message_frames, fcr_wait_release;
static const char* const fcr_ports[] = { "Port 1", "Port 2", "Port 3", "Port 4" };
static const char* const fcr_targets[] = {
    "A", "B", "X", "Y", "Z", "START", "L click", "R click",
    "D-pad Up", "D-pad Down", "D-pad Left", "D-pad Right",
    "Stick Right", "Stick Left", "Stick Up", "Stick Down",
    "C-stick Right", "C-stick Left", "C-stick Up", "C-stick Down", "Analog L", "Analog R"
};
static const char* const fcr_conflicts[] = { "Swap", "Also" };
static const char* const fcr_presets[] = { "Default", "Shoulder Jump", "Face Friendly" };
static const char* const fcr_shields[] = { "Original", "Light", "Full" };
static int fcr_ready(void) { return Controls_Connected(fcr_port); }
static int fcr_get_port(void) { return fcr_port; }
static void fcr_set_port(int p) { fcr_port = p; Controls_Select(p); }
static int fcr_get_target(void) { return fcr_target; }
static void fcr_set_target(int t) { fcr_target = t; }
static int fcr_get_also(void) { return fcr_also; }
static void fcr_set_also(int v) { fcr_also = v; }
static void fcr_bind(void) { Controls_Capture(fcr_target, fcr_also); }
static void fcr_pick(void) { Controls_Capture(-1, 0); }
static void fcr_fmt_binding(int v, char* out) { (void)v; Controls_Binding(fcr_target, out, FE_STR); }
static void fcr_fmt_tester(int v, char* out) { (void)v; Controls_Tester(out, FE_STR); }
static void fcr_fmt_profile(int v, char* out) { (void)v; Controls_ProfileName(out, FE_STR); }
static void fcr_new(void) { Controls_ProfileAction(0); }
static void fcr_rename(void) { Controls_ProfileAction(1); }
static void fcr_delete(void) { Controls_ProfileAction(2); }
static void fcr_reset(void) { Controls_ProfileAction(3); }
static int fcr_get_preset(void) { return fcr_preset; }
static void fcr_set_preset(int v) { fcr_preset = v; }
static void fcr_apply_preset(void) { Controls_Preset(fcr_preset); }
static int fcr_get_assign(void) { return fcr_assign; }
static void fcr_set_assign(int v) { fcr_assign = v; }
static void fcr_assign_profile(void) {
    fe_ol_notice(Controls_Assign(fcr_assign) ? "Profile assigned." : "Connect the same kind of controller first.", false);
}
#define FCR_OPTION(name, index) \
    static int fcr_get_##name(void) { return Controls_Option(index); } \
    static void fcr_set_##name(int v) { Controls_SetOption(index, v); }
FCR_OPTION(swap, 0)
FCR_OPTION(analog, 1)
FCR_OPTION(main_dz, 3)
FCR_OPTION(c_dz, 4)
FCR_OPTION(rumble, 5)
#undef FCR_OPTION
static int fcr_get_shield(void) {
    int v = Controls_Option(2); return v == 0 ? 0 : v < 140 ? 1 : 2;
}
static void fcr_set_shield(int v) { Controls_SetOption(2, v == 0 ? 0 : v == 1 ? 80 : 140); }
static void fcr_fmt_dz(int v, char* out) {
    if (v < 0) sprintf(out, "%s", "Inherit calibration"); else sprintf(out, "%d%%", v);
}
static void fcr_back(void);
static const FrontendItem fe_items_remap[] = {
    { FE_CHOICE, 0, "Controller", "Choose a connected port, or use Pick Controller below.", fcr_get_port, fcr_set_port, 0, 3, 1, fcr_ports },
    { FE_ACTION, FE_DO_CALL, "Pick Controller", "Press any input on the controller you want. START+B cancels.", NULL, NULL, 0, 0, 0, NULL, NULL, NULL, fcr_pick },
    { FE_CHOICE, 0, "Game Input", "Choose what the game should receive, then choose Bind Input.", fcr_get_target, fcr_set_target, 0, 21, 1, fcr_targets },
    { FE_ACTION, FE_DO_CALL, "Bind Input", "Press a button, tilt a stick, or pull a trigger. START+B cancels; 8 second timeout.", NULL, NULL, 0, 0, 0, NULL, fcr_fmt_binding, NULL, fcr_bind, fcr_ready, "Connect a controller to this port first." },
    { FE_CHOICE, 0, "Conflicts", "Swap exchanges bindings. Also keeps both: several jump buttons, or one button for two inputs.", fcr_get_also, fcr_set_also, 0, 1, 1, fcr_conflicts },
    { FE_SLIDER, 0, "Game Receives", "Live mapped inputs. Menu navigation always uses the original layout here.", fsc_zero, NULL, 0, 0, 0, NULL, fcr_fmt_tester },
    { FE_CHOICE, 0, "Profile", "Left/Right chooses one of four profiles for this device.", Controls_Profile, Controls_SetProfile, 0, 3, 1, NULL, fcr_fmt_profile },
    { FE_ACTION, FE_DO_CALL, "New Profile", "Copy this profile into the first empty slot (four slots per device).", NULL, NULL, 0, 0, 0, NULL, NULL, NULL, fcr_new, fcr_ready, "Connect a controller to this port first." },
    { FE_ACTION, FE_DO_CALL, "Rename Profile", "Cycle names: Default, Main, Practice, Tournament, Friends.", NULL, NULL, 0, 0, 0, NULL, NULL, NULL, fcr_rename, fcr_ready, "Connect a controller to this port first." },
    { FE_ACTION, FE_DO_CALL, "Delete Profile", "Delete the current profile and return to slot 1's saved layout.", NULL, NULL, 0, 0, 0, NULL, NULL, NULL, fcr_delete, fcr_ready, "Connect a controller to this port first." },
    { FE_CHOICE, 0, "Preset", "Default: original. Shoulder Jump: left shoulder jumps. Face Friendly: south/east attack/special.", fcr_get_preset, fcr_set_preset, 0, 2, 1, fcr_presets, NULL, NULL, fcr_apply_preset },
    { FE_TOGGLE, 0, "Swap Sticks", "Exchange movement and C-stick attacks. C-stick never becomes a camera.", fcr_get_swap, fcr_set_swap, 0, 1, 1 },
    { FE_TOGGLE, 0, "Trigger Analog Off", "Ignore continuous trigger pulls; digital shield bindings remain.", fcr_get_analog, fcr_set_analog, 0, 1, 1 },
    { FE_CHOICE, 0, "Digital Shield", "Light sends analog 80 without a hard-shield click. Full sends analog 140 and the click.", fcr_get_shield, fcr_set_shield, 0, 2, 1, fcr_shields },
    { FE_SLIDER, 0, "Main Dead Zone", "Inherit uses existing SDL deadzone or GameCube calibration. Percent of full travel.", fcr_get_main_dz, fcr_set_main_dz, -1, 50, 1, NULL, fcr_fmt_dz },
    { FE_SLIDER, 0, "C-stick Dead Zone", "Independent C-stick threshold; inherit uses the existing controller setting.", fcr_get_c_dz, fcr_set_c_dz, -1, 50, 1, NULL, fcr_fmt_dz },
    { FE_TOGGLE, 0, "Rumble (this profile)", "Allow rumble when this controller supports it. CONTROLS has a Rumble row per game port too.", fcr_get_rumble, fcr_set_rumble, 0, 1, 1 },
    { FE_CHOICE, 0, "Assign to Port", "A copies this profile to a connected controller of the same kind on this port.", fcr_get_assign, fcr_set_assign, 0, 3, 1, fcr_ports, NULL, NULL, fcr_assign_profile },
    { FE_ACTION, FE_DO_CALL, "Reset to Default", "Restore this profile. Anywhere in kit menus, hold original START+B for 2 seconds.", NULL, NULL, 0, 0, 0, NULL, NULL, NULL, fcr_reset, fcr_ready, "Connect a controller to this port first." },
    { FE_ACTION, FE_DO_CALL, "Done", "Return to controls.", NULL, NULL, 0, 0, 0, NULL, NULL, NULL, fcr_back },
};
static const FrontendScreen fe_screen_remap = {
    "SETTINGS > CONTROLS", "REMAP CONTROLLER", fe_items_remap,
    sizeof fe_items_remap / sizeof fe_items_remap[0]
};
static void fcr_open(void) {
    int c;
    for (c = 0; c < 4 && !Controls_Connected(c); ++c) {}
    fcr_port = c < 4 ? c : 0;
    Controls_Menu(fcr_port); Controls_Select(fcr_port);
    fcr_result = Controls_Result(); fcr_message_frames = 0; fcr_wait_release = 0;
    fe_switch_screen(&fe_screen_remap);
    OSReport("frontend: controller remapping\n");
}
/* Returns true while capturing: ordinary menu navigation is frozen, but the
 * default-layout START+B chord cancels. Pressing B alone can itself be bound. */
static bool fcr_frame(void) {
    static const char* const messages[] = {
        "Original layout navigates; START+B restores.",
        "Press and release an input. START+B cancels. 8 seconds.", "Saved.", "Timed out; not changed.",
        "Cancelled; not changed.", "Defaults restored.", "Profile copied.", "All four profile slots are full.",
        "Conflict: bindings swapped.", "Conflict: both bindings kept.",
        "Could not save. Check settings storage; this change may be temporary."
    };
    int result = Controls_Result();
    char tester[FE_STR], line[FE_HELP_STR];
    fcr_port = Controls_Port();
    Controls_Menu(fcr_port);
    if (Controls_Capturing() && (Pad_Value(fcr_port, 0) & 0x1200) == 0x1200) Controls_Cancel();
    if (result != fcr_result) {
        fcr_result = result; fcr_message_frames = 120;
        if (result != 1) fcr_wait_release = 1;
    }
    if (fcr_wait_release && !Controls_Held()) fcr_wait_release = 0;
    if (fcr_message_frames > 0) --fcr_message_frames;
    Controls_Tester(tester, sizeof tester);
    sprintf(line, "%s | %s", tester, messages[Controls_Capturing() ? 1 :
        fcr_message_frames && result >= 0 && result < 11 ? result : 0]);
    fe_ol_notice(line, false);
    return Controls_Capturing() != 0 || fcr_wait_release != 0;
}
