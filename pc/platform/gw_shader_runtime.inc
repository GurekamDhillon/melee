// Included by gw_fx_render.cpp: no new link object, Aurora and shim_gx stay untouched.
#include "gw_shader.h"
#include "gw_shader_contract.hpp"
#include "gw_shader_paths.hpp"
#include "gw_profiler.h"
#include <atomic>
#include <chrono>
#include <condition_variable>
#include <filesystem>
#include <memory>
#include <cstdio>
#include <thread>

namespace shader_runtime {
using namespace gwshader;
using aurora::gfx::Range;
using Clock = std::chrono::steady_clock;
struct ProfScope {
  bool enabled;
  ProfScope(unsigned zone, const std::string& name) : enabled(gw_prof_enabled()!=0) {
    if (!enabled) return;
    unsigned detail=2166136261u;
    for(unsigned char c:name) detail=(detail^c)*16777619u;
    gw_prof_detail_name(detail,name.c_str()); gw_prof_begin(zone,detail);
  }
  ~ProfScope() { if(enabled) gw_prof_end(); }
};
struct Program {
  Source source; Schema schema;
  std::string identity;
  wgpu::ShaderModule module;
  bool valid=false;
  std::string error, pipeline_error; // pipeline_error is guarded by state_mutex
};
struct Asset {
  int owner=0; std::string root, fragment, vertex, params_json, builtin;
  uint64_t stamp=0; Clock::time_point poll{};
  std::shared_ptr<Program> program;
  Schema schema;
  std::vector<float> values;
  bool effect=false;
};
struct Post { int shader=0, order=0, stage=0; uint32_t serial=0; bool half=false,owns_shader=false,engine=false; std::vector<float> values; std::shared_ptr<Program> prepared; };
Handles<std::shared_ptr<Asset>,128> assets;
Handles<Post,32> posts;
std::map<uint64_t,std::vector<std::weak_ptr<Program>>> cache;
std::mutex state_mutex;
GwShaderPerf perf{};
int light_owner=0;
struct Override { int owner=0,shader=0; };
std::map<std::pair<int,int>,Override> overrides;
std::atomic<bool> has_overrides{false};
float light_dir[4]={.25f,1,.55f,0}, light_color[4]={235.f/255,235.f/255,240.f/255,0};
float ambient[4]={92.f/255,96.f/255,112.f/255,0};

void error_out(const std::string& s,char* out,int cap) { if (out&&cap>0) std::snprintf(out,size_t(cap),"%s",s.c_str()); }
std::string sv(wgpu::StringView v) { return v.data?std::string(v.data,v.length==WGPU_STRLEN?std::strlen(v.data):v.length):std::string(); }

// Open and inspect the same handle that supplies bytes (including hot reload).
bool read_contained(const std::string& root,const std::string& rel,std::string& text,uint64_t& stamp,std::string& error,
                    unsigned limit=65536,bool binary=false) {
  if (!valid_path(rel)) { error="invalid relative shader path: "+rel; return false; }
#ifdef _WIN32
  char basepath[1024], final[1024];
  HANDLE base=CreateFileA(root.c_str(),FILE_READ_ATTRIBUTES,FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,
                          nullptr,OPEN_EXISTING,FILE_FLAG_BACKUP_SEMANTICS,nullptr);
  if (base==INVALID_HANDLE_VALUE) { error="shader root unavailable"; return false; }
  DWORD n=GetFinalPathNameByHandleA(base,basepath,sizeof basepath,FILE_NAME_NORMALIZED|VOLUME_NAME_DOS);
  CloseHandle(base);
  if (!n||n>=sizeof basepath) { error="shader root path too long"; return false; }
  std::string path=root+"/"+rel;
  HANDLE h=CreateFileA(path.c_str(),GENERIC_READ,FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,
                       nullptr,OPEN_EXISTING,FILE_FLAG_BACKUP_SEMANTICS,nullptr);
  if (h==INVALID_HANDLE_VALUE) { error="shader file unavailable: "+rel; return false; }
  BY_HANDLE_FILE_INFORMATION info{};
  n=GetFinalPathNameByHandleA(h,final,sizeof final,FILE_NAME_NORMALIZED|VOLUME_NAME_DOS);
  bool ok=n&&n<sizeof final&&inside(basepath,final)&&GetFileType(h)==FILE_TYPE_DISK&&
          GetFileInformationByHandle(h,&info)&&!(info.dwFileAttributes&FILE_ATTRIBUTE_DIRECTORY)&&
          !info.nFileSizeHigh&&info.nFileSizeLow<=limit;
  if (!ok) { CloseHandle(h); error="shader escapes mod root, is not a regular file, or exceeds its byte limit: "+rel; return false; }
  text.resize(info.nFileSizeLow); DWORD got=0,extra=0; char tail;
  ok=ReadFile(h,text.data(),info.nFileSizeLow,&got,nullptr)&&got==info.nFileSizeLow&&ReadFile(h,&tail,1,&extra,nullptr)&&!extra;
  BY_HANDLE_FILE_INFORMATION after{};
  ok=ok&&GetFileInformationByHandle(h,&after)&&after.nFileSizeLow==info.nFileSizeLow&&
     after.ftLastWriteTime.dwHighDateTime==info.ftLastWriteTime.dwHighDateTime&&
     after.ftLastWriteTime.dwLowDateTime==info.ftLastWriteTime.dwLowDateTime;
  CloseHandle(h);
  if (!ok) { error="shader changed while reading: "+rel; return false; }
  stamp=uint64_t(info.ftLastWriteTime.dwHighDateTime)<<32|info.ftLastWriteTime.dwLowDateTime;
  stamp^=uint64_t(info.nFileSizeLow)*1099511628211ull;
#else
  // Non-Windows port: canonical root and final file are checked before reading.
  // Windows is the supported author workflow; do not claim POSIX race resistance.
  std::error_code ec;
  auto base=std::filesystem::canonical(root,ec); if (ec) { error="shader root unavailable"; return false; }
  auto path=std::filesystem::canonical(base/rel,ec);
  if (ec||!inside(base.string(),path.string())||!std::filesystem::is_regular_file(path)||std::filesystem::file_size(path)>limit) { error="shader path escapes root or unavailable"; return false; }
  FILE* f=std::fopen(path.string().c_str(),"rb"); if (!f) { error="shader unreadable"; return false; }
  text.resize(size_t(std::filesystem::file_size(path))); bool ok=std::fread(text.data(),1,text.size(),f)==text.size(); std::fclose(f);
  if (!ok) { error="shader read failed"; return false; }
  stamp=uint64_t(std::filesystem::last_write_time(path).time_since_epoch().count());
#endif
  if (!binary&&text.find('\0')!=text.npos) { error="shader contains NUL bytes: "+rel; return false; }
  return true;
}

const char* header=R"WGSL(
struct U {
  view: array<vec4f,3>, proj: array<vec4f,4>, screen: vec4f, clock: vec4f,
  dir: vec4f, light: vec4f, ambient: vec4f, tint: vec4f, material: vec4f,
};
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var<storage,read> vertices: array<vec4f>;
@group(0) @binding(2) var t0: texture_2d<f32>;
@group(0) @binding(3) var t1: texture_2d<f32>;
@group(0) @binding(4) var t2: texture_2d<f32>;
@group(0) @binding(5) var scene: texture_2d<f32>;
@group(0) @binding(6) var depth: texture_2d<f32>;
@group(0) @binding(7) var previous: texture_2d<f32>;
@group(0) @binding(8) var linear_sampler: sampler;
struct Input { uv: vec2f, screen_uv: vec2f, color: vec4f, color1: vec4f,
  normal: vec3f, world: vec3f, fresnel: f32, param: f32, ii: u32 };
fn scene_color(uv: vec2f) -> vec4f { return textureSampleLevel(scene,linear_sampler,uv,0.0); }
fn previous_color(uv: vec2f) -> vec4f { return textureSampleLevel(previous,linear_sampler,uv,0.0); }
fn scene_depth(uv: vec2f) -> f32 {
  let size=textureDimensions(depth); let xy=clamp(vec2i(uv*vec2f(size)),vec2i(0),vec2i(size)-1);
  return textureLoad(depth,xy,0).r;
}
// Solve clip.z/clip.w for eye.z; works for perspective, ortho and reversed Z.
fn linear_depth(z: f32) -> f32 {
  let denom=z*u.proj[3].z-u.proj[2].z;
  return abs((u.proj[2].w-z*u.proj[3].w)/select(denom,1e-7,abs(denom)<1e-7));
}
fn time_seconds() -> f32 { return u.clock.x; }
fn resolution() -> vec2f { return u.screen.xy; }
fn builtin_material(in: Input) -> vec4f {
  let albedo=textureSampleLevel(t0,linear_sampler,in.uv,0.0)*u.tint*in.color;
  let glow=textureSampleLevel(t2,linear_sampler,in.uv,0.0).rgb*u.material.z;
  let map=textureSampleLevel(t1,linear_sampler,in.uv,0.0).xyz*2.0-1.0;
  let dx=dpdx(in.world); let dy=dpdy(in.world); let ux=dpdx(in.uv); let uy=dpdy(in.uv);
  let n=normalize(in.normal+vec3f(1e-7));
  let tangent=normalize(dx*uy.y-dy*ux.y+vec3f(1e-7));
  let bitangent=normalize(-dx*uy.x+dy*ux.x+vec3f(1e-7));
  let mapped=normalize(tangent*map.x+bitangent*map.y+n*map.z);
  let nn=select(n,mapped,u.material.w>0.5);
  let key=max(dot(nn,normalize(u.dir.xyz)),0.0);
  var rgb=albedo.rgb*min(u.ambient.rgb+u.light.rgb*key,vec3f(1.0))+glow;
  if (u.clock.w>0.5) { rgb=albedo.rgb+glow; }
  var alpha=albedo.a*u.material.x;
  if (u.clock.z>0.5) {
    let edge=pow(clamp(in.fresnel,0.0,1.0),mix(5.0,1.0,u.material.y));
    rgb=mix(rgb,u.tint.rgb,edge*0.6);
    // Authored opacity replaces the old atlas alpha; that atlas depended on repeated GX draws.
    alpha=u.material.x*u.tint.a*in.color.a;
    alpha=clamp(alpha+edge*(1.0-alpha)*0.65,0.0,1.0);
  }
  return vec4f(rgb,alpha);
}
)WGSL";
const char* tail=R"WGSL(
struct V { @builtin(position) pos: vec4f, @location(0) uv: vec2f,
 @location(1) normal: vec3f, @location(2) world: vec3f, @location(3) fresnel: f32 };
