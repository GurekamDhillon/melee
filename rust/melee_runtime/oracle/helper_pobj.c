/* Oracle scenarios for the logic of pobj.c (build_pobj.sh). Compiled with the game flags;
 * includes pobj.c and aobj.c (statics reachable: mtx_mark, ShapeSetSetAnimResult, PObjUpdateFunc,
 * decode_*_xyz, the AObj counters). Generated inputs only.
 *
 * Rows (hex words): <op> <inputs...> | <outputs...> <state hash>. A persistent chain of 3 PObjs
 * is reset every 40 ops (row "ffffffff" + the reset inputs). The hash covers each PObj's flags,
 * its shape set (flags, bl, bp[0..8], AObj state incl. every FObj), mtx_mark and the AObj
 * InterpretAnim counters. An AObjDesc is written as: present, flags, end_frame, nf, then per FObj
 * length startframe type frac_value frac_slope <256 hex bytes>. */
#include "../../../src/sysdolphin/baselib/pobj.c"
#include "../../../src/sysdolphin/baselib/aobj.c"

unsigned drv_rand(void);
float drv_randf(float r);
unsigned drv_rword(void);
void drv_u32(unsigned v);
void drv_hex(const unsigned char* p, int n);
void drv_sep(void);
void drv_eol(void);
void drv_hash_begin(void);
void drv_hash_end(void);

static u32 fb(f32 f) { return *(u32*) &f; }
static f32 bf(u32 u) { return *(f32*) &u; }
static u32 rn(u32 n) { return drv_rand() % n; }
static f32 rf(void)
{
    switch (rn(8)) {
    case 0: return 0.0F;
    case 1: return bf(drv_rword());
    case 2: return drv_randf(1e10F);
    case 3: return drv_randf(2.0F);
    case 4: return 1.0F;
    default: return drv_randf(0.5F) + 0.5F;
    }
}
static f32 rframe(void)
{
    u32 m = rn(8);
    return m == 0 ? 0.f : m < 4 ? (f32) (int) drv_randf(40.f) : m < 7 ? drv_randf(60.f) : drv_randf(3000.f);
}

#define BUF 256
#define NP 3
static HSD_PObj po[NP];
static HSD_ShapeSet ss[NP];
static f32 bp[NP][8];
static u8 bufs[64][BUF];
static int nbuf;
static int objs[4];

static int put_varint(u8* buf, int n, u32 v) { do { u8 b = v & 0x7F; v >>= 7; if (v) b |= 0x80; buf[n++] = b; } while (v); return n; }
static u8 rfrac(void)
{
    static const u8 kinds[] = { 0x00, 0x20, 0x40, 0x60, 0x80 };
    u8 k = kinds[rn(5)];
    if (k == 0) return 0;
    return k | (u8) rn(9);
}
static int put_val(u8* buf, int n, u8 frac)
{
    u8 k = frac & 0xE0;
    int sz = (k == 0x20 || k == 0x40) ? 2 : (k == 0x60 || k == 0x80) ? 1 : 0;
    if (frac == 0) {
        u32 b = fb(drv_randf(10.f)); int i;
        for (i = 0; i < 4; i++) buf[n++] = (u8) (b >> (8 * i));
        return n;
    }
    while (sz--) buf[n++] = (u8) drv_rand();
    return n;
}
static int gen_stream(u8* buf, u8 fv, u8 fs)
{
    int n = 0;
    memset(buf, 0, BUF);
    while (n < BUF - 64) {
        u32 op = 1 + rn(6);
        u32 nb = 1 + rn(4), k;
        buf[n++] = (u8) ((op & 0xF) | (((nb - 1) & 7) << 4));
        for (k = 0; k < nb && n < BUF - 32; k++) {
            if (op == 5) { n = put_val(buf, n, fs); continue; }
            n = put_val(buf, n, fv);
            if (op == 4) n = put_val(buf, n, fs);
            n = put_varint(buf, n, rn(12));
        }
        if (rn(6) == 0) break;
    }
    return n;
}

/* A fresh AObjDesc (or NULL) for pobj slot i; track types stay in the shape set's weight range. */
static HSD_FObjDesc fd[NP][3];
static HSD_AObjDesc ad[NP];
static HSD_AObjDesc* gen_desc(int i)
{
    int k, nf = 1 + rn(3);
    u32 fl;
    if (rn(6) == 0) { drv_u32(0); return NULL; }
    for (k = 0; k < nf; k++) {
        u8* b = bufs[nbuf++ % 64];
        fd[i][k].next = k + 1 < nf ? &fd[i][k + 1] : 0;
        fd[i][k].frac_value = rfrac(); fd[i][k].frac_slope = rfrac();
        fd[i][k].type = (u8) (HSD_A_S_W0 + rn(ss[i].nb_shape));
        fd[i][k].dummy0 = 0;
        fd[i][k].length = (u32) gen_stream(b, fd[i][k].frac_value, fd[i][k].frac_slope);
        fd[i][k].ad = b;
        fd[i][k].startframe = (f32) (int) drv_randf(20.f);
    }
    fl = rn(2) ? 1u << 29 : 0;
    if (rn(4) == 0) fl |= 1u << 28;
    if (rn(4) == 0) fl |= 1u << 30;
    ad[i].flags = fl; ad[i].end_frame = rframe(); ad[i].fobjdesc = rn(10) == 0 ? 0 : fd[i]; ad[i].obj_id = 0;
    if (!ad[i].fobjdesc) nf = 0;
    drv_u32(1); drv_u32(ad[i].flags); drv_u32(fb(ad[i].end_frame)); drv_u32(nf);
    for (k = 0; k < nf; k++) {
        drv_u32(fd[i][k].length); drv_u32(fb(fd[i][k].startframe)); drv_u32(fd[i][k].type);
        drv_u32(fd[i][k].frac_value); drv_u32(fd[i][k].frac_slope);
        drv_hex(fd[i][k].ad, BUF);
    }
    return &ad[i];
}

