/* Shared layout of the game_camera stand-in for the camera accessor oracle
 * (gen_camera_acc_ref.sh). camera_acc_tu.c (game flags) defines it with the game's types, the
 * driver camera_acc_ref.c with plain ones; the member list and order are the same. Members are
 * the cm/types.h names of everything the cut bodies touch. */
#ifndef CAMERA_ACC_H
#define CAMERA_ACC_H
#define CAM_ACC_MEMBERS(V3, TS, COLOR, V2)                                                      \
    void* gobj;                                                                                \
    unsigned mode;                                                                             \
    COLOR background_color;                                                                    \
    float nearz, farz;                                                                         \
    TS transform;                                                                              \
    V2 translation;                                                                            \
    float quake_scale, x2B0;                                                                   \
    short x2B8;                                                                                \
    float x2BC;                                                                                \
    V3 pause_up;                                                                               \
    unsigned char x341_b0 : 1, x341_b1_b2 : 2, x341_b3_b4 : 2, x341_b5_b6 : 2, x341_b7 : 1;    \
    V3 x350, x368;                                                                             \
    float x374;                                                                                \
    unsigned char x398_b0 : 1, x398_b1 : 1, x398_b2 : 1, x398_b3 : 1, x398_b4 : 1,             \
        x398_b5 : 1, x398_b6_b7 : 2;                                                           \
    unsigned char x399_b0_b1 : 2, x399_b2 : 1, x399_b3 : 1, x399_b4 : 1, x399_b5 : 1,          \
        x399_b6 : 1, x399_b7 : 1;                                                              \
    unsigned char x39A_b0 : 1, x39A_b1 : 1, x39A_b2 : 1, x39A_b3 : 1, x39A_b4 : 1,             \
        x39A_b5 : 1, x39A_b6 : 1, x39A_b7 : 1;
/* op ids: see camera_acc_tu.c acc_op */
int acc_op(int op, int iarg, const float* fin, float* fout);
#endif