@vertex fn vs_full(@builtin(vertex_index) vi:u32) -> V {
 let xy=array<vec2f,3>(vec2f(-1,1),vec2f(-1,-3),vec2f(3,1))[vi]; var o:V;
 o.pos=vec4f(xy,0,1); o.uv=vec2f(xy.x*.5+.5,.5-xy.y*.5); o.normal=vec3f(0,0,1); o.world=vec3f(0); o.fresnel=0; return o;
}
@vertex fn vs_model(@builtin(vertex_index) vi:u32) -> V {
 let a=vertices[vi*3u]; let b=vertices[vi*3u+1u];
 let w=vec4f(mod_vertex(a.xyz,b.yzw,vec2f(a.w,b.x)),1);
 let vp=vec4f(dot(u.view[0],w),dot(u.view[1],w),dot(u.view[2],w),1); var o:V;
 o.pos=vec4f(dot(u.proj[0],vp),dot(u.proj[1],vp),dot(u.proj[2],vp),dot(u.proj[3],vp));
 o.uv=vec2f(a.w,b.x); o.normal=b.yzw; o.world=w.xyz;
 o.fresnel=1.0-abs(dot(normalize(b.yzw+vec3f(1e-7)),normalize(u.view[2].xyz))); return o;
}
@fragment fn fs_main(v:V) -> @location(0) vec4f {
 var in:Input; in.uv=v.uv; in.screen_uv=v.pos.xy*u.screen.zw; in.normal=v.normal;
 in.world=v.world; in.color=vec4f(1); in.color1=vec4f(1); in.fresnel=v.fresnel; in.param=1; in.ii=0u;
 let c=mod_fragment(in); if (c.a<=0.0) { discard; } return c;
}
@fragment fn fs_post(v:V) -> @location(0) vec4f {
 var in:Input; in.uv=v.uv; in.screen_uv=v.pos.xy*u.screen.zw; in.normal=v.normal;
 in.world=v.world; in.color=vec4f(1); in.color1=vec4f(1); in.fresnel=v.fresnel; in.param=1; in.ii=0u;
 return mod_fragment(in);
}
)WGSL";

