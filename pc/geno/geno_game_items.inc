/* Standalone, snapshotted items. Immutable definitions cross the native boundary
 * as scalars; all simulation state lives in Item.xDD4_itemVar. No DAT required. */
#include "geno_items.h"
#include "geno_item_errors.h"
extern int Item_ScriptSpawnAtCap(SpawnItem* spawn);
extern int Item_ScriptSpawnCapTest(void);
#include <melee/mp/mplib.h>
#include <melee/it/it_3F14.h>
#include <sysdolphin/baselib/state.h>
extern int Geno_ItemParam(int def, int field);
extern int Mex_ItemRangeAvailable(int base, int count);
extern void ItemEvent(int what, int handle, int def, int port, int xb,
                      int yb, int colour, int amount);
typedef struct GenoItemVars {
 int handle, owner, colour, amount, age, terminal, floor, blink_started;
 Vec3 prev;
} GenoItemVars;
static Article geno_item_desc[GENO_ITEM_MAX_DEFS];
static ItemAttr geno_item_attr[GENO_ITEM_MAX_DEFS];
static ItemModelDesc geno_item_model[GENO_ITEM_MAX_DEFS];
static HSD_Joint geno_item_joint;
static ItemStateArray geno_item_anims;
/* Vanilla/m-ex own their variable block. Script ownership is in game globals,
 * also snapshotted; x1C protects against heap address reuse after destruction. */
typedef struct GenoFamilyItem { HSD_GObj* g; int serial,handle,owner,kind,terminal,colour,amount; } GenoFamilyItem;
static GenoFamilyItem geno_family_items[128];
extern int Mex_ItemReady(int kind);
int Geno_ItemActive(int kind) {
 int d=kind-GENO_ITEM_KIND_BASE;
 return d>=0 && d<GENO_ITEM_MAX_DEFS && Geno_ItemParam(d,GENO_IP_DEFINED) &&
        Mex_ItemRangeAvailable(GENO_ITEM_KIND_BASE,GENO_ITEM_MAX_DEFS);
}
static int geno_item_bits(f32 f) { union { int i; f32 f; } u; u.f=f; return u.i; }
static f32 geno_item_float(int i) { union { int i; f32 f; } u; u.i=i; return u.f; }
static f32 geno_item_paramf(int d,int f) { return geno_item_float(Geno_ItemParam(d,f)); }
static int geno_item_def(Item* ip) {
 int d=ip->kind-GENO_ITEM_KIND_BASE;
 return d>=0 && d<GENO_ITEM_MAX_DEFS && Geno_ItemActive(ip->kind) ? d : -1;
}
static GenoItemVars* geno_item_vars(Item* ip) { return (GenoItemVars*)&ip->xDD4_itemVar; }
static void geno_item_event(Item* ip,int what,int port) {
 GenoItemVars* v=geno_item_vars(ip);
 if(v->terminal) return;
 v->terminal=1;
 ItemEvent(what,v->handle,geno_item_def(ip),port,geno_item_bits(ip->pos.x),
           geno_item_bits(ip->pos.y),v->colour,v->amount);
}
/* The floor anchor remains physics-owned. Only the visual/pickup span lifts.
 * Height is authored in unscaled model units; zero keeps old spherical pickup. */
