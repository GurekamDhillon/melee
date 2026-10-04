// Runs the production shader code with Dawn's Null backend. No game or GPU.
#include "../platform/gw_fx_render.cpp"
#include <fstream>
#include <iostream>

static wgpu::Device test_device;
static bool test_recording=false;
static bool test_draw_accept=true;
static wgpu::TextureView test_color;
namespace aurora::gfx {
wgpu::Device device() noexcept { return test_device; }
wgpu::Queue queue() noexcept { return test_device.GetQueue(); }
wgpu::TextureFormat color_format() noexcept { return wgpu::TextureFormat::RGBA8Unorm; }
wgpu::TextureFormat depth_format() noexcept { return wgpu::TextureFormat::Depth24Plus; }
bool uses_reversed_z() noexcept { return true; }
RenderTargetLayout scene_render_target_layout() noexcept {
  RenderTargetLayout t;t.key=1;t.colorAttachmentCount=1;t.colorAttachments[0].format=color_format();
  t.colorAttachments[0].width=640;t.colorAttachments[0].height=480;t.depthStencilFormat=depth_format();return t;
}
DrawTypeId register_draw_type(const DrawTypeDescriptor&) { return 1; }
bool push_custom_draw(DrawTypeId,const void*,size_t) { return test_recording && test_draw_accept; }
Range push_uniform(const uint8_t*,size_t size) { return test_recording ? Range{0,uint32_t(size)} : Range{}; }
Range push_storage(const uint8_t*,size_t size) { return test_recording ? Range{0,uint32_t(size)} : Range{}; }
bool resolve_pass(const ResolveDesc&,ResolvedTargets& out) { if(!test_recording)return false;out.color=test_color;out.width=640;out.height=480;out.colorFormat=color_format();return true; }
bool create_pass(uint32_t,uint32_t) { return test_recording; }
bool create_pass_profiled(uint32_t w,uint32_t h,const char*) { return create_pass(w,h); }
bool is_offscreen() noexcept { return false; }
uint32_t current_frame() noexcept { return 0; }
}
extern "C" void gw_log(const char* format,...) { va_list args;va_start(args,format);vfprintf(stderr,format,args);fputc('\n',stderr);va_end(args); }
extern "C" void gw_test_register(const char*,gw_test_fn) {}
extern "C" void gw_test_fail(const char*,...) {}
extern "C" const fx_state* gw_fx_state(void) { return nullptr; }
extern "C" const fx_pkg* gw_fx_pkg(int) { return nullptr; }
extern "C" float gw_fx_curve_at(const fx_curve*,float,int) { return 0; }
extern "C" int gw_Fx_Find(const char*) { return -1; }
extern "C" void GXGetProjectionv(f32* out) { std::fill(out,out+7,0); }
extern "C" void GXAuroraCallback(void (*)(const void*,u32),const void*,u32) {}

extern "C" int gw_prof_enabled(void) { return 0; }
extern "C" void gw_prof_begin(unsigned,unsigned) {}
extern "C" void gw_prof_end(void) {}
extern "C" void gw_prof_detail_name(unsigned,const char*) {}

