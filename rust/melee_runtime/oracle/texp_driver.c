/* Synthetic, deterministic native-C oracle; includes every local texp.c helper.
 * Each output word is a return, named field in texp_fields.h, or external call
 * argument. Pointers are normalized to regions; padding is never compared. */
#include "../../../src/sysdolphin/baselib/texp.c"
extern void emit(u32);
extern void oracle_abort(void);
static HSD_TExp nodes[12], pool[64];
static HSD_TExpTevDesc descs[4];
static HSD_TObj textures[2];
static double values[8][4];
static int used;
static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static u32 PI(const void* p) {
    unsigned long long a=(unsigned long long)p;
    const void* bases[5]={nodes,pool,descs,textures,values};
    unsigned sizes[5]={sizeof(nodes),sizeof(pool),sizeof(descs),sizeof(textures),sizeof(values)};
    int i;
    if (!p) return 0;
    if (p==HSD_TEXP_TEX) return 0xffffffff;
    if (p==HSD_TEXP_RAS) return 0xfffffffe;
    for(i=0;i<5;i++) if(a>=(unsigned long long)bases[i] && a<(unsigned long long)bases[i]+sizes[i])
        return ((i+1)<<16)+(u32)(a-(unsigned long long)bases[i]);
    return 0xffff0000;
}
#define HM emit
#define HDBG 0
extern int printf(const char*, ...);
#include "texp_fields.h"
static void dump_desc(HSD_TExpTevDesc* p) {
    emit(PI(p->desc.next)); emit(p->desc.flags); emit(p->desc.stage);
    emit(p->desc.coord); emit(p->desc.map); emit(p->desc.color);
    dump_HSD_TevConf(&p->desc.u.tevconf); emit(PI(p->tobj));
}
void __assert(const char* f,u32 l,const char* c) { printf("ASSERT %u %s\n",l,c); oracle_abort(); }
void HSD_Panic(const char* f,u32 l,const char* c) { oracle_abort(); }
void* hsdAllocMemPiece(s32 n) { emit(100); emit(n); return &pool[used++]; }
void hsdFreeMemPiece(void* p,s32 n) { emit(101); emit(PI(p)); emit(n); }
void GXPixModeSync(void) { emit(102); }
void GXSetTevColor(GXTevRegID id,GXColor c) { emit(103); emit(id); emit(c.r); emit(c.g); emit(c.b); emit(c.a); }
void GXSetTevKColor(GXTevKColorID id,GXColor c) { emit(104); emit(id); emit(c.r); emit(c.g); emit(c.b); emit(c.a); }
void HSD_StateInvalidate(s32 mask) { emit(105); emit(mask); }
int HSD_StateAssignTev(void) { emit(106); return 0; }
void HSD_SetupTevStage(HSD_TevDesc* p) { emit(107); dump_desc((HSD_TExpTevDesc*)p); }
int HSD_Index2TevStage(int i) { emit(108); emit(i); return i; }
int HSD_TExpSimplify(HSD_TExp* p) { emit(109); emit(PI(p)); return 0; }
int HSD_TExpSimplify2(HSD_TExp* p) { emit(110); emit(PI(p)); return 0; }
int HSD_TExpMakeDag(HSD_TExp* p,HSD_TExpDag* d) { emit(111); emit(PI(p)); d[0].tev=&p->tev; return 1; }
void HSD_TExpSchedule(int n,HSD_TExpDag* d,HSD_TExp** order,HSD_TExpRes* r) {
    emit(112); emit(n); dump_HSD_TExpRes(r); order[0]=(HSD_TExp*)d[0].tev;
    order[0]->tev.c_dst=3; order[0]->tev.a_dst=3;
}
static void reset(u32 seed) {
    rs=seed; used=0;
    memset(nodes,0,sizeof(nodes)); memset(pool,0,sizeof(pool));
    memset(descs,0,sizeof(descs)); memset(textures,0,sizeof(textures)); memset(values,0,sizeof(values));
}
static void init_tev(HSD_TExp* p) {
    int i; memset(p,0,sizeof(*p)); p->type=HSD_TE_TEV;
    p->tev.c_op=rnd()%16; p->tev.a_op=rnd()%16;
    p->tev.c_ref=rnd()%3; p->tev.a_ref=rnd()%3;
    p->tev.c_clamp=1; p->tev.a_clamp=1;
    p->tev.c_dst=rnd()%4; p->tev.a_dst=rnd()%4;
    p->tev.c_bias=rnd()%3; p->tev.a_bias=rnd()%3;
    p->tev.c_scale=rnd()%4; p->tev.a_scale=rnd()%4;
    p->tev.tex_swap=p->tev.ras_swap=p->tev.kcsel=p->tev.kasel=255;
    p->tev.chan=255;
    for(i=0;i<4;i++) { p->tev.c_in[i].sel=7; p->tev.a_in[i].sel=7; p->tev.c_in[i].arg=15; p->tev.a_in[i].arg=7; }
}
static void init_cnst(HSD_TExp* p,int scalar) {
    memset(p,0,sizeof(*p)); p->type=HSD_TE_CNST;
    p->cnst.comp=scalar?HSD_TE_X:HSD_TE_RGB; p->cnst.ctype=rnd()%5;
    p->cnst.reg=255; p->cnst.idx=255; p->cnst.ref=rnd()%5; p->cnst.val=values;
}
static void snapshot(void) {
    int i; for(i=0;i<12;i++) {
        if(nodes[i].type==HSD_TE_TEV) dump_HSD_TETev(&nodes[i].tev);
        else if(nodes[i].type==HSD_TE_CNST) dump_HSD_TECnst(&nodes[i].cnst);
        else emit(nodes[i].type);
    }
    emit(used);
    for(i=0;i<used;i++) {
        if(pool[i].type==HSD_TE_TEV) dump_HSD_TETev(&pool[i].tev);
        else if(pool[i].type==HSD_TE_CNST) dump_HSD_TECnst(&pool[i].cnst);
        else emit(pool[i].type);
    }
}
void run_case(int op,int seed) {
    HSD_TExp* list=NULL; HSD_TExp* p; HSD_TExp* q; HSD_TExpRes res;
    HSD_TExpTevDesc* out=NULL; int i,j,r=0,cp=1,ap=1; u32 sel;
    reset(seed); memset(&res,0,sizeof(res));
    emit(op); emit(seed);
    init_tev(&nodes[0]); init_tev(&nodes[1]); init_cnst(&nodes[2],seed&1);
    switch(op) {
    case 0:
        if(seed%5==0) nodes[0].tev.c_ref=nodes[0].tev.a_ref=0x7fffffff;
        if(seed%7==0) nodes[1].tev.c_ref=nodes[1].tev.a_ref=(-2147483647-1);
        nodes[2].cnst.ref=255;
        emit(HSD_TExpGetType(NULL)); emit(HSD_TExpGetType(HSD_TEXP_TEX)); emit(HSD_TExpGetType(HSD_TEXP_RAS));
        for(i=0;i<3;i++) { HSD_TExpRef(&nodes[i],seed%7); emit(HSD_TExpGetType(&nodes[i])); HSD_TExpUnref(&nodes[i],seed%7); }
        nodes[0].tev.c_ref=1; nodes[0].tev.a_ref=0;
        nodes[0].tev.c_in[0].exp=&nodes[1]; nodes[0].tev.c_in[0].sel=1;
        nodes[0].tev.a_in[0].exp=&nodes[2]; HSD_TExpUnref(nodes,1); HSD_TExpUnref(NULL,0);
        break;
    case 1:
        p=HSD_TExpTev(&list); q=HSD_TExpCnst(values,HSD_TE_RGB,HSD_TE_F64,&list);
        emit(PI(HSD_TExpCnst(values,HSD_TE_RGB,HSD_TE_F64,&list)));
        emit(PI(HSD_TExpCnst(values,HSD_TE_0,HSD_TE_U8,&list)));
        p->tev.c_ref=seed%2; q->cnst.ref=(seed>>1)%2;
        list=HSD_TExpFreeList(list,seed%2?HSD_TE_ALL:HSD_TE_CNST,(seed>>2)%2); emit(PI(list));
        HSD_TExpFree(HSD_TEXP_TEX); break;
    case 2:
        HSD_TExpColorOp(nodes,seed%16,(seed/2)%3,(seed/3)%4,seed%3);
        HSD_TExpAlphaOp(nodes,(seed/3)%16,(seed/5)%3,(seed/7)%4,(seed/2)%3); break;
    case 3:
        sel=1+seed%15; i=seed%4;
        p=sel<=5?(seed%2?HSD_TEXP_TEX:HSD_TEXP_RAS):NULL;
        HSD_TExpColorInSub(&nodes[0].tev,sel,p,i);
        HSD_TExpAlphaInSub(&nodes[0].tev,sel>=7?sel:HSD_TE_A,sel>=7?NULL:p,i); break;
    case 4:
        nodes[2].cnst.comp=HSD_TE_X;
        HSD_TExpColorIn(nodes,1,&nodes[1],5,&nodes[1],6,&nodes[2],7,NULL);
        HSD_TExpAlphaIn(nodes,5,&nodes[1],6,&nodes[2],5,HSD_TEXP_TEX,5,HSD_TEXP_RAS); break;
    case 5: HSD_TExpOrder(nodes,seed%2?textures:NULL,seed%9); break;
    case 6:
        for(i=0;i<8;i++) { res.reg[i].color=rnd()%4; res.reg[i].alpha=rnd()%2; }
        nodes[2].cnst.reg=seed%3?255:(seed/3)%8; nodes[2].cnst.idx=seed%4;
        nodes[0].tev.c_in[0].exp=nodes+2; nodes[0].tev.a_in[0].exp=nodes+2;
        switch(seed%4) {
        case 0:r=AssignColorReg(&nodes[0].tev,0,&res);break;
        case 1:r=AssignAlphaReg(&nodes[0].tev,0,&res);break;
        case 2:r=AssignColorKonst(&nodes[0].tev,0,&res);break;
        case 3:r=AssignAlphaKonst(&nodes[0].tev,0,&res);break;
        } dump_HSD_TExpRes(&res); break;
    case 7:
        nodes[0].tev.c_op=nodes[0].tev.a_op=0;
        nodes[0].tev.c_bias=nodes[0].tev.a_bias=nodes[0].tev.c_scale=nodes[0].tev.a_scale=0;
        nodes[0].tev.c_in[3].exp=nodes+2; nodes[0].tev.c_in[3].type=4;
        init_cnst(nodes+3,1); nodes[0].tev.a_in[3].exp=nodes+3; nodes[0].tev.a_in[3].type=4;
        if(seed%2) nodes[0].tev.kcsel=nodes[0].tev.kasel=0;
        if(seed%3==0) { nodes[0].tev.c_in[0].sel=0; nodes[0].tev.a_in[0].sel=0; }
        r=TExpAssignReg(nodes,&res); dump_HSD_TExpRes(&res);
        emit(IsThroughColor(nodes)); emit(IsThroughAlpha(nodes)); break;
    case 8:
        cp=seed%2; ap=(seed>>1)%2;
        if(seed%4==0) nodes[0].tev.c_op=255;
        if(seed%5==0) nodes[0].tev.a_op=255;
        if(seed%3==0) nodes[0].tev.tex=textures;
        for(i=0;i<4;i++) { nodes[0].tev.c_in[i].arg=(seed+i)%3?rnd()%16:255; nodes[0].tev.a_in[i].arg=(seed+i)%4?rnd()%8:255; }
        TExp2TevDesc(nodes,descs,&cp,&ap); dump_desc(descs); emit(cp);emit(ap);break;
    case 9:
        for(i=0;i<8;i++) {
            init_cnst(nodes+i,(seed+i)%3!=0); nodes[i].cnst.reg=i; nodes[i].cnst.idx=(seed+i)%4;
            nodes[i].cnst.ctype=(seed+i)%5; nodes[i].cnst.next=i==7?NULL:nodes+i+1; nodes[i].cnst.val=values[i];
            for(j=0;j<4;j++) {
                u32 v=rnd();
                switch(nodes[i].cnst.ctype) {
                case 0:((u8*)values[i])[j]=v;break;
                case 1:((u16*)values[i])[j]=v;break;
                case 2:((u32*)values[i])[j]=v;break;
                case 3: { u32 bits[8]={0x7fc00000,0x7f800000,0xff800000,0x80000000,0,0x3f800000,0x3b808081,v}; ((f32*)values[i])[j]=*(f32*)&bits[(seed+j)%8]; break; }
                default:values[i][j]=((int)(v%1024)-512)/255.0;break;
                }
            }
        } HSD_TExpSetReg(nodes); HSD_TExpSetReg(NULL); break;
    case 10:
        textures[0].id=seed%8; textures[0].coord=seed%9;
        descs[0].desc.next=&descs[1].desc; descs[0].tobj=textures;
        HSD_TExpSetupTev(descs,NULL); dump_desc(descs); dump_desc(descs+1);break;
    case 11:
        list=HSD_TExpTev(&list); HSD_TExpColorOp(list,0,0,0,1);HSD_TExpAlphaOp(list,0,0,0,1);
        r=HSD_TExpCompile(list,&out,&list); emit(PI(list));emit(PI(out));dump_desc(out);HSD_TExpFreeTevDesc(out); break;
    case 12:
        descs[0].desc.next=&descs[1].desc; HSD_TExpFreeTevDesc(descs); HSD_TExpFreeTevDesc(NULL);break;
    case 13:
        p=TevAlloc();p->type=HSD_TE_TEV; q=CnstAlloc();q->type=HSD_TE_CNST;
        HSD_TExpFree(p);HSD_TExpFree(q);break;
    }
    emit(r); snapshot();
}
