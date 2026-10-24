
u8 *hsd_804D79A0, *hsd_804D79A4, *hsd_804D79B8, *hsd_804D79BC;
s32 hsd_804D79A8, hsd_804D79AC, hsd_804D79C0, hsd_804D79C4;
u8 hsd_804D79B0[8], hsd_804D79C8;
void hsd_803B3344(u8 byte) { JpegWork *w=&hsd_804D2648; hsd_803B3CD8_write_byte(byte,&w); }
void hsd_803B3398(void *src,size_t n) {
    if(hsd_804D79A0 < hsd_804D79A4+hsd_804D79A8-n) {
        memcpy(hsd_804D79A0,src,n); hsd_804D79A0+=n;
    } else longjmp(oracle_jmp,1);
}
static u32 rng=0x945952;
static u32 next(void) { rng=rng*1664525u+1013904223u; return rng; }
static void ints(const s32 *v,int n) { for(int i=0;i<n;i++) printf(" %d",v[i]); }
static void bytes(const u8 *v,int n) { for(int i=0;i<n;i++) printf(" %02x",v[i]); }
static void outinit(u8 *out,int n) {
    memset(out,0xcc,32768); hsd_804D79A0=hsd_804D79A4=out; hsd_804D79A8=n;
    hsd_804D79AC=0; memset(hsd_804D79B0,0,8);
}
static void ininit(u8 *src,int n) {
    hsd_804D79B8=hsd_804D79BC=src; hsd_804D79C0=n; hsd_804D79C4=0; hsd_804D79C8=0;
}
static void native_words(u8 *p,int n) { for(int i=0;i<n;i++,p+=2) { u16 w=read_be16(p); memcpy(p,&w,2); } }
int main(void) {
    u8 output[32768], stream[32768]; u16 image[1024]; s32 block[64];
    native_words(encode_storage+0x80,162); native_words(encode_storage+0x268,162);
    native_words(lbl_80431090+0x8c,162); native_words(lbl_80431090+0x3bc,162);
    for(int test=0;test<256;test++) {
        printf("D"); for(int i=0;i<64;i++) { block[i]=test<192?(s32)(next()%16385)-8192:(s32)next(); printf(" %d",block[i]); }
        fn_803B376C((u8*)block); ints(block,64); fn_803B61B4(block); ints(block,64); puts("");
    }
    for(int test=0;test<64;test++) {
        for(int i=0;i<1024;i++) image[i]=(u16)next();
        int w=(test&1)?32:16,x=(test&1)*16,y=((test>>1)&1)*16;
        printf("P %d %d %d",w,x,y); for(int i=0;i<1024;i++) printf(" %04x",image[i]);
        hsd_803B3408((u8*)image,x,y,w,32); ints(hsd_804D2648.x118,384);
        JpegState *s=(JpegState*)&hsd_804D2E70;
        memcpy(&s->work,hsd_804D2648.x118,384*4); memset(output,0xcc,sizeof(output));
        fn_803B6820(output,x,y,w,32); ints(s->work.luma,256); bytes(output,2048); puts("");
    }
    for(int i=0;i<1000;i++) {
        s32 v=(s32)next(); float f=(float)((s32)(next()%200001)-100000)/100.f;
        u16 pixel=0; JpegOutput o={&pixel}; s32 y=(s32)(next()%1024)-512,cb=(s32)(next()%1024)-512,cr=(s32)(next()%1024)-512;
        jpeg_store_rgb565(&o,0,y,cb,cr);
        printf("H %d %d %d %.9g %d %d %d %d %u\n",v,hsd_803B3CD8_bit_length(v),jpeg_run_bit_length(v),f,jpeg_clamp(f),y,cb,cr,pixel);
    }
    for(int kind=0;kind<7;kind++) for(int cap=0;cap<160;cap++) {
        JpegWork *w=&hsd_804D2648; outinit(output,cap); hsd_803B5C2C(cap%14-2);
        int failed=setjmp(oracle_jmp);
        if(!failed) switch(kind) {
        case 0: hsd_803B46D4(); break; case 1: hsd_803B4A2C(); break;
        case 2: hsd_803B4D64(0x1234,0x5678); break;
        case 3: hsd_803B3CD8_write_byte(255,&w); break;
        case 4: hsd_803B3CD8_write_bits(-1,25,&w); break;
        case 5: jpeg_write_run_payload(254,16,&w); break;
        case 6: hsd_803B3CD8_write_bits(0x1bcdef,25,&w); break;
        }
        printf("W %d %d %d %d %d %u",kind,cap,failed,(int)(hsd_804D79A0-output),hsd_804D79AC,hsd_804D79B0[0]); bytes(output,160);puts("");
    }
    for(int test=0;test<128;test++) {
        JpegWork *w=&hsd_804D2648; int component=test%3;
        memset(w,0,sizeof(*w)); memset(&decoder_storage,0,sizeof decoder_storage);
        outinit(output,32768); printf("C %d",component);
        for(int i=0;i<64;i++) { w->coef[i]=next()%5==0?(s32)(next()%127)-63:0; printf(" %d",w->coef[i]); }
        s32 av=0,run=0,val=0,index=0;
        jpeg_encode_component(component,&av,&run,&val,&index);
        printf(" %d %d %d %d %d %d %u",av,run,val,index,(int)(hsd_804D79A0-output),hsd_804D79AC,hsd_804D79B0[0]); bytes(output,128);
        hsd_803B3CD8_write_bits(0,8,&w);
        ininit(output,(int)(hsd_804D79A0-output)); int failed=setjmp(oracle_jmp);
        if(!failed) hsd_803B5EA0(component);
        printf(" %d %d %d %u",failed,(int)(hsd_804D79B8-output),hsd_804D79C4,hsd_804D79C8);
        ints(((JpegState*)&hsd_804D2E70)->work.coeff,64); puts("");
    }
    for(int test=0;test<128;test++) {
        outinit(output,32768); JpegWork *w=&hsd_804D2648;
        int bits=test%31+1; u32 v=next(); hsd_803B3CD8_write_bits((s32)v,bits,&w); hsd_803B3CD8_write_bits(0,8,&w);
        ininit(output,32768); s32 initial=(s32)(next()%10000),got=hsd_803B5C4C_read(initial,bits);
        printf("R %u %d %d %d %d %d %u\n",v,bits,initial,got,(int)(hsd_804D79B8-output),hsd_804D79C4,hsd_804D79C8);
    }
    for(int test=0;test<32;test++) {
        int width=test&1?32:16, height=test&2?32:16,cap=test<8?test*100:32768;
        for(int i=0;i<1024;i++) image[i]=(u16)next();
        hsd_803B5C2C(test%10+1); outinit(output,cap); printf("E %d %d %d %d",width,height,cap,test%10+1);
        for(int i=0;i<1024;i++) printf(" %04x",image[i]);
        int n=hsd_803B51C8((u8*)image,width,height,(char*)output,cap);
        printf(" %d %d %d %u",n,(int)(hsd_804D79A0-output),hsd_804D79AC,hsd_804D79B0[0]); bytes(output,4096);
        memset(stream,0xcc,sizeof stream);int size=hsd_803B6BE4((char*)output,n,stream);
        printf(" %d",size);bytes(stream,2048);
        ints(hsd_804D2648.x118,451); ints(((JpegState*)&hsd_804D2E70)->work.luma,451);
        bytes(lbl_80431090,128); printf(" %d %d %u",(int)(hsd_804D79B8-output),hsd_804D79C4,hsd_804D79C8);puts("");
    }
    for(int cap=0;cap<700;cap+=7) {
        JpegComment c; JpegHuffDc dl,dc; JpegHuffAc al,ac;
        memset(&c,0xdd,sizeof c);memset(&dl,0xdd,sizeof dl);memset(&dc,0xdd,sizeof dc);
        memset(&al,0xdd,sizeof al);memset(&ac,0xdd,sizeof ac);outinit(output,cap);
        int n=hsd_803B51C8_inline((u8*)image,0,0,(char*)output,cap,&lbl_803B9670,&c,&dl,&dc,&al,&ac);
        printf("I %d %d %d",cap,n,(int)(hsd_804D79A0-output));bytes((u8*)&c,21);bytes(dl.data,28);bytes(dc.data,28);bytes(al.data,178);bytes(ac.data,178);bytes(output,700);puts("");
    }
    for(int ac=0;ac<2;ac++) for(int component=0;component<2;component++) for(int index=0;index<(ac?162:12);index++) {
        int len,code;JpegWork *w=&hsd_804D2648;outinit(output,32768);
        if(ac) {len=lbl_80431090[(component?0x318:0x1d0)+index];code=((u16*)(lbl_80431090+(component?0x3bc:0x8c)))[index];}
        else {len=(component?lbl_804316B4:lbl_80431690)[index];code=(component?lbl_8043169C:lbl_80431678)[index];}
        hsd_803B3CD8_write_bits(code,len,&w);hsd_803B3CD8_write_bits(0,16,&w);ininit(output,32768);
        int v=hsd_803B5D70(ac,component);
        printf("F %d %d %d %d %d %d %d %u\n",ac,component,code,len,v,(int)(hsd_804D79B8-output),hsd_804D79C4,hsd_804D79C8);
    }
    for(int test=0;test<24;test++) {
        memset(output,0,sizeof output);output[0]=(test&1)?255:0x93;output[1]=(test&2)?0:1;
        ininit(output,test%4);int v=0,failed=setjmp(oracle_jmp);if(!failed)v=hsd_803B5C4C(test/4+1);
        printf("B %d %d %d %d %d %u\n",test,failed,v,(int)(hsd_804D79B8-output),hsd_804D79C4,hsd_804D79C8);
    }
    return 0;
}