struct Validation {
  std::mutex mutex; std::condition_variable cv; bool done=false, ok=false; std::string error;
};
template<class F> bool wait_validation(F fn,std::string& error) {
  auto v=std::make_shared<Validation>(); fn(v);
  std::unique_lock<std::mutex> lk(v->mutex);
  if (!v->cv.wait_for(lk,std::chrono::seconds(2),[&]{return v->done;})) { error="shader validation timed out; draw skipped"; return false; }
  error=v->error; return v->ok;
}
std::shared_ptr<Program> compile(const Source& source,const Schema& schema,const std::string& fragment,const std::string& vertex,std::string& error) {
  uint64_t hash=content_hash(source.text);
  for (auto& weak:cache[hash]) if (auto p=weak.lock()) if (p->valid && p->source.text==source.text) {
    ++perf.cache_hits; error=p->error; return p;
  }
  ProfScope profile(GW_PROF_SHADER_COMPILE,fragment);
  auto p=std::make_shared<Program>(); p->source=source; p->schema=schema; p->identity=fragment;
  auto dev=aurora::gfx::device();
  if (!dev) { error="render device unavailable (headless); shader not compiled"; p->error=error; return p; }
  ++perf.compiles;
  dev.PushErrorScope(wgpu::ErrorFilter::Validation);
  wgpu::ShaderSourceWGSL wgsl{}; wgsl.code=source.text.c_str();
  wgpu::ShaderModuleDescriptor md{}; md.nextInChain=&wgsl; md.label=fragment.c_str();
  p->module=dev.CreateShaderModule(&md);
  std::string scoped;
  bool scope_ok=wait_validation([&](auto v) { dev.PopErrorScope(wgpu::CallbackMode::AllowSpontaneous,
    [v](wgpu::PopErrorScopeStatus status,wgpu::ErrorType type,wgpu::StringView msg) {
      std::lock_guard<std::mutex> lk(v->mutex); v->ok=status==wgpu::PopErrorScopeStatus::Success&&type==wgpu::ErrorType::NoError;
      v->error=sv(msg); v->done=true; v->cv.notify_all(); }); },scoped);
  std::string diagnostic;
  bool info_ok=wait_validation([&](auto v) { p->module.GetCompilationInfo(wgpu::CallbackMode::AllowSpontaneous,
    [v,source,fragment,vertex](wgpu::CompilationInfoRequestStatus status,const wgpu::CompilationInfo* info) {
      std::lock_guard<std::mutex> lk(v->mutex); v->ok=status==wgpu::CompilationInfoRequestStatus::Success&&info;
      if (info) for (size_t i=0;i<info->messageCount;++i) {
        const auto& m=info->messages[i];
        if (m.type!=wgpu::CompilationMessageType::Error) continue;
        v->ok=false; bool vert=m.lineNum>=source.vertex_line&&m.lineNum<source.vertex_end_line;
        v->error+=(vert?vertex:fragment)+":"+std::to_string(source.line(unsigned(m.lineNum),vert))+":"+std::to_string(m.linePos)+": "+sv(m.message)+"\n";
      }
      if (!v->ok&&v->error.empty()) v->error="shader compilation information unavailable";
      v->done=true; v->cv.notify_all(); }); },diagnostic);
  p->valid=scope_ok&&info_ok;
  if (!p->valid) { error=diagnostic.empty()?scoped:diagnostic; p->error=error; ++perf.errors; gw_log("shader: %s",error.c_str()); }
  cache[hash].push_back(p);
  if(cache.size()>256) {
    for(auto it=cache.begin();it!=cache.end();) {
      auto& bucket=it->second;bucket.erase(std::remove_if(bucket.begin(),bucket.end(),[](auto& w){return w.expired();}),bucket.end());
      if(bucket.empty())it=cache.erase(it);else ++it;
    }
    if(cache.size()>256)cache.erase(cache.begin());
  }
  return p;
}

