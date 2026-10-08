#include "../platform/gw_netrun.h"
#include <assert.h>
static void rehash(char *s) { char d[17]; char *p=strrchr(s,'|'); gw_nr_digest_text(s,(size_t)(p-s),d); memcpy(p+1,d,17); }
int main(void) {
    GwNrRecord r,p,q; char text[GW_NR_TEXT_MAX+1],before[GW_NR_TEXT_MAX+1],why[96]; int bars=0;
    gw_nr_clear(&r); r.seed=42; r.mode=GW_ENVOY_MODE_COOP; r.stocks=8;
    assert(gw_nr_encode(&r,text,sizeof text));
    for (const char *c=text; *c; ++c) bars+=*c=='|';
    assert(bars==18 && "RN2 must carry shared stocks, continue token and lost-stage state");
    assert(gw_nr_parse(text,&p,why,sizeof why)); assert(p.stocks==8 && p.continues==1 && !p.lost);
    snprintf(before,sizeof before,"%s",text); r.stocks=7; assert(gw_nr_encode(&r,text,sizeof text)); assert(strcmp(before,text));
    assert(gw_nr_parse(text,&q,why,sizeof why)); assert(gw_nr_compare(&p,&q)==2);
    r.continues=0; assert(gw_nr_encode(&r,text,sizeof text)); assert(strcmp(q.digest,r.digest));
    r.continues=1; gw_nr_lose_stage(&r); assert(r.lost && r.stocks==0);
    assert(gw_nr_encode(&r,text,sizeof text)); assert(gw_nr_parse(text,&q,why,sizeof why) && q.lost);
    assert(!gw_nr_continue(&r,1,8) && !gw_nr_continue(&r,2,8) && !gw_nr_continue(&r,3,0));
    assert(gw_nr_continue(&r,3,8) && r.continues==0 && r.stocks==8 && !r.lost);
    gw_nr_lose_stage(&r); assert(!gw_nr_continue(&r,3,8));
    r.stocks=1; assert(!gw_nr_encode(&r,text,sizeof text)); r.stocks=0; r.continues=2; assert(!gw_nr_encode(&r,text,sizeof text));
    snprintf(text,sizeof text,"%s",before); char *tail=strrchr(text,'|'); tail[-3]='2'; rehash(text); assert(!gw_nr_parse(text,&q,why,sizeof why));
    {long n; assert(!gw_nr_num("9999999999",10,&n));assert(!gw_nr_num("2147483648",10,&n));assert(gw_nr_num("2147483647",10,&n));}
    /* Maximal resource fields still fit settings.cfg and the reliable chunk receiver. */
    gw_nr_clear(&r);r.seed=2147483646L;r.game=31;r.round=31;r.started=31;r.loop=99999;r.stocks=198;r.flags=2147483647L;r.ext=2147483647L;
    memset(r.x,'a',16);r.x[16]=0;
    for(int g=1;g<=31;++g){r.gstage[g]=0xabcdef;r.pick[g][0]=3;r.pick[g][1]=2;}
    assert(gw_nr_encode(&r,text,sizeof text));assert(strlen(text)<511);assert(gw_nr_parse(text,&q,why,sizeof why));
    assert(q.stocks==198 && q.continues==1 && q.started==31);
    puts("PASS RN2 resources: round trip, digest, conflict, shared loss, agreed single continue, strict bounds, 31 stages");return 0;
}
