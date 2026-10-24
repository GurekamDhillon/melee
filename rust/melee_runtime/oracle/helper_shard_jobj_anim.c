#include <sysdolphin/baselib/jobj.c>
void shard_emit(unsigned,unsigned,unsigned);
static unsigned ev,hash;
static HSD_JObj j[5];static HSD_DObj d[5];
static unsigned did(HSD_DObj*p){return p?(unsigned)(p-d)+1:0;}
static void event(unsigned n){ev=ev*33+n;}
static void word(unsigned n){hash=(hash^n)*16777619u;}
void HSD_DObjAddAnimAll(HSD_DObj*p,HSD_MatAnim*m,HSD_ShapeAnimDObj*s){event(1);event(did(p));event((u32)m);event((u32)s);}
void HSD_DObjAnimAll(HSD_DObj*p){event(2);event(did(p));}
void HSD_DObjRemoveAnimAllByFlags(HSD_DObj*p,u32 f){event(3);event(did(p));event(f);}
void HSD_DObjReqAnimAllByFlags(HSD_DObj*p,f32 x,u32 f){event(4);event(did(p));event(*(u32*)&x);event(f);}
void HSD_RObjAddAnimAll(HSD_RObj*r,HSD_RObjAnimJoint*a){}
void HSD_RObjAnimAll(HSD_RObj*r){}
void HSD_RObjRemoveAnimAllByFlags(HSD_RObj*r,u32 f){}
void HSD_RObjReqAnimAllByFlags(HSD_RObj*r,f32 x,u32 f){}
HSD_RObj *HSD_RObjGetByType(HSD_RObj*r,u32 t,u32 s){return NULL;}
int HSD_RObjGetGlobalPosition(HSD_RObj*r,int t,Vec3*v){return 0;}
void HSD_RObjUpdateAll(HSD_RObj*r,void*j,HSD_ObjUpdateFunc f){}
static void dump(unsigned seed,unsigned step){unsigned i;hash=2166136261u;word(ev);for(i=0;i<5;i++){HSD_AObj*a=j[i].aobj;word(j[i].flags);word(a!=NULL);if(a){word(a->flags);word(*(u32*)&a->curr_frame);word(*(u32*)&a->rewind_frame);word(*(u32*)&a->end_frame);word(*(u32*)&a->framerate);}}shard_emit(seed,step,hash);}
void shard_jobj_run(unsigned seed){unsigned i,k,x=seed+12345;HSD_AnimJoint a[4]={0};HSD_MatAnimJoint m[4]={0};HSD_ShapeAnimJoint s[4]={0};HSD_AObjDesc ad[4]={0};HSD_FObj fs[5]={0};HSD_AObj sort={0};HSD_FObj*f;
 memset(j,0,sizeof j);memset(d,0,sizeof d);ev=0;
 for(i=0;i<5;i++){j[i].flags=64;j[i].scale.x=j[i].scale.y=j[i].scale.z=1;j[i].u.dobj=&d[i];fs[i].obj_type=i==(seed%6)?12:5+i;fs[i].next=i<4?&fs[i+1]:NULL;}
 j[0].child=&j[1];j[1].parent=&j[0];j[1].next=&j[2];j[2].parent=&j[0];j[1].child=&j[3];j[3].parent=&j[1];j[3].next=&j[4];j[4].parent=&j[1];
 if(seed&1)j[1].flags|=JOBJ_INSTANCE;if(seed&2)j[2].flags|=JOBJ_PTCL;if(seed&4)j[4].flags|=JOBJ_SPLINE;
 for(i=0;i<4;i++){a[i].flags=(seed+i)&1;ad[i].flags=(seed&8)?(1<<29):0;ad[i].end_frame=3+i;a[i].aobjdesc=&ad[i];m[i].matanim=(void*)(i+1);s[i].shapeanimdobj=(void*)(i+9);}
 a[0].child=&a[1];a[1].next=&a[2];a[1].child=&a[3];m[0].child=&m[1];m[1].next=&m[2];m[1].child=&m[3];s[0].child=&s[1];s[1].next=&s[2];s[1].child=&s[3];
 sort.fobj=fs;JObjSortAnim(NULL);JObjSortAnim(&sort);hash=2166136261u;for(f=sort.fobj;f;f=f->next)word(f->obj_type);shard_emit(seed,0,hash);
 for(k=0;k<78;k++){unsigned op=k%13,idx,flags;float frame;x=x*1664525+1013904223;idx=(x>>16)%6;x=x*1664525+1013904223;flags=(x>>16)&0x7ff;frame=(int)(x&31)-8;HSD_JObj*p=idx<5?&j[idx]:NULL;
 switch(op){case 0:HSD_JObjAddAnim(p,a,m,s);break;case 1:HSD_JObjAddAnimAll(p,a,(seed&16)?NULL:m,(seed&32)?NULL:s);break;
 case 2:HSD_JObjReqAnimByFlags(p,flags,frame);break;case 3:HSD_JObjReqAnimAllByFlags(p,flags,frame);break;case 4:HSD_JObjReqAnim(p,frame);break;case 5:HSD_JObjReqAnimAll(p,frame);break;case 6:HSD_JObjAnim(p);break;case 7:JObjAnimAll(p);break;case 8:HSD_JObjAnimAll(p);break;case 9:HSD_JObjRemoveAnimByFlags(p,flags);break;case 10:HSD_JObjRemoveAnimAllByFlags(p,flags);break;case 11:HSD_JObjRemoveAnim(p);break;case 12:HSD_JObjRemoveAnimAll(p);break;}
 dump(seed,k+1);
 }
}
