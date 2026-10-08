#ifndef GENO_PLAN_H
#define GENO_PLAN_H
/* The engine's flat reading of a `base: "none"` define's package (plan.json from tools/geno/build_courier.sh).
 * The host registry parses the JSON once (gw_Geno_DefinePlan) and the game side reads these plain arrays;
 * both compile this header, so the layout is one definition. No pointers, no disc data: original numbers only. */
#define GPL_MAX_JOINTS 256   /* slice 8: was 128; Sora has 175 */
#define GPL_MAX_OWN_ROWS 128 /* slice 8: rows a package declares past Mario's 303 (geno.json fighter.rows) */
#define GPL_MAX_PARTS 56
#define GPL_MAX_CLIPS 256
#define GPL_MAX_MOTIONS 351
#define GPL_MAX_ROWS 512
#define GPL_NONE 255
typedef struct GenoPlan {
    int ok;                                     /* parsed and consistent */
    int joints;                                 /* joint count (the model's, depth-first, TopN = 0) */
    int nparts;                                 /* part slots in part_to_joint (54) */
    unsigned char joint_to_part[GPL_MAX_JOINTS];/* GPL_NONE = the joint is no Melee part */
    unsigned char part_to_joint[GPL_MAX_PARTS]; /* GPL_NONE = the fighter lacks that part */
    int nclips;
    char clip_name[GPL_MAX_CLIPS][32];          /* "Wait" */
    char clip_symbol[GPL_MAX_CLIPS][72];        /* "PlyCourier_Share_ACTION_Wait_figatree" */
    unsigned clip_offset[GPL_MAX_CLIPS];        /* byte offset in the animation bank */
    unsigned clip_bytes[GPL_MAX_CLIPS];
    int clip_frames[GPL_MAX_CLIPS];
    int nmotions;
    int ft_x8[5], ft_x34[1], ft_x38[2], ft_x44[6], ft_x54[5], ft_x58[10]; /* ftData joint-index fields, by role (plan "ftdata"), -1 = none */
    int nhurt;                                  /* hurtbox capsules (the engine holds 15) */
    int hurt_joint[16], hurt_height[16], hurt_grab[16];
    float hurt_a[16][3], hurt_b[16][3], hurt_radius[16]; /* joint-local, host byte order (the game gets bit patterns) */
    int nown;                                   /* declared own rows (fighter.rows); their clip index is own_clip */
    short own_clip[GPL_MAX_OWN_ROWS];           /* clip index per declared row (303 + i), -1 = the Wait clip */
    unsigned row_flags[GPL_MAX_ROWS];           /* slice 8: the row's animation flags above the author-kind bits (0x80000000 = the clip drives the root) */
    unsigned char row_blend[GPL_MAX_ROWS][2];   /* slice 8: the row's two blend bytes (ftData x10), has_blend says the package gives them */
    unsigned char has_blend[GPL_MAX_ROWS];
    unsigned char has_flags[GPL_MAX_ROWS];      /* 1 = the package gives this row's flags; otherwise the donor's row keeps its own */
    short row_clip[GPL_MAX_ROWS];               /* clip per animation ROW (the engine's subaction numbers, ftCo_SM_*), -1 = no name matches */
    short motion_clip[GPL_MAX_MOTIONS];         /* clip index per engine motion row, -1 = the row plays none */
} GenoPlan;
#endif
