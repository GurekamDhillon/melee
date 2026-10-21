/* Round 1 (night 2) part of the melee_items C oracle; #included by oracle.c after its helpers
 * (bits, fbits, next, rf, `it`). Function bodies are copied verbatim from the decomp C they name;
 * GET_ITEM reads the Item directly (the oracle passes Item* where the game passes a GObj).
 * Built with contraction on + gwtool lowering (gen_expected.sh), as the shipping build. */
#include "melee/ft/types.h"
#include "melee/lb/types.h"
#include "melee/mp/forward.h"
#undef GET_ITEM
#define GET_ITEM(g) ((Item*) (g))
#ifndef M_TAU
#define M_TAU 6.283185307179586 /* src/Runtime/platform.h */
#endif
#ifndef M_PI_2
#define M_PI_2 (M_PI / 2) /* src/MSL/math.h */
#endif
#define Item_GObj void

/* itrshell.c (file-local type) */
typedef struct itRShell_Attrs {
    float x0; float x4; float x8; float xC; float x10; Vec3 x14; float x20; float x24; float x28;
    float x2C; char pad30[0x38 - 0x30]; float x38; float x3C; float x40; float x44; Vec x48; s32 x54;
} itRShell_Attrs;

/* itmaplib.c */
static inline float product_xy(Vec3* a, Vec3* b) { return (a->x * b->x + a->y * b->y); }
s32 it_802763E0(Item_GObj* item_gobj)
{
    int result = 0;
    Item* item = GET_ITEM(item_gobj);
    CollData* colldata = &item->x378_itemColl;
    if ((item->x378_itemColl.env_flags & Collide_CeilingMask)) {
        item->xC30 = colldata->ceiling.index;
        result = 2;
    }
    return result;
}
/* itzako.c */
static inline float product_xyz(Vec3* a, Vec3* b) { return (a->x * b->x + a->y * b->y + a->z * b->z); }
static inline f32 it_8027B798_CalcAngle(GroundOrAir ground_or_air, s32 angle, ftCommonData* common_data, f32 knockback)
{
    if (angle != 0x169) {
        return 0.017453292f * angle;
    } else if (ground_or_air == GA_Air) {
        return common_data->x144_radians;
    } else if (knockback < common_data->x14C) {
        return 0.0f;
    } else {
        f32 temp_angle =
            0.017453292f *
            ((common_data->x148 * ((knockback - common_data->x14C) /
                                   (common_data->x150 - common_data->x14C))) +
             1.0f);
        if (temp_angle > 0.017453292f * common_data->x148) {
            temp_angle = 0.017453292f * common_data->x148;
        }
        return temp_angle;
    }
}
/* itlinkboomerang.c */
static void clamp_tau(f32* angle) { if (*angle > M_TAU) { *angle -= M_TAU; } else if (*angle < -M_TAU) { *angle += M_TAU; } }
static void clamp_pi_tau(f32* angle) { if (*angle < -M_PI) { *angle += M_TAU; } else if (*angle > M_PI) { *angle -= M_TAU; } }
static inline void clamp_angle_pi(f32* angle) { if (*angle <= 0.0f) { *angle += M_PI; } else { *angle -= M_PI; } }
static inline void norm_xF74_from_angle(Item* ip, f32 angle_in)
{
    f64 angle;
    if (ip->facing_dir == 1.0f) { angle = angle_in; } else { angle = angle_in - M_PI; }
    ip->xDD4_itemVar.linkboomerang.xF78 = angle;
}
static inline f32 itLinkboomerang_UnkMotion3_Phys_add_clamp(f32 x, f32 y, f32 max) { x += y; if (x > max) { x = max; } return x; }
/* itlinkbomb.c */
static inline f32 fsign_inline(f32 x) { if (x >= 0.0F) { return 1.0F; } else { return -1.0F; } }
static inline s32 float_sign_int_inline(f32 x) { if (x == -1.0F) { return -1; } else { return 1; } }
/* itlinkhookshot.c */
static inline f32 it_link_lerp(f32 a, f32 b, f32 t) { return t * a + (1.0F - t) * b; }
/* itleadead.c */
static inline f32 neg_dot(Vec3* a, Vec3* b) { return -(a->x * b->x) - (a->y * b->y) - (a->z * b->z); }
/* ittaru.c */
static inline f32 inline_fabsf(f32 x) { if (x < 0.0F) { return -x; } else { return +x; } }
/* itrshell.c */
static inline void itRshell_ClampVel(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    f32 vel = ip->x40_vel.x;
    itRShell_Attrs* attrs = ip->xC4_article_data->x4_specialAttributes;
    f32 abs_vel;
    if (vel < 0.0f) { abs_vel = -vel; } else { abs_vel = vel; }
    if (abs_vel > attrs->xC) {
        s32 sign;
        if (vel < 0.0f) { sign = -1; } else { sign = 1; }
        ip->x40_vel.x = (f32) sign;
        ip->x40_vel.x *= attrs->xC;
    }
}
static inline void itRshell_UM5_Accel(Item_GObj* gobj, Vec3* target)
{
    Item* ip = GET_ITEM(gobj);
    itRShell_Attrs* attrs = ip->xC4_article_data->x4_specialAttributes;
    s32 sign = (target->x - ip->pos.x < 0.0f) ? -1 : 1;
    ip->xDD4_itemVar.rshell.xDE0 = attrs->x8 * (f32) sign;
}
static inline bool itRshell_UM5_AddVelAndCheck(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    s32 sign_xDE0; s32 sign_vx;
    ip->x40_vel.x += ip->xDD4_itemVar.rshell.xDE0;
    sign_xDE0 = ip->xDD4_itemVar.rshell.xDE0 < 0.0f ? -1 : 1;
    sign_vx = ip->x40_vel.x < 0.0f ? -1 : 1;
    return sign_vx == sign_xDE0;
}
static inline void itRshell_UM5C_Reverse(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->x40_vel.x = -ip->x40_vel.x;
    ip->xDD4_itemVar.rshell.xDE0 = -ip->xDD4_itemVar.rshell.xDE0;
}
/* itseakchain.c */
static inline void itSeakChain_clamp_x10(ItemLink* link, itSeakChain_Attrs* attrs)
{
    f32 value = link->vel.x; f32 limit = attrs->x10;
    if (value > limit) { link->vel.x = value - limit; } else if (value < -limit) { link->vel.x = value + limit; } else { link->vel.x = 0.0f; }
}
static inline void itSeakChain_clamp_x14(ItemLink* link, itSeakChain_Attrs* attrs)
{
    f32 value = link->vel.x; f32 limit = attrs->x14;
    if (value > limit) { link->vel.x = value - limit; } else if (value < -limit) { link->vel.x = value + limit; } else { link->vel.x = 0.0f; }
}
/* itcoll.c */
static bool itColl_chkECBOverlap(f32 pos_x, f32 pos_y, itECB* ecb_a, itECB* ecb_b, Vec3* target)
{
    f32 top; f32 left; f32 right; f32 bottom;
    top = pos_y + (ecb_a->top + ecb_b->top);
    bottom = pos_y + (ecb_a->bottom + ecb_b->bottom);
    right = pos_x + (ecb_a->right + ecb_b->right);
    left = pos_x + (ecb_a->left + ecb_b->left);
    if (top >= target->y && bottom <= target->y && right <= target->x && left >= target->x) { return true; } else { return false; }
}
/* tydisplay.c */
static inline f32 _tyDisplay_80318CB4_calc_dist_sq(f32 dz, f32 dx) { return dx * dx + dz * dz; }
/* itpikachuthunder.c */
static f32 pika_scale(f32 a, f32 b) { f32 f = 10000.0f * a / b; f = ((s32) (f + 1)) / 10000.0f; return f; }
/* itoctarockstone.c */
static inline f32 getX(f32 x1, f32 x2) { f32 res = x1 - x2; if (res < 0.0f) { return -res; } return res; }
/* itnokonoko.c */
f32 static inline lt_zero(f32 val) { if (val < 0.0f) { return -1.0f; } else { return 1.0f; } }
/* itpeachturnip.c */
static bool decrease_lifetimer(Item_GObj* item_gobj)
{
    Item* item; item = GET_ITEM(item_gobj);
    item->xD44_lifeTimer--;
    if (item->xD44_lifeTimer <= 0.0F) { return true; }
    return false;
}
/* tyfigupon.c */
static inline void tyFigupon_StoreDigits(s32* digits, s32 value) { s32 i = 0; do { digits[i++] = value % 10; value /= 10; } while (value > 0); }
/* itdosei.c */
static inline f32 itDosei_FacingAngle(Item_GObj* gobj, f32 m) { f32 facing_dir = GET_ITEM(gobj)->facing_dir; f32 angle = (facing_dir * M_PI_2) + (facing_dir * m); return angle; }
/* it_2725.c (it_804D6D28 is the oracle's own global) */
static ItemCommonData* it_804D6D28_o;
#define it_804D6D28 it_804D6D28_o
void it_80275158(Item_GObj* item_gobj, f32 lifetime)
{
    Item* item; item = GET_ITEM(item_gobj);
    item->xD44_lifeTimer = lifetime;
    item->xD48_halfLifeTimer = lifetime * it_804D6D28->x4C_float;
}
#undef it_804D6D28
#undef Item_GObj