static f32 geno_item_lift(int d,f32 age) {
 return geno_item_paramf(d,GENO_IP_HOVER)+geno_item_paramf(d,GENO_IP_BOB)*
  sinf(age*geno_item_paramf(d,GENO_IP_BOB_SPEED)*(f32)M_PI/180.0f);
}
static bool geno_item_in_capsule(f32 dx,f32 dy,f32 radius,f32 top) {
 if(top<0) top=0;
 if(dy>top) dy-=top; else if(dy>0) dy=0;
 return dx*dx+dy*dy<=radius*radius;
}
static bool geno_item_anim(Item_GObj* g) {
 Item* ip=GET_ITEM(g); GenoItemVars* v=geno_item_vars(ip);
 int d=geno_item_def(ip),life=Geno_ItemParam(d,GENO_IP_LIFETIME);
 HSD_GObj* cur;
 if(v->terminal) return true;
 ++v->age;
 if(life>0 && Geno_ItemParam(d,GENO_IP_BLINK)>0 &&
    life-v->age<=Geno_ItemParam(d,GENO_IP_BLINK) && !v->blink_started) {
  v->blink_started=1;
  OSReport("geno item: blink start handle=%d kind=%d age=%d remaining=%d\n",
           v->handle,ip->kind,v->age,life-v->age);
 }
 if(life>0 && v->age>=life) { geno_item_event(ip,1,-1); return true; }
 if(Geno_ItemParam(d,GENO_IP_COLLECT)!=1 || HSD_GObjPLinkHead==NULL) return false;
 for(cur=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER];cur;cur=cur->next) {
  Fighter* fp=GET_FIGHTER(cur); f32 dx,dy,r;
  if(!fp || fp->player_id>=6 || fp->is_sub_fighter || fp->motion_id<=ftCo_MS_RebirthWait || (unsigned)fp->team>=32 ||
     !(Geno_ItemParam(d,GENO_IP_PORT_MASK)&(1<<fp->player_id)) ||
     !((u32)Geno_ItemParam(d,GENO_IP_TEAM_MASK)&(1U<<fp->team))) continue;
  dx=fp->cur_pos.x-ip->pos.x; dy=fp->cur_pos.y-ip->pos.y;
  r=geno_item_paramf(d,GENO_IP_RADIUS)*geno_item_paramf(d,GENO_IP_SCALE);
  if(geno_item_in_capsule(dx,dy,r,geno_item_lift(d,(f32)v->age)+
     geno_item_paramf(d,GENO_IP_HEIGHT)*geno_item_paramf(d,GENO_IP_SCALE))) { geno_item_event(ip,0,fp->player_id); return true; }
 }
 return false;
}
static void geno_item_phys(Item_GObj* g) {
 Item* ip=GET_ITEM(g); GenoItemVars* v=geno_item_vars(ip); int d=geno_item_def(ip);
 f32 terminal=geno_item_paramf(d,GENO_IP_TERMINAL);
 if(v->floor>=0) {
  Vec3 delta={0,0,0};
  if(mpGetSpeed(v->floor,&ip->pos,&delta)) {
   ip->pos.x+=delta.x;ip->pos.y+=delta.y;
  } else v->floor=-1;
 }
 v->prev=ip->pos;
 ip->x40_vel.y-=geno_item_paramf(d,GENO_IP_GRAVITY);
 if(terminal>0 && ip->x40_vel.y < -terminal) ip->x40_vel.y=-terminal;
}
static void geno_item_floor_response(Item* ip,f32 y) {
 int d=geno_item_def(ip); f32 bounce=geno_item_paramf(d,GENO_IP_BOUNCE);
 ip->pos.y=y;
 ip->x40_vel.y=-ip->x40_vel.y*bounce;
 ip->x40_vel.x*=1.0f-geno_item_paramf(d,GENO_IP_FRICTION);
 if(Geno_ItemParam(d,GENO_IP_FLOOR_REST) && ip->x40_vel.y<0.5f) ip->x40_vel.y=0;
}
static bool geno_item_coll(Item_GObj* g) {
 Item* ip=GET_ITEM(g); GenoItemVars* v=geno_item_vars(ip); int d=geno_item_def(ip),line=-1; Vec3 hit;
 f32 bounce=geno_item_paramf(d,GENO_IP_BOUNCE);
 v->floor=-1;
 if(!Geno_ItemParam(d,GENO_IP_STAGE_COLLISION)) return false;
 if(Geno_ItemParam(d,GENO_IP_STAGE_COLLISION) && ip->x40_vel.y<=0 &&
    mpCheckFloor(v->prev.x,v->prev.y+0.01f,ip->pos.x,ip->pos.y-0.01f,
                 0,&hit,&line,NULL,NULL,-1,-1,-1,NULL,NULL)) {
  geno_item_floor_response(ip,hit.y);
  if(ip->x40_vel.y==0 && Geno_ItemParam(d,GENO_IP_FLOOR_REST)) v->floor=line;
 }
 if(ip->x40_vel.y>0 && mpCheckCeiling(v->prev.x,v->prev.y-0.01f,ip->pos.x,ip->pos.y+0.01f,
    &hit,NULL,NULL,NULL,-1,-1)) {ip->pos.y=hit.y-0.01f;ip->x40_vel.y=-ip->x40_vel.y*bounce;}
 if(ip->x40_vel.x!=0 && (mpCheckLeftWall(v->prev.x,v->prev.y,ip->pos.x,ip->pos.y,
    &hit,NULL,NULL,NULL,-1,-1) || mpCheckRightWall(v->prev.x,v->prev.y,ip->pos.x,ip->pos.y,
    &hit,NULL,NULL,NULL,-1,-1))) {
  ip->pos.x=hit.x+(ip->x40_vel.x>0?-0.01f:0.01f);ip->x40_vel.x=-ip->x40_vel.x*bounce;
 }
 return false;
}
static void geno_item_destroyed(Item_GObj* g) { geno_item_event(GET_ITEM(g),2,-1); }
static ItemStateTable geno_item_states[1]={{-1,geno_item_anim,geno_item_phys,geno_item_coll}};
static ItemLogicTable geno_item_logic={geno_item_states,NULL,geno_item_destroyed,
 NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,geno_art_unref};