static void dump_aobj(HSD_AObj* a)
{
    HSD_FObj* f; u32 n = 0;
    if (!a) { drv_u32(0); return; }
    drv_u32(1);
    drv_u32(a->flags); drv_u32(fb(a->curr_frame)); drv_u32(fb(a->rewind_frame)); drv_u32(fb(a->end_frame));
    drv_u32(fb(a->framerate));
    for (f = a->fobj; f; f = f->next) n++;
    drv_u32(n);
    for (f = a->fobj; f; f = f->next) {
        drv_u32(f->flags); drv_u32(f->op); drv_u32(f->op_intrp); drv_u32(f->nb_pack); drv_u32(f->fterm);
        drv_u32(fb(f->time)); drv_u32(fb(f->p0)); drv_u32(fb(f->p1)); drv_u32(fb(f->d0)); drv_u32(fb(f->d1));
        drv_u32(f->ad ? (u32) (f->ad - f->ad_head) : 0);
    }
}
static u32 oidx(void* p)
{
    int i;
    if (p == NULL) return 0xFFFFFFFFu;
    for (i = 0; i < 4; i++) if (p == &objs[i]) return i;
    return 0xEEEEEEEEu;
}
static void dump(void)
{
    int i, k;
    drv_hash_begin();
    for (i = 0; i < NP; i++) {
        drv_u32(po[i].flags);
        if (pobj_type((&po[i])) == POBJ_SHAPEANIM) {
            drv_u32(ss[i].flags);
            if (ss[i].flags & SHAPESET_ADDITIVE) for (k = 0; k < 8; k++) drv_u32(fb(bp[i][k]));
            else drv_u32(fb(ss[i].blend.bl));
            dump_aobj(ss[i].aobj);
        }
    }
    for (i = 0; i < 2; i++) { drv_u32(oidx(mtx_mark[i].obj)); drv_u32(mtx_mark[i].mark); }
    drv_u32((u32) HSD_AObj_804D762C); drv_u32((u32) HSD_AObj_804D7630);
    drv_hash_end();
}

static void reset(void)
{
    int i, k;
    for (i = 0; i < NP; i++) {
        if (pobj_type((&po[i])) == POBJ_SHAPEANIM && ss[i].aobj) HSD_AObjRemove(ss[i].aobj);
    }
    memset(po, 0, sizeof po); memset(ss, 0, sizeof ss);
    HSD_AObj_804D762C = HSD_AObj_804D7630 = 0;
    drv_u32(0xFFFFFFFFu);
    for (i = 0; i < NP; i++) {
        u32 t = rn(4);
        po[i].flags = (u16) ((drv_rand() & 0xCFFF) | ((t == 0 ? 0 : t == 3 ? 2 : 1) << 12));
        po[i].next = i + 1 < NP ? &po[i + 1] : NULL;
        drv_u32(po[i].flags);
        if (pobj_type((&po[i])) == POBJ_SHAPEANIM) {
            ss[i].flags = (u16) (drv_rand() & 0xFFFF);
            ss[i].nb_shape = (u16) (1 + rn(8));
            for (k = 0; k < 8; k++) bp[i][k] = rf();
            if (ss[i].flags & SHAPESET_ADDITIVE) ss[i].blend.bp = bp[i];
            else ss[i].blend.bl = rf();
            drv_u32(ss[i].flags); drv_u32(ss[i].nb_shape);
            for (k = 0; k < 8; k++) drv_u32(fb(bp[i][k]));
            drv_u32(fb(ss[i].blend.bl));
            po[i].u.shape_set = &ss[i];
            ss[i].aobj = rn(2) ? HSD_AObjLoadDesc(gen_desc(i)) : (drv_u32(0), NULL);
        }
    }
    dump();
    drv_eol();
}

static u32 rpflags(void)
{
    u32 f = drv_rword();
    return rn(2) ? f | POBJ_ANIM : f & ~POBJ_ANIM;
}
static int all_shape(int from)
{
    int i;
    for (i = from; i < NP; i++) if (pobj_type((&po[i])) != POBJ_SHAPEANIM) return 0;
    return 1;
}

