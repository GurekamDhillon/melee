/* FX1: bounded presentation history. Registry on the game thread; history on
 * the GX recording thread. FIFO payloads copy observations/options; replayed
 * interpolated presents redraw but never add samples. Nothing writes MEM1.
 * Restores/rollback explicitly clear histories and refill from live frames.
 */
#include "gw_motion.h"
#include "gw_motion_history.hpp"
#include "gw.h"
extern "C" {
#include "gw_test.h"
}
#include "gw_profiler.h"
#include <aurora/motion.hpp>
#include <aurora/gfx.hpp>
#include <dolphin/gx/GXAurora.h>
#include <algorithm>
#include <array>
#include <cmath>
#include <cstring>
#include <map>
#include <vector>
#include <chrono>
#include <climits>
#include <atomic>
#include <mutex>
extern "C" int gw_ScriptGame_MotionAnchorI(int,int,int,int,int,int);
extern "C" float gw_ScriptGame_MotionAnchorF(int,int,int,int,int,int);
extern "C" int gw_ScriptGame_MotionHeldDraw(int,int);
namespace {
struct MotionProfile {
    bool enabled;
    MotionProfile(unsigned detail,const char* name):enabled(gw_prof_enabled()!=0){
        if(enabled){gw_prof_detail_name(detail,name);gw_prof_begin(GW_PROF_OBJECT_CALLBACK,detail);}
    }
    ~MotionProfile(){if(enabled)gw_prof_end();}
};
using namespace gw_motion;
constexpr int MaxAfter=12,MaxTracer=64,MaxEmitters=MaxAfter+MaxTracer;
struct Emitter { int handle=0;unsigned owner=0;GwMotionOptions o{};uint32_t program=0; };
std::array<Emitter,MaxEmitters> registry;
int serial,logic_frame,after_count,tracer_count;float global_intensity=0.65f;
int draw_port,draw_sub;bool draw_held,traversing_held;
std::array<int,12> held_frame{{-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1}};
std::atomic<uint64_t> samples,copy_draws,skipped,ribbon_vertices,reset_count;
std::atomic<uint64_t> copy_budget_skips;
uint32_t programs[3];
const char* surfaces[]={
 "fn gd_surface(prev:vec4f,s:GdSurfaceInput)->vec4f{return vec4f(prev.rgb*s.params[0].rgb,prev.a*s.params[0].a);}",
 "fn gd_surface(prev:vec4f,s:GdSurfaceInput)->vec4f{return vec4f(s.params[0].rgb,prev.a*s.params[0].a);}",
 "fn gd_surface(prev:vec4f,s:GdSurfaceInput)->vec4f{let t=clamp(s.uv0.y,0.0,1.0);return vec4f(mix(s.params[1].rgb,s.params[0].rgb,t),prev.a*s.params[0].a);}"};
bool init_surface(int n,const char** error){
    if(programs[n])return true;
    char why[512]{};programs[n]=GXAuroraSurfaceRegister(surfaces[n],"builtin/motion-surface",why,sizeof why);
    if(!programs[n]){static char message[512];snprintf(message,sizeof message,"%s",why);*error=message;return false;}
    return true;
}
Emitter* find(unsigned owner,int handle){for(auto& e:registry)if(e.handle==handle&&e.owner==owner&&handle>0)return &e;return nullptr;}
bool valid(const GwMotionOptions& o){
    if(o.kind<1||o.kind>2||o.port<1||o.port>6||o.sub<0||o.sub>1||o.anchor<0||o.anchor>5||o.index<-5||o.index>254||
       o.copies<1||o.copies>GW_MOTION_MAX_COPIES||o.spacing<1||o.spacing>8||o.lifetime<2||o.lifetime>GW_MOTION_MAX_AGE||o.length<2||o.length>GW_MOTION_MAX_AGE||
       o.smoothing<1||o.smoothing>8||o.blend<0||o.blend>1||o.trigger<0||o.trigger>2||o.surface<0||o.surface>2||
       o.shader<0||o.shader>5||o.depth<0||o.depth>1)return false;
    const float scalars[]={o.speed,o.scale,o.curve,o.width,o.taper,o.intensity};
    for(float f:scalars)if(!std::isfinite(f))return false;
    if(o.speed<0||o.speed>100||o.scale<0.25f||o.scale>2||o.curve<0.25f||o.curve>4||o.width<0.05f||o.width>24||o.taper<0||o.taper>4||o.intensity<0||o.intensity>1)return false;
    for(const float* color:{o.tint,o.tail,o.edge})for(int i=0;i<4;++i)if(!std::isfinite(color[i])||color[i]<0||color[i]>1)return false;
    for(float f:o.params)if(!std::isfinite(f)||f<0||f>10)return false;
    for(float f:o.offset)if(!std::isfinite(f)||std::abs(f)>100)return false;
    if(o.anchor==1&&(o.index<0||o.index>4))return false;
    if(o.anchor==4&&o.item<=0)return false;
    return true;
}
struct SavedPose { std::vector<aurora::gx::motion::Pose> draws;Matrix view;Point root; };
size_t pose_bytes(const SavedPose& p){size_t n=0;for(const auto& d:p.draws)n+=aurora::gx::motion::bytes(d);return n;}
size_t pose_draws(const SavedPose& p){size_t n=0;for(const auto& d:p.draws)n+=aurora::gx::motion::draws(d);return n;}
struct GhostHistory { History<SavedPose,GW_MOTION_HISTORY> poses;SavedPose pending{};bool pending_failed=false;
    int pending_frame=-1,entity=0,costume=-1,falls=-1,last_present=-1,port=1,sub=0; };
struct RibbonHistory { History<Point,GW_MOTION_HISTORY> points;int element=0,last=-1,entity=0,costume=-1,falls=-1; };
std::array<uint64_t,12> logged_failures{};
std::array<uint64_t,12> logged_fallbacks{};
std::array<bool,12> logged_copy_budget{};
uint64_t active_fallbacks[aurora::gx::motion::FallbackCount]{};
void diagnose(int port,int sub,aurora::gx::motion::Failure failure){
    using namespace aurora::gx::motion;
    if(failure==Failure::Count)return;
    auto& mask=logged_failures[size_t((port-1)*2+sub)];uint64_t bit=uint64_t(1)<<unsigned(failure);
    if(!(mask&bit)){mask|=bit;gw_log("motion: capture failure reason=%s fighter=P%d sub=%d",failure_name(failure),port,sub);}
}
std::map<int,GhostHistory> ghosts;
std::map<std::pair<int,int>,RibbonHistory> ribbons;
struct Begin { Emitter emitter;Matrix view;Point root;int frame,entity,hitlag,held,costume,falls;float speed,intensity; };
Begin active_begin{};bool capturing;
int budget_frame=-1;Budget budget{0,0};
void clear_callback(const void* data,uint32_t size){
    if(aurora::gx::motion::replaying())return;
    if(size!=sizeof(int))return;int h;memcpy(&h,data,sizeof h);
    if(h==0){ghosts.clear();ribbons.clear();++reset_count;budget_frame=-1;}
    else {ghosts.erase(h);for(auto i=ribbons.begin();i!=ribbons.end();)if(i->first.first==h)i=ribbons.erase(i);else ++i;}
}
void clear_history(int h){GXAuroraMotionCallback(clear_callback,&h,sizeof h);}
void begin_callback(const void* data,uint32_t size){
    aurora::gx::motion::set_timing(gw_prof_enabled()!=0);
    MotionProfile profile(0x4d4f0001,"motion/afterimage-replay");
    if(size!=sizeof(Begin))return;Begin b;memcpy(&b,data,size);active_begin=b;
    capturing=false;
    auto& history=ghosts[b.emitter.handle];
    if(history.entity!=b.entity || history.costume!=b.costume || b.frame<history.poses.latest() ||
       (b.emitter.o.clear_on_respawn && history.falls!=b.falls)){history.poses.clear();history.pending={};history.pending_frame=-1;}
    history.entity=b.entity;history.costume=b.costume;history.falls=b.falls;
    history.port=b.emitter.o.port;history.sub=b.emitter.o.sub;
    const auto& o=b.emitter.o;
    int present=int(aurora::gfx::current_frame());
    if(budget_frame!=present){budget_frame=present;budget={48*1024*1024,8192};}
    // Keep only the poses this emitter can still draw (the oldest copy's age, bounded by its lifetime).
    history.poses.trim(b.frame,std::min(int(b.emitter.o.lifetime),int(b.emitter.o.copies)*int(b.emitter.o.spacing))+1);
    std::array<const SavedPose*,GW_MOTION_MAX_COPIES> chosen{};
    const bool first_present=history.last_present!=present;history.last_present=present;
    for(int n=1;first_present&&b.intensity*o.intensity>0&&n<=o.copies;++n){
        int age=n*o.spacing;auto p=history.poses.age(b.frame,age);
        if(age<o.lifetime&&p&&!p->draws.empty()){
            if(budget.take(pose_bytes(*p),pose_draws(*p)))chosen[n-1]=p;
            else {++skipped;++copy_budget_skips;auto& logged=logged_copy_budget[size_t((o.port-1)*2+o.sub)];
                if(!logged){logged=true;gw_log("motion: replay skip reason=copy_budget fighter=P%d sub=%d need_bytes=%u need_draws=%u",o.port,o.sub,unsigned(pose_bytes(*p)),unsigned(pose_draws(*p)));}
            }
        }
    }
    for(int n=o.copies;n>=1;--n){
        int age=n*o.spacing;if(age>=o.lifetime)continue;
        auto p=chosen[n-1];if(!p||p->draws.empty())continue;
        float strength=fade(age,o.lifetime,o.curve)*o.intensity*b.intensity;
        if(strength<=0)continue;
        Matrix inv;if(!inverse(p->view,inv)){++skipped;continue;}
        auto world=identity();world[0]=world[5]=world[10]=o.scale;
        Point shift=o.follow?b.root-p->root:Point{};
        world[3]=p->root.x*(1-o.scale)+shift.x+o.offset[0];
        world[7]=p->root.y*(1-o.scale)+shift.y+o.offset[1];
        world[11]=p->root.z*(1-o.scale)+shift.z+o.offset[2];
        auto delta=multiply(b.view,multiply(world,inv));
        float params[20]{};memcpy(params+4,o.tint,16);memcpy(params+8,o.tail,16);
        params[7]*=strength;
        if(!aurora::gx::motion::replay_group(p->draws,delta,params)){++skipped;diagnose(o.port,o.sub,aurora::gx::motion::last_failure());continue;}
        copy_draws+=pose_draws(*p);
    }
    bool trigger=o.trigger==0||(o.trigger==1&&b.speed>o.speed*o.speed)||(o.trigger==2&&o.flag);
    { static int probe; if(probe<6 && b.intensity*o.intensity>0){++probe;gw_log("motion: probe begin replaying=%d trigger=%d hitlag=%d intensity=%.2f latest=%d frame=%d copies=%d",int(aurora::gx::motion::replaying()),int(trigger),int(b.hitlag),double(b.intensity*o.intensity),int(history.poses.latest()),int(b.frame),int(o.copies));} }
    if(!aurora::gx::motion::replaying() && trigger && !b.hitlag && b.intensity>0 && o.intensity>0 && history.poses.latest()!=b.frame){
        if(history.pending_frame!=b.frame){history.pending={};history.pending_frame=b.frame;history.pending_failed=false;}
        if(history.pending_failed)return;
        history.pending.view=b.view;history.pending.root=b.root;
        uint64_t failures[aurora::gx::motion::FailureCount],warm_count;
        aurora::gx::motion::diagnostics(failures,active_fallbacks,&warm_count);
        aurora::gx::motion::begin(b.emitter.program,o.blend,false,o.surface!=0);capturing=true;
    }
}
void end_callback(const void*,uint32_t){
    MotionProfile profile(0x4d4f0002,"motion/pose-retain");
    if(!capturing)return;capturing=false;
    auto pose=aurora::gx::motion::end();
    uint64_t failures[aurora::gx::motion::FailureCount],fallbacks[aurora::gx::motion::FallbackCount],warm_count;
    aurora::gx::motion::diagnostics(failures,fallbacks,&warm_count);
    static const char* fallback_names[]={"no_fog","textureless","copy_silhouette","palette_warm","held_draws","held_geometry"};
    auto& mask=logged_fallbacks[size_t((active_begin.emitter.o.port-1)*2+active_begin.emitter.o.sub)];
    for(size_t i=0;i<aurora::gx::motion::FallbackCount;++i)if(fallbacks[i]>active_fallbacks[i]&&!(mask&(uint64_t(1)<<i))){
        mask|=uint64_t(1)<<i;gw_log("motion: fallback reason=%s fighter=P%d sub=%d",fallback_names[i],active_begin.emitter.o.port,active_begin.emitter.o.sub);
    }
    auto& h=ghosts[active_begin.emitter.handle];
    if(pose){h.pending.draws.push_back(pose);
        if(pose_bytes(h.pending)>4*1024*1024 || pose_draws(h.pending)>2048){
            auto reason=pose_draws(h.pending)>2048?aurora::gx::motion::Failure::DrawCount:aurora::gx::motion::Failure::PoseBytes;
            aurora::gx::motion::rejected(reason);h.pending_failed=true;diagnose(active_begin.emitter.o.port,active_begin.emitter.o.sub,reason);
        }
    }else if(aurora::gx::motion::last_failure()!=aurora::gx::motion::Failure::Count){
        h.pending_failed=true;diagnose(active_begin.emitter.o.port,active_begin.emitter.o.sub,aurora::gx::motion::last_failure());
    }
    { static int probe; if(probe<6 && (pose||aurora::gx::motion::last_failure()!=aurora::gx::motion::Failure::Count)){++probe;gw_log("motion: probe end pose=%d draws=%d bytes=%d fighter=P%d sub=%d reason=%s",int(bool(pose)),pose?int(aurora::gx::motion::draws(pose)):-1,pose?int(aurora::gx::motion::bytes(pose)):-1,active_begin.emitter.o.port,active_begin.emitter.o.sub,aurora::gx::motion::failure_name(aurora::gx::motion::last_failure()));} }
}
// Once per presented frame: report the GX-thread time the afterimage code spent (capture, replay, ...) as profiler details
// of the object_callback zone, one observation per frame per category. Drained even when the profiler is off.
void report_timings(){
    for(unsigned i=0;i<aurora::gx::motion::TimingCount;++i){
        uint64_t ns=aurora::gx::motion::take_timing(i);
        if(!ns||!gw_prof_enabled())continue;
        char name[64];snprintf(name,sizeof name,"motion/%s",aurora::gx::motion::timing_name(i));
        unsigned detail=0x4d4f0100u+i;gw_prof_detail_name(detail,name);
        gw_prof_sample(GW_PROF_OBJECT_CALLBACK,double(ns)/1e6,detail);
    }
}
void finish_callback(const void*,uint32_t){
    if(aurora::gx::motion::replaying())return;
    report_timings();
    for(auto& entry:ghosts){auto& h=entry.second;if(h.pending_frame<0)continue;
        if(h.pending_failed||h.pending.draws.empty()){
            if(!h.pending_failed){aurora::gx::motion::rejected(aurora::gx::motion::Failure::EmptyPose);diagnose(h.port,h.sub,aurora::gx::motion::Failure::EmptyPose);}
            ++skipped;
        }else{
            static int probe;if(probe<6){++probe;gw_log("motion: pose assembled fighter=P%d sub=%d parts=%u draws=%u bytes=%u frame=%d",h.port,h.sub,unsigned(h.pending.draws.size()),unsigned(pose_draws(h.pending)),unsigned(pose_bytes(h.pending)),h.pending_frame);}
            h.poses.push(h.pending_frame,std::move(h.pending));++samples;
        }
        h.pending={};h.pending_frame=-1;h.pending_failed=false;
    }
}
void only_callback(const void* data,uint32_t size){if(size==sizeof(int)){int on;memcpy(&on,data,size);aurora::gx::motion::capture_only(on!=0);if(on)aurora::gx::motion::held_draws();}}
struct Warm {uint32_t program;int blend,begin,flat,port;};
void warm_callback(const void* data,uint32_t size){
    if(size!=sizeof(Warm))return;Warm w;memcpy(&w,data,size);
    if(w.begin)aurora::gx::motion::begin(w.program,w.blend,true,w.flat!=0);else {
        aurora::gx::motion::end();uint64_t f[aurora::gx::motion::FailureCount],b[aurora::gx::motion::FallbackCount],n;
        aurora::gx::motion::diagnostics(f,b,&n);gw_log("motion: warm fighter=P%d variants=%llu includes_motion=1",w.port,(unsigned long long)n);
    }
}
// Ribbons are uploaded once on the GX thread; immutable ranges are consumed by
// the worker. The GPU library below has no host time or random simulation input.
#include "gw_motion_ribbons.inc"
struct Sample { Emitter emitter;Point point;int index,valid,element,entity,costume,falls; };
struct World {Matrix view;int frame,count;float intensity;Sample samples[MaxTracer*5];};
void world_callback(const void* data,uint32_t size){
    MotionProfile profile(0x4d4f0003,"motion/ribbons");
    if(size<offsetof(World,samples))return;World w{};memcpy(&w,data,std::min(size,uint32_t(sizeof w)));
    if(w.count<0||w.count>MaxTracer*5||size!=offsetof(World,samples)+w.count*sizeof(Sample))return;
    int present=int(aurora::gfx::current_frame());
    if(ribbon_frame_tag!=present){ribbon_frame_tag=present;ribbon_frame_vertices=0;}
    std::vector<RibbonVertex> vertices;
    for(int i=0;i<w.count;++i){
        const auto& s=w.samples[i];const auto& o=s.emitter.o;auto& h=ribbons[{s.emitter.handle,s.index}];
        if(!aurora::gx::motion::replaying()){
            if(w.frame<h.points.latest()||h.element!=s.element||h.entity!=s.entity||h.costume!=s.costume||
               (o.clear_on_respawn&&h.falls!=s.falls)||(!s.valid && h.last==w.frame-1))h.points.clear();
            h.entity=s.entity;h.costume=s.costume;h.falls=s.falls;
            const auto* old=h.points.at(w.frame-1);
            bool trigger=o.trigger==0||(o.trigger==2&&o.flag)||(o.trigger==1&&(!old||distance2(*old,s.point)>o.speed*o.speed));
            if(s.valid && trigger){
                if(h.last>=0 && w.frame>h.last+1)h.points.clear();
                const auto* previous=h.points.at(w.frame-1);
                if(previous&&distance2(*previous,s.point)>2500)h.points.clear(); // teleport: don't bridge the room
                h.points.push(w.frame,s.point);h.last=w.frame;h.element=s.element;
            }
        }
        vertices.clear();
        build_ribbon(h.points,w.frame,o,w.intensity,w.view,s.element,vertices);
        if(vertices.empty())continue;
        if(vertices.size()>16384-ribbon_frame_vertices){++skipped;continue;}
        ribbon_frame_vertices+=vertices.size();ribbon_vertices+=vertices.size();
        auto draw_options=o;
        if(o.anchor==GW_ANCHOR_HITS)draw_options.shader=s.element==1?2:s.element==2?3:s.element==5?4:s.element==13?5:0;
        record_ribbon(vertices,draw_options,w.view,w.frame);
    }
}
}
extern "C" void gw_motion_defaults(GwMotionOptions* o,int kind){
    memset(o,0,sizeof *o);o->kind=kind;o->port=1;o->copies=3;o->spacing=3;o->lifetime=16;o->length=12;o->smoothing=4;
    o->index=-2;o->scale=1;o->curve=1.5f;o->width=0.8f;o->taper=1;o->intensity=0.45f;o->depth=1;o->params[0]=1;
    float tint[]={0.35f,0.8f,1,0.65f},tail[]={0.15f,0.25f,0.7f,0},edge[]={1,1,1,0};
    memcpy(o->tint,tint,16);memcpy(o->tail,tail,16);memcpy(o->edge,edge,16);
    /* Owner rule: no always-on afterimages. They start lowered (flag trigger) until a script raises the flag, binds it to a
       status, or opens a window; "always" and "moving" need debug=true in the options (gw_script_earned.inc). */
    if(kind==GW_MOTION_AFTERIMAGE){o->trigger=2;o->flag=0;}
}
extern "C" int gw_motion_add(unsigned owner,const GwMotionOptions* o,const char** error){
    if(!owner||!o||!valid(*o)){*error="invalid motion options";return 0;}
    int count=0;Emitter* slot=nullptr;
    for(auto& e:registry){if(!e.handle){if(!slot)slot=&e;continue;}if(e.o.kind==o->kind)++count;
        if(o->kind==1&&e.o.kind==1&&e.o.port==o->port&&(e.o.surface!=o->surface||e.o.blend!=o->blend)){*error="paired afterimages must share surface and blend for warm traversal";return 0;}
        if(o->kind==1&&e.o.kind==1&&e.o.port==o->port&&e.o.sub==o->sub){*error="afterimage entity already has an emitter";return 0;}}
    if(!slot||count>=(o->kind==1?MaxAfter:MaxTracer)||serial==INT_MAX){*error="motion capacity exhausted";return 0;}
    if(o->kind==1&&!init_surface(o->surface,error))return 0;
    *slot={++serial,owner,*o,o->kind==1?programs[o->surface]:0};
    if(o->kind==1){++after_count;held_frame[(o->port-1)*2+o->sub]=-1;}else ++tracer_count;
    gw_log("motion: emitter %d kind=%d port=%d sub=%d added",slot->handle,o->kind,o->port,o->sub);
    return slot->handle;
}
extern "C" int gw_motion_get(unsigned owner,int h,GwMotionOptions* o){auto e=find(owner,h);if(!e)return 0;*o=e->o;return 1;}
extern "C" int gw_motion_set(unsigned owner,int h,const GwMotionOptions* o,const char** error){
    auto e=find(owner,h);if(!e){*error="stale or foreign motion handle";return 0;}
    if(!valid(*o)||o->kind!=e->o.kind||o->port!=e->o.port||o->sub!=e->o.sub){*error="invalid options or immutable entity";return 0;}
    for(const auto& other:registry)if(other.handle&&other.handle!=h&&other.o.kind==1&&o->kind==1&&other.o.port==o->port&&
        (other.o.surface!=o->surface||other.o.blend!=o->blend)){*error="paired afterimages must share surface and blend";return 0;}
    if(o->kind==1&&!init_surface(o->surface,error))return 0;
    bool reset=o->anchor!=e->o.anchor||o->index!=e->o.index||o->item!=e->o.item||o->surface!=e->o.surface||o->blend!=e->o.blend;
    e->o=*o;e->program=o->kind==1?programs[o->surface]:0;if(reset){clear_history(h);if(o->kind==1)held_frame[(o->port-1)*2+o->sub]=-1;}return 1;
}
extern "C" int gw_motion_remove(unsigned owner,int h){auto e=find(owner,h);if(!e)return 0;if(e->o.kind==1)--after_count;else --tracer_count;*e={};clear_history(h);return 1;}
extern "C" void gw_motion_release(unsigned owner){
    bool any=false;
    for(auto& e:registry)if(e.handle&&(!owner||e.owner==owner)){any=true;clear_history(e.handle);if(e.o.kind==1)--after_count;else --tracer_count;e={};}
    if(!owner&&any)clear_history(0);
}
extern "C" void gw_motion_frame(int frame,int replay){logic_frame=frame;if(replay){held_frame.fill(-1);if(after_count||tracer_count)clear_history(0);}}
extern "C" void gw_motion_intensity(float f){if(std::isfinite(f))global_intensity=std::clamp(f,0.f,1.f);}
extern "C" void gw_motion_stats(uint64_t out[GW_MOTION_STATS_COUNT]){
    static_assert(GW_MOTION_STATS_COUNT==8+aurora::gx::motion::FailureCount+aurora::gx::motion::FallbackCount+1+4);
    memset(out,0,GW_MOTION_STATS_COUNT*sizeof *out);for(auto& e:registry)if(e.handle)++out[e.o.kind-1];
    out[2]=samples;out[3]=copy_draws;out[4]=skipped;out[5]=ribbon_vertices;out[6]=reset_count;out[7]=copy_budget_skips;
    aurora::gx::motion::diagnostics(out+8,out+8+aurora::gx::motion::FailureCount,out+GW_MOTION_STATS_COUNT-5);
    aurora::gx::motion::pool_stats(out+GW_MOTION_STATS_COUNT-4);
}
extern "C" const char* gw_motion_stat_name(unsigned i){
    static const char* names[]={"afterimages","tracers","poses","copy_draws","skipped","ribbon_vertices","resets","copy_budget_skips",
        "fail_pipeline_missing","fail_vertex_arena","fail_index_arena","fail_uniform_arena","fail_copy_texture","fail_index_array","fail_palette","fail_fog_lut",
        "fail_pose_bytes","fail_draw_count","fail_frame_arena","fail_resident_bytes","fail_empty_pose","fail_pipeline_pending","fail_replay_arena","fail_geometry",
        "fallback_no_fog","fallback_textureless","fallback_copy_silhouette","warm_palette_variants","held_draw_scopes","held_retained_draws","warm_variants",
        "pool_bytes","pool_hits","pool_misses","resident_bytes"};
    static_assert(sizeof names/sizeof names[0]==GW_MOTION_STATS_COUNT);
    return i<GW_MOTION_STATS_COUNT?names[i]:"invalid";
}
extern "C" int gw_MotionFighterBegin(int port,int sub,int entity,int view,int root,int hitlag,int held,int costume,int falls,int speed){
    if(!after_count || traversing_held)return 0;
    { static int probe; if(probe<6){++probe;gw_log("motion: probe fighter draw port=%d sub=%d hitlag=%d held=%d intensity=%.2f frame=%d",port,sub,hitlag,held,double(global_intensity),int(logic_frame));} }
    for(auto& e:registry)if(e.handle&&e.o.kind==1&&e.o.port==port&&e.o.sub==sub){
        draw_port=port;draw_sub=sub;draw_held=held!=0 && !hitlag && global_intensity>0 && e.o.intensity>0;
        Begin b{};b.emitter=e;b.frame=logic_frame;b.entity=entity;b.hitlag=hitlag;b.held=held;b.costume=costume;b.falls=falls;b.intensity=global_intensity;
        for(int i=0;i<12;++i)b.view[i]=gw_rf32(reinterpret_cast<const char*>(uintptr_t(uint32_t(view)))+i*4);
        auto p=reinterpret_cast<const char*>(uintptr_t(uint32_t(root)));b.root={gw_rf32(p),gw_rf32(p+4),gw_rf32(p+8)};
        memcpy(&b.speed,&speed,4);GXAuroraMotionCallback(begin_callback,&b,sizeof b);return 1;
    }
    return 0;
}
extern "C" void gw_MotionFighterEnd(void){
    const int slot=(draw_port-1)*2+draw_sub;
    if(draw_held && slot>=0 && slot<12 && held_frame[slot]!=logic_frame){
        held_frame[slot]=logic_frame;int on=1;GXAuroraMotionCallback(only_callback,&on,sizeof on);
        // Current held item uses its actual renderer/visibility, all passes, once
        // per logic frame. Aurora retains this traversal but never draws it live.
        traversing_held=true;gw_ScriptGame_MotionHeldDraw(draw_port-1,draw_sub);traversing_held=false;
        on=0;GXAuroraMotionCallback(only_callback,&on,sizeof on);
    }
    GXAuroraMotionCallback(end_callback,nullptr,0);
}
extern "C" void gw_motion_warm(int port,int begin){
    for(auto& e:registry)if(e.handle&&e.o.kind==1&&e.o.port==port){
        if(!begin){traversing_held=true;gw_ScriptGame_MotionHeldDraw(port-1,0);gw_ScriptGame_MotionHeldDraw(port-1,1);traversing_held=false;}
        Warm w{e.program,e.o.blend,begin,e.o.surface!=0,port};GXAuroraMotionCallback(warm_callback,&w,sizeof w);return;
    }
}
static void arena_fill_callback(const void* data,uint32_t size){
    if(size!=sizeof(unsigned))return;unsigned leave;memcpy(&leave,data,sizeof leave);
    aurora::gx::motion::test_fill_uniform_arena(leave);
}
extern "C" void gw_motion_test_arena_fill(unsigned leave){GXAuroraMotionCallback(arena_fill_callback,&leave,sizeof leave);}
extern "C" void gw_motion_prepare_ribbons(void){GXAuroraMotionCallback(ribbon_warm_callback,nullptr,0);}
extern "C" void gw_MotionWorldDraw(int view){
    if(after_count)GXAuroraMotionCallback(finish_callback,nullptr,0);
    if(!tracer_count)return;
    World w{};w.frame=logic_frame;w.intensity=global_intensity;
    for(int i=0;i<12;++i)w.view[i]=gw_rf32(reinterpret_cast<const char*>(uintptr_t(uint32_t(view)))+i*4);
    auto flush=[&](){if(w.count)GXAuroraMotionCallback(world_callback,&w,uint32_t(offsetof(World,samples)+w.count*sizeof(Sample)));w.count=0;};
    for(auto& e:registry)if(e.handle&&e.o.kind==2){
        int count=e.o.anchor==GW_ANCHOR_HITS?5:1;
        for(int n=0;n<count;++n){if(w.count==128)flush();Sample& s=w.samples[w.count++];s.emitter=e;s.index=n;
            int anchor=e.o.anchor==GW_ANCHOR_HITS?GW_ANCHOR_HITBOX:e.o.anchor,index=count>1?n:e.o.index;
            s.valid=gw_ScriptGame_MotionAnchorI(e.o.port-1,e.o.sub,anchor,index,e.o.item,0);
            s.element=gw_ScriptGame_MotionAnchorI(e.o.port-1,e.o.sub,anchor,index,e.o.item,1);
            s.entity=gw_ScriptGame_MotionAnchorI(e.o.port-1,e.o.sub,anchor,index,e.o.item,2);
            s.falls=gw_ScriptGame_MotionAnchorI(e.o.port-1,e.o.sub,anchor,index,e.o.item,3);
            s.costume=gw_ScriptGame_MotionAnchorI(e.o.port-1,e.o.sub,anchor,index,e.o.item,4);
            float* p=&s.point.x;for(int k=0;k<3;++k)p[k]=gw_ScriptGame_MotionAnchorF(e.o.port-1,e.o.sub,anchor,index,e.o.item,k)+e.o.offset[k];
            for(int k=0;k<3;++k)if(!std::isfinite(p[k])||std::abs(p[k])>100000)s.valid=0;
        }
    }
    flush();
}
#include "gw_motion_tests.inc"
