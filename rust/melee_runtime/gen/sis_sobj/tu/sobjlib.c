#include TU_SRC
#include "mtu_common.h"
static HSD_SObj S[4]; static HSD_GObj G; static HSD_SObjDesc2 D; static HSD_ImageDesc IM[2]; static HSD_Tlut TL; static HSD_CObj CAM;
static void callback(HSD_SObj* s) { T(2000); T(PI(s)); s->x48++; }
#include "sobjlib_gen.h"
int main(void) {
 preg(S,sizeof S); preg(&G,sizeof G); preg(&D,sizeof D); preg(IM,sizeof IM); preg(&TL,sizeof TL); preg(&CAM,sizeof CAM); preg(&HSD_SObjLib_804D10E0,sizeof HSD_SObjLib_804D10E0);
 freg_all();freg(callback);
 for(int op=0;op<11;op++) for(int k=0;k<500;k++) {
 tu_case(op,k); u32 ret=0;
 for(int i=0;i<4;i++) fill_HSD_SObj(&S[i]); fill_HSD_GObj(&G); for(int i=0;i<2;i++) fill_HSD_ImageDesc(&IM[i]); fill_HSD_Tlut(&TL); fill_HSD_ObjAllocData(&HSD_SObjLib_804D10E0);
 int n=rnd()%4; unsigned ix=rnd()%4; u8 pr=fv(); u32 ws=fv()%3,wt=fv()%3;u16 ww=fv(),hh=fv();
 for(int i=0;i<4;i++){S[i].x0=0;S[i].gobj=&G;S[i].prev=(i>0&&i<n)?&S[i-1]:0;S[i].next=(i+1<n)?&S[i+1]:0;S[i].x44=i*60;S[i].x40=fv()%32;S[i].x18=(float)((int)(fv()%101)-50)/8;S[i].x4C_callback=(fv()%2)?callback:0;}
 HSD_SObjLib_804D7960=23;G.obj_kind=k%7?23:24; G.hsd_obj=n?S:0;
 D.desc.image=&IM[0];D.desc.tlut=k%2?&TL:0;D.image2=&IM[1]; IM[0].image_ptr=0;IM[1].image_ptr=0;TL.lut=0;
 switch(op){
case 0:{HSD_SObjLib_803A44A4();}break;
case 1:{HSD_SObjLib_803A44D4(&G,&S[ix],pr);}break;
case 2:{HSD_SObjLib_803A466C(k%5==0?0:&S[ix]);}break;
case 3:{HSD_SObjLib_803A4740((HSD_SObj*)G.hsd_obj);}break;
case 4:{ret=PI(HSD_SObjLib_803A477C(&G,&D.desc,ws,wt,pr,k%2));}break;
case 5:{HSD_SObjLib_803A49E0(&G,ix);}break;
case 6:{HSD_SObjLib_803A4A68(&S[ix]);}break;
case 7:{HSD_SObjLib_803A54EC(&G,ix);}break;
case 8:{HSD_SObjLib_803A55DC(&G,ww,hh,pr);}break;
case 9:{GXColor c=discard_color(S[ix].x38_color); ret=RH(&c,sizeof c);}break;
case 10:{ret=PI(HSD_GObjGetHSDObj(&G)); GXEnd();}break;
}
 for(int i=0;i<4;i++)hash_HSD_SObj(&S[i]);hash_HSD_GObj(&G);for(int i=0;i<2;i++)hash_HSD_ImageDesc(&IM[i]);hash_HSD_Tlut(&TL);hash_HSD_ObjAllocData(&HSD_SObjLib_804D10E0);
 printf("%d %d %08X %08X %08X\n",op,k,ret,H,TR);
 }return 0;}
