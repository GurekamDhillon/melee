/* What the functions cut from gw_rollback.c see: GwRbInput (cut from gw_rollback.h) and the
 * fields of the file-static `rb` they read. */
#include "gwrbinput.h"
#define RB_FIRST (-123)
static struct {
    unsigned gen_seed, seed;
    int delay, fake_lat, fake_jit, fake_loss;
} rb;