void* Geno_ItemDesc(int kind) {
 int d=kind-GENO_ITEM_KIND_BASE; Article* a; ItemAttr* at;
 if(!Geno_ItemActive(kind)) return NULL;
 a=&geno_item_desc[d]; at=&geno_item_attr[d];
 geno_zero(at,sizeof(*at)); at->x60_scale=1; at->destroy_gfx=-1; at->destroy_sfx=-1;
 /* Unsupported grab cannot accidentally match a zero-sized pickup rectangle. */
 at->x38_grab_range.x=at->x38_grab_range.y=-1;
 geno_zero(&geno_item_joint,sizeof(geno_item_joint));
 geno_item_joint.scale.x=geno_item_joint.scale.y=geno_item_joint.scale.z=1;
 geno_zero(&geno_item_model[d],sizeof(geno_item_model[d]));
 geno_item_model[d].x0_joint=&geno_item_joint;
 geno_zero(a,sizeof(*a)); a->x0_common_attr=at; a->xC_itemStates=&geno_item_anims;
 a->x10_modelDesc=&geno_item_model[d]; return a;
}
void* Geno_ItemLogic(int kind) { return &geno_item_logic; }
int Geno_ItemCap(int kind) { return Geno_ItemParam(kind-GENO_ITEM_KIND_BASE,GENO_IP_CAP) ? 0 : 8; }
static Item* geno_item_index(int index) {
 HSD_GObj* cur;
 if(index<0 || !HSD_GObjPLinkHead) return NULL;
 for(cur=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM];cur;cur=cur->next) {
  Item* ip=GET_ITEM(cur);
  if(ip && geno_item_def(ip)>=0 && !geno_item_vars(ip)->terminal && index--==0) return ip;
 }
 return NULL;
}
/* Ordinary script item enumeration contains vanilla/m-ex rows too; its index
 * cannot be passed to the standalone-only visual getters. */
