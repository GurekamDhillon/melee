/* gd-ui-v1 / gd-lab-v1 shared constants (docs/rust-port/notes/wasm-ui-lab-api.md).
 * Rust mirror: rust/melee_modhost/src/pres.rs. Interface stubs only; the gd_ui / gd_lab imports are not linked yet.
 * Layer ids, gd_match_setup and gd_lab_fighter_v1 are added by the UI and Lab halves. */
#ifndef GD_UI_LAB_V1_H
#define GD_UI_LAB_V1_H

/* ABI versions (major<<16 | minor). A mod that declares any UI (LAB) feature bit, required or optional, exports
 * the i32 global gdmod_ui_abi (gdmod_lab_abi) whose VALUE is the version: major must equal the host's, minor may
 * be lower. */
#define GD_UI_ABI_VERSION  (1u << 16)
#define GD_LAB_ABI_VERSION (1u << 16)

/* features bits, extending GD_FEATURE_* in gdmod.h */
enum {
    GD_FEATURE_UI_DRAW      = 1u << 4,
    GD_FEATURE_UI_MENU      = 1u << 5,
    GD_FEATURE_UI_INPUT     = 1u << 6,
    GD_FEATURE_UI_SCENE     = 1u << 7,
    GD_FEATURE_UI_SIM_INPUT = 1u << 8,
    GD_FEATURE_UI_HTML      = 1u << 9,  /* reserved, refused by v1 */
    GD_FEATURE_LAB          = 1u << 10,
    GD_FEATURE_LAB_STATE    = 1u << 11,
    GD_FEATURE_LAB_INPUT    = 1u << 12,
    GD_FEATURE_LAB_CPU      = 1u << 13,
    GD_FEATURE_LAB_OVERLAY  = 1u << 14,
    GD_FEATURE_UI_VECTOR    = 1u << 15
};

/* results: 0 ok, negative = error */
enum {
    GD_OK       = 0,
    GD_EINVAL   = -1,
    GD_ECAP     = -2,
    GD_EOFFLINE = -3, /* not allowed in this session (normal/ranked netplay, replay, synctest) */
    GD_ESTATE   = -4,
    GD_EFULL    = -5
};

#endif
