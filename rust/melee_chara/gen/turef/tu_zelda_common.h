/* Shared state-oracle harness for the group-4 Zelda TUs (ftzeldaspecial{lw,n,s}.c).
 *
 * Each driver includes its TU verbatim (the game's own code, compiled natively with the shipping
 * contraction by gen/turef/tu_shard457.py), then this header, then calls the TU's functions on a
 * Fighter filled from a seeded xorshift stream. Callees outside the TU that Rust reaches through a
 * hook are logging stubs here; the Rust test installs hooks that log the same text. Callees the
 * Rust side calls as real group-3 functions are compiled from the game source (extract .inc).
 * The fingerprint line format is mirrored by rust/melee_chara/tests/tu_<stem>.rs. */
#ifndef TU_ZELDA_COMMON_H
#define TU_ZELDA_COMMON_H
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

static char g_log[8192];
static size_t g_loglen;
/* Minimal formatter (the game's libc has no vsnprintf): %d %ld %u %x %08x %s, appended to g_log. */
static void app_c(char c)
{
    if (g_loglen + 1 < sizeof g_log) {
        g_log[g_loglen++] = c;
        g_log[g_loglen] = 0;
    }
}
static void app_u(unsigned long v, unsigned base, int width, int zeropad)
{
    char t[40];
    int n = 0;
    do {
        t[n++] = "0123456789abcdef"[v % base];
        v /= base;
    } while (v);
    while (n < width) {
        t[n++] = zeropad ? '0' : ' ';
    }
    while (n) {
        app_c(t[--n]);
    }
}
static void lg(const char* fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    while (*fmt) {
        int width = 0;
        int zp = 0;
        int lng = 0;
        if (*fmt != '%') {
            app_c(*fmt++);
            continue;
        }
        fmt++;
        if (*fmt == '0') {
            zp = 1;
            fmt++;
        }
        while (*fmt >= '0' && *fmt <= '9') {
            width = width * 10 + (*fmt++ - '0');
        }
        if (*fmt == 'l') {
            lng = 1;
            fmt++;
        }
        switch (*fmt++) {
        case 'd': {
            long v = lng ? va_arg(ap, long) : (long) va_arg(ap, int);
            if (v < 0) {
                app_c('-');
                app_u((unsigned long) -v, 10, 0, 0);
            } else {
                app_u((unsigned long) v, 10, 0, 0);
            }
            break;
        }
        case 'u': app_u(lng ? va_arg(ap, unsigned long) : (unsigned long) va_arg(ap, unsigned), 10, 0, 0); break;
        case 'x': app_u((unsigned long) va_arg(ap, unsigned), 16, width, zp); break;
        case 's': {
            const char* p = va_arg(ap, const char*);
            while (*p) {
                app_c(*p++);
            }
            break;
        }
        }
    }
    va_end(ap);
}

/* Per-case controls, written by the driver from the seeded stream (mirrored in Rust). */
static int g_frames_remaining;
static int g_ground_ret;
static int g_air_ret;
static int g_item_ret;
static int g_owner_self;
static void* g_dat;

/* Item and fighter identities for x222C / returns. Joints are compared by index only. */
static HSD_GObj ITEM_A_OBJ;
static HSD_GObj ITEM_B_OBJ;
static HSD_GObj G; /* the gobj the functions under test receive; its user_data is &F */
/* Joint identities: byte-array slots (HSD_JObj is only forward-declared in the TU headers). */
#define JSTRIDE 64
static char JOINT_BASE[128 * JSTRIDE];

static u32 g_s = 0x2468ACEu;
static u32 rnd(void)
{
    g_s ^= g_s << 13;
    g_s ^= g_s >> 17;
    g_s ^= g_s << 5;
    return g_s;
}
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 4001) - 2000) / 997.0f;
    }
}
static u32 bits(float f)
{
    union {
        float f;
        u32 u;
    } c;
    c.f = f;
    return c.u;
}
/* Fill n bytes (a multiple of 4) as 32-bit words of rf() bit patterns. */
static void fill_words(void* p, size_t n)
{
    u32* w = (u32*) p;
    size_t i;
    for (i = 0; i < n / 4; i++) {
        w[i] = bits(rf());
    }
}
static const char* cbname(void* p)
{
    size_t i;
    if (p == NULL) return "0";
    for (i = 0; i < sizeof CB_TABLE / sizeof CB_TABLE[0]; i++) {
        if (CB_TABLE[i].p == p) return CB_TABLE[i].n;
    }
    return "?";
}
static const char* itemname(void* p)
{
    if (p == NULL) return "0";
    if (p == (void*) &ITEM_A_OBJ) return "A";
    if (p == (void*) &ITEM_B_OBJ) return "B";
    if (p == (void*) &G) return "self";
    return "?";
}
static int jointidx(void* j)
{
    return (int) (((char*) j - JOINT_BASE) / JSTRIDE);
}

