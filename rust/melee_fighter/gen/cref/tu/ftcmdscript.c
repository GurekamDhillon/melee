/* TU: src/melee/ft/ftcmdscript.c */
/* C oracle for src/tu/ftcmdscript.rs (mirrored by tests/tu_ftcmdscript.rs). Op 0: the
 * interpreter ftCo_800B3E04 over random scripts, 8 frames; op 1: the writers (4A78, 4880,
 * 46B8, 4778, 49F4) then 8 interpreter frames. Arguments are drawn in named locals: clang's
 * MS ABI evaluates call arguments right to left. Each line: op case hash trace. */
#include TU_SRC
#include "tu_common.h"

struct Fighter_804D64FC_t* Fighter_804D64FC;
static float LA;
float lb_8000D008(float y, float x) { T(B(y)); T(B(x)); return LA; }

static const u8 CODES[] = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                            21, 22, 23, 24, 25, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136,
                            137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 192,
                            193, 194 };
static u8 code(void) { return CODES[rnd() % sizeof CODES]; }
static u8 arg(void) { return (rnd() & 1) ? (u8) (rnd() % 4) : (u8) rnd(); }

static u8 SC[4][40];
static u8* SCP[4];
static struct Fighter_804D64FC_t TB;

static void hstate(void)
{
    struct CpuFighter* d = &F.cpu;
    HM(d->buttons); HM((u8) d->lstick.x); HM((u8) d->lstick.y); HM((u8) d->cstick.x);
    HM((u8) d->cstick.y); HM(d->ltrigger); HM(d->rtrigger); HM(d->x18); HM(d->command_duration);
    HM(d->csP ? (u32) (d->csP - d->buffer) : 0xFFFFFFFFu);
}

int main(void)
{
    int op, k, i, j;
    tu_init();
    TB.cmdscripts = SCP;
    Fighter_804D64FC = &TB;
    for (op = 0; op < 2; op++) {
        for (k = 0; k < 3000; k++) {
            tu_case(op, k);
            for (i = 0; i < 256; i++) {
                u32 r = rnd() % 8;
                F.cpu.buffer[i] = (s8) (r == 0 ? 127 : r < 6 ? code() : arg());
            }
            F.cpu.buffer[253] = F.cpu.buffer[254] = F.cpu.buffer[255] = 127;
            F.cur_pos.x = rw(); F.cur_pos.y = rw();
            F2.cur_pos.x = rw(); F2.cur_pos.y = rw();
            F.cpu.x54.x = rw(); F.cpu.x54.y = rw();
            F.facing_dir = rg();
            F.motion_id = rnd() % 4;
            F.cpu.x44 = (rnd() & 1) ? &F2 : NULL;
            F.cpu.lstick.x = (s8) rnd(); F.cpu.lstick.y = (s8) rnd();
            F.cpu.buttons = rnd();
            LA = rw();
            if (op == 0) {
                F.cpu.csP = (rnd() % 8) ? F.cpu.buffer + rnd() % 32 : NULL;
                F.cpu.command_duration = rnd() % 3;
            } else {
                for (j = 0; j < 4; j++) {
                    int n = 0, m = (int) (rnd() % 8);
                    for (i = 0; i < m; i++) {
                        u8 c = code();
                        if (c == 127) continue;
                        SC[j][n++] = c;
                        if (c > 127) SC[j][n++] = arg();
                        if (c > 191) SC[j][n++] = arg();
                    }
                    SC[j][n] = 127;
                    SCP[j] = SC[j];
                }
                ftCo_800B4A78(&F);
                ftCo_800B4880(&F, (int) (rnd() % 4));
                if (rnd() & 1) { u8 a = code(), b = arg(); ftCo_800B46B8(&F, a, b); }
                if (rnd() & 1) { u8 a = code(), b = arg(), d = arg(); ftCo_800B4778(&F, a, b, d); }
                if (rnd() & 1) ftCo_800B4880(&F, (int) (rnd() % 4));
                ftCo_800B49F4(&F);
            }
            H = 0x811C9DC5u;
            if (op == 1) {
                HM((u32) (F.cpu.write_pos - F.cpu.buffer));
                for (i = 0; i < 64; i++) HM((u8) F.cpu.buffer[i]);
                F.cpu.buffer[253] = F.cpu.buffer[254] = F.cpu.buffer[255] = 127;
            }
            for (j = 0; j < 8; j++) {
                LA = rw();
                ftCo_800B3E04(&F);
                hstate();
            }
            printf("%d %d %08X %08X\n", op, k, H, TR);
        }
    }
    return 0;
}