void orc_pobj_run(int nops)
{
    int n;
    for (n = 0; n < nops; n++) {
        u32 op = rn(13);
        int i = (int) rn(NP), nul = rn(10) == 0;
        HSD_PObj* p = nul ? NULL : &po[i];
        if (n % 40 == 0) reset();
        if ((op == 3 && !nul && pobj_type((&po[i])) != POBJ_SHAPEANIM) || (op == 4 && !all_shape(i)))
            op = 0;
        drv_u32(op);
        switch (op) {
        case 0:
            drv_u32(nul); drv_u32(i); drv_sep();
            drv_u32(HSD_PObjGetFlags(p));
            break;
        case 1: case 2: {
            u32 f = rpflags();
            drv_u32(nul); drv_u32(i); drv_u32(f); drv_sep();
            if (op == 1) HSD_PObjRemoveAnimByFlags(p, f);
            else HSD_PObjRemoveAnimAllByFlags(p, f);
            break;
        }
        case 3: {
            HSD_ShapeAnim sa;
            int nosa = rn(8) == 0;
            drv_u32(nul); drv_u32(i); drv_u32(nosa);
            sa.next = NULL;
            if (!nul && !nosa) sa.aobjdesc = gen_desc(i);
            drv_sep();
            HSD_PObjAddAnim(p, nosa ? NULL : &sa);
            break;
        }
        case 4: {
            HSD_ShapeAnim sa[NP];
            int k, none = rn(8) == 0;
            drv_u32(nul); drv_u32(i); drv_u32(none);
            for (k = i; k < NP; k++) {
                sa[k].next = k + 1 < NP ? &sa[k + 1] : NULL;
                if (!nul && !none) sa[k].aobjdesc = gen_desc(k);
            }
            drv_sep();
            HSD_PObjAddAnimAll(p, none ? NULL : &sa[i]);
            break;
        }
        case 5: case 6: {
            f32 fr = rframe();
            u32 f = rpflags();
            drv_u32(nul); drv_u32(i); drv_u32(fb(fr)); drv_u32(f); drv_sep();
            if (op == 5) HSD_PObjReqAnimByFlags(p, fr, f);
            else HSD_PObjReqAnimAllByFlags(p, fr, f);
            break;
        }
        case 7: {
            u32 ty;
            f32 fv = rf();
            HSD_ObjData d;
            if (!nul && pobj_type((&po[i])) == POBJ_SHAPEANIM && (ss[i].flags & SHAPESET_ADDITIVE))
                ty = HSD_A_S_W0 + rn(8);
            else
                ty = rn(40);
            drv_u32(nul); drv_u32(i); drv_u32(ty); drv_u32(fb(fv)); drv_sep();
            d.fv = fv;
            PObjUpdateFunc(p, ty, &d);
            break;
        }
        case 8: case 9: {
            int k, reps = 1 + rn(4);
            drv_u32(nul); drv_u32(i); drv_u32(reps); drv_sep();
            for (k = 0; k < reps; k++) {
                if (op == 8) HSD_PObjAnim(p);
                else HSD_PObjAnimAll(p);
            }
            break;
        }
        case 10: {
            u32 sub = rn(3), mark = drv_rword();
            int idx = (int) rn(5) - 1, o = (int) rn(5);
            void* obj = o == 4 ? NULL : &objs[o];
            void* go; u32 gm;
            if (sub == 1 && idx < 0) idx = 0; /* the C stores out of bounds there */
            drv_u32(sub); drv_u32((u32) idx); drv_u32(oidx(obj)); drv_u32(mark); drv_sep();
            if (sub == 0) HSD_PObjClearMtxMark(obj, mark);
            else if (sub == 1) HSD_PObjSetMtxMark(idx, obj, mark);
            else { HSD_PObjGetMtxMark(idx, &go, &gm); drv_u32(oidx(go)); drv_u32(gm); }
            break;
        }
        default: {
            u8 src[6]; f32 dst[3];
            int k, scale, kind = (int) (op - 11) * 2 + (int) rn(2);
            u32 m = rn(4);
            for (k = 0; k < 6; k++) src[k] = (u8) drv_rand();
            scale = m == 0 ? 1 << rn(16) : m == 1 ? (int) drv_rword() : m == 2 ? 0 : (int) rn(300) - 150;
            drv_u32(kind); drv_u32(scale);
            for (k = 0; k < 6; k++) drv_u32(src[k]);
            drv_sep();
            if (kind == 0) decode_u8_xyz(src, dst, scale);
            else if (kind == 1) decode_s8_xyz(src, dst, scale);
            else if (kind == 2) decode_u16_xyz(src, dst, scale);
            else decode_s16_xyz(src, dst, scale);
            for (k = 0; k < 3; k++) drv_u32(dst[k] != dst[k] ? 0x7FC00000u : fb(dst[k]));
            break;
        }
        }
        dump();
        drv_eol();
    }
}
