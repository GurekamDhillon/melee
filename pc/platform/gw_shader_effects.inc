// Adapter over the established particle/mesh contract. No changes to emission or rollback.
namespace shader_runtime {
void replace_all(std::string& source,const std::string& before,const std::string& after) {
  size_t at=0;while((at=source.find(before,at))!=source.npos){source.replace(at,before.size(),after);at+=after.size();}
}
Source effect_source(const Schema& schema,const std::string& fragment,const std::string& vertex) {
  std::string module=kWGSL;
  replace_all(module,"  @location(3) fres: f32,", "  @location(4) normal: vec3f,\n  @location(5) world: vec3f,\n  @location(3) fres: f32,");
  replace_all(module,"  o.fres = 1.0;", "  o.normal = u.view[2].xyz; o.world = p.p0.xyz;\n  o.fres = 1.0;");
  replace_all(module,"  o.fres = 1.0 - abs(dot(u.view[2].xyz, nw));", "  o.normal = nw; o.world = w.xyz;\n  o.fres = 1.0 - abs(dot(u.view[2].xyz, nw));");
  replace_all(module,"fn shade(in: V)","fn builtin_shade(in: V)");
  replace_all(module,"p.p0.xyz + p.ax.xyz * c.x + p.ay.xyz * c.y",
              "mod_vertex(p.p0.xyz + p.ax.xyz * c.x + p.ay.xyz * c.y, normalize(p.az.xyz + vec3f(1e-7)), c + 0.5)");
  replace_all(module,"let w = vec4f(p.p0.xyz, 1.0);","let w = vec4f(mod_vertex(p.p0.xyz, u.view[2].xyz, c + 0.5), 1.0);");
  replace_all(module,"p.p0.xyz + p.ax.xyz * a.x + p.ay.xyz * a.y + p.az.xyz * a.z",
              "mod_vertex(p.p0.xyz + p.ax.xyz * a.x + p.ay.xyz * a.y + p.az.xyz * a.z, b.yzw, vec2f(a.w, b.x))");
  module+=R"WGSL(
struct Input { uv: vec2f, screen_uv: vec2f, color: vec4f, color1: vec4f,
 normal: vec3f, world: vec3f, fresnel: f32, param: f32, ii: u32 };
struct Runtime { clock:vec4f };
@group(1) @binding(1) var<uniform> runtime:Runtime;
fn time_seconds() -> f32 { return runtime.clock.x; }
fn resolution() -> vec2f { return u.screen.xy; }
fn scene_color(uv:vec2f) -> vec4f { return textureSampleLevel(tsrc,sl,uv,0.0); }
fn scene_depth(uv:vec2f) -> f32 {
 let size=textureDimensions(tdepth);return textureLoad(tdepth,clamp(vec2i(uv*vec2f(size)),vec2i(0),vec2i(size)-1),0).r;
}
fn linear_depth(z:f32) -> f32 {
 let d=z*u.proj[3].z-u.proj[2].z;
 return abs((u.proj[2].w-z*u.proj[3].w)/select(d,1e-7,abs(d)<1e-7));
}
)WGSL";
  const char* adapter=R"WGSL(
fn shade(v:V) -> vec4f {
 let p=parts[v.ii];var in:Input;in.uv=v.uv;in.screen_uv=v.pos.xy*u.screen.zw;
 in.color=p.c0*v.vc;in.color1=p.c1;in.world=v.world;in.normal=v.normal;
 in.fresnel=v.fres;in.param=p.p1.z;in.ii=v.ii;return mod_fragment(in);
}
)WGSL";
  return splice(module,schema,fragment,vertex,adapter);
}
struct FxPrepared {std::shared_ptr<Program> program;Range params,clock;};
std::map<uint64_t,FxPrepared> fx_prepared;uint64_t fx_serial=0;
std::mutex fx_mutex;
wgpu::BindGroupLayout fx_param_bgl;
wgpu::PipelineLayout fx_layout;
void fx_layout_init() {
  if(fx_layout)return;
  wgpu::BindGroupLayoutEntry e[2]{};for(unsigned i=0;i<2;++i){e[i].binding=i;e[i].visibility=wgpu::ShaderStage::Vertex|wgpu::ShaderStage::Fragment;e[i].buffer.type=wgpu::BufferBindingType::Uniform;}
  wgpu::BindGroupLayoutDescriptor bd{};bd.entries=e;bd.entryCount=2;auto dev=aurora::gfx::device();fx_param_bgl=dev.CreateBindGroupLayout(&bd);
  wgpu::BindGroupLayout groups[]={g_bgl,fx_param_bgl};wgpu::PipelineLayoutDescriptor ld{};ld.bindGroupLayoutCount=2;ld.bindGroupLayouts=groups;fx_layout=dev.CreatePipelineLayout(&ld);
}
uint64_t fx_prepare(int handle) {
  if(handle<=0){if(handle<0){std::lock_guard<std::mutex> state(state_mutex);++perf.skipped;}return 0;}
  std::lock_guard<std::mutex> state(state_mutex);auto a=find(0,handle);std::string error;
  if(!a||!reload(*a,false,error)){++perf.skipped;return 0;}
  FxPrepared ready;ready.program=a->program;
  ready.params=aurora::gfx::push_uniform(reinterpret_cast<const uint8_t*>(a->values.data()),a->values.size()*4);
  static auto epoch=Clock::now();float time[4]={float(std::chrono::duration<double>(Clock::now()-epoch).count()),0,0,0};
  ready.clock=aurora::gfx::push_uniform(reinterpret_cast<const uint8_t*>(time),sizeof time);
  if(!ready.params.size||!ready.clock.size){++perf.skipped;return 0;}
  perf.uniform_bytes+=a->values.size()*4+sizeof time;
  std::lock_guard<std::mutex> lock(fx_mutex);uint64_t id=++fx_serial;fx_prepared.emplace(id,ready);return id;
}
bool fx_take(uint64_t id,FxPrepared& out) {
  std::lock_guard<std::mutex> lock(fx_mutex);auto it=fx_prepared.find(id);if(it==fx_prepared.end())return false;out=it->second;fx_prepared.erase(it);return true;
}
struct FxPipe {std::shared_ptr<Program> program;uint64_t key;uint32_t kind,blend,depth;wgpu::RenderPipeline pipe;};
std::vector<FxPipe> fx_pipelines;
wgpu::RenderPipeline fx_pipeline(const FxPrepared& custom,const wgpu::RenderPipelineDescriptor& input,
                                uint64_t key,uint32_t kind,uint32_t blend,uint32_t depth) {
  for(auto& p:fx_pipelines)if(p.program==custom.program&&p.key==key&&p.kind==kind&&p.blend==blend&&p.depth==depth)return p.pipe;
  fx_layout_init();auto rd=input;auto fs=*input.fragment;rd.layout=fx_layout;rd.vertex.module=custom.program->module;fs.module=custom.program->module;rd.fragment=&fs;
  auto result=std::make_shared<wgpu::RenderPipeline>();std::string error;
  bool ok=wait_validation([&](auto v){aurora::gfx::device().CreateRenderPipelineAsync(&rd,wgpu::CallbackMode::AllowSpontaneous,
    [v,result](wgpu::CreatePipelineAsyncStatus status,wgpu::RenderPipeline pipe,wgpu::StringView msg){
      std::lock_guard<std::mutex> lock(v->mutex);v->ok=status==wgpu::CreatePipelineAsyncStatus::Success;
      if(v->ok)*result=pipe;v->error=sv(msg);v->done=true;v->cv.notify_all();});},error);
  if(!ok){std::lock_guard<std::mutex> state(state_mutex);++perf.errors;custom.program->pipeline_error=error;gw_log("shader effect pipeline: %s",error.c_str());}
  fx_pipelines.push_back({custom.program,key,kind,blend,depth,ok?*result:wgpu::RenderPipeline{}});
  if(fx_pipelines.size()>128)fx_pipelines.erase(fx_pipelines.begin(),fx_pipelines.begin()+32);
  return ok?*result:wgpu::RenderPipeline{};
}

