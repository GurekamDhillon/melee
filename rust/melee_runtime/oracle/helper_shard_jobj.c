/* Native state oracle: compile this inclusion with shipping contraction + lower.py.
 * Inclusion exposes jobj.c's own static helpers, not copies of their algorithms. */
#include <sysdolphin/baselib/jobj.c>
void *orc_alloc(unsigned);
void shard_emit(unsigned,unsigned,unsigned);
static unsigned ev, hash, count;
static int init_status;
static HSD_JObj *objects[32];
static HSD_IDKey keys[32];
static HSD_JObj *values[32];
static HSD_ClassInfo parent_info;
HSD_ClassInfo hsdObj;
static HSD_CObj camera;
static unsigned pid(HSD_JObj *p) { unsigned i; if(!p)return 0; for(i=0;i<count;i++)if(objects[i]==p)return i+1;return 99; }
static void event(unsigned n){ev=ev*33+n;}
static void word(unsigned n){hash=(hash^n)*16777619u;}
static void fl(float f){word(*(unsigned*)&f);}
static int parent_init(HSD_Class *o){event(1);return init_status;}
static void parent_release(HSD_Class *o){event(2);}
static void parent_amnesia(HSD_ClassInfo *i){event(3);}
static void destroy(HSD_Class *o){event(4);}
void hsdInitClassInfo(HSD_ClassInfo *i,HSD_ClassInfo *p,char *lib,char *name,int is,int os){event(5);i->head.parent=p;i->head.info_size=is;i->head.obj_size=os;i->destroy=destroy;}
bool hsdIsDescendantOf(void *i,void *p){event(6);return 1;}
HSD_ClassInfo *hsdSearchClassInfo(const char *n){event(7);return n[0]=='x'?HSD_CLASS_INFO(&hsdJObj):NULL;}
void *hsdNew(HSD_ClassInfo *i){HSD_JObj*j=orc_alloc(sizeof(HSD_JObj));objects[count++]=j;j->object.parent.class_info=i;i->init((HSD_Class*)j);return j;}
HSD_DObj *HSD_DObjLoadDesc(HSD_DObjDesc*d){event(8+(d!=NULL));return d?orc_alloc(sizeof(HSD_DObj)):NULL;}
HSD_RObj *HSD_RObjLoadDesc(HSD_RObjDesc*d){event(10+(d!=NULL));return d?orc_alloc(sizeof(HSD_RObj)):NULL;}
void HSD_DObjResolveRefsAll(HSD_DObj *j,HSD_DObjDesc*d){event(12+(j!=NULL));}
void HSD_RObjResolveRefsAll(HSD_RObj *j,HSD_RObjDesc*d){event(14+(j!=NULL));}
void HSD_DObjRemoveAll(HSD_DObj*j){event(16);}
void HSD_RObjRemoveAll(HSD_RObj*j){event(17);}
void HSD_AObjRemove(HSD_AObj*j){event(18);}
void HSD_IDInsertToTable(HSD_IDTable*t,HSD_IDKey id,void*j){unsigned i;event(19);for(i=0;i<32;i++)if(keys[i]==0||keys[i]==id){keys[i]=id;values[i]=j;return;}}
void *HSD_IDGetDataFromTable(HSD_IDTable*t,HSD_IDKey id,s32*s){unsigned i;event(20);for(i=0;i<32;i++)if(keys[i]==id)return values[i];return NULL;}
void HSD_IDRemoveByIDFromTable(HSD_IDTable*t,HSD_IDKey id){unsigned i;event(21);for(i=0;i<32;i++)if(keys[i]==id){keys[i]=0;values[i]=NULL;return;}}
HSD_CObj *HSD_CObjGetCurrent(void){event(22);return &camera;}
void HSD_JObjDisp(HSD_JObj*j,Mtx m,HSD_TrspMask f,u32 r){int a,b;event(23+pid(j));for(a=0;a<3;a++)for(b=0;b<4;b++)event(*(unsigned*)&m[a][b]);event(f);event(r);}
HSD_RObj *HSD_RObjGetByType(HSD_RObj*r,u32 t,u32 s){return NULL;}
int HSD_RObjGetGlobalPosition(HSD_RObj*r,int t,Vec3*v){return 0;}
void HSD_RObjUpdateAll(HSD_RObj*r,void*j,HSD_ObjUpdateFunc f){}
static void dump(unsigned seed,unsigned stage){unsigned i,k;hash=2166136261u;word(ev);word(count);word(pid(current_jobj));word(default_class!=NULL);word(ufc_callbacks!=NULL);
 for(i=0;i<count;i++){HSD_JObj*j=objects[i];word(pid(j->next));word(pid(j->parent));word(pid(j->child));word(j->flags);word(j->object.ref_count);word(j->object.ref_count_individual);
 for(k=0;k<10;k++)fl((&j->rotate.x)[k]);for(k=0;k<12;k++)fl(j->mtx[k/4][k%4]);word(j->scl!=NULL);word(j->envelopemtx!=NULL);if(j->envelopemtx)for(k=0;k<12;k++)fl(j->envelopemtx[k/4][k%4]);word(j->u.dobj!=NULL);word(j->robj!=NULL);word(j->aobj!=NULL);}
 shard_emit(seed,stage,hash);
}
void shard_jobj_run(unsigned seed){HSD_Joint d[4]={0};HSD_SList s[2]={0};Mtx m;unsigned i;HSD_JObj*j;HSD_JObj local={0};HSD_RObj r={0};
 ev=0;count=0;init_status=0;memset(keys,0,sizeof keys);memset(values,0,sizeof values);memset(&hsdJObj,0,sizeof hsdJObj);default_class=NULL;current_jobj=NULL;ufc_callbacks=NULL;
 parent_info.init=parent_init;parent_info.release=parent_release;parent_info.amnesia=parent_amnesia;hsdObj=parent_info;JObjInfoInit();HSD_JObjSetDefaultClass(HSD_CLASS_INFO(&hsdJObj));
 for(i=0;i<4;i++){d[i].flags=(1<<18)|(1<<28);d[i].scale.x=1+(seed%3);d[i].scale.y=2;d[i].scale.z=3;d[i].rotation.x=seed*.001f;d[i].rotation.y=i*.1f;d[i].rotation.z=-.25f;d[i].position.x=i;d[i].position.y=seed;d[i].position.z=-2;}
 d[0].child=&d[1];d[1].next=&d[2];d[0].next=&d[3];d[3].child=&d[1];d[3].flags|=JOBJ_INSTANCE;
 d[1].class_name=seed&1?"x":"z";d[2].flags|=seed%3==0?JOBJ_PTCL:(seed%3==1?JOBJ_SPLINE:0);
 s[0].next=&s[1];s[0].data=(void*)0x123;s[1].data=(void*)0x456;
 if(seed%3==0)d[2].u.ptcl=s;else if(seed%3==1)d[2].u.spline=(void*)0x100;else d[2].u.dobjdesc=(void*)0x200;
 d[0].robjdesc=(void*)0x300;PSMTXIdentity(m);m[0][3]=seed;d[0].mtx=m;PSMTXIdentity(camera.view_mtx);
 j=HSD_JObjLoadJoint(d);dump(seed,0);
 /* Class metadata, local static helpers and parent init failure/success. */
 hash=2166136261u;word(hsdJObj.load==JObjLoad);word(hsdJObj.make_mtx==HSD_JObjMakeMatrix);word(hsdJObj.release_child==JObjReleaseChild);word(has_scl(NULL));word(has_scl(j));word(robj_set_next(NULL,&r)==&r);word(robj_set_next(&r,NULL)==&r);word(r.next==NULL);if(seed%3==0){word((u32)s[0].data);word((u32)s[1].data);}shard_emit(seed,1,hash);
 local.object.parent.class_info=HSD_CLASS_INFO(&hsdJObj);init_status=-3;word(JObjInit((HSD_Class*)&local));init_status=0;word(JObjInit((HSD_Class*)&local));word(local.flags);fl(local.scale.x);shard_emit(seed,2,hash);
 HSD_JObjSetCurrent(j);HSD_JObjSetCurrent(objects[3]);HSD_JObjSetCurrent(NULL);dump(seed,3);
 /* Clean matrices avoid constraints and exercise instance relative display concatenation. */
 for(i=0;i<count;i++){objects[i]->flags&=~JOBJ_MTX_DIRTY;objects[i]->mtx[0][3]=i;}
 HSD_JObjDispAll(j,m,1,seed);HSD_JObjDispAll(objects[3],m,1,seed);objects[3]->flags|=JOBJ_HIDDEN;HSD_JObjDispAll(objects[3],m,1,seed);dump(seed,4);
 /* Keep records allocated in the oracle to inspect post-release fields. */
 objects[2]->aobj=orc_alloc(sizeof(HSD_AObj));JObjReleaseChild(objects[2]);dump(seed,5);
 for(i=0;i<4;i++){local.object.ref_count=(i==3?65535:i);local.object.ref_count_individual=seed%3;local.child=NULL;local.parent=NULL;local.robj=NULL;local.aobj=NULL;local.u.dobj=NULL;local.scl=NULL;local.envelopemtx=NULL;local.id=0;
 HSD_JObjUnref(&local);hash=2166136261u;word(ev);word(local.object.ref_count);word(local.object.ref_count_individual);HSD_JObjUnrefThis(&local);word(ev);word(local.object.ref_count);word(local.object.ref_count_individual);shard_emit(seed,6+i,hash);}
 ufc_callbacks=s;current_jobj=j;JObjAmnesia(HSD_CLASS_INFO(&hsdJObj));dump(seed,10);
 JObjRelease((HSD_Class*)j);dump(seed,11);
}