// Effect adapter is supplied below, beside the original effect module.
Source effect_source(const Schema&,const std::string&,const std::string&);
bool reload(Asset& a,bool force,std::string& error) {
  auto now=Clock::now(); if (!force&&now<a.poll) { error=a.program?a.program->error:"shader unavailable"; return a.program&&a.program->valid; }
  a.poll=now+std::chrono::milliseconds(250);
  std::string f,v; uint64_t fs=0,vs=0;
  if (!a.builtin.empty()) f=a.builtin;
  else if (!read_contained(a.root,a.fragment,f,fs,error)||(!a.vertex.empty()&&!read_contained(a.root,a.vertex,v,vs,error))) {
    // Deletion or a junction escape invalidates the draw too. Retry after the next stamp poll.
    if (!a.program || a.program->valid || a.program->error!=error) { ++perf.errors; gw_log("shader: %s",error.c_str()); }
    auto failed=std::make_shared<Program>(); failed->error=error; a.program=failed; a.stamp=0; return false;
  }
  // We already read bounded bytes for handle containment: include content too,
  // so atomic replacements/equal stamps cannot keep a stale shader alive.
  uint64_t stamp=fs^(vs*1000003ull)^content_hash(f)^(content_hash(v)*1099511628211ull);
  if (!force&&stamp==a.stamp&&a.program) { error=a.program->error; return a.program->valid; }
  if (!parse_schema(a.params_json,a.schema,error)) return false;
  auto source=a.effect?effect_source(a.schema,f,v):splice(header,a.schema,f,v,tail);
  a.program=compile(source,a.schema,a.fragment,a.vertex,error); a.stamp=stamp;
  if (a.values.empty()) a.values=a.schema.defaults();
  if (!force) ++perf.reloads;
  return a.program->valid;
}
std::shared_ptr<Asset> find(int owner,int h) { auto p=assets.get(uint32_t(h),owner); return p?*p:nullptr; }

struct alignas(16) Uniform {
  float view[3][4],proj[4][4],screen[4],clock[4],dir[4],light[4],ambient[4],tint[4],material[4];
};
static_assert(sizeof(Uniform)==224,"WGSL uniform contract");
void projection(float out[4][4]) {
  f32 pv[7]; GXGetProjectionv(pv); std::memset(out,0,64);
  out[0][0]=pv[1]; out[1][1]=pv[3]; out[2][2]=pv[5]; out[2][3]=pv[6];
  if (pv[0]!=0) { out[0][3]=pv[2];out[1][3]=pv[4];out[3][3]=1; }
  else { out[0][2]=pv[2];out[1][2]=pv[4];out[3][2]=-1; }
  for (int c=0;c<4;++c) { if (aurora::gfx::uses_reversed_z()) out[2][c]=-out[2][c]; else out[2][c]+=out[3][c]; }
}
void fill(Uniform& u,uint32_t w,uint32_t h) {
  u.screen[0]=float(w);u.screen[1]=float(h);u.screen[2]=1.f/std::max(w,1u);u.screen[3]=1.f/std::max(h,1u);
  static auto epoch=Clock::now();
  u.clock[0]=float(std::chrono::duration<double>(Clock::now()-epoch).count());
  std::copy(light_dir,light_dir+4,u.dir);std::copy(light_color,light_color+4,u.light);std::copy(ambient,ambient+4,u.ambient);
  std::fill(u.tint,u.tint+4,1); u.material[0]=u.material[2]=1;u.material[1]=.5f;
}
wgpu::BindGroupLayout bgl, param_bgl;
wgpu::PipelineLayout layout;
wgpu::Sampler sampler;
wgpu::TextureView white,black,flat_normal,depth0;
void gpu_init() {
  if (layout) return;
  auto dev=aurora::gfx::device();auto queue=aurora::gfx::queue();
  wgpu::BindGroupLayoutEntry e[9]{};
  for (unsigned i=0;i<9;++i) {e[i].binding=i;e[i].visibility=wgpu::ShaderStage::Fragment;}
  e[0].visibility|=wgpu::ShaderStage::Vertex; e[0].buffer.type=wgpu::BufferBindingType::Uniform;
  e[1].visibility=wgpu::ShaderStage::Vertex;e[1].buffer.type=wgpu::BufferBindingType::ReadOnlyStorage;
  for (int i=2;i<8;++i) {e[i].texture.viewDimension=wgpu::TextureViewDimension::e2D;e[i].texture.sampleType=i==6?wgpu::TextureSampleType::UnfilterableFloat:wgpu::TextureSampleType::Float;}
  e[8].sampler.type=wgpu::SamplerBindingType::Filtering;
  wgpu::BindGroupLayoutDescriptor bd{};bd.entryCount=9;bd.entries=e;bgl=dev.CreateBindGroupLayout(&bd);
  wgpu::BindGroupLayoutEntry pe{};pe.binding=0;pe.visibility=wgpu::ShaderStage::Vertex|wgpu::ShaderStage::Fragment;pe.buffer.type=wgpu::BufferBindingType::Uniform;
  bd.entryCount=1;bd.entries=&pe;param_bgl=dev.CreateBindGroupLayout(&bd);
  wgpu::BindGroupLayout groups[]={bgl,param_bgl};wgpu::PipelineLayoutDescriptor ld{};ld.bindGroupLayoutCount=2;ld.bindGroupLayouts=groups;layout=dev.CreatePipelineLayout(&ld);
  sampler=make_sampler(dev,wgpu::AddressMode::ClampToEdge);
  uint8_t w[]={255,255,255,255},b[]={0,0,0,255},n[]={128,128,255,255};float z=0;
  white=solid_view(dev,queue,wgpu::TextureFormat::RGBA8Unorm,w,4);black=solid_view(dev,queue,wgpu::TextureFormat::RGBA8Unorm,b,4);
  flat_normal=solid_view(dev,queue,wgpu::TextureFormat::RGBA8Unorm,n,4);depth0=solid_view(dev,queue,wgpu::TextureFormat::R32Float,&z,4);
}
struct Pipe { std::shared_ptr<Program> program; uint64_t key=0; int kind=0,blend=0;
              wgpu::RenderPipeline pipeline; };
