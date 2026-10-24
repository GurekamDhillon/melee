/* Native source oracle. Test fixtures/stubs only; all functions under test are included. */
#include "tev_native.c"
#include "src/sysdolphin/baselib/state.c"
extern int printf(const char*, ...);
extern void abort(void);
/* math_ppc.h emits sqrtf with gnu89-inline; no state/TEV function calls it. */
double __frsqrte(double x) { abort(); return 0; }
static u32 trace[4096], trace_count;
static void T(u32 x) { trace[trace_count++] = x; }
static u32 bits(float f) { union { float f; u32 u; } v; v.f=f; return v.u; }
static float flt(u32 u) { union { float f; u32 u; } v; v.u=u; return v.f; }
#include "gx_stubs.h"
void __assert(const char* f, u32 l, const char* s) { printf("ASSERT %s %u %s\n",f,l,s); abort(); }
static HSD_LObj lights[4];
static u32 diffuse, specular, alpha;
static int nlights;
s32 HSD_LObjGetLightMaskSpecular(void) { return specular; }
GXLightID HSD_LObjGetLightMaskDiffuse(void) { return diffuse; }
s32 HSD_LObjGetLightMaskAlpha(void) { return alpha; }
int HSD_LObjGetNbActive(void) { return nlights; }
HSD_LObj* HSD_LObjGetActiveByIndex(int i) { return &lights[i]; }
HSD_LObj* HSD_LObjGetActiveByID(GXLightID id) {
    int i; for(i=0;i<nlights;i++) if((lights[i].flags&3)==0) return &lights[i]; return NULL;
}
void HSD_LObjSetup(HSD_LObj* p, GXColor c, float s) {
    T(29); T(p->id); T(c.r); T(c.g); T(c.b); T(c.a); T(bits(s));
}
void HSD_MulColor(GXColor* a, GXColor* b, GXColor* d) {
    d->r=a->r*b->r/255; d->g=a->g*b->g/255; d->b=a->b*b->b/255; d->a=a->a*b->a/255;
}
void HSD_ObjAllocInit(HSD_ObjAllocData* d, size_t size, u32 align) {
    T(30); T(d==&render_alloc_data?1:d==&tevreg_alloc_data?2:3); T(size); T(align);
    d->size=size; d->align=align;
}
static u32 words[512], nw;
static void W(u32 x) { words[nw++]=x; }
static void color(GXColor c) { W(c.r); W(c.g); W(c.b); W(c.a); }
static void channel(HSD_Chan* c) {
    W(c->chan); W(c->flags); color(c->amb_color); color(c->mat_color);
    W(c->enable); W(c->amb_src); W(c->mat_src); W(c->light_mask); W(c->diff_fn); W(c->attn_fn);
}
static void alloc(HSD_ObjAllocData* d) {
    W(d->num_limit_flag); W(d->heap_limit_flag); W(d->freehead==NULL); W(d->used); W(d->free); W(d->peak);
    W(d->num_limit); W(d->heap_limit_size); W(d->heap_limit_num); W(d->size); W(d->align); W(d->next==NULL);
}
static void snapshot(void) {
    int i;
    W(state_line_width); W(state_line_tex_offsets); W(state_point_size); W(HSD_State_804D75BC);
    W(state_cull_mode); W(state_blend_type); W(state_src_factor); W(state_dst_factor); W(state_logic_op);
    W(state_z_enable); W(state_z_func); W(state_z_update); W(state_alpha_comp0); W(state_alpha_ref0);
    W(state_alpha_op); W(state_alpha_comp1); W(state_alpha_ref1); W(state_color_update); W(state_alpha_update);
    W(state_enable_dst_alpha); W(state_dst_alpha); W(state_before_tex); W(state_dither);
    color(matstate.ambient); color(matstate.diffuse); color(matstate.specular); W(matstate.alpha); W(bits(matstate.shininess));
    channel(&HSD_State_80405A38); channel(&HSD_State_80405A38_30); channel(&HSD_State_80405A38_60);
    channel(&HSD_State_80405A38_90); channel(&HSD_State_80405A38_C0); channel(&HSD_State_80405A38_F0);
    for(i=0;i<4;i++) channel(&prev_ch[i]);
    W(current_tev); W(num_tex_gens); W(prev_num_chans); W(committed_tev_stages);
    W(prev_mat_invalid[0]); W(prev_mat_invalid[1]); W(prev_amb_invalid[0]); W(prev_amb_invalid[1]);
    for(i=0;i<4;i++) { W(TevReg[i].a.r); W(TevReg[i].a.g); W(TevReg[i].a.b); W(TevReg[i].a.a); W(TevReg[i].c); }
    alloc(&render_alloc_data); alloc(&tevreg_alloc_data); alloc(&chan_alloc_data);
}
static u32 rnd(u32* s) { *s^=*s<<13; *s^=*s>>17; *s^=*s<<5; return *s; }
static GXColor make_color(u32* v) { GXColor c={v[0],v[1],v[2],v[3]}; return c; }
static void inputs(u32 seed,u32* v,HSD_Chan* ch,HSD_TevDesc* td,HSD_PEDesc* pe) {
    int i; u32* p;
    for(i=0;i<128;i++) v[i]=rnd(&seed);
    memset(ch,0,4*sizeof(*ch)); memset(td,0,4*sizeof(*td));
    for(i=0;i<4;i++) {
        p=&v[i*24]; ch[i].chan=(p[0]%7==6)?255:p[0]%7; ch[i].flags=p[1];
        ch[i].amb_color=make_color(p+2); ch[i].mat_color=make_color(p+6);
        ch[i].enable=p[10]&1; ch[i].amb_src=p[11]&1; ch[i].mat_src=p[12]&1;
        ch[i].light_mask=p[13]&255; ch[i].diff_fn=p[14]%3; ch[i].attn_fn=p[15]%3;
        ch[i].next=i<3?&ch[i+1]:NULL;
        td[i].next=i<3?&td[i+1]:NULL; td[i].flags=p[0]&1; td[i].stage=p[1]%16;
        td[i].coord=p[2]%8; td[i].map=p[3]%8; td[i].color=p[4]%6;
        td[i].u.tevconf.clr_op=p[5]%2; td[i].u.tevconf.clr_a=p[6]%16; td[i].u.tevconf.clr_b=p[7]%16;
        td[i].u.tevconf.clr_c=p[8]%16; td[i].u.tevconf.clr_d=p[9]%16; td[i].u.tevconf.clr_scale=p[10]%4;
        td[i].u.tevconf.clr_bias=p[11]%3; td[i].u.tevconf.clr_clamp=p[12]&1; td[i].u.tevconf.clr_out_reg=p[13]%4;
        td[i].u.tevconf.alpha_op=p[14]%2; td[i].u.tevconf.alpha_a=p[15]%8; td[i].u.tevconf.alpha_b=p[16]%8;
        td[i].u.tevconf.alpha_c=p[17]%8; td[i].u.tevconf.alpha_d=p[18]%8; td[i].u.tevconf.alpha_scale=p[19]%4;
        td[i].u.tevconf.alpha_bias=p[20]%3; td[i].u.tevconf.alpha_clamp=p[21]&1; td[i].u.tevconf.alpha_out_reg=p[22]%4;
        td[i].u.tevconf.ras_swap=p[3]%4; td[i].u.tevconf.tex_swap=p[4]%4;
        td[i].u.tevconf.kcsel=p[5]%32; td[i].u.tevconf.kasel=p[6]%32;
        lights[i].flags=(p[8]&31); lights[i].id=i; lights[i].color=make_color(p+16);
    }
    for(i=0;i<12;i++) ((u8*)pe)[i]=v[96+i];
    nlights=v[108]%5; diffuse=v[109]&255; specular=v[110]&255; alpha=(v[111]&1)?v[112]&255:0;
}
int main(void) {
    u32 n,i,k,seed,v[128],op; HSD_Chan ch[4]; HSD_TevDesc td[4]; HSD_PEDesc pe; float f;
    for(n=0;n<3200;n++) {
        seed=0x96997600u+n; op=n%50; inputs(seed,v,ch,td,&pe); trace_count=0; nw=0;
        f=flt((n/50)%4==0 ? (u32[]){0,0x80000000,0x7f800000,0xff800000,0x7fc00000,0xbf800000,0x3f800000,0x4f000000}[(n/200)%8] : v[120]);
        switch(op) {
        case 0: HSD_SetMaterialColor(make_color(v),make_color(v+4),make_color(v+8),f); break;
        case 1: HSD_SetMaterialShininess(f); break;
        case 2: HSD_StateSetLineWidth(v[0],v[1]); break;
        case 3: HSD_StateSetPointSize(v[0],v[1]); break;
        case 4: HSD_StateSetCullMode(v[0]%4); break;
        case 5: HSD_StateSetBlendMode(v[0]%4,v[1]%8,v[2]%8,v[3]%16); break;
        case 6: HSD_StateSetZMode((v[0]%3)-1,v[1]%8,(v[2]%3)-1); break;
        case 7: HSD_StateSetAlphaCompare(v[0]%8,v[1],v[2]%4,v[3]%8,v[4]); break;
        case 8: HSD_StateSetColorUpdate((v[0]%3)-1); break;
        case 9: HSD_StateSetAlphaUpdate((v[0]%3)-1); break;
        case 10: HSD_StateSetDstAlpha((v[0]%3)-1,v[1]); break;
        case 11: HSD_StateSetZCompLoc((v[0]%3)-1); break;
        case 12: HSD_StateSetDither((v[0]%3)-1); break;
        case 13: _HSD_StateInvalidatePrimitive(); break;
        case 14: _HSD_StateInvalidateVtxAttr(); break;
        case 15: _HSD_StateInvalidateRenderMode(); break;
        case 16: HSD_StateInvalidate(v[0]&127); break;
        case 17: HSD_SetupPEMode(v[0],v[1]&1?&pe:NULL); break;
        case 18: HSD_SetupChannelMode(v[0]&15); break;
        case 19: setupTevMode_last(); break;
        case 20: HSD_SetupRenderModeWithCustomPE(v[0],v[1]&1?&pe:NULL); break;
        case 21: HSD_SetupRenderMode(v[0]); break;
        case 22: HSD_RenderInitAllocData(); break;
        case 23: W(HSD_RenderGetAllocData()==&render_alloc_data); W(HSD_TevRegGetAllocData()==&tevreg_alloc_data); W(HSD_ChanGetAllocData()==&chan_alloc_data); break;
        case 24: {
            GXColor a=make_color(v),b=a;
            ((u8*)&b)[(n/50)%4]^=1;
            W(CompareRGB(&a,&b)); W(CompareRGBA(&a,&b)); CopyRGB(&a,&b); color(a); break;
        }
        case 25: HSD_SetupChannel(v[0]&1?&ch[0]:NULL); break;
        case 26: k=v[0]%5; if(k) ch[k-1].next=NULL; HSD_SetupChannelAll(k?ch:NULL); break;
        case 27: HSD_StateSetNumChans(v[0]%3); break;
        case 28: HSD_StateRegisterTexGen(v[0]%9==8?255:v[0]%9); break;
        case 29: HSD_StateSetNumTexGens(); break;
        case 30: HSD_StateInitTev(); break;
        case 31: W(HSD_StateGetNumTevStages()); W(HSD_StateGetCommittedTevStages()); break;
        case 32: W(HSD_StateAssignTev()); break;
        case 33: HSD_StateSetNumTevStages(); break;
        case 34: HSD_SetupTevStage(td); break;
        case 35: k=v[0]%5; if(k) td[k-1].next=NULL; HSD_SetupTevStageAll(k?td:NULL); break;
        case 36: W(HSD_Channel2Num(v[0]%7==6?255:v[0]%7)); break;
        case 37: W(HSD_Index2TevStage(v[0]%16)); break;
        case 38: W(HSD_TevStage2Index(v[0]%16)); break;
        case 39: W(HSD_TevStage2Num(v[0]%16)); break;
        case 40: for(i=0;i<4;i++) { TevReg[i].a=(GXColorS10){v[i*5],v[i*5+1],v[i*5+2],v[i*5+3]}; TevReg[i].c=v[i*5+4]%3; } HSD_SetTevRegAll(); break;
        case 41: W(HSD_TexCoordID2Num(v[0]%9==8?255:v[0]%9)); break;
        case 42: ChanUpdateFunc(v[0]&1?ch:NULL,(n/50)%16,&f); channel(ch); break;
        case 43: _HSD_StateInvalidateColorChannel(); break;
        case 44: _HSD_StateInvalidateTevStage(); break;
        case 45: for(i=0;i<4;i++) TevReg[i].c=v[i]; _HSD_StateInvalidateTevRegister(); break;
        case 46: _HSD_StateInvalidateTexCoordGen(); break;
        case 47: HSD_StateSetLineWidth(v[0],v[1]); HSD_StateSetLineWidth(v[0],v[1]); HSD_StateSetPointSize(v[2],v[3]); HSD_StateSetPointSize(v[2],v[3]); break;
        case 48: HSD_SetupChannel(ch); HSD_SetupChannel(ch); break;
        case 49: HSD_StateAssignTev(); HSD_StateAssignTev(); HSD_StateSetNumTevStages(); HSD_SetupTevStageAll(td); W(HSD_StateGetCommittedTevStages()); break;
        }
        printf("%u %08x %u",op,seed,nw); for(i=0;i<nw;i++) printf(" %08x",words[i]);
        nw=0; snapshot(); printf(" %u",nw); for(i=0;i<nw;i++) printf(" %08x",words[i]);
        printf(" %u",trace_count); for(i=0;i<trace_count;i++) printf(" %08x",trace[i]); printf("\n");
    }
    return 0;
}
