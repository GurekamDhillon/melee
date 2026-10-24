/* TU: src/melee/ft/kinds/ftCommon/ftCo_09F4.c */
/* C oracle for src/tu/ftCo_09f4.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

s8 HSD_GObj_LightKind;
static HSD_LObj L0, L1, LF, LS, LN;
static HSD_CObj CO;
static ColorOverlay OV;
static HSD_JObj JO;
static HSD_GObj LG;
static LightList* GRL[2];

static u32 EP(const void* p)
{
    if (p == 0) return 0;
    if (p == &L0) return 1;
    if (p == &L1) return 2;
    if (p == &LF) return 3;
    if (p == &LS) return 4;
    if (p == &LN) return 5;
    if (p == &CO) return 6;
    if (p == &LG) return 7;
    if (p == &GO) return 8;
    if (p == &JO) return 9;
    if (p == GRL) return 10;
    if (p == node2) return 11;
    return 0xFFFF;
}
static void tv(const Vec3* v) { T(B(v->x)); T(B(v->y)); T(B(v->z)); }

HSD_GObj* GObj_Create(u16 c, u8 p, u8 pr) { T(1); T(c); T(p); T(pr); return &LG; }
LightList** Ground_801C49B4(void) { T(2); return GRL; }
HSD_LObj* lb_80011AC4(LightList** l)
{
    float* f;
    T(3); T(EP(l));
    if (l == node2) {
        /* the static light data: list -> node1 -> node0 (flags, colour, the position record's 5 words) */
        LightList* n1 = l[0];
        HSD_LightDesc* d = n1->desc;
        f = (float*) d->position;
        T(n1->desc == &node0); T(n1->anims == NULL); T(l[1] == NULL);
        T(d->flags); T(d->attnflags); T(d->color.r); T(d->color.g); T(d->color.b); T(d->color.a);
        T(d->class_name == NULL); T(d->next == NULL); T(d->interest == NULL);
        T(B(f[0])); T(B(f[1])); T(B(f[2])); T(B(f[3])); T(B(f[4]));
        return &LF;
    }
    return &LS;
}
HSD_LObj* lb_8000CDC0(HSD_LObj* l) { T(4); T(EP(l)); return &LN; }
void Ground_801C2374(HSD_LObj* l) { T(5); T(EP(l)); }
void HSD_GObjObject_80390A70(HSD_GObj* g, u8 kind, void* o) { T(6); T(EP(g)); T(kind); T(EP(o)); }
void GObj_SetupGXLink(HSD_GObj* g, GObj_RenderFunc cb, u8 gx, u32 prio)
{ T(7); T(EP(g)); T(cb == ftCo_8009F54C); T(gx); T(prio); }
HSD_GObjProc* HSD_GObj_SetupProc(HSD_GObj* g, HSD_GObjEvent f, u8 prio)
{ T(8); T(EP(g)); T(f == ftCo_8009F480); T(prio); return NULL; }
void HSD_LObjAnimAll(HSD_LObj* l) { T(9); T(EP(l)); }
void HSD_LObjReqAnimAll(HSD_LObj* l, f32 f) { T(10); T(EP(l)); T(B(f)); }
void HSD_LObj_803668EC(HSD_LObj* l) { T(11); T(EP(l)); }
void HSD_LObjSetupInit(HSD_CObj* c) { T(12); T(EP(c)); }
HSD_CObj* HSD_CObjGetCurrent(void) { T(13); return &CO; }
void HSD_LObjSetPosition(HSD_LObj* l, Vec3* v) { T(14); T(EP(l)); tv(v); }
void HSD_LObjSetColor(HSD_LObj* l, GXColor c) { T(15); T(EP(l)); T(c.r); T(c.g); T(c.b); T(c.a); }
void HSD_LObjSetFlags(HSD_LObj* l, u32 f) { T(16); T(EP(l)); T(f); }
void HSD_LObjClearFlags(HSD_LObj* l, u32 f) { T(17); T(EP(l)); T(f); }
void HSD_LObjSetCurrentAll(HSD_LObj* l) { T(18); T(EP(l)); }
void HSD_LObjAddCurrent(HSD_LObj* l) { T(19); T(EP(l)); }
ColorOverlay* ftCo_800C0658(Fighter* fp) { T(20); T(fp == &F); return &OV; }

static void hx(u32 v) { HM(v); }
int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 3000; k++) {
            tu_case(op, k);
            zero(&OV, sizeof OV); zero(&JO, sizeof JO);
            HSD_GObj_LightKind = (s8) (rnd() % 20);
            lobj0 = (rnd() & 1) ? &L0 : NULL;
            lobj1 = (rnd() & 1) ? &L1 : NULL;
            GO.hsd_obj = &JO; GO.user_data = &F; F.gobj = &GO;
            JO.rotate.y = rw();
            F.x588 = (rnd() & 1) ? &LF : &LS;
            F.x2221_b3 = rnd() & 1;
            OV.x7C_flag2 = rnd() & 1; OV.x7C_light_enable = rnd() & 1;
            OV.x74_light_rot_x = rw(); OV.x78_light_rot_yz = rw();
            OV.x50_light_color.r = (u8) rnd(); OV.x50_light_color.g = (u8) rnd(); OV.x50_light_color.b = (u8) rnd(); OV.x50_light_color.a = (u8) rnd();
            switch (op) {
            case 0: ftCo_8009F480(&GO); break;
            case 1: ftCo_8009F4A4(); break;
            case 2: ftCo_8009F54C(&GO, (int) rnd()); break;
            case 3: ftCo_8009F578(&F); break;
            case 4: ftCo_8009F5AC(&F); break;
            case 5: ftCo_8009F75C(&F, rnd() & 1); break;
            case 6: ftCo_8009F7F8(&F); break;
            }
            H = 0x811C9DC5u;
            hx(EP(F.x588)); hx(F.x2221_b3); hx(EP(lobj0)); hx(EP(lobj1));
            printf("%d %d %08X %08X %08X\n", op, k, 0u, H, TR);
        }
    }
    return 0;
}