std::vector<Pipe> pipes;
wgpu::RenderPipeline get_pipeline(std::shared_ptr<Program> p,const aurora::gfx::RenderTargetLayout& target,int kind,int blend) {
  if (!p||!p->valid) return {};
  for (auto& entry:pipes) if (entry.program==p&&entry.key==target.key&&entry.kind==kind&&entry.blend==blend) return entry.pipeline;
  ProfScope profile(GW_PROF_PIPELINE_COMPILE,p->identity);
  gpu_init();
  wgpu::BlendState bs{};bs.color.srcFactor=blend?wgpu::BlendFactor::SrcAlpha:wgpu::BlendFactor::One;
  bs.color.dstFactor=blend?wgpu::BlendFactor::OneMinusSrcAlpha:wgpu::BlendFactor::Zero;
  bs.alpha.srcFactor=kind==0?wgpu::BlendFactor::One:wgpu::BlendFactor::Zero;
  bs.alpha.dstFactor=kind==0?wgpu::BlendFactor::Zero:wgpu::BlendFactor::One;
  wgpu::ColorTargetState colors[aurora::gfx::MaxColorAttachments]{};
  for (unsigned i=0;i<target.colorAttachmentCount;++i) {colors[i].format=target.colorAttachments[i].format;colors[i].writeMask=i==0?wgpu::ColorWriteMask::All:wgpu::ColorWriteMask::None;}
  colors[0].blend=&bs;
  wgpu::FragmentState fs{};fs.module=p->module;fs.entryPoint=kind==1?"fs_main":"fs_post";fs.targetCount=target.colorAttachmentCount;fs.targets=colors;
  wgpu::DepthStencilState ds{};ds.format=target.depthStencilFormat;ds.depthWriteEnabled=kind==1&&!blend;
  ds.depthCompare=kind==1?(aurora::gfx::uses_reversed_z()?wgpu::CompareFunction::GreaterEqual:wgpu::CompareFunction::LessEqual):wgpu::CompareFunction::Always;
  wgpu::RenderPipelineDescriptor rd{};rd.label="mod shader";rd.layout=layout;rd.vertex.module=p->module;
  rd.vertex.entryPoint=kind==1?"vs_model":"vs_full";rd.primitive.topology=wgpu::PrimitiveTopology::TriangleList;
  rd.depthStencil=ds.format==wgpu::TextureFormat::Undefined?nullptr:&ds;rd.multisample.count=target.sampleCount;rd.fragment=&fs;
  auto pipe=std::make_shared<wgpu::RenderPipeline>();
  // Compile before recording. Failed pipelines never reach SetPipeline, including half-res copies.
  std::string error;
  bool ok=wait_validation([&](auto v){aurora::gfx::device().CreateRenderPipelineAsync(&rd,wgpu::CallbackMode::AllowSpontaneous,
    [v,pipe](wgpu::CreatePipelineAsyncStatus status,wgpu::RenderPipeline compiled,wgpu::StringView msg){
      std::lock_guard<std::mutex> lk(v->mutex);v->ok=status==wgpu::CreatePipelineAsyncStatus::Success;
      if(v->ok)*pipe=compiled;v->error=sv(msg);v->done=true;v->cv.notify_all();});},error);
  if(!ok){++perf.errors;p->pipeline_error=error;gw_log("shader: pipeline: %s",error.c_str());}
  pipes.push_back({p,target.key,kind,blend,ok?*pipe:wgpu::RenderPipeline{}});
  // Keep cache bounded across edit/save sessions. Recorded packets retain their own pipelines.
  if(pipes.size()>128)pipes.erase(pipes.begin(),pipes.begin()+32);
  return ok?*pipe:wgpu::RenderPipeline{};
}