int Geno_ItemIndexForId(int serial) {
 int i=0; HSD_GObj* cur;
 if(!HSD_GObjPLinkHead) return -1;
 for(cur=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM];cur;cur=cur->next) {
  Item* ip=GET_ITEM(cur);
  if(!ip || geno_item_def(ip)<0 || geno_item_vars(ip)->terminal) continue;
  if(ip->x1C==serial) return i;
  ++i;
 }
 return -1;
}
int Geno_ItemI(int index,int field) {
 Item* ip=geno_item_index(index); GenoItemVars* v; int d,life,blink;
 if(!ip) return -1; v=geno_item_vars(ip); d=geno_item_def(ip);
 switch(field) {
 case GENO_II_HANDLE:return v->handle; case GENO_II_DEF:return d;
 case GENO_II_OWNER:return v->owner; case GENO_II_COLOUR:return v->colour;
 case GENO_II_AMOUNT:return v->amount; case GENO_II_AGE:return v->age;
 case GENO_II_KIND:return ip->kind;
 case GENO_II_VISIBLE:life=Geno_ItemParam(d,GENO_IP_LIFETIME); blink=Geno_ItemParam(d,GENO_IP_BLINK);
  return !(life>0 && blink>0 && life-v->age<=blink && (v->age&4));
 } return -1;
}
int Geno_ItemF(int index,int field) {
 Item* ip=geno_item_index(index); if(!ip) return 0;
 switch(field) { case GENO_IF_X:return geno_item_bits(ip->pos.x);
 case GENO_IF_Y:return geno_item_bits(ip->pos.y); case GENO_IF_Z:return geno_item_bits(ip->pos.z);
 case GENO_IF_VX:return geno_item_bits(ip->x40_vel.x); case GENO_IF_VY:return geno_item_bits(ip->x40_vel.y); }
 return 0;
}
int Geno_ItemVisual(int index,int field) {
 Item* ip=geno_item_index(index); int d; f32 age;
 if(!ip) return 0; d=geno_item_def(ip); age=(f32)geno_item_vars(ip)->age;
 switch(field) {
 case GENO_IV_Y:return geno_item_bits(ip->pos.y+geno_item_lift(d,age));
 case GENO_IV_ROT_Y:return geno_item_bits(age*geno_item_paramf(d,GENO_IP_SPIN));
 case GENO_IV_SCALE:return Geno_ItemParam(d,GENO_IP_SCALE);
 case GENO_IV_TILT:return Geno_ItemParam(d,GENO_IP_TILT);
 } return 0;
}
int Geno_ItemSpawn(int d,int xb,int yb,int vxb,int vyb,int handle,int owner,int colour,int amount) {
 SpawnItem s; Item_GObj* g; Item* ip; GenoItemVars* v; int i;
 if(d<0 || d>=GENO_ITEM_MAX_DEFS || !Geno_ItemParam(d,GENO_IP_DEFINED)) return GENO_ITEM_ERR_UNSUPPORTED;
 if(!Mex_ItemRangeAvailable(GENO_ITEM_KIND_BASE,GENO_ITEM_MAX_DEFS)) return GENO_ITEM_ERR_CONFLICT;
 if(!Geno_ItemDesc(GENO_ITEM_KIND_BASE+d)) return GENO_ITEM_ERR_UNINITIALIZED;
 for(i=0;geno_item_index(i)!=NULL;++i) if(i>=127) return GENO_ITEM_ERR_POOL;
 geno_zero(&s,sizeof(s)); s.kind=GENO_ITEM_KIND_BASE+d; s.pos.x=geno_item_float(xb);
 s.pos.y=geno_item_float(yb); s.prev_pos=s.pos;
 Item_InitSpawnCommonFields(&s,NULL,1,true);
 s.vel.x=geno_item_float(vxb); s.vel.y=geno_item_float(vyb);
 if(Item_ScriptSpawnAtCap(&s)) return GENO_ITEM_ERR_CAP;
 g=Item_80268B18(&s); if(!g) return GENO_ITEM_ERR_POOL;
 ip=GET_ITEM(g); v=geno_item_vars(ip); geno_zero(v,sizeof(*v));
 v->handle=handle; v->owner=owner; v->colour=colour; v->amount=amount; v->prev=ip->pos;v->floor=-1;
 Item_80268E5C(g,0,ITEM_ANIM_UPDATE); ip->x40_vel=s.vel;
 ip->xDC8_word.flags.x15=0; return handle;
}
int Geno_ItemDespawn(int handle,int owner) {
 int i; Item* ip; HSD_GObj* cur;
 for(i=0;i<128;++i) {
  GenoFamilyItem* f=&geno_family_items[i];
  if(!f->g || f->handle!=handle || (owner>=0 && f->owner!=owner)) continue;
  for(cur=HSD_GObjPLinkHead ? HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM] : NULL;cur;cur=cur->next) {
   if(cur==f->g && GET_ITEM(cur)->x1C==f->serial) {
    f->terminal=1; Item_8026A8EC(cur); f->g=NULL; return 1;
   }
  }
  f->g=NULL; return 0;
 }
 for(i=0;(ip=geno_item_index(i))!=NULL;++i) {
  GenoItemVars* v=geno_item_vars(ip);
  if(v->handle==handle && (owner<0 || v->owner==owner)) {
   v->terminal=1; Item_8026A8EC(ip->entity); return 1;
  }
 } return 0;
}
void Geno_ItemReleaseOwner(int owner) {
 int i=0; Item* ip;
 while((ip=geno_item_index(i))!=NULL) {
  if(owner==0 || geno_item_vars(ip)->owner==owner) Geno_ItemDespawn(geno_item_vars(ip)->handle,owner==0 ? -1 : owner);
  else ++i;
 }
 for(i=0;i<128;++i) if(geno_family_items[i].g &&
  ((owner==0 && geno_family_items[i].owner>=0) || geno_family_items[i].owner==owner))
  Geno_ItemDespawn(geno_family_items[i].handle,owner==0 ? -1 : owner);
}
void Geno_ItemsSceneReset(void) { geno_zero(geno_family_items,sizeof(geno_family_items)); }