/* input pool: random values plus edge cases (signed zeros, +-1, pi/tau edges, inf, nan, tiny) */
static const unsigned SPECIAL[] = { 0x00000000u, 0x80000000u, 0x3F800000u, 0xBF800000u, 0x40490FDBu,
    0xC0490FDBu, 0x40C90FDBu, 0xC0C90FDBu, 0x40C90FDAu, 0x40490FDAu, 0x7F800000u, 0xFF800000u,
    0x7FC00000u, 0x00000001u, 0x3F7FFFFFu, 0x447A0000u, 0x461C4000u, 0x3DCCCCCDu };
#define NSPECIAL ((int) (sizeof SPECIAL / sizeof SPECIAL[0]))
static f32 pick(int i)
{
    unsigned r = next();
    if (i < NSPECIAL) return fbits(SPECIAL[i]);
    if ((r & 7) == 0) return fbits(SPECIAL[(r >> 8) % NSPECIAL]);
    if ((r & 7) == 1) return fbits(next()); /* any bit pattern */
    return rf() / (f32) (1 << ((r >> 4) & 15));
}

#define N1 96
static void row(const unsigned* v, int n)
{
    int k;
    printf("    [");
    for (k = 0; k < n; k++) printf("%s0x%08X", k ? ", " : "", v[k]);
    printf("],\n");
}
#define BEGIN(name, n) printf("pub const " name ": &[[u32; %d]] = &[\n", n)
#define END() printf("];\n")