/* ---- callees outside the TU (signatures from the game headers) ---- */
void Fighter_ChangeMotionState(Fighter_GObj* gobj, FtMotionId msid, MotionFlags flags, f32 start, f32 speed,
                               f32 blend, Fighter_GObj* arg3)
{
    lg("cms(%d,%u,%08x,%08x,%08x,%s)", (int) msid, (unsigned) flags, bits(start), bits(speed), bits(blend),
       arg3 == NULL ? "0" : "g");
    (void) gobj;
}
void ftAnim_8006EBA4(Fighter_GObj* g)
{
    (void) g;
    lg("anim8006EBA4");
}
bool ftAnim_IsFramesRemaining(Fighter_GObj* g)
{
    (void) g;
    lg("framesrem");
    return g_frames_remaining;
}
void ft_8008A2BC(Fighter_GObj* g)
{
    (void) g;
    lg("8008A2BC");
}
void ftCo_Fall_Enter(Fighter_GObj* g)
{
    (void) g;
    lg("cofall");
}
GroundOrAir ft_80081D0C(Fighter_GObj* g)
{
    (void) g;
    lg("81D0C");
    return g_air_ret;
}
GroundOrAir ft_80082708(Fighter_GObj* g)
{
    (void) g;
    lg("82708");
    return g_ground_ret;
}
void ft_80084F3C(Fighter_GObj* g)
{
    (void) g;
    lg("84F3C");
}
void ftColl_8007AEF8(Fighter_GObj* g)
{
    (void) g;
    lg("coll8007AEF8");
}
void ftCo_80096900(Fighter_GObj* g, int a, int b, int allow, f32 mob, f32 land)
{
    (void) g;
    lg("co96900(%d,%d,%d,%08x,%08x)", a, b, allow, bits(mob), bits(land));
}
void ftColl_CreateReflectHit(Fighter_GObj* g, ReflectDesc* d, HSD_GObjEvent cb)
{
    (void) g;
    lg("reflect(%ld,%s)", (long) ((char*) d - (char*) g_dat), cbname((void*) cb));
}
void* efSync_Spawn(s32 gfx_id, HSD_GObj* arg_gobj, ...)
{
    (void) arg_gobj;
    lg("efsync(%d)", (int) gfx_id);
    return NULL;
}
void lb_8000B1CC(HSD_JObj* jobj, Vec3* pos0, Vec3* pos1)
{
    int i = jointidx(jobj);
    (void) pos0;
    pos1->x = (f32) i;
    pos1->y = (f32) i * 2.0f;
    pos1->z = 3.0f;
    lg("lb8000B1CC(j%d)", i);
}
void lb_800119DC(Vec3* v, int n, float a, float b, float c)
{
    lg("lb119DC(%08x,%08x,%08x,%d,%08x,%08x,%08x)", bits(v->x), bits(v->y), bits(v->z), n, bits(a), bits(b),
       bits(c));
}
Item_GObj* it_802C3BAC(Item_GObj* gobj, Vec3* vec, f32 dir, f32 x)
{
    (void) gobj;
    lg("it3BAC(%08x,%08x,%08x,%08x)", bits(vec->x), bits(vec->y), bits(dir), bits(x));
    return g_item_ret ? (Item_GObj*) &ITEM_A_OBJ : NULL;
}
Fighter_GObj* itZeldaDinFire_GetOwner(Item_GObj* gobj)
{
    lg("getowner(%s)", itemname((void*) gobj));
    return g_owner_self ? (Fighter_GObj*) &G : (Fighter_GObj*) &ITEM_B_OBJ;
}
void it_802C3D44(Item_GObj* gobj)
{
    lg("it3D44(%s)", itemname((void*) gobj));
}
void efLib_PauseAll(HSD_GObj* gobj)
{
    (void) gobj;
}
void efLib_ResumeAll(HSD_GObj* gobj)
{
    (void) gobj;
}
void ftCommon_Fall(Fighter* fp, float gravity, float terminal_vel)
{
    (void) fp;
    lg("fall(%08x,%08x)", bits(gravity), bits(terminal_vel));
}
void ftCommon_8007D5D4(Fighter* fp)
{
    (void) fp;
    lg("d5d4");
}
void ftCommon_8007D7FC(Fighter* fp)
{
    (void) fp;
    lg("d7fc");
}
void ftCommon_8007EFC8(Fighter_GObj* gobj, void (*cb)(Fighter_GObj*))
{
    (void) gobj;
    lg("8007EFC8(%s)", cbname((void*) cb));
}
void ftPartSetRotX(Fighter* fp, int part_idx, f32 rotate_x)
{
    (void) fp;
    lg("prot(%d,%08x)", part_idx, bits(rotate_x));
}
void ftZd_Init_801393AC(HSD_GObj* gobj)
{
    (void) gobj;
    lg("init");
}