/* Only ordinary common objects (before character/projectile ranges) or a
 * completely resolved m-ex row. m-ex's incomplete loader reports not-ready. */
int Geno_ItemSpawnKind(int kind,int xb,int yb,int vxb,int vyb,int handle,
                       int owner,int colour,int amount) {
 SpawnItem s; HSD_GObj* g; int i; HSD_GObj* cur;
 if(kind<0 || kind>=5000) return GENO_ITEM_ERR_UNSUPPORTED;
 /* Standalone item slots are conditional: an m-ex row may own 4608 when
  * native definition admission refused an overlapping item range. */
 if((kind>=GENO_ART_KIND_BASE && kind<GENO_ART_KIND_END) || Geno_ItemActive(kind)) return GENO_ITEM_ERR_CONFLICT;
 if(kind>It_Kind_M_Ball && kind<237) return GENO_ITEM_ERR_UNSAFE;
 if(kind>=237 && !Mex_ItemReady(kind)) return GENO_ITEM_ERR_UNINITIALIZED;
 if(kind<=It_Kind_M_Ball && (!it_804D6D24 || !it_804D6D24[kind] ||
    !it_804D6D24[kind]->x0_common_attr || !it_804D6D24[kind]->x10_modelDesc ||
    !it_803F14C4[kind].states || !it_803F14C4[kind].spawned)) return GENO_ITEM_ERR_UNINITIALIZED;
 /* Reclaim metadata only after checking the current item list, never stale pointers. */
 for(i=0;i<128;++i) {
  GenoFamilyItem* f=&geno_family_items[i]; int live=0;
  if(f->g) for(cur=HSD_GObjPLinkHead ? HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM] : NULL;cur;cur=cur->next)
   if(cur==f->g && GET_ITEM(cur)->x1C==f->serial) { live=1; break; }
  if(!live) { f->g=NULL; break; }
 }
 if(i==128) return GENO_ITEM_ERR_POOL;
 geno_zero(&s,sizeof(s)); s.kind=kind; s.pos.x=geno_item_float(xb);
 s.pos.y=geno_item_float(yb); s.prev_pos=s.pos;
 Item_InitSpawnCommonFields(&s,NULL,1,true);
 s.vel.x=geno_item_float(vxb); s.vel.y=geno_item_float(vyb);
 if(Item_ScriptSpawnAtCap(&s)) return GENO_ITEM_ERR_CAP;
 g=Item_80268B18(&s); if(!g) return GENO_ITEM_ERR_POOL;
 geno_family_items[i].g=g; geno_family_items[i].serial=GET_ITEM(g)->x1C;
 geno_family_items[i].handle=handle; geno_family_items[i].owner=owner;
 GET_ITEM(g)->x40_vel=s.vel;
 geno_family_items[i].kind=kind; geno_family_items[i].terminal=0;
 geno_family_items[i].colour=colour; geno_family_items[i].amount=amount; return handle;
}

