#include TU_SRC
#include "mtu_common.h"
static u64 ARENA[1024]; static HSD_Text TEX[3];static sislib_UnkAlloc3 E[3];static SisBlock BL[9];static HSD_GObj G[4];static char STR[3][32];static SIS SI[5];static SIS* TABLE[5];static u64 AR[8];static HSD_CObj CAM;static int kcase;
void OSPanic(const char* f,int line,const char* fmt,...){printf("PANIC %d\n",line);__builtin_trap();}
#include "sislib_gen.h"
int main(void){
 preg(ARENA,sizeof ARENA);preg(TEX,sizeof TEX);preg(E,sizeof E);preg(BL,sizeof BL);preg(G,sizeof G);preg(STR,sizeof STR);preg(SI,sizeof SI);preg(TABLE,sizeof TABLE);preg(AR,sizeof AR);preg(&CAM,sizeof CAM);freg_all();
 for(int op=0;op<22;op++)for(int k=0;k<400;k++){
 tu_case(op,k);kcase=k;u32 ret=0;
 for(int i=0;i<sizeof ARENA;i++)((u8*)ARENA)[i]=(u8)fv();
 for(int i=0;i<3;i++){fill_HSD_Text(&TEX[i]);fill_sislib_UnkAlloc3(&E[i]);}for(int i=0;i<9;i++)fill_SisBlock(&BL[i]);for(int i=0;i<4;i++)fill_HSD_GObj(&G[i]);
 HSD_SisLib_803A6048(sizeof ARENA);TR=0;
 for(int i=0;i<3;i++){
 TEX[i].next=i<2?&TEX[i+1]:0;TEX[i].entity=(fv()%2)?&G[i]:0;TEX[i].font_idx=i%2;TEX[i].alloc_data=0;TEX[i].string_buffer=(fv()%2)?STR[i]:0;TEX[i].render_callback=0;
 E[i].x0=i<2?&E[i+1]:0;E[i].x4=(fv()%2)?&G[i]:0;E[i].xA=i%2;E[i].xE%=16;
 for(int j=0;j<32;j++)STR[i][j]=(u8)fv();
 BL[i].data=&TEX[i];BL[i+3].data=(HSD_Text*)&E[i];BL[i+6].data=(HSD_Text*)STR[i];
 }
 for(int i=0;i<9;i++){BL[i].next=i<8?&BL[i+1]:0;BL[i].size=32;}
 used_head=BL;HSD_SisLib_804D7978=TEX;HSD_SisLib_804D797C=E;
 for(int i=0;i<5;i++){TABLE[i]=&SI[i];HSD_SisLib_804D1124[i]=(k%3)?(SIS*)TABLE:0;HSD_SisLib_804D1110[i]=(k%2)?(HSD_Archive*)AR:0;}
 int ix=rnd()%4,font=rnd()%3,sz=8*(1+rnd()%16);float fx=rw(),fy=rw(),fz=rw(),fw=rw(),fh=rw();u16 cl=fv();u8 pl=fv(),pp=fv(),gl=fv(),gp=fv();u32 rp=fv();
 switch(op){
case 0:{ret=PI(HSD_SisLib_Alloc(sz));}break;
case 1:{HSD_SisLib_Free(ix<3?&TEX[ix]:0);}break;
case 2:{HSD_SisLib_803A5A2C(ix<3?&TEX[ix]:0);}break;
case 3:{ret=PI(HSD_SisLib_803A5ACC(0,k%3-1,fx,fy,fz,fw,fh));}break;
case 4:{HSD_SisLib_803A5CC4(ix<3?&TEX[ix]:0);}break;
case 5:{HSD_SisLib_803A5D30();}break;
case 6:{HSD_SisLib_803A5DA0_inline0(font);}break;
case 7:{HSD_SisLib_803A5DA0(font);}break;
case 8:{HSD_SisLib_803A5E70();}break;
case 9:{HSD_SisLib_803A5F50(font);}break;
case 10:{HSD_SisLib_803A5FBC();}break;
case 11:{HSD_SisLib_803A6048(8192);}break;
case 12:{fn_803A60EC(&G[ix]);}break;
case 13:{sislib_WideCamera(&G[ix],font);}break;
case 14:{HSD_SisLib_SetWideCanvas(font,k%4);}break;
case 15:{ret=HSD_SisLib_803A611C(font,k%2?&G[0]:0,cl,pl,pp,gl,gp,rp);}break;
case 16:{HSD_SisLib_803A62A0(font,"archive","symbol");}break;
case 17:{HSD_SisLib_803A6368(&TEX[ix%3],k%5);}break;
case 18:{used_head=0;void* a=HSD_SisLib_Alloc(sz);void*b=HSD_SisLib_Alloc(64);HSD_SisLib_Free(b);HSD_SisLib_Free(a);}break;
case 19:{BL[7].next=0;BL[8].next=0;BL[8].size=32;free_head->next=&BL[8];ret=PI(HSD_SisLib_Alloc(sz));}break;
case 20:{ret=PI(HSD_SisLib_Alloc(sz-3));}break;
case 21:{used_head=0;void*a=HSD_SisLib_Alloc(sz-7);void*b=HSD_SisLib_Alloc(16);HSD_SisLib_Free(b);HSD_SisLib_Free(a);}break;
}
 for(int i=0;i<3;i++){hash_HSD_Text(&TEX[i]);hash_sislib_UnkAlloc3(&E[i]);}for(int i=0;i<9;i++)hash_SisBlock(&BL[i]);for(int i=0;i<4;i++)hash_HSD_GObj(&G[i]);
 HM(PI(used_head));HM(PI(free_head));HM(HSD_SisLib_804D7968);HM(PI(HSD_SisLib_804D796C));HM(PI(HSD_SisLib_804D7978));HM(PI(HSD_SisLib_804D797C));
 for(int i=0;i<5;i++){HM(PI(HSD_SisLib_804D1110[i]));HM(PI(HSD_SisLib_804D1124[i]));}
 for(SisBlock*b=used_head;b;b=b->next)hash_SisBlock(b);for(SisBlock*b=free_head;b;b=b->next)hash_SisBlock(b);
 for(HSD_Text*t=HSD_SisLib_804D7978;t;t=t->next){hash_HSD_Text(t);if(op==17&&t==&TEX[ix%3])for(int j=0;j<16;j++)HM((u8)t->string_buffer[j]);}
 for(sislib_UnkAlloc3*e=HSD_SisLib_804D797C;e;e=e->x0)hash_sislib_UnkAlloc3(e);
 printf("%d %d %08X %08X %08X\n",op,k,ret,H,TR);
 }return 0;}
