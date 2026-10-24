/* TU: src/melee/lb/lbspdisplay.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/lbtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#include "lbtu_common.h"
static HSD_JObj JO[9];
static HSD_DObj DO[9];
static HSD_MObj MO[9];
static HSD_ImageDesc IMG[1];
static GXTexObj TEX[1];
static HSD_GObj GO[1];
static HSD_CObj CO[1];
static CameraBlurData BD[1];
static LightList LL[4];
static HSD_LObj LO[4];
static HSD_LightDesc LD[4];
static HSD_CameraDescPerspective CD[1];
static unsigned char MEM[256];
static HSD_JObj* OUT[6];
static u16 IX[6];
static LightList* LS[5];
typedef struct CameraBlurData CameraBlurData;
static HSD_LightAnim* AN[1];
static void evt(HSD_GObj* p) { T(9001); p->p_priority ^= 7; }
static void draw(HSD_GObj* p, int pass) { T(9002); T(pass); }
#include "lbspdisplay_gen.h"

static void setup(void)
{
    int i, k;
    (void) i; (void) k;
    zero(JO, sizeof JO);
    zero(DO, sizeof DO);
    zero(MO, sizeof MO);
    zero(IMG, sizeof IMG);
    zero(TEX, sizeof TEX);
    zero(GO, sizeof GO);
    zero(CO, sizeof CO);
    zero(BD, sizeof BD);
    zero(LL, sizeof LL);
    zero(LO, sizeof LO);
    zero(LD, sizeof LD);
    zero(CD, sizeof CD);
    zero(MEM, sizeof MEM);
    zero(OUT, sizeof OUT);
    zero(IX, sizeof IX);
    zero(LS, sizeof LS);
    for (i = 0; i < 9; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < 9; i++) fill_HSD_DObj(&DO[i]);
    for (i = 0; i < 9; i++) fill_HSD_MObj(&MO[i]);
    for (i = 0; i < 1; i++) fill_HSD_ImageDesc(&IMG[i]);
    for (i = 0; i < 1; i++) fill_GXTexObj(&TEX[i]);
    for (i = 0; i < 1; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < 1; i++) fill_HSD_CObj(&CO[i]);
    for (i = 0; i < 1; i++) fill_CameraBlurData(&BD[i]);
    for (i = 0; i < 4; i++) fill_LightList(&LL[i]);
    for (i = 0; i < 4; i++) fill_HSD_LObj(&LO[i]);
    for (i = 0; i < 4; i++) fill_HSD_LightDesc(&LD[i]);
    for (i = 0; i < 1; i++) fill_HSD_CameraDescPerspective(&CD[i]);
    int n = rnd()%5;
    for(k=0;k<4;k++) { LL[k].desc=&LD[k]; LL[k].anims=(rnd()&1)?AN:0; LS[k]=k<n?&LL[k]:0; }
    for(k=0;k<9;k++) { JO[k].flags &= ~((1u<<5)|(1u<<14)|(1u<<12)); JO[k].flags |= (rnd()%8==0)?(1u<<12):0; JO[k].u.dobj=(rnd()&1)?&DO[k]:0; DO[k].mobj=&MO[k]; DO[k].next=(k<8 && (rnd()&1))?&DO[k+1]:0; }
    JO[0].child=&JO[1]; JO[1].parent=JO[2].parent=JO[3].parent=JO;
    JO[1].next=&JO[2]; JO[2].next=&JO[3]; JO[1].child=&JO[4]; JO[4].next=&JO[5]; JO[4].parent=JO[5].parent=&JO[1];
    JO[2].child=&JO[6]; JO[6].parent=&JO[2]; JO[6].child=&JO[7]; JO[7].parent=&JO[6]; JO[0].next=&JO[8];
    for(k=0;k<6;k++) IX[k]=rnd()%14;
    IMG[0].width=(rnd()%641); IMG[0].height=(rnd()%481); IMG[0].image_ptr=MEM;
    TEX[0].dummy[0]=IMG[0].width; TEX[0].dummy[1]=IMG[0].height;
    BD[0].mode=rnd()%3; BD[0].efb_copy=IMG; BD[0].callback=(rnd()&1)?evt:0;
    GO[0].user_data=BD; GO[0].hsd_obj=CO;

}
static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
for(k=0;k<9;k++) { HM(PI(JO[k].u.dobj)); JO[k].u.dobj=0; }
    for (i = 0; i < 9; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 9; i++) hash_HSD_DObj(&DO[i]);
    for (i = 0; i < 9; i++) hash_HSD_MObj(&MO[i]);
    for (i = 0; i < 1; i++) hash_HSD_ImageDesc(&IMG[i]);
    for (i = 0; i < 1; i++) hash_GXTexObj(&TEX[i]);
    for (i = 0; i < 1; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < 1; i++) hash_HSD_CObj(&CO[i]);
    for (i = 0; i < 1; i++) hash_CameraBlurData(&BD[i]);
    for (i = 0; i < 4; i++) hash_LightList(&LL[i]);
    for (i = 0; i < 4; i++) hash_HSD_LObj(&LO[i]);
    for (i = 0; i < 4; i++) hash_HSD_LightDesc(&LD[i]);
    for (i = 0; i < 1; i++) hash_HSD_CameraDescPerspective(&CD[i]);
for(k=0;k<6;k++) HM(PI(OUT[k]));
}
int main(void)
{
    int op, k;
    preg(JO, sizeof JO);
    preg(DO, sizeof DO);
    preg(MO, sizeof MO);
    preg(IMG, sizeof IMG);
    preg(TEX, sizeof TEX);
    preg(GO, sizeof GO);
    preg(CO, sizeof CO);
    preg(BD, sizeof BD);
    preg(LL, sizeof LL);
    preg(LO, sizeof LO);
    preg(LD, sizeof LD);
    preg(CD, sizeof CD);
    preg(MEM, sizeof MEM);
    preg(OUT, sizeof OUT);
    preg(IX, sizeof IX);
    preg(LS, sizeof LS);
    freg_all();
freg(evt); freg(draw);
    for (op = 0; op < 20; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {   ret = PI(lb_80011AC4(LS)); } break; /* lb_80011AC4 */
            case 1: {  u32 v1 = (u32) (fv()); lb_80011B74(&DO[0], v1); } break; /* lb_80011B74 */
            case 2: { ret=checkJObjFlags(&JO[0]); } break; /* checkJObjFlags */
            case 3: {  u32 v1 = (u32) (fv()); lb_80011C18(&JO[0], v1); } break; /* lb_80011C18 */
            case 4: { ret=lb_80011E24(JO, OUT, (int)IX[0], (int)IX[1], (int)IX[2], (int)IX[3], (int)IX[4], (int)IX[5], -1); } break; /* lb_80011E24 */
            case 5: {  int v3 = (int) ((int)(fv()%7)-1); ret = (u32) lb_8001204C(&JO[0], OUT, IX, v3); } break; /* lb_8001204C */
            case 6: {  s16 v1 = (s16) ((int) fv()); ret = PI(setImageFromPreloadedArchive(&IMG[0], v1)); } break; /* setImageFromPreloadedArchive */
            case 7: { IMG[0].image_ptr=0; int v1 = (int) ((int) fv()); int v2 = (int) ((int) fv()); GXTexFmt v3 = (GXTexFmt) ((int) fv()); s16 v4 = (s16) ((int) fv()); ret = PI(lb_800121FC(&IMG[0], v1, v2, v3, v4)); } break; /* lb_800121FC */
            case 8: {  u16 v1 = (u16) (fv()); u16 v2 = (u16) (fv()); bool v3 = (bool) ((int) fv()); lb_800122C8(&IMG[0], v1, v2, v3); } break; /* lb_800122C8 */
            case 9: {  f32 v2 = (f32) (rf()); lb_800122F0(&IMG[0], &TEX[0], v2); } break; /* lb_800122F0 */
            case 10: {  float v1 = (float) (rf()); float v2 = (float) (rf()); float v3 = (float) (rf()); float v4 = (float) (rf()); float v5 = (float) (rf()); float v6 = (float) (rf()); lb_8001271C(&TEX[0], v1, v2, v3, v4, v5, v6); } break; /* lb_8001271C */
            case 11: {   lb_8001285C(&IMG[0], &TEX[0]); } break; /* lb_8001285C */
            case 12: { setTevAlpha((u8)fv()); } break; /* setTevAlpha */
            case 13: {  u8 v1 = (u8) (fv()); u8 v2 = (u8) (fv()); f32 v3 = (f32) (rf()); f32 v4 = (f32) (rf()); f32 v5 = (f32) (rf()); f32 v6 = (f32) (rf()); f32 v7 = (f32) (rf()); lb_80012994(&IMG[0], v1, v2, v3, v4, v5, v6, v7); } break; /* lb_80012994 */
            case 14: {   fn_80013614(&GO[0]); } break; /* fn_80013614 */
            case 15: {   fn_800138AC(MEM); } break; /* fn_800138AC */
            case 16: {  HSD_GObjEvent v1 = (HSD_GObjEvent) ((fv()&1)?evt:0); lb_800138CC(&GO[0], v1); } break; /* lb_800138CC */
            case 17: {  s8 v1 = (s8) ((int) fv()); lb_800138D8(&GO[0], v1); } break; /* lb_800138D8 */
            case 18: {  GObj_RenderFunc v1 = (GObj_RenderFunc) ((fv()&1)?draw:0); int v2 = (int) ((int) fv()); s8 v3 = (s8) ((int) fv()); float v4 = (float) (rf()); float v5 = (float) (rf()); float v6 = (float) (rf()); float v7 = (float) (rf()); lb_800138EC(&IMG[0], v1, v2, v3, v4, v5, v6, v7); } break; /* lb_800138EC */
            case 19: {   ret = PI(lb_80013B14(&CD[0])); } break; /* lb_80013B14 */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