int Geno_ItemMeta(int serial,int field) {
 int i; Item* ip;
 for(i=0;(ip=geno_item_index(i))!=NULL;++i) if(ip->x1C==serial) return Geno_ItemI(i,field);
 for(i=0;i<128;++i) if(geno_family_items[i].g && geno_family_items[i].serial==serial) {
  GenoFamilyItem* f=&geno_family_items[i];
  switch(field) { case GENO_II_HANDLE:return f->handle; case GENO_II_OWNER:return f->owner;
   case GENO_II_KIND:return f->kind; case GENO_II_DEF:return -(f->kind+1);
   case GENO_II_COLOUR:return f->colour; case GENO_II_AMOUNT:return f->amount; }
 }
 return -1;
}
int Geno_ItemHandleForId(int serial) { return Geno_ItemMeta(serial,GENO_II_HANDLE); }
void Geno_ItemFamilyEvent(HSD_GObj* g,int what,int port) {
 Item* ip=GET_ITEM(g); int i,slot=-1; HSD_GObj* cur;
 if(geno_item_def(ip)>=0) return;
 for(i=0;i<128;++i) {
  GenoFamilyItem* f=&geno_family_items[i];
  if(f->g!=g || f->serial!=ip->x1C || f->terminal) continue;
  f->terminal=1;
  ItemEvent(what,f->handle,-(ip->kind+1),port,geno_item_bits(ip->pos.x),geno_item_bits(ip->pos.y),f->colour,f->amount);
  return;
 }
 /* Engine-owned rows expose the item's game serial (script handles are in
  * 0x40000000..). Only collection needs a tombstone; destruction frees it. */
 for(i=0;i<128;++i) {
  GenoFamilyItem* f=&geno_family_items[i]; int live=0;
  if(f->g==g && f->serial==ip->x1C) return;
  if(f->g) for(cur=HSD_GObjPLinkHead ? HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM] : NULL;cur;cur=cur->next)
   if(cur==f->g && GET_ITEM(cur)->x1C==f->serial) { live=1; break; }
  if(!live && slot<0) slot=i;
 }
 if(what!=2) {
  if(slot<0) return; /* bounded snapshot-owned event deduplication */
  geno_zero(&geno_family_items[slot],sizeof(geno_family_items[slot]));
  geno_family_items[slot].g=g; geno_family_items[slot].serial=ip->x1C;
  geno_family_items[slot].handle=ip->x1C; geno_family_items[slot].owner=-1;
  geno_family_items[slot].kind=ip->kind; geno_family_items[slot].terminal=1;
 }
 ItemEvent(what,ip->x1C,-(ip->kind+1),port,geno_item_bits(ip->pos.x),geno_item_bits(ip->pos.y),0,0);
}
extern void ItemDraw(int index,int view,int code);
void Geno_ItemDraw(HSD_GObj* g,int code) {
 int i; Item* ip; HSD_CObj* c;
 if(code!=0 && code!=2) return;
 c=HSD_CObjGetCurrent(); if(!c) return;
 for(i=0;(ip=geno_item_index(i))!=NULL;++i) if(ip->entity==g) {
  HSD_StateInvalidate(-1);
  ItemDraw(i,(int)HSD_CObjGetViewingMtxPtr(c),code);
  HSD_StateInvalidate(-1); return;
 }
}

/* Real C fixture, called with a registered touch definition: zero bounce,
 * floor_rest=true, finite lifetime, port0/team0 allowed, positive radius.
 * Saves the whole Item bytes just as the snapshot heap does. No host mirrors. */