struct Packet {
  wgpu::RenderPipeline pipeline;
  wgpu::TextureView textures[6]; // albedo, normal, emissive, scene, depth, previous
  Range uniform,params,vertices;
  uint32_t count=3;
  bool full=true;
};
std::mutex packets_mutex;
std::map<uint64_t,Packet> packets; uint64_t next_packet=0;
aurora::gfx::DrawTypeId draw_type=0;
void draw_callback(const aurora::gfx::DrawContext& ctx,const wgpu::RenderPassEncoder& pass,const void* bytes,size_t size,void*) {
  if(size!=sizeof(uint64_t))return;uint64_t id;std::memcpy(&id,bytes,sizeof id);Packet d;
  {std::lock_guard<std::mutex> lock(packets_mutex);auto it=packets.find(id);if(it==packets.end())return;d=it->second;packets.erase(it);}
  if(!d.pipeline)return;
  wgpu::BindGroupEntry e[9]{};for(unsigned i=0;i<9;++i)e[i].binding=i;
  e[0].buffer=ctx.uniformBuffer;e[0].offset=d.uniform.offset;e[0].size=d.uniform.size;
  e[1].buffer=ctx.storageBuffer;e[1].offset=d.vertices.offset;e[1].size=d.vertices.size;
  for(int i=0;i<6;++i)e[i+2].textureView=d.textures[i];e[8].sampler=sampler;
  wgpu::BindGroupDescriptor bd{};bd.layout=bgl;bd.entries=e;bd.entryCount=9;auto bg=ctx.device.CreateBindGroup(&bd);
  wgpu::BindGroupEntry pe{};pe.binding=0;pe.buffer=ctx.uniformBuffer;pe.offset=d.params.offset;pe.size=d.params.size;
  bd.layout=param_bgl;bd.entries=&pe;bd.entryCount=1;auto pb=ctx.device.CreateBindGroup(&bd);
  pass.SetPipeline(d.pipeline);pass.SetBindGroup(0,bg);pass.SetBindGroup(1,pb);
  if(d.full){uint32_t w=ctx.layout.colorAttachments[0].width,h=ctx.layout.colorAttachments[0].height;
    pass.SetViewport(0,0,float(w),float(h),0,1);pass.SetScissorRect(0,0,w,h);}
  pass.Draw(d.count);
}
bool record(Packet d,const Uniform& u,const std::vector<float>& values) {
  if(!d.pipeline){++perf.skipped;return false;}
  d.uniform=aurora::gfx::push_uniform(reinterpret_cast<const uint8_t*>(&u),sizeof u);
  d.params=aurora::gfx::push_uniform(reinterpret_cast<const uint8_t*>(values.data()),values.size()*4);
  if(!d.vertices.size) {float dummy[12]{};d.vertices=aurora::gfx::push_storage(reinterpret_cast<const uint8_t*>(dummy),sizeof dummy);}
  if(!d.uniform.size||!d.params.size||!d.vertices.size){++perf.skipped;return false;}
  if(!draw_type){aurora::gfx::DrawTypeDescriptor desc{};desc.label="mod shader";desc.draw=draw_callback;draw_type=aurora::gfx::register_draw_type(desc);}
  uint64_t id;
  {std::lock_guard<std::mutex> lock(packets_mutex);id=++next_packet;packets.emplace(id,d);}
  if(!aurora::gfx::push_custom_draw(draw_type,&id,sizeof id)) {std::lock_guard<std::mutex> lock(packets_mutex);packets.erase(id);++perf.skipped;return false;}
  ++perf.draws;perf.uniform_bytes+=sizeof u+values.size()*4;return true;
}
} // namespace shader_runtime