static void round1(void)
{
    int i;
    unsigned v[16];
    Vec3 a, b;
    BEGIN("PRODUCT_XYZ", 8); /* ax ay az bx by bz | xy xyz */
    for (i = 0; i < N1; i++) {
        a.x = pick(i); a.y = pick(i + 3); a.z = pick(i + 5); b.x = pick(i + 1); b.y = pick(i + 2); b.z = pick(i + 7);
        v[0] = bits(a.x); v[1] = bits(a.y); v[2] = bits(a.z); v[3] = bits(b.x); v[4] = bits(b.y); v[5] = bits(b.z);
        v[6] = bits(product_xy(&a, &b)); v[7] = bits(product_xyz(&a, &b)); row(v, 8);
    }
    END();
    BEGIN("NEG_DOT", 7);
    for (i = 0; i < N1; i++) {
        a.x = pick(i); a.y = pick(i + 3); a.z = pick(i + 5); b.x = pick(i + 1); b.y = pick(i + 2); b.z = pick(i + 7);
        v[0] = bits(a.x); v[1] = bits(a.y); v[2] = bits(a.z); v[3] = bits(b.x); v[4] = bits(b.y); v[5] = bits(b.z);
        v[6] = bits(neg_dot(&a, &b)); row(v, 7);
    }
    END();
    BEGIN("CALC_ANGLE", 8); /* ga angle x144 x148 x14C x150 kb | out */
    for (i = 0; i < N1; i++) {
        ftCommonData cd; f32 kb = pick(i) * 100.0f; s32 ang = (i % 3) ? 0x169 : (s32) (next() % 400) - 20;
        GroundOrAir ga = (i & 4) ? GA_Air : GA_Ground;
        memset(&cd, 0, sizeof cd); cd.x144_radians = pick(i + 1); cd.x148 = (f32) (next() % 90);
        cd.x14C = (f32) (next() % 120); cd.x150 = cd.x14C + (f32) (next() % 50) + pick(i + 4);
        v[0] = (unsigned) ga; v[1] = (unsigned) ang; v[2] = bits(cd.x144_radians); v[3] = bits(cd.x148);
        v[4] = bits(cd.x14C); v[5] = bits(cd.x150); v[6] = bits(kb);
        v[7] = bits(it_8027B798_CalcAngle(ga, ang, &cd, kb)); row(v, 8);
    }
    END();
    BEGIN("ANGLES", 6); /* x | clamp_tau clamp_pi_tau clamp_angle_pi norm(fd=1) norm(fd=-1) */
    for (i = 0; i < N1; i++) {
        f32 x = pick(i) * ((i & 1) ? 8.0f : 1.0f), t;
        v[0] = bits(x);
        t = x; clamp_tau(&t); v[1] = bits(t);
        t = x; clamp_pi_tau(&t); v[2] = bits(t);
        t = x; clamp_angle_pi(&t); v[3] = bits(t);
        memset(&it, 0, sizeof it); it.facing_dir = 1.0f; norm_xF74_from_angle(&it, x);
        v[4] = bits(it.xDD4_itemVar.linkboomerang.xF78);
        it.facing_dir = -1.0f; norm_xF74_from_angle(&it, x); v[5] = bits(it.xDD4_itemVar.linkboomerang.xF78);
        row(v, 6);
    }
    END();
    BEGIN("SCALAR3", 12); /* x y z | add_clamp lerp dist_sq pika getX fsign fsign_int fabs lt_zero */
    for (i = 0; i < N1; i++) {
        f32 x = pick(i), y = pick(i + 2), z = pick(i + 4);
        v[0] = bits(x); v[1] = bits(y); v[2] = bits(z);
        v[3] = bits(itLinkboomerang_UnkMotion3_Phys_add_clamp(x, y, z));
        v[4] = bits(it_link_lerp(x, y, z));
        v[5] = bits(_tyDisplay_80318CB4_calc_dist_sq(x, y));
        v[6] = bits(pika_scale(x, y));
        v[7] = bits(getX(x, y));
        v[8] = bits(fsign_inline(x));
        v[9] = (unsigned) float_sign_int_inline((i & 3) == 0 ? -1.0f : x);
        v[10] = bits(inline_fabsf(x));
        v[11] = bits(lt_zero(x));
        row(v, 12);
    }
    END();
    BEGIN("DOSEI", 3); /* fd m | angle */
    for (i = 0; i < N1; i++) {
        f32 fd = (i < 4) ? pick(i) : ((i & 1) ? 1.0f : -1.0f), m = pick(i + 1);
        memset(&it, 0, sizeof it); it.facing_dir = fd;
        v[0] = bits(fd); v[1] = bits(m); v[2] = bits(itDosei_FacingAngle((void*) &it, m)); row(v, 3);
    }
    END();
    BEGIN("RSHELL", 11); /* vx xDE0 x8 xC posx tx | clampvel accel.xDE0 addvel.vx addvel.ret reverse(vx^xDE0) */
    for (i = 0; i < N1; i++) {
        static itRShell_Attrs at; static Article art; Vec3 tgt;
        memset(&at, 0, sizeof at); memset(&art, 0, sizeof art); art.x4_specialAttributes = &at;
        at.x8 = pick(i + 1); at.xC = pick(i + 2);
        memset(&it, 0, sizeof it); it.xC4_article_data = &art;
        v[0] = bits(pick(i)); v[1] = bits(pick(i + 3)); v[2] = bits(at.x8); v[3] = bits(at.xC);
        v[4] = bits(pick(i + 5)); v[5] = bits(pick(i + 6));
        it.x40_vel.x = fbits(v[0]); itRshell_ClampVel((void*) &it); v[6] = bits(it.x40_vel.x);
        it.pos.x = fbits(v[4]); tgt.x = fbits(v[5]); tgt.y = tgt.z = 0;
        itRshell_UM5_Accel((void*) &it, &tgt); v[7] = bits(it.xDD4_itemVar.rshell.xDE0);
        it.x40_vel.x = fbits(v[0]); it.xDD4_itemVar.rshell.xDE0 = fbits(v[1]);
        v[9] = (unsigned) itRshell_UM5_AddVelAndCheck((void*) &it); v[8] = bits(it.x40_vel.x);
        itRshell_UM5C_Reverse((void*) &it); v[10] = bits(it.x40_vel.x) ^ bits(it.xDD4_itemVar.rshell.xDE0);
        row(v, 11);
    }
    END();
    BEGIN("SEAK_CLAMP", 5); /* vx x10 x14 | clamp_x10 clamp_x14 */
    for (i = 0; i < N1; i++) {
        static ItemLink l; static itSeakChain_Attrs at;
        memset(&l, 0, sizeof l); memset(&at, 0, sizeof at);
        v[0] = bits(pick(i)); at.x10 = pick(i + 1); at.x14 = -pick(i + 2); v[1] = bits(at.x10); v[2] = bits(at.x14);
        l.vel.x = fbits(v[0]); itSeakChain_clamp_x10(&l, &at); v[3] = bits(l.vel.x);
        l.vel.x = fbits(v[0]); itSeakChain_clamp_x14(&l, &at); v[4] = bits(l.vel.x);
        row(v, 5);
    }
    END();
    BEGIN("ECB_OVERLAP", 13); /* px py a(t b r l) b(t b r l) tx ty | ret */
    for (i = 0; i < N1; i++) {
        itECB ea, eb; Vec3 t; int k;
        f32* fa = &ea.top; f32* fb = &eb.top;
        for (k = 0; k < 4; k++) { fa[k] = pick(i + k) * 0.25f; fb[k] = pick(i + k + 4) * 0.25f; }
        v[0] = bits(pick(i + 8) * 0.01f); v[1] = bits(pick(i + 9) * 0.01f);
        t.x = pick(i + 10) * 0.01f; t.y = pick(i + 11) * 0.01f; t.z = 0;
        for (k = 0; k < 4; k++) { v[2 + k] = bits(fa[k]); v[6 + k] = bits(fb[k]); }
        v[10] = bits(t.x); v[11] = bits(t.y);
        v[12] = (unsigned) itColl_chkECBOverlap(fbits(v[0]), fbits(v[1]), &ea, &eb, &t); row(v, 13);
    }
    END();
    BEGIN("LIFETIMER", 6); /* life x4C | dec.life dec.ret set.life set.half */
    for (i = 0; i < N1; i++) {
        static ItemCommonData cd; f32 l = pick(i), h = pick(i + 1);
        memset(&cd, 0, sizeof cd); cd.x4C_float = h; it_804D6D28_o = &cd;
        memset(&it, 0, sizeof it); it.xD44_lifeTimer = l;
        v[0] = bits(l); v[1] = bits(h); v[3] = (unsigned) decrease_lifetimer((void*) &it); v[2] = bits(it.xD44_lifeTimer);
        it_80275158((void*) &it, l); v[4] = bits(it.xD44_lifeTimer); v[5] = bits(it.xD48_halfLifeTimer); row(v, 6);
    }
    END();
    BEGIN("CEILING", 4); /* env_flags index | ret xC30 */
    for (i = 0; i < 32; i++) {
        memset(&it, 0, sizeof it); it.xC30 = -7;
        it.x378_itemColl.env_flags = (s32) (next() & 0x7FFF);
        if (i < 4) it.x378_itemColl.env_flags = (s32) (0x2000u << (i & 1)) * (i >> 1);
        it.x378_itemColl.ceiling.index = (s32) (next() % 300) - 1;
        v[0] = (unsigned) it.x378_itemColl.env_flags; v[1] = (unsigned) it.x378_itemColl.ceiling.index;
        v[2] = (unsigned) it_802763E0((void*) &it); v[3] = (unsigned) it.xC30; row(v, 4);
    }
    END();
    BEGIN("DIGITS", 11); /* value | digits[0..9], unwritten = -1 */
    for (i = 0; i < 40; i++) {
        static const s32 fixed[] = { 0, 9, 10, -5, 2147483647, -2147483647 - 1 };
        s32 d[10]; s32 val; int k;
        val = i < 6 ? fixed[i] : (s32) (next() % ((i & 1) ? 100000u : 1000000000u));
        for (k = 0; k < 10; k++) d[k] = -1;
        tyFigupon_StoreDigits(d, val);
        v[0] = (unsigned) val;
        for (k = 0; k < 10; k++) v[1 + k] = (unsigned) d[k];
        row(v, 11);
    }
    END();
}
