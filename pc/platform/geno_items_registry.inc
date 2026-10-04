/* Immutable standalone definitions; included by geno_registry.c. Per-frame
 * state belongs to Item.xDD4 in the game heap, never to this host registry. */
#include "../geno/geno_items.h"
#include <math.h>
extern int gw_Mex_ItemRangeAvailable(int base, int count);
typedef struct {
 char name[49], root[260], path[260], model[181], glass[181], asset_mod[64];
 int param[GENO_IP_COUNT], colour, amount, colour_variants;
 unsigned tint; unsigned long long hash;
} GnItemDef;
static GnItemDef gn_items[GENO_ITEM_MAX_DEFS];
static int gn_item_count;
static int gn_item_bits(float f) { union { float f; int i; } v; v.f=f; return v.i; }
static int gn_item_name(const char* s) {
 int n=0; for (;*s;++s,++n) if (n>=48 || !((*s>='a'&&*s<='z') || (*s>='0'&&*s<='9') || *s=='_' || *s=='-')) return 0;
 return n>0;
}
/* The shared Geno reader accepts comments and historical number spellings.
 * Standalone item files use a stricter subset: ordinary JSON objects/numbers,
 * booleans, and unescaped printable ASCII strings (all names/paths are ASCII).
 * Validate before its DOM reader; do not change existing fighter compatibility. */