namespace shader_runtime {
std::atomic<unsigned> post_stages{0};
float world_projection[4][4]{};
bool have_world_projection=false;
void refresh_stages() { unsigned bits=0;posts.each([&](uint32_t,int,Post& p){bits|=1u<<p.stage;});post_stages.store(bits); }
bool update_values(const Schema& schema,std::vector<float>& values,const char* json,std::string& error) {
  Json j;if(!Reader(json?json:"{}").parse(j)||j.kind!=Json::Object){error="params must be an object";return false;}
  auto next=values;
  for(const auto& pair:j.object){std::vector<float> v;if(!floats(pair.second,v)||!set_param(schema,next,pair.first,v,error))return false;}
  values=std::move(next);return true;
}
struct PostWork {int stage=0;Uniform uniform{};};
std::mutex work_mutex;
std::map<uint64_t,PostWork> post_work;uint64_t next_work=0;
std::shared_ptr<Program> copy_program;
void post_record(const void* bytes,u32 size) {
  if(size!=sizeof(uint64_t))return;uint64_t id;std::memcpy(&id,bytes,sizeof id);PostWork work;
  {std::lock_guard<std::mutex> lock(work_mutex);auto it=post_work.find(id);if(it==post_work.end())return;work=it->second;post_work.erase(it);}
  std::lock_guard<std::mutex> state(state_mutex);
  auto start=Clock::now();
  std::vector<Post> chain;posts.each([&](uint32_t,int,Post& p){if(p.stage==work.stage)chain.push_back(p);});
  std::sort(chain.begin(),chain.end(),[](const Post& a,const Post& b){return a.order!=b.order?a.order<b.order:a.serial<b.serial;});
  if(chain.empty()||aurora::gfx::is_offscreen())return;
  struct Ready{Post post;std::shared_ptr<Program> program;};std::vector<Ready> ready;
  for(auto& post:chain){auto a=find(0,post.shader);std::string error;if(a&&reload(*a,false,error))ready.push_back({post,a->program});else ++perf.skipped;}
  if(ready.empty())return;
  gpu_init();
  auto target=aurora::gfx::scene_render_target_layout();
  // Offscreen create_pass uses Aurora's colour format and fixed single-sample layout.
  auto offscreen_layout=target;offscreen_layout.key=0xfade0001ull;offscreen_layout.colorAttachmentCount=1;offscreen_layout.sampleCount=1;
  offscreen_layout.colorAttachments[0].format=aurora::gfx::color_format();offscreen_layout.depthStencilFormat=aurora::gfx::depth_format();
  if(!copy_program){Schema empty;std::string error;copy_program=compile(splice(header,empty,"return previous_color(in.uv);","",tail),empty,"<post composite>","",error);}
  auto composite=get_pipeline(copy_program,target,2,0);if(!composite){++perf.skipped;return;}
  // Preflight every pass before taking snapshots or overwriting the EFB.
  std::vector<wgpu::RenderPipeline> compiled;
  for(auto& r:ready)compiled.push_back(get_pipeline(r.program,offscreen_layout,0,0));
  bool any=false;for(auto& p:compiled)any|=bool(p);if(!any)return;
  aurora::gfx::ResolveDesc resolve{};resolve.depth=true;aurora::gfx::ResolvedTargets scene;
  if(!aurora::gfx::resolve_pass(resolve,scene)||!scene.color){++perf.skipped;return;}++perf.resolves;
  auto previous=scene.color;
  for(size_t i=0;i<ready.size();++i){
    if(!compiled[i]){++perf.skipped;continue;}
    uint32_t w=std::max(1u,scene.width/(ready[i].post.half?2u:1u)),h=std::max(1u,scene.height/(ready[i].post.half?2u:1u));
    if(!aurora::gfx::create_pass_profiled(w,h,ready[i].program->identity.c_str())){++perf.skipped;continue;}
    Uniform u=work.uniform;fill(u,w,h);
    Packet p;p.pipeline=compiled[i];p.textures[0]=white;p.textures[1]=flat_normal;p.textures[2]=black;
    p.textures[3]=scene.color;p.textures[4]=scene.depth?scene.depth:depth0;p.textures[5]=previous;
    bool drawn=record(p,u,ready[i].post.values);
    aurora::gfx::ResolvedTargets out;aurora::gfx::ResolveDesc color_only{};
    bool closed=aurora::gfx::resolve_pass(color_only,out);++perf.resolves;
    // Never adopt the cleared offscreen output if the draw could not be recorded.
    if(drawn&&closed&&out.color){
      previous=out.color;++perf.passes;perf.pixels+=uint64_t(w)*h;
      if(auto live=posts.get(ready[i].post.serial,0)) live->prepared=ready[i].program;
    }
    if(!closed) { ++perf.skipped; return; }
  }
  if(previous.Get()!=scene.color.Get()){
    Uniform u=work.uniform;fill(u,scene.width,scene.height);Packet p;p.pipeline=composite;
    p.textures[0]=white;p.textures[1]=flat_normal;p.textures[2]=black;p.textures[3]=scene.color;p.textures[4]=scene.depth?scene.depth:depth0;p.textures[5]=previous;
    record(p,u,copy_program->schema.defaults());perf.pixels+=uint64_t(scene.width)*scene.height;
  }
  perf.record_ms+=std::chrono::duration<double,std::milli>(Clock::now()-start).count();
}
} // namespace shader_runtime