int Geno_ItemFixture(int def) {
 Item ip, saved; HSD_GObj g, fighter; Fighter fp; HSD_GObj** old;
 HSD_GObj* heads[32]; int failed=0,handle=731,life;
 if(def<0 || def>=GENO_ITEM_MAX_DEFS || !Geno_ItemParam(def,GENO_IP_DEFINED)) return -1;
 geno_zero(&ip,sizeof(ip)); geno_zero(&g,sizeof(g));
 geno_zero(&fighter,sizeof(fighter)); geno_zero(&fp,sizeof(fp)); geno_zero(heads,sizeof(heads));
 ip.kind=GENO_ITEM_KIND_BASE+def; ip.entity=&g; g.user_data=&ip;
 geno_item_vars(&ip)->handle=handle; geno_item_vars(&ip)->owner=19;
 geno_item_vars(&ip)->colour=3; geno_item_vars(&ip)->amount=7;
 geno_item_vars(&ip)->floor=-1;
 ip.pos.x=9; ip.pos.y=22; ip.x40_vel.y=-4;
 saved=ip; geno_item_phys(&g); ip.pos.y=-100;
 geno_item_floor_response(&ip,5);
 if(ip.pos.y!=5 || ip.x40_vel.y!=0) ++failed;
 ip=saved;
 if(ip.pos.y!=22 || ip.x40_vel.y!=-4 || geno_item_vars(&ip)->handle!=handle ||
    geno_item_vars(&ip)->colour!=3 || geno_item_vars(&ip)->amount!=7) ++failed;
 /* Capsule includes both rounded ends, standing beneath and airborne touch;
  * points outside its horizontal radius or above its top cap are refused. */
 if(!geno_item_in_capsule(0,0,5,17.375f) ||
    !geno_item_in_capsule(0,17.375f,5,17.375f) ||
    !geno_item_in_capsule(3,21.375f,5,17.375f) ||
    geno_item_in_capsule(5.01f,6,5,17.375f) ||
    geno_item_in_capsule(0,22.385f,5,17.375f) ||
    !geno_item_in_capsule(3,4,5,0) || geno_item_in_capsule(3,4.01f,5,0)) ++failed;
 old=HSD_GObjPLinkHead; HSD_GObjPLinkHead=heads; heads[HSD_GOBJ_PLINK_ITEM]=&g;
 if(Geno_ItemF(0,GENO_IF_Y)!=geno_item_bits(22) ||
    Geno_ItemVisual(0,GENO_IV_Y)!=geno_item_bits(22+geno_item_lift(def,0))) ++failed;
 geno_item_vars(&ip)->age=15;
 if(Geno_ItemVisual(0,GENO_IV_Y)!=geno_item_bits(ip.pos.y+geno_item_lift(def,15))) ++failed;
 if(Geno_ItemVisual(0,GENO_IV_ROT_Y)!=geno_item_bits(15*geno_item_paramf(def,GENO_IP_SPIN)) ||
    Geno_ItemVisual(0,GENO_IV_TILT)!=Geno_ItemParam(def,GENO_IP_TILT) ||
    Geno_ItemVisual(0,GENO_IV_SCALE)!=Geno_ItemParam(def,GENO_IP_SCALE)) ++failed;
 geno_item_vars(&ip)->age=0;
 fighter.user_data=&fp; fp.player_id=0; fp.team=0; fp.motion_id=ftCo_MS_Wait; fp.cur_pos=ip.pos;
 heads[HSD_GOBJ_PLINK_FIGHTER]=&fighter;
 /* Use production collection, not just the geometry helper: radius is scaled
  * and the upper cap reaches the rendered model. */
 {
  f32 radius=geno_item_paramf(def,GENO_IP_RADIUS)*geno_item_paramf(def,GENO_IP_SCALE);
  fp.cur_pos.y=ip.pos.y+geno_item_lift(def,1)+
   geno_item_paramf(def,GENO_IP_HEIGHT)*geno_item_paramf(def,GENO_IP_SCALE);
  fp.cur_pos.x=ip.pos.x+radius+0.1f;
  if(geno_item_anim(&g) || geno_item_vars(&ip)->terminal) ++failed;
  geno_item_vars(&ip)->age=0;
  fp.cur_pos.x=ip.pos.x+radius*0.99f;
 }
 if(!geno_item_anim(&g) || !geno_item_vars(&ip)->terminal) ++failed;
 /* Terminal guard prevents a second callback or destroy callback from emitting. */
 if(!geno_item_anim(&g)) ++failed; geno_item_destroyed(&g);
 ip=saved; heads[HSD_GOBJ_PLINK_FIGHTER]=NULL;
 life=Geno_ItemParam(def,GENO_IP_LIFETIME); geno_item_vars(&ip)->age=life-1;
 if(life<=0 || !geno_item_anim(&g) || !geno_item_vars(&ip)->terminal) ++failed;
 /* Refusals return distinct negative reasons before any spawn mutation. */
 if(Geno_ItemSpawnKind(-1,0,0,0,0,1,0,0,0)!=GENO_ITEM_ERR_UNSUPPORTED ||
    Geno_ItemSpawnKind(5000,0,0,0,0,1,0,0,0)!=GENO_ITEM_ERR_UNSUPPORTED ||
    Geno_ItemSpawnKind(35,0,0,0,0,1,0,0,0)!=GENO_ITEM_ERR_UNSAFE ||
    Geno_ItemSpawnKind(200,0,0,0,0,1,0,0,0)!=GENO_ITEM_ERR_UNSAFE ||
    Geno_ItemSpawnKind(GENO_ART_KIND_BASE,0,0,0,0,1,0,0,0)!=GENO_ITEM_ERR_CONFLICT ||
    Geno_ItemSpawn(-1,0,0,0,0,1,0,0,0)!=GENO_ITEM_ERR_UNSUPPORTED) ++failed;
 /* Kind 34 is decimal Poke Ball (0x22), inside the safe common range.
  * Missing registration is distinct from unsafe, including with a full cap. */
 {
  Article** articles=it_804D6D24;
  it_804D6D24=NULL;
  if(Geno_ItemSpawnKind(34,0,0,0,0,1,0,0,0)!=GENO_ITEM_ERR_UNINITIALIZED) ++failed;
  it_804D6D24=articles;
 }
 failed+=Item_ScriptSpawnCapTest();
 /* Rendering reads age bit 2: four visible frames, four hidden frames.
  * Crossing the threshold once marks/logs blink in snapshot-owned state. */
 if(life>0 && Geno_ItemParam(def,GENO_IP_BLINK)>0) {
  int age;
  ip=saved; geno_item_vars(&ip)->age=life-Geno_ItemParam(def,GENO_IP_BLINK)-1;
  if(geno_item_vars(&ip)->age>=0) {
   geno_item_anim(&g);
   if(!geno_item_vars(&ip)->blink_started) ++failed;
  }
  for(age=life-Geno_ItemParam(def,GENO_IP_BLINK);age<life && age<life-Geno_ItemParam(def,GENO_IP_BLINK)+8;++age) {
   geno_item_vars(&ip)->age=age;
   if(Geno_ItemI(0,GENO_II_VISIBLE)!=!(age&4)) ++failed;
  }
 }
 HSD_GObjPLinkHead=old; return failed;
}

