/* Driver for the efLib list walkers (gen_trivial_r3_ref.sh). Each case builds the two effect
 * lists (UNK0, UNK1) of random length with random parents (one of 3 GObjs, some effects NULL),
 * runs one op, and prints, all hex:
 *   E op arg n0 n1 | per-node: parent(0-2 or n for no effect) expire state ... | after: expire state ...
 * op: 0 SetFlags(gobj[0], arg), 1 PauseAll(gobj[0]), 2 ResumeAll(gobj[0]).
 * tests/eflib_r3_vs_c.rs rebuilds the same lists and replays. */
#include <stdint.h>
#include <stdio.h>
typedef struct HSD_GObj HSD_GObj;
struct HSD_GObj { HSD_GObj* next; void* user_data; };
typedef struct EF_Effect {
    struct EF_Effect* next; HSD_GObj* gobj; HSD_GObj* parent_gobj; void* attach_jobj;
    void* update; void* user_data; float params[3]; uint16_t lifetime; int8_t expire_flags;
    uint8_t scale_flags; uint8_t state_flags; uint8_t is_async; int8_t x2A; int8_t x2B;
} EF_Effect;
extern HSD_GObj* HSD_GObjPLinkHead[16];
void efLib_SetFlags(HSD_GObj*, int32_t);
void efLib_PauseAll(HSD_GObj*);
void efLib_ResumeAll(HSD_GObj*);
static uint32_t rng = 0x9E3779B9u;
static uint32_t nx(void) { rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5; return rng; }
int main(void)
{
    static HSD_GObj nodes[2][8], parents[3];
    static EF_Effect effs[2][8];
    for (int c = 0; c < 3000; c++) {
        int op = nx() % 3;
        int32_t arg = (nx() % 4 == 0) ? (int32_t) nx() : (int32_t) (nx() & 0xFF) - 0x80;
        int n[2] = { nx() % 9, nx() % 9 };
        int par[2][8];
        printf("E %x %x %x %x |", op, (uint32_t) arg, n[0], n[1]);
        for (int l = 0; l < 2; l++) {
            HSD_GObjPLinkHead[11 + l] = n[l] ? &nodes[l][0] : NULL;
            for (int i = 0; i < n[l]; i++) {
                nodes[l][i].next = i + 1 < n[l] ? &nodes[l][i + 1] : NULL;
                par[l][i] = nx() % 4; /* 3 = no effect */
                if (par[l][i] == 3) {
                    nodes[l][i].user_data = NULL;
                    printf(" n");
                } else {
                    EF_Effect* e = &effs[l][i];
                    nodes[l][i].user_data = e;
                    e->parent_gobj = &parents[par[l][i]];
                    e->expire_flags = (int8_t) nx();
                    e->state_flags = (uint8_t) nx();
                    printf(" %x %x %x", par[l][i], (uint8_t) e->expire_flags, e->state_flags);
                }
            }
        }
        if (op == 0) efLib_SetFlags(&parents[0], arg);
        else if (op == 1) efLib_PauseAll(&parents[0]);
        else efLib_ResumeAll(&parents[0]);
        printf(" |");
        for (int l = 0; l < 2; l++)
            for (int i = 0; i < n[l]; i++)
                if (par[l][i] != 3)
                    printf(" %x %x", (uint8_t) effs[l][i].expire_flags, effs[l][i].state_flags);
        printf("\n");
    }
    return 0;
}