extern "C" int gw_Shader_Load(int owner,const char* root,const char* fragment,const char* vertex,int effect,const char* params_json,char* out,int cap) {
  using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);std::string error;
  if(owner<=0||!root||!fragment){error_out("shader requires an owning mod",out,cap);return 0;}
  auto a=std::make_shared<Asset>();a->owner=owner;a->root=root;a->fragment=fragment;a->vertex=vertex?vertex:"";a->params_json=params_json?params_json:"{}";
  a->effect=effect!=0;
  if(!reload(*a,true,error)){error_out(error,out,cap);return 0;}
  int h=int(assets.add(owner,a));if(!h)error_out("shader handle capacity exhausted (128)",out,cap);return h;
}
extern "C" int gw_Shader_Set(int owner,int handle,const char* params_json,char* out,int cap) {
  using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);auto a=find(owner,handle);std::string error;
  if(!a){error_out("shader handle expired or belongs to another script",out,cap);return 0;}
  if(!update_values(a->schema,a->values,params_json,error)){error_out(error,out,cap);return 0;}return 1;
}
extern "C" int gw_Shader_Status(int owner,int handle,char* out,int cap) {
  using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);auto a=find(owner,handle);std::string error;
  if(!a){error_out("shader handle expired or belongs to another script",out,cap);return 0;}
  bool ok=reload(*a,false,error);
  if(!a->program->pipeline_error.empty()){error=a->program->pipeline_error;ok=false;}
  error_out(error,out,cap);return ok?1:-1;
}
extern "C" int gw_Post_Add(int owner,int shader,int order,int stage,int half,int owns_shader,char* out,int cap) {
  using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);auto a=find(owner,shader);
  if(!a||a->effect||stage<0||stage>1){if(a&&owns_shader)assets.remove(uint32_t(shader),owner);error_out("post requires a post shader owned by this script and world/final stage",out,cap);return 0;}
  Post p;p.shader=shader;p.order=order;p.stage=stage;p.half=half!=0;p.values=a->values;p.owns_shader=owns_shader!=0;
  int h=int(posts.add(owner,p));if(h)posts.get(uint32_t(h),owner)->serial=uint32_t(h);
  else {if(owns_shader)assets.remove(uint32_t(shader),owner);error_out("post chain capacity exhausted (32)",out,cap);}
  refresh_stages();return h;
}
#include "gw_shader_stage_cover.inc"
extern "C" int gw_Post_Set(int owner,int handle,const char* params_json,char* out,int cap) {
  using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);auto p=posts.get(uint32_t(handle),owner);std::string error;
  auto a=p?find(owner,p->shader):nullptr;
  if(!a){error_out("post handle expired or belongs to another script",out,cap);return 0;}
  if(!update_values(a->schema,p->values,params_json,error)){error_out(error,out,cap);return 0;}return 1;
}
extern "C" int gw_Post_Ready(int owner,int handle) {
  using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);
  if(owner<=0)return -1;
  auto p=posts.get(uint32_t(handle),owner);
  auto a=p?find(owner,p->shader):nullptr;
  if(!a)return -1;
  return p->prepared && p->prepared==a->program && a->program->pipeline_error.empty()?1:0;
}
extern "C" int gw_Post_Remove(int owner,int h) {using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);
  auto p=posts.get(uint32_t(h),owner);if(p&&p->owns_shader)assets.remove(uint32_t(p->shader),owner);
  bool ok=posts.remove(uint32_t(h),owner);refresh_stages();return ok;}
extern "C" void gw_Post_Clear(int owner) {using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);
  // Zero means no caller, never global authority. Scene cleanup uses Release.
  std::vector<uint32_t> remove;
  posts.each([&](uint32_t h,int o,Post& p){if(owner>0&&owner==o&&!p.engine){
    if(p.owns_shader)assets.remove(uint32_t(p.shader),o);remove.push_back(h);}});
  for(auto h:remove)posts.remove(h,owner);refresh_stages();}
extern "C" void gw_Post_Protect(int owner,int handle) {using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);
  if(owner>0)if(auto p=posts.get(uint32_t(handle),owner))p->engine=true;}
extern "C" void gw_Shader_Release(int owner) {
  using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);posts.clear(owner);
  if(owner)assets.clear(owner);else{std::vector<std::pair<uint32_t,int>> remove;assets.each([&](uint32_t h,int o,std::shared_ptr<Asset>&){if(o!=0x7fffffff)remove.push_back({h,o});});for(auto h:remove)assets.remove(h.first,h.second);}
  for(auto it=overrides.begin();it!=overrides.end();)if(!owner||it->second.owner==owner)it=overrides.erase(it);else ++it;
  has_overrides.store(!overrides.empty());refresh_stages();
  if(!owner||light_owner==owner){light_owner=0;float d[]={.25f,1,.55f,0},c[]={235.f/255,235.f/255,240.f/255,0},a[]={92.f/255,96.f/255,112.f/255,0};std::copy(d,d+4,light_dir);std::copy(c,c+4,light_color);std::copy(a,a+4,ambient);}
  if(!owner)have_world_projection=false;
}
extern "C" void gw_Shader_Perf(GwShaderPerf* out) {using namespace shader_runtime;if(!out)return;std::lock_guard<std::mutex> state(state_mutex);*out=perf;}
extern "C" void gw_Shader_Lights(int owner,const float* dir,const float* color,const float* amb) {
  using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);light_owner=owner;
  std::copy(dir,dir+3,light_dir);std::copy(color,color+3,light_color);std::copy(amb,amb+3,ambient);
}
extern "C" void gw_Shader_PostDraw(int stage) {
  using namespace shader_runtime;
  if(stage<0||stage>1)return;
  // Capture the world projection even for a final-only chain. No GX callback when inactive.
  if(!post_stages.load())return;
  PostWork work;work.stage=stage;
  if(stage==0){projection(world_projection);have_world_projection=true;}
  if(!(post_stages.load()&(1u<<stage)))return;
  if(have_world_projection)std::memcpy(work.uniform.proj,world_projection,sizeof world_projection);else projection(work.uniform.proj);
  uint64_t id;{std::lock_guard<std::mutex> lock(work_mutex);id=++next_work;post_work.emplace(id,work);}
  GXAuroraCallback(post_record,&id,sizeof id);
}