/* Actual gw_snap acceptance fixture. Static game objects are in the snapshot's
 * BSS arena; restore also brings the item list and payload back, then visual
 * reads derive from that restored list. Never launch a match for this test. */
static Item geno_item_snap_fixture;
static HSD_GObj geno_item_snap_gobj;
static HSD_GObj* geno_item_snap_heads[32];
extern int snap_open(int k);
extern void snap_save(int frame);
extern int snap_load(int frame);
int Geno_ItemSnapshotFixture(int def) {
 HSD_GObj** old=HSD_GObjPLinkHead; int result=0;
 Item* ip=&geno_item_snap_fixture; GenoItemVars* v;
 if(def<0 || def>=GENO_ITEM_MAX_DEFS || !Geno_ItemParam(def,GENO_IP_DEFINED)) return -1;
 geno_zero(ip,sizeof(*ip)); geno_zero(&geno_item_snap_gobj,sizeof(geno_item_snap_gobj));
 geno_zero(geno_item_snap_heads,sizeof(geno_item_snap_heads));
 ip->kind=GENO_ITEM_KIND_BASE+def; ip->x1C=731;
 ip->entity=&geno_item_snap_gobj; geno_item_snap_gobj.user_data=ip;
 ip->pos.x=9; ip->pos.y=22; ip->x40_vel.y=-4;
 v=geno_item_vars(ip); v->handle=0x40000731; v->owner=19; v->colour=3; v->amount=7;v->floor=-1;
 geno_item_snap_heads[HSD_GOBJ_PLINK_ITEM]=&geno_item_snap_gobj;
 HSD_GObjPLinkHead=geno_item_snap_heads;
 if(snap_open(1)!=0) { HSD_GObjPLinkHead=old; return -2; }
 snap_save(0);
 ip->pos.y=55; ip->x40_vel.y=100; v->colour=99; v->amount=88;
 geno_item_snap_heads[HSD_GOBJ_PLINK_ITEM]=NULL;
 if(snap_load(0)!=0) result=1;
 else if(Geno_ItemF(0,GENO_IF_Y)!=geno_item_bits(22) ||
         Geno_ItemF(0,GENO_IF_VY)!=geno_item_bits(-4) ||
         Geno_ItemI(0,GENO_II_HANDLE)!=0x40000731 ||
         Geno_ItemI(0,GENO_II_COLOUR)!=3 || Geno_ItemI(0,GENO_II_AMOUNT)!=7 ||
         Geno_ItemVisual(0,GENO_IV_Y)!=geno_item_bits(22+geno_item_lift(def,0))) result=2;
 HSD_GObjPLinkHead=old;
 return result;
}

#include "geno_items_interleave_tests.inc"
