/* TU: src/melee/if/ifprize.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/if_drv.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
int sprintf(char* s, const char* f, ...) { __builtin_va_list ap; char* o = s; __builtin_va_start(ap, f); while (*f) { if (*f != 37) { *o++ = *f++; continue; } f++; { int zero = 0, w = 0, v, neg, n = 0; char t[16]; if (*f == 48) { zero = 1; f++; } while (*f >= 48 && *f <= 57) { w = w * 10 + (*f - 48); f++; } f++; v = __builtin_va_arg(ap, int); neg = v < 0; { unsigned u = neg ? 0u - (unsigned) v : (unsigned) v; do { t[n++] = (char) (48 + u % 10); u /= 10; } while (u); } if (neg && zero) { *o++ = 45; w--; } while (w > n + (neg && !zero ? 1 : 0)) { *o++ = zero ? 48 : 32; w--; } if (neg && !zero) *o++ = 45; while (n) *o++ = t[--n]; } } *o = 0; __builtin_va_end(ap); return (int) (o - s); }
static DynamicModelDesc* PRZM[2];
static s16 S16A[4];
static u8 U8A[4];
static int S32A[4];

#include "ifprize_gen.h"



static void setup_extra(void)
{
    int i, n;
    (void) i; (void) n;

}

int main(void)
{
    int op, k;
    preg_state();
    freg_all();
    tu_case(99, 0); hash_init_state(); printf("%d %d %08X %08X %08X\n", 99, 0, 0u, H, TR);
    for (op = 0; op < 12; op++) {
        static const int NC[] = {400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400};
        for (k = 0; k < NC[op]; k++) {
            u32 ret = 0;
            tu_case(op, k);
            reset_state();
            setup_extra();
            switch (op) {
            case 0: { int a0; int a1; s16 * a2; s16 * a3;  a0 = (int) ((0 + (int) (rnd() % 71u))); a1 = (int) ((0 + (int) (rnd() % 31u))); a2 = (s16 *) (((rnd() & 1) ? &S16A[0] : 0)); a3 = (s16 *) (((rnd() & 1) ? &S16A[2] : 0)); un_802FE3F8(a0, a1, a2, a3); } break;
            case 1: { int a0; int a1; s16 * a2; s16 * a3;  a0 = (int) ((0 + (int) (rnd() % 71u))); a1 = (int) ((0 + (int) (rnd() % 31u))); a2 = (s16 *) (((rnd() & 1) ? &S16A[0] : 0)); a3 = (s16 *) (((rnd() & 1) ? &S16A[2] : 0)); un_802FE3F8_inner(a0, a1, a2, a3); } break;
            case 2: { int a0; int a1; s16 * a2; s16 * a3;  a0 = (int) ((0 + (int) (rnd() % 71u))); a1 = (int) ((0 + (int) (rnd() % 31u))); a2 = (s16 *) (((rnd() & 1) ? &S16A[0] : 0)); a3 = (s16 *) (((rnd() & 1) ? &S16A[2] : 0)); un_802FE3F8_noinline(a0, a1, a2, a3); } break;
            case 3: { HSD_GObj * a0; int a1;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); a1 = (int) (fv()); ifPrize_RenderGuarded(a0, a1); } break;
            case 4: { HSD_GObj * a0;  a0 = (HSD_GObj *) (pick_HSD_GObj(0)); fn_802FE470(a0); } break;
            case 5: {    un_802FE6A8(); } break;
            case 6: {    un_802FE8CC(); } break;
            case 7: {    setArchive(); } break;
            case 8: { unsigned char * a0; int * a1;  a0 = (unsigned char *) (&U8A[0]); a1 = (int *) (&S32A[0]); un_802FE918_update_x3(a0, a1); } break;
            case 9: { int a0; int a1; int a2;  a0 = (int) ((0 + (int) (rnd() % 71u))); a1 = (int) ((0 + (int) (rnd() % 31u))); a2 = (int) (fv()); un_802FE918(a0, a1, a2); } break;
            case 10: { void * a0;  a0 = (void *) (pick_un_802FEBE0_OnEnter_arg0(0)); ifPrize_Scene_OnEnter(a0); } break;
            case 11: { void * a0;  a0 = (void *) (0); ifPrize_Scene_OnExit(a0); } break;
            }
            hash_state();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