extern "C" int gw_Shader_FxOverride(int owner,const char* package,const char* emitter,int shader,char* out,int cap) {
  using namespace shader_runtime;
  // Find may compile a package, so do not hold the shader registry lock across it.
  extern int gw_Fx_Find(const char*);
  int pk=gw_Fx_Find(package);const fx_pkg* p=gw_fx_pkg(pk);int em=-1;
  if(p)for(int i=0;i<p->nem;++i)if(!std::strcmp(p->em[i].name,emitter)){em=i;break;}
  if(em<0){error_out("effect package/emitter unavailable",out,cap);return 0;}
  std::lock_guard<std::mutex> state(state_mutex);auto a=shader?find(owner,shader):nullptr;
  if(shader&&(!a||!a->effect)){error_out("fx_shader requires an owned effect shader",out,cap);return 0;}
  auto key=std::make_pair(pk,em);auto old=overrides.find(key);
  if(old!=overrides.end()&&old->second.owner!=owner){error_out("emitter shader override belongs to another script",out,cap);return 0;}
  if(shader)overrides[key]={owner,shader};else overrides.erase(key);has_overrides.store(!overrides.empty());return 1;
}
extern "C" int gw_Shader_FxSelect(int package,int emitter,int fallback) {
  using namespace shader_runtime;if(!has_overrides.load())return fallback;
  std::lock_guard<std::mutex> state(state_mutex);auto it=overrides.find({package,emitter});return it==overrides.end()?fallback:it->second.shader;
}
void fx_bind(const FxPrepared& custom,const aurora::gfx::DrawContext& ctx,const wgpu::RenderPassEncoder& pass) {
  wgpu::BindGroupEntry e[2]{};e[0].binding=0;e[0].buffer=ctx.uniformBuffer;e[0].offset=custom.params.offset;e[0].size=custom.params.size;
  e[1].binding=1;e[1].buffer=ctx.uniformBuffer;e[1].offset=custom.clock.offset;e[1].size=custom.clock.size;
  wgpu::BindGroupDescriptor bd{};bd.layout=fx_param_bgl;bd.entries=e;bd.entryCount=2;auto bg=ctx.device.CreateBindGroup(&bd);pass.SetBindGroup(1,bg);
  {std::lock_guard<std::mutex> state(state_mutex);++perf.draws;}
}
} // namespace shader_runtime

extern "C" int gw_Shader_FxLoad(const char* package,const char* fragment,const char* vertex,const char* params_json) {
  using namespace shader_runtime;auto a=std::make_shared<Asset>();a->effect=true;a->owner=0x7fffffff;
  a->root=infer_root(package);a->fragment=relative_path(a->root,(std::filesystem::path(package).parent_path()/fragment).string());
  if(vertex&&vertex[0])a->vertex=relative_path(a->root,(std::filesystem::path(package).parent_path()/vertex).string());
  a->params_json=params_json?params_json:"{}";std::string error;
  if(a->root.empty()||!valid_path(fragment)||(!a->vertex.empty()&&!valid_path(vertex))){gw_log("shader: %s: invalid custom shader path or mod root",package);return -1;}
  std::lock_guard<std::mutex> state(state_mutex);reload(*a,true,error);
  int h=int(assets.add(a->owner,a));if(!h)gw_log("shader: effect shader capacity exhausted");return h?h:-1;
}