static void gn_item_ws(const char** p) { while(**p==' '||**p=='\t'||**p=='\r'||**p=='\n') ++*p; }
static int gn_item_json_string(const char** p) {
 if(*(*p)++!='"') return 0;
 while(**p && **p!='"') { if((unsigned char)**p<32||(unsigned char)**p>126||**p=='\\') return 0; ++*p; }
 if(!**p) return 0; ++*p;return 1;
}
static int gn_item_json_value(const char** p,int depth) {
 gn_item_ws(p);if(depth>8) return 0;
 if(**p=='{') {
  ++*p;gn_item_ws(p);if(**p=='}'){++*p;return 1;}
  for(;;) {
   gn_item_ws(p);if(**p!='"'||!gn_item_json_string(p)) return 0;
   gn_item_ws(p);if(*(*p)++!=':')return 0;
   if(!gn_item_json_value(p,depth+1))return 0;
   gn_item_ws(p);if(**p=='}'){++*p;return 1;}if(*(*p)++!=',')return 0;
  }
 }
 if(**p=='"')return gn_item_json_string(p);
 if(!strncmp(*p,"true",4)){*p+=4;return 1;}if(!strncmp(*p,"false",5)){*p+=5;return 1;}
 if(**p=='-')++*p;
 if(**p=='0')++*p;
 else {if(**p<'1'||**p>'9')return 0;while(**p>='0'&&**p<='9')++*p;}
 if(**p=='.'){++*p;if(**p<'0'||**p>'9')return 0;while(**p>='0'&&**p<='9')++*p;}
 if(**p=='e'||**p=='E'){++*p;if(**p=='+'||**p=='-')++*p;if(**p<'0'||**p>'9')return 0;while(**p>='0'&&**p<='9')++*p;}
 return 1;
}
static int gn_item_model_path(const char* s) {
 int n=0,part=0; for (;*s;++s,++n) {
  if(n>=180) return 0;
  if(*s=='/') { if(!part) return 0; part=0; }
  else if((*s>='a'&&*s<='z')||(*s>='A'&&*s<='Z')||(*s>='0'&&*s<='9')||*s=='_'||*s=='-') ++part;
  else return 0;
 } return part>0;
}
static int gn_item_members(const jdoc* d,int obj,const char* const* keys) {
 int c,k,p;
 if(obj<0) return 1;
 if(d->n[obj].type!=JN_OBJ) return 0;
 for(c=d->n[obj].first;c>=0;c=d->n[c].next) {
  for(k=0;keys[k]&&strcmp(keys[k],d->n[c].key);++k) {}
  if(!keys[k]) return 0;
  for(p=d->n[obj].first;p!=c;p=d->n[p].next) if(!strcmp(d->n[p].key,d->n[c].key)) return 0;
 } return 1;
}
static int gn_item_number(const jdoc* d,int obj,const char* key,int* out,double lo,double hi,int integer) {
 int v=jd_get(d,obj,key); double n;
 if(v<0) return 1;
 n=d->n[v].num;
 if(d->n[v].type!=JN_NUM || !isfinite(n) || n<lo || n>hi || (integer&&floor(n)!=n)) return 0;
 *out=integer?(int)n:gn_item_bits((float)n); return 1;
}
static int gn_item_bool(const jdoc* d,int obj,const char* key,int* out) {
 int v=jd_get(d,obj,key); if(v<0) return 1;
 if(d->n[v].type!=JN_BOOL) return 0;
 *out=d->n[v].num!=0; return 1;
}
static int gn_item_string(const jdoc* d,int obj,const char* key,char* out,size_t cap) {
 int v=jd_get(d,obj,key); if(v<0) return 1;
 if(d->n[v].type!=JN_STR || strlen(d->n[v].str)>=cap) return 0;
 snprintf(out,cap,"%s",d->n[v].str); return 1;
}
static const char* const gn_item_colours[]={"red","green","blue","yellow","white"};
int gw_Geno_ItemFind(const char* name) {
 int i; for(i=0;i<gn_item_count;++i) if(!strcmp(gn_items[i].name,name)) return i; return -1;
}
int gw_Geno_ItemCount(void) { return gn_item_count; }
int gw_Geno_ItemParam(int def,int field) {
 return def>=0&&def<gn_item_count&&field>=0&&field<GENO_IP_COUNT ? gn_items[def].param[field] : 0;
}
int gw_Geno_ItemPayload(int def,int field) {
 if(def<0||def>=gn_item_count) return 0;
 return field==0?gn_items[def].colour:gn_items[def].amount;
}
const char* gw_Geno_ItemName(int def) { return def>=0&&def<gn_item_count?gn_items[def].name:""; }
/* Host-only accessor; never called by retargeted code. */
const char* gw_Geno_ItemText(int def,int field) {
 const GnItemDef* d=def>=0&&def<gn_item_count?&gn_items[def]:NULL;
 if(!d) return "";
 switch(field) { case 0:return d->root; case 1:return d->model; case 2:return d->glass; case 3:return d->asset_mod; }
 return "";
}
unsigned gw_Geno_ItemTint(int def) { return def>=0&&def<gn_item_count?gn_items[def].tint:0xFFFFFFFFu; }
int gw_Geno_ItemColourVariants(int def) { return def>=0&&def<gn_item_count?gn_items[def].colour_variants:0; }
int gw_Geno_ItemDefineText(const char* text,const char* root,const char* path,char* error,int cap) {
 static const char* const top[]={"name","version","physics","collection","radius","ports","teams","lifetime","blink","count_against_cap","held","thrown","payload","visual",NULL};
 static const char* const physics[]={"gravity","terminal_velocity","bounce","friction","floor_rest","stage_collision",NULL};
 static const char* const payload[]={"colour","amount",NULL};
 static const char* const visual[]={"model","glass","asset_mod","scale","spin","bob","bob_speed","hover","tilt","height","tint","colour_variants",NULL};
 GnItemDef def; jdoc* d=malloc(sizeof *d); int obj,p,v,i,old; char collect[16]="none",colour[16]="red";
 unsigned long long hash=14695981039346656037ull; const unsigned char* bytes=(const unsigned char*)text;
 const char* why="invalid item definition";
 if(!d) { snprintf(error,cap,"out of memory"); return -1; }
 memset(&def,0,sizeof def);
 { const char* p=text; if(!gn_item_json_value(&p,0)) goto bad;gn_item_ws(&p);if(*p)goto bad; }
 if(strlen(root)>=sizeof def.root||strlen(path)>=sizeof def.path) goto bad;
 while(*bytes) { hash^=*bytes++; hash*=1099511628211ull; }
 def.hash=hash; snprintf(def.root,sizeof def.root,"%s",root); snprintf(def.path,sizeof def.path,"%s",path);
 def.param[GENO_IP_DEFINED]=1; def.param[GENO_IP_GRAVITY]=gn_item_bits(.12f);
 def.param[GENO_IP_TERMINAL]=gn_item_bits(4); def.param[GENO_IP_FRICTION]=gn_item_bits(.15f);
 def.param[GENO_IP_FLOOR_REST]=def.param[GENO_IP_STAGE_COLLISION]=1;
 def.param[GENO_IP_LIFETIME]=900; def.param[GENO_IP_BLINK]=120;
 def.param[GENO_IP_RADIUS]=gn_item_bits(7); def.param[GENO_IP_PORT_MASK]=63; def.param[GENO_IP_TEAM_MASK]=15;
 def.param[GENO_IP_SCALE]=gn_item_bits(1); def.param[GENO_IP_SPIN]=gn_item_bits(3); def.amount=1; def.tint=0xFFFFFFFFu;
 obj=jd_parse(d,text);
 if(obj<0||!gn_item_members(d,obj,top)) goto bad;
 if(!gn_item_string(d,obj,"name",def.name,sizeof def.name)||!gn_item_name(def.name)) goto bad;
 v=jd_get(d,obj,"version"); if(v>=0&&(d->n[v].type!=JN_NUM||d->n[v].num!=1)) goto bad;
 p=jd_get(d,obj,"physics"); if(!gn_item_members(d,p,physics)) goto bad;
 if(!gn_item_number(d,p,"gravity",&def.param[GENO_IP_GRAVITY],0,10,0)||
    !gn_item_number(d,p,"terminal_velocity",&def.param[GENO_IP_TERMINAL],.01,100,0)||
    !gn_item_number(d,p,"bounce",&def.param[GENO_IP_BOUNCE],0,1,0)||
    !gn_item_number(d,p,"friction",&def.param[GENO_IP_FRICTION],0,1,0)||
    !gn_item_bool(d,p,"floor_rest",&def.param[GENO_IP_FLOOR_REST])||
    !gn_item_bool(d,p,"stage_collision",&def.param[GENO_IP_STAGE_COLLISION])) goto bad;
 if(!gn_item_string(d,obj,"collection",collect,sizeof collect)) goto bad;
 if(!strcmp(collect,"touch")) def.param[GENO_IP_COLLECT]=1;
 else if(strcmp(collect,"none")) { why="grab collection is not supported; use touch or none"; goto bad; }
 for(i=0;i<2;++i) { v=jd_get(d,obj,i?"thrown":"held"); if(v>=0&&(d->n[v].type!=JN_BOOL||d->n[v].num)) { why="held/thrown standalone items are not supported"; goto bad; } }
 if(!gn_item_number(d,obj,"radius",&def.param[GENO_IP_RADIUS],.01,100,0)||
    !gn_item_number(d,obj,"ports",&def.param[GENO_IP_PORT_MASK],1,63,1)||
    !gn_item_number(d,obj,"teams",&def.param[GENO_IP_TEAM_MASK],0,15,1)||
    !gn_item_number(d,obj,"lifetime",&def.param[GENO_IP_LIFETIME],1,360000,1)||
    !gn_item_number(d,obj,"blink",&def.param[GENO_IP_BLINK],0,360000,1)||
    !gn_item_bool(d,obj,"count_against_cap",&def.param[GENO_IP_CAP])) goto bad;
 if(def.param[GENO_IP_BLINK]>def.param[GENO_IP_LIFETIME]) goto bad;
 p=jd_get(d,obj,"payload"); if(!gn_item_members(d,p,payload)||!gn_item_string(d,p,"colour",colour,sizeof colour)||
    !gn_item_number(d,p,"amount",&def.amount,0,1000000,1)) goto bad;
 for(i=0;i<5&&strcmp(colour,gn_item_colours[i]);++i) {} if(i==5) goto bad; def.colour=i;
 p=jd_get(d,obj,"visual"); if(!gn_item_members(d,p,visual)||
    !gn_item_string(d,p,"model",def.model,sizeof def.model)||!gn_item_string(d,p,"glass",def.glass,sizeof def.glass)||
    !gn_item_string(d,p,"asset_mod",def.asset_mod,sizeof def.asset_mod)||
    !gn_item_number(d,p,"scale",&def.param[GENO_IP_SCALE],.001,100,0)||
    !gn_item_number(d,p,"spin",&def.param[GENO_IP_SPIN],-360,360,0)||
    !gn_item_number(d,p,"bob",&def.param[GENO_IP_BOB],0,100,0)||
    !gn_item_number(d,p,"bob_speed",&def.param[GENO_IP_BOB_SPEED],0,360,0)||
    !gn_item_number(d,p,"hover",&def.param[GENO_IP_HOVER],0,100,0)||
    !gn_item_number(d,p,"tilt",&def.param[GENO_IP_TILT],-90,90,0)||
    !gn_item_number(d,p,"height",&def.param[GENO_IP_HEIGHT],0,1000,0)||
    !gn_item_bool(d,p,"colour_variants",&def.colour_variants)) goto bad;
 if((def.model[0]&&!gn_item_model_path(def.model))||(def.glass[0]&&!gn_item_model_path(def.glass))||(def.asset_mod[0]&&!gn_item_name(def.asset_mod))) goto bad;
 v=jd_get(d,p,"tint"); if(v>=0) { double n=d->n[v].num; if(d->n[v].type!=JN_NUM||!isfinite(n)||n<0||n>4294967295.0||floor(n)!=n) goto bad; def.tint=(unsigned)n; }
 old=gw_Geno_ItemFind(def.name);
 if(old>=0) {
  if(gn_items[old].hash==def.hash&&!strcmp(gn_items[old].root,root)&&!strcmp(gn_items[old].path,path)) { free(d); return old; }
  why="item name already registered; definitions are immutable until restart"; goto bad;
 }
 if(gn_item_count==GENO_ITEM_MAX_DEFS) { why="item definition registry full (64)"; goto bad; }
 if(!gw_Mex_ItemRangeAvailable(GENO_ITEM_KIND_BASE,GENO_ITEM_MAX_DEFS)) { why="m-ex item tables overlap native item range"; goto bad; }
 i=gn_item_count++; gn_items[i]=def; free(d);
 gw_log("geno item: registered %s kind=%d cap=%d",def.name,GENO_ITEM_KIND_BASE+i,def.param[GENO_IP_CAP]); return i;
bad:
 free(d); snprintf(error,cap,"%s",why); return -1;
}
