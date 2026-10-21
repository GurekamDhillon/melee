/* C reference for the Rust NHV1 fighter word (src/neutral_hash.rs).
 *
 * Built natively by gen/cref/run.py: the real `struct Fighter` from ft/types.h, plus the
 * nh_Mix / nh_CpuFlags / nh_Fighter source cut verbatim out of src/melee/ft/fighter.c into
 * nh_extract.inc (never hand-copied). Each case fills the hashed fields from a xorshift32 stream
 * in a fixed order that tests/nh_ref.rs repeats, then prints the C word. */
#include "ft/types.h"
#include "ft/forward.h"

static int g_stocks;
static int g_cpu;
s32 Player_GetStocks(int slot) { (void) slot; return g_stocks; }
bool ftCo_IsCpuControlled(Fighter* fp) { (void) fp; return g_cpu; }

static u32 ftRb_Bits(f32 f)
{
    union {
        f32 f;
        u32 u;
    } c;
    c.f = f;
    return c.u;
}

#include "nh_extract.inc"

int printf(const char*, ...);
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static f32 rf(void)
{
    union { u32 u; f32 f; } c;
    c.u = rnd();
    if (((c.u >> 23) & 0xFF) == 0xFF) c.u &= ~(1u << 23); /* no NaN/inf */
    return c.f;
}

static Fighter fp;

int main(void)
{
    int k;
    for (k = 0; k < 64; k++) {
        struct CpuFighter* c = &fp.cpu;
        u32 off;
        zero(&fp, sizeof fp);
        rs = 0x9E3779B9u ^ (u32) k * 0x85EBCA6Bu;
        if (rs == 0) rs = 1;
        fp.motion_id = (FtMotionId) (rnd() % 400);
        fp.cur_pos.x = rf(); fp.cur_pos.y = rf(); fp.cur_pos.z = rf();
        fp.self_vel.x = rf(); fp.self_vel.y = rf();
        fp.dmg.x1830_percent = rf();
        fp.facing_dir = rf();
        fp.ground_or_air = (GroundOrAir) (rnd() & 1);
        fp.x8c_kb_vel.x = rf(); fp.x8c_kb_vel.y = rf();
        fp.gr_vel = rf();
        fp.dmg.x195c_hitlag_frames = rf();
        fp.x2219_b5 = rnd() & 1;
        fp.shield_health = rf();
        fp.x3E4_fighterCmdScript.frame_count = rf();
        fp.x1968_jumpsUsed = (u8) rnd();
        fp.x67F = (u8) rnd();
        fp.x680 = (u8) rnd();
        g_stocks = (int) (rnd() % 7) - 1;
        g_cpu = (int) (rnd() & 1);
        c->buttons = rnd();
        c->lstick.x = (s8) rnd(); c->lstick.y = (s8) rnd();
        c->cstick.x = (s8) rnd(); c->cstick.y = (s8) rnd();
        c->ltrigger = (u8) rnd(); c->rtrigger = (u8) rnd();
        c->kind = (CpuKind) (rnd() % 32);
        c->level = (int) (rnd() % 10);
        c->x14 = (int) rnd(); c->x18 = (int) rnd(); c->x1C = (int) rnd(); c->x20 = (int) rnd();
        c->x24 = (int) rnd(); c->x28 = (int) rnd(); c->x2C = (int) rnd(); c->x30 = (int) rnd();
        c->x34 = (int) rnd();
        c->x38 = rf(); c->x3C = rf(); c->x40 = rf();
        c->x7C = (int) rnd(); c->x80 = (int) rnd(); c->x84 = (int) rnd();
        c->xC8 = (u8) rnd(); c->xEC = (u8) rnd();
        c->command_duration = rnd();
        off = rnd() % 300; /* > 256: outside the buffer -> NULL */
        c->csP = off <= 256 ? (char*) c->buffer + off : NULL;
#define B(f, w) c->f = rnd() & ((1u << (w)) - 1u)
        B(xF8_b0, 1); B(xF8_b12, 2); B(xF8_b34, 2); B(xF8_b5, 1); B(xF8_b6, 1); B(xF8_b7, 1);
        B(xF9_b0, 1); B(xF9_b1, 1); B(xF9_b2, 1); B(xF9_b3, 1); B(xF9_b4, 1); B(xF9_b5, 1); B(xF9_b6, 1); B(xF9_b7, 1);
        B(xFA_b0, 1); B(xFA_b1, 1); B(xFA_b2, 1); B(xFA_b34, 2); B(xFA_b5, 1); B(xFA_b6, 1); B(xFA_b7, 1);
        B(xFB_b0, 1); B(xFB_b1, 1); B(xFB_b2, 1); B(xFB_b3, 1); B(xFB_b4, 1); B(xFB_b5, 1); B(xFB_b6, 1); B(xFB_b7, 1);
#undef B
        printf("%d %08X %08X\n", k, nh_Fighter(&fp, k % 12), nh_CpuFlags(c));
    }
    return 0;
}
