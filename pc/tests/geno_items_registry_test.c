/* Standalone fixture: generated prefix is the production Geno JSON reader. */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include "../geno/geno_items.h"
static int gw_Mex_ItemRangeAvailable(int base, int count) { return base == GENO_ITEM_KIND_BASE && count == GENO_ITEM_MAX_DEFS; }
static void gw_log(const char* fmt, ...) { (void)fmt; }
#include "geno_items_json_reader.inc"
#include "../platform/geno_items_registry.inc"
int main(void) {
 char error[256]; int d;
 assert(gw_Geno_ItemDefineText("{\"name\":\"drive\",\"collection\":\"touch\",\"payload\":{\"colour\":\"blue\",\"amount\":3}}", "fixture", "items/drive/item.json", error, sizeof error) == 0);
 d=gw_Geno_ItemFind("drive"); assert(d==0);
 assert(gw_Geno_ItemParam(d,GENO_IP_DEFINED)==1);
 assert(gw_Geno_ItemParam(d,GENO_IP_COLLECT)==1);
 assert(gw_Geno_ItemPayload(d,0)==2 && gw_Geno_ItemPayload(d,1)==3);
 assert(gw_Geno_ItemDefineText("{\"name\":\"bad\",\"collection\":\"grab\"}","fixture","items/bad/item.json",error,sizeof error)<0);
 assert(gw_Geno_ItemDefineText("{\"name\":\"bad\",\"physics\":{\"gravity\":-1}}","fixture","items/bad/item.json",error,sizeof error)<0);
 assert(gw_Geno_ItemDefineText("{\"name\":\"bad\",\"lifetime\":1e100}","fixture","items/bad/item.json",error,sizeof error)<0);
 assert(gw_Geno_ItemDefineText("{\"name\":\"bad\",\"unknown\":2}","fixture","items/bad/item.json",error,sizeof error)<0);
 assert(gw_Geno_ItemDefineText("{\"name\":\"bad\",\"physics\":{\"bounce\":2}}","fixture","items/bad/item.json",error,sizeof error)<0);
 assert(gw_Geno_ItemDefineText("{\"name\":\"bad\",\"visual\":{\"model\":\"../stolen\"}}","fixture","items/bad/item.json",error,sizeof error)<0);
 assert(gw_Geno_ItemDefineText("{\"name\":\"bad\",\"payload\":{\"colour\":\"purple\"}}","fixture","items/bad/item.json",error,sizeof error)<0);
 assert(gw_Geno_ItemDefineText("{\"name\":\"drive\",\"collection\":\"none\"}","fixture","items/drive/item.json",error,sizeof error)<0);
 assert(gw_Geno_ItemFind("bad")==-1);
 puts("geno_items_registry: PASS"); return 0;
}