/* ---- per-case setup and fingerprint (order of draws is mirrored in the Rust test) ---- */
#define NCASES 200
static Fighter F;
static ftZelda_DatAttrs DA;
static FighterBone PARTS[128];
static ftCommonData CDATA;

static void setup_case(void)
{
    int k;
    int choice;
    memset(&F, 0, sizeof F);
    G.user_data = &F;
    F.dat_attrs = &DA;
    g_dat = &DA;
    fill_words(&DA, sizeof DA);
    F.parts = PARTS;
    for (k = 0; k < 128; k++) {
        PARTS[k].joint = (HSD_JObj*) (JOINT_BASE + k * JSTRIDE);
    }
    F.facing_dir = rf();
    F.self_vel.x = rf();
    F.self_vel.y = rf();
    F.gr_vel = rf();
    F.cur_anim_frame = rf();
    for (k = 0; k < 4; k++) {
        F.cmd_vars[k] = rnd() % 4;
    }
    F.x2219_b0 = rnd() & 1;
    F.reflecting = rnd() & 1;
    F.x1968_jumpsUsed = rnd() % 4;
    F.motion_id = 340 + rnd() % 12;
    F.ground_or_air = rnd() % 2;
    F.input.held_buttons[0] = rnd();
    F.co_attrs.aerial_friction = rf();
    F.co_attrs.terminal_velocity = rf();
    choice = rnd() % 3;
    F.u.zd.x222C = choice == 0 ? NULL : choice == 1 ? &ITEM_A_OBJ : &ITEM_B_OBJ;
    F.mv.zd.specials.x0 = rnd() % 5;
    F.mv.zd.specials.x4 = rnd() % 5;
    F.mv.zd.specials.x8 = rnd() % 5;
    F.mv.zd.specials.xC = rnd() % 5;
    g_frames_remaining = rnd() & 1;
    g_ground_ret = rnd() & 1;
    g_air_ret = rnd() & 1;
    g_item_ret = rnd() & 1;
    g_owner_self = rnd() & 1;
    F.accessory4_cb = (HSD_GObjEvent) CB_TABLE[rnd() % CB_COUNT].p;
    F.death2_cb = (HSD_GObjEvent) CB_TABLE[rnd() % CB_COUNT].p;
    F.take_dmg_cb = (HSD_GObjEvent) CB_TABLE[rnd() % CB_COUNT].p;
    F.pre_hitlag_cb = (HSD_GObjEvent) CB_TABLE[rnd() % CB_COUNT].p;
    F.post_hitlag_cb = (HSD_GObjEvent) CB_TABLE[rnd() % CB_COUNT].p;
    F.x74_self_accel.x = rf();
    F.x74_self_accel.y = rf();
    CDATA.aerial_friction_oob = rf();
    p_ftCommonData = &CDATA;
}

static void fingerprint(int f, int i, const char* name)
{
    printf("%d.%d %s|log=%s|acc=%s|death=%s|take=%s|pre=%s|post=%s|x2219=%d|cmd=%u,%u,%u,%u|sv=%08x,%08x|"
           "gr=%08x|fd=%08x|cur=%08x|sp=%d,%d,%d,%d|refl=%d|u=%s|jumps=%u|held=%u|acc74=%08x,%08x|ret=%d\n",
           f, i, name, g_log, cbname((void*) F.accessory4_cb), cbname((void*) F.death2_cb),
           cbname((void*) F.take_dmg_cb), cbname((void*) F.pre_hitlag_cb), cbname((void*) F.post_hitlag_cb),
           (int) F.x2219_b0, (unsigned) F.cmd_vars[0], (unsigned) F.cmd_vars[1], (unsigned) F.cmd_vars[2],
           (unsigned) F.cmd_vars[3], bits(F.self_vel.x), bits(F.self_vel.y), bits(F.gr_vel), bits(F.facing_dir),
           bits(F.cur_anim_frame), (int) F.mv.zd.specials.x0, (int) F.mv.zd.specials.x4,
           (int) F.mv.zd.specials.x8, (int) F.mv.zd.specials.xC, (int) F.reflecting, itemname((void*) F.u.zd.x222C),
           (unsigned) F.x1968_jumpsUsed, (unsigned) F.input.held_buttons[0], bits(F.x74_self_accel.x),
           bits(F.x74_self_accel.y), g_ret);
}

/* Runs every function of the driver's table on NCASES seeded states each. */
static void run_all(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES; i++) {
            setup_case();
            g_loglen = 0;
            g_log[0] = 0;
            g_ret = 0;
            call_fn(f, &G);
            fingerprint(f, i, FUNC_NAMES[f]);
        }
    }
}

#endif