int main(int argc,char** argv) {
  using namespace shader_runtime;
  CHECK(argc==3);
  CHECK(shader_contract_test()==0);
#ifdef _WIN32
  CHECK(relative_path(R"(C:\mods\sample)", R"(\\?\C:\mods\sample\models\drive.material.json)") == "models/drive.material.json");
  CHECK(relative_path(R"(\\?\C:\mods\sample)", R"(C:\mods\sample\models\drive.material.json)") == "models/drive.material.json");
  CHECK(relative_path(R"(\\server\share\mods\sample)", R"(\\?\UNC\server\share\mods\sample\models\drive.material.json)") == "models/drive.material.json");
  CHECK(relative_path(R"(C:\mods\sample)", R"(\\?\C:\mods\sample2\models\drive.material.json)").empty());
#endif
  auto instance=wgpu::CreateInstance();
  wgpu::RequestAdapterOptions options{};options.backendType=wgpu::BackendType::Null;
  wgpu::Adapter adapter;
  std::string error;
  CHECK(wait_validation([&](auto v){instance.RequestAdapter(&options,wgpu::CallbackMode::AllowSpontaneous,
    [v,&adapter](wgpu::RequestAdapterStatus status,wgpu::Adapter a,wgpu::StringView message){
      std::lock_guard<std::mutex> lock(v->mutex);v->ok=status==wgpu::RequestAdapterStatus::Success;
      adapter=a;v->error=sv(message);v->done=true;v->cv.notify_all();});},error));
  CHECK(wait_validation([&](auto v){adapter.RequestDevice(nullptr,wgpu::CallbackMode::AllowSpontaneous,
    [v](wgpu::RequestDeviceStatus status,wgpu::Device d,wgpu::StringView message){
      std::lock_guard<std::mutex> lock(v->mutex);v->ok=status==wgpu::RequestDeviceStatus::Success;
      test_device=d;v->error=sv(message);v->done=true;v->cv.notify_all();});},error));
  Schema schema; CHECK(parse_schema("{}",schema,error));
  auto basic=compile(splice(header,schema,"return previous_color(in.uv);","",tail),schema,"test.wgsl","",error);
  CHECK(basic->valid);
  auto reused=compile(basic->source,schema,"same.wgsl","",error);CHECK(reused==basic && perf.cache_hits);
  auto bad=compile(splice(header,schema,"\nreturn does_not_exist;","",tail),schema,"bad.wgsl","",error);
  CHECK(!bad->valid && error.find("bad.wgsl:2:")!=std::string::npos);
  auto wrong_vertex=compile(splice(header,schema,"return vec4f(1);","\nreturn missing;",tail),schema,"ok.wgsl","vertex.wgsl",error);
  CHECK(!wrong_vertex->valid && error.find("vertex.wgsl:2:")!=std::string::npos);
  auto glass=compile(splice(header,schema,"return builtin_material(in);","",tail),schema,"glass","",error);CHECK(glass->valid);
  CHECK(get_pipeline(glass,aurora::gfx::scene_render_target_layout(),1,0));
  CHECK(get_pipeline(basic,aurora::gfx::scene_render_target_layout(),0,0));
  std::string root=argv[1], text;uint64_t stamp=0;
  CHECK(read_contained(root,"shaders/vignette.wgsl",text,stamp,error));
  CHECK(!read_contained(root,"../escape.wgsl",text,stamp,error));
  CHECK(read_contained(argv[2],"good.wgsl",text,stamp,error));
  CHECK(!read_contained(argv[2],"junction/stolen.wgsl",text,stamp,error));
  auto validate_example=[&](const char* file,const char* params){
    if(!read_contained(root,std::string("shaders/")+file,text,stamp,error)||!parse_schema(params,schema,error))return false;
    auto p=compile(splice(header,schema,text,"",tail),schema,file,"",error);return p->valid;
  };
  CHECK(validate_example("grade.wgsl",R"({"lift":[0,0,0,0],"gamma":[1,1,1,1],"gain":[1,1,1,1],"saturation":1})"));
  CHECK(validate_example("vignette.wgsl",R"({"strength":0.4,"radius":0.3,"softness":0.4})"));
  CHECK(validate_example("outline.wgsl",R"({"width":1,"threshold":0.02,"strength":0.8,"tint":[0,0,0,1]})"));
  CHECK(validate_example("bloom.wgsl",R"({"threshold":0.7,"intensity":0.5,"radius":1})"));
  CHECK(validate_example("bloom-compose.wgsl","{}"));
  CHECK(read_contained(root,"shaders/effect.wgsl",text,stamp,error));
  CHECK(parse_schema("{}",schema,error));
  auto effect=compile(effect_source(schema,text,"return position;"),schema,"effect.wgsl","vertex.wgsl",error);CHECK(effect->valid);
  aurora::gfx::DrawContext ctx;ctx.device=test_device;ctx.queue=test_device.GetQueue();ctx.layout=aurora::gfx::scene_render_target_layout();
  init_gpu(ctx);FxPrepared prepared;prepared.program=effect;
  CHECK(pipeline(ctx,KParticles,FX_BLEND_ALPHA,1,&prepared));
  CHECK(pipeline(ctx,KMesh,FX_BLEND_ADD,1,&prepared));
  CHECK(pipeline(ctx,KBloomParticles,FX_BLEND_ALPHA,0,&prepared));
  CHECK(pipeline(ctx,KBloomMesh,FX_BLEND_ALPHA,0,&prepared));
  int shader=gw_Shader_Load(7,root.c_str(),"shaders/vignette.wgsl","",0,R"({"strength":0.4,"radius":0.3,"softness":0.4})",nullptr,0);
  CHECK(shader>0);
  char msg[2048];int post=gw_Post_Add(7,shader,0,0,1,0,msg,sizeof msg);CHECK(post>0);
  CHECK(gw_Post_Ready(7,post)==0 && gw_Post_Ready(8,post)==-1 && gw_Post_Ready(0,post)==-1);
  // Recording contract fixture: no game or real rendered pixels. Dawn Null
  // validates the real pipelines; controllable Aurora boundary tests readiness.
  wgpu::TextureDescriptor color_desc{};color_desc.size={640,480,1};color_desc.format=wgpu::TextureFormat::RGBA8Unorm;
  color_desc.usage=wgpu::TextureUsage::TextureBinding|wgpu::TextureUsage::RenderAttachment;
  test_color=test_device.CreateTexture(&color_desc).CreateView();
  auto submit_post=[&] { uint64_t id=++next_work;post_work[id]={};post_record(&id,sizeof id); };
  test_recording=true;test_draw_accept=false;submit_post();CHECK(gw_Post_Ready(7,post)==0);
  test_draw_accept=true;submit_post();CHECK(gw_Post_Ready(7,post)==1);
  test_recording=false;
  CHECK(!gw_Post_Set(8,post,"{}",msg,sizeof msg));
  CHECK(gw_Post_Set(7,post,R"({"strength":0.6})",msg,sizeof msg));
  CHECK(!gw_Post_Set(7,post,R"({"strength":[1,1,1,1]})",msg,sizeof msg));
  CHECK(gw_Post_Remove(7,post) && !gw_Post_Remove(7,post));
  CHECK(gw_Post_Ready(7,post)==-1);
  post=gw_Post_Add(7,shader,0,1,0,0,msg,sizeof msg);CHECK(post>0);
  gw_Shader_Release(7);CHECK(!gw_Post_Set(7,post,"{}",msg,sizeof msg));CHECK(!gw_Shader_Status(7,shader,msg,sizeof msg));
  // Repeated path-owned passes must release their hidden shader handles at clear.
  for(int i=0;i<160;++i){
    int s=gw_Shader_Load(7,root.c_str(),"shaders/vignette.wgsl","",0,R"({"strength":0.4,"radius":0.3,"softness":0.4})",msg,sizeof msg);
    CHECK(s>0);int p=gw_Post_Add(7,s,0,0,0,1,msg,sizeof msg);CHECK(p>0);gw_Post_Clear(7);CHECK(!gw_Shader_Status(7,s,msg,sizeof msg));
  }
  // Stamp changes recompile; removal invalidates immediately after the next poll.
  Asset reloadable;reloadable.root=argv[2];reloadable.fragment="good.wgsl";reloadable.params_json="{}";
  CHECK(reload(reloadable,true,error));auto old=reloadable.program;
  {std::ofstream file(std::filesystem::path(argv[2])/"good.wgsl");file<<"return vec4f(0.5, 0.25, 0.0, 1.0);\n";}
  reloadable.poll={};CHECK(reload(reloadable,false,error)&&reloadable.program!=old);
  {std::ofstream file(std::filesystem::path(argv[2])/"good.wgsl");file<<"return missing_value;\n";}
  reloadable.poll={};CHECK(!reload(reloadable,false,error)&&error.find("good.wgsl:1:")!=std::string::npos);
  {std::ofstream file(std::filesystem::path(argv[2])/"good.wgsl");file<<"return vec4f(1.0);\n";}
  reloadable.poll={};CHECK(reload(reloadable,false,error));
  std::cout<<"shader_runtime (Dawn Null): PASS\n";
  return 0;
}
