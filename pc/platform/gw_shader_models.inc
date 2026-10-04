// Opt-in kit materials. Called at precisely the existing opaque/far/near draw sites.
namespace shader_runtime {
struct Image { uint32_t w=0,h=0;std::vector<uint8_t> rgba;wgpu::TextureView view; };
struct Model {
  Material material;std::shared_ptr<Asset> shader;
  Image albedo,normal,emissive;
};
Handles<std::shared_ptr<Model>,128> models;
std::string infer_root(const std::string& path) {
  auto p=std::filesystem::path(path).parent_path();
  while(!p.empty()){
    std::error_code ec;if(std::filesystem::is_regular_file(p/"mod.json",ec))return p.string();
    auto parent=p.parent_path();if(parent==p)break;p=parent;
  }
  return {};
}

bool gxtex(const std::string& root,const std::string& path,Image& out,std::string& error) {
  std::string bytes;uint64_t stamp;
  if(!read_contained(root,path,bytes,stamp,error,64u*1024*1024,true))return false;
  auto b=reinterpret_cast<const unsigned char*>(bytes.data());
  if(bytes.size()<64||gw_r32(b)!=0x47585458||gw_r32(b+4)!=1||gw_r32(b+8)!=6){error="material atlas must be GXTX v1 RGBA8: "+path;return false;}
  uint32_t w=gw_r32(b+12),h=gw_r32(b+16),size=gw_r32(b+28),offset=gw_r32(b+36);
  if(w<4||h<4||w>4096||h>4096||(w&3)||(h&3)||uint64_t(offset)+size>bytes.size()||uint64_t(w)*h*4>size){error="invalid material atlas dimensions/bytes: "+path;return false;}
  out.w=w;out.h=h;out.rgba.resize(size_t(w)*h*4);
  for(uint32_t y=0;y<h;++y)for(uint32_t x=0;x<w;++x){
    size_t block=(size_t(y/4)*(w/4)+x/4)*64,at=((y&3)*4+(x&3))*2,dst=(size_t(y)*w+x)*4;
    out.rgba[dst]=b[offset+block+at+1];out.rgba[dst+1]=b[offset+block+32+at];
    out.rgba[dst+2]=b[offset+block+32+at+1];out.rgba[dst+3]=b[offset+block+at];
  }
  return true;
}
wgpu::TextureView image_view(Image& image,wgpu::TextureView fallback) {
  if(!image.w)return fallback;
  if(!image.view){auto dev=aurora::gfx::device();wgpu::TextureDescriptor td{};td.size={image.w,image.h,1};td.format=wgpu::TextureFormat::RGBA8Unorm;
    td.usage=wgpu::TextureUsage::TextureBinding|wgpu::TextureUsage::CopyDst;auto tex=dev.CreateTexture(&td);
    wgpu::TexelCopyTextureInfo dst{};dst.texture=tex;wgpu::TexelCopyBufferLayout lay{};lay.bytesPerRow=image.w*4;lay.rowsPerImage=image.h;
    wgpu::Extent3D extent{image.w,image.h,1};aurora::gfx::queue().WriteTexture(&dst,image.rgba.data(),image.rgba.size(),&lay,&extent);image.view=tex.CreateView();}
  return image.view;
}
struct ModelWork { std::shared_ptr<Model> model;Uniform uniform{};std::vector<float> vertices;bool alpha=false; };
std::map<uint64_t,ModelWork> model_work;
void model_record(const void* bytes,u32 size) {
  if(size!=sizeof(uint64_t))return;uint64_t id;std::memcpy(&id,bytes,sizeof id);ModelWork work;
  {std::lock_guard<std::mutex> lock(work_mutex);auto it=model_work.find(id);if(it==model_work.end())return;work=std::move(it->second);model_work.erase(it);}
  std::lock_guard<std::mutex> state(state_mutex);auto start=Clock::now();auto m=work.model;
  std::string error;
  if(!m->material.fragment.empty()&&!reload(*m->shader,false,error)){++perf.skipped;return;}
  auto pipeline=get_pipeline(m->shader->program,aurora::gfx::scene_render_target_layout(),1,work.alpha);
  if(!pipeline){++perf.skipped;return;}
  auto target=aurora::gfx::scene_render_target_layout();uint32_t w=target.colorAttachments[0].width,h=target.colorAttachments[0].height;
  auto& u=work.uniform;auto saved_tint=std::array<float,4>{u.tint[0],u.tint[1],u.tint[2],u.tint[3]};fill(u,w,h);
  std::copy(saved_tint.begin(),saved_tint.end(),u.tint);
  u.clock[2]=m->material.glass;u.clock[3]=m->material.unlit;
  u.material[0]=m->material.opacity;u.material[1]=m->material.roughness;u.material[2]=m->material.emission;u.material[3]=m->normal.w?1.f:0.f;
  Packet p;p.pipeline=pipeline;p.count=uint32_t(work.vertices.size()/12);p.full=false;
  p.textures[0]=image_view(m->albedo,white);p.textures[1]=image_view(m->normal,flat_normal);p.textures[2]=image_view(m->emissive,black);
  p.textures[3]=black;p.textures[4]=depth0;p.textures[5]=black;
  // Custom model shaders may sample a copy; only their opt-in draws pay this cost.
  if(!m->material.fragment.empty()){
    aurora::gfx::ResolveDesc desc{};desc.depth=true;aurora::gfx::ResolvedTargets scene;
    if(aurora::gfx::resolve_pass(desc,scene)){++perf.resolves;p.textures[3]=scene.color?scene.color:black;p.textures[4]=scene.depth?scene.depth:depth0;p.textures[5]=p.textures[3];}
  }
  p.vertices=aurora::gfx::push_storage(reinterpret_cast<const uint8_t*>(work.vertices.data()),work.vertices.size()*4);
  if(!p.vertices.size){++perf.skipped;return;}
  perf.vertex_bytes+=work.vertices.size()*4;record(p,u,m->shader->values);
  perf.record_ms+=std::chrono::duration<double,std::milli>(Clock::now()-start).count();
}
} // namespace shader_runtime

extern "C" int gw_Shader_ModelLoad(const char* root_arg,const char* mesh_path,const char* atlas_path,const char* glow_path,char* out,int cap) {
  using namespace shader_runtime;
  std::string side=mesh_path;auto suffix=side.rfind(".gxmesh");if(suffix==side.npos)return 0;side.replace(suffix,7,".material.json");
#ifdef _WIN32
  DWORD attr=GetFileAttributesA(side.c_str());if(attr==INVALID_FILE_ATTRIBUTES){DWORD why=GetLastError();if(why==ERROR_FILE_NOT_FOUND||why==ERROR_PATH_NOT_FOUND)return 0;}
#else
  if(!std::filesystem::exists(side))return 0;
#endif
  std::string root=root_arg?root_arg:infer_root(mesh_path),error,text;uint64_t stamp=0;
  auto fail=[&](){error_out(error,out,cap);gw_log("shader material: %s: %s",side.c_str(),error.c_str());return -1;};
  if(root.empty()){error="material has no containing mod manifest";return fail();}
  if(!read_contained(root,relative_path(root,side),text,stamp,error))return fail();
  auto m=std::make_shared<Model>();if(!parse_material(text,m->material,error))return fail();if(!m->material.enabled)return 0;
  auto dir=std::filesystem::path(side).parent_path();
  auto material_rel=[&](const std::string& path){return relative_path(root,(dir/path).string());};
  auto albedo=m->material.albedo.empty()?relative_path(root,atlas_path):material_rel(m->material.albedo);
  if(!gxtex(root,albedo,m->albedo,error))return fail();
  if(!m->material.normal.empty()&&!gxtex(root,material_rel(m->material.normal),m->normal,error))return fail();
  if(!m->material.emissive.empty()){if(!gxtex(root,material_rel(m->material.emissive),m->emissive,error))return fail();}
  else if(glow_path&&glow_path[0]){if(!gxtex(root,relative_path(root,glow_path),m->emissive,error))return fail();}
  m->shader=std::make_shared<Asset>();m->shader->root=root;m->shader->params_json="{}";m->shader->schema=m->material.schema;
  m->shader->values=m->material.schema.defaults();
  std::lock_guard<std::mutex> state(state_mutex);
  if(m->material.fragment.empty()){
    auto source=splice(header,m->material.schema,"return builtin_material(in);","",tail);
    m->shader->program=compile(source,m->material.schema,side,"",error);
    if(!m->shader->program->valid)return fail();
  }else{
    m->shader->fragment=material_rel(m->material.fragment);m->shader->vertex=m->material.vertex.empty()?"":material_rel(m->material.vertex);
    // Preserve the sidecar's declaration defaults through hot reload.
    Json schema;schema.kind=Json::Object;
    std::string json="{";for(const auto& f:m->material.schema.fields){if(json.size()>1)json+=",";json+='"'+f.name+"\":";
      if(f.count==1)json+=std::to_string(f.value[0]);else{json+="[";for(int i=0;i<4;++i){if(i)json+=",";json+=std::to_string(f.value[i]);}json+="]";}}
    m->shader->params_json=json+"}";
    if(!reload(*m->shader,true,error))return fail();
  }
  int h=int(models.add(1,m));if(!h){error="material capacity exhausted (128)";return fail();}return h;
}
extern "C" void gw_Shader_ModelsReset(void){using namespace shader_runtime;std::lock_guard<std::mutex> state(state_mutex);models.clear(0);}
extern "C" int gw_Shader_ModelDraw(int material,const void* view_be,const float* local,unsigned tint,int alpha,
                                    const unsigned char* mesh,const float* batch,int batch_count) {
  using namespace shader_runtime;if(!material)return 0;if(material<0)return 1;
  ModelWork work;
  {std::lock_guard<std::mutex> state(state_mutex);auto m=models.get(uint32_t(material),1);if(!m)return 1;work.model=*m;}
  if(!mesh||!view_be||!local)return 1;
  for(int i=0;i<12;++i)reinterpret_cast<float*>(work.uniform.view)[i]=gw_rf32(static_cast<const unsigned char*>(view_be)+i*4);
  projection(work.uniform.proj);work.alpha=alpha!=0;
  for(int i=0;i<4;++i)work.uniform.tint[i]=float((tint>>(24-i*8))&255)/255*work.model->material.tint[i];
  uint32_t count=batch_count?uint32_t(batch_count):gw_r32(mesh+12),offset=gw_r32(mesh+28),indices=gw_r32(mesh+32);
  bool normals=gw_r32(mesh+4)==2;uint32_t stride=normals?32:20;
  work.vertices.resize(size_t(count)*12);
  for(uint32_t k=0;k<count;++k){float p[3],uv[2],normal[3]={0,0,1};float* o=&work.vertices[size_t(k)*12];
    if(batch_count){auto v=batch+size_t(k)*8;std::copy(v,v+3,p);uv[0]=v[3];uv[1]=v[4];if(normals)std::copy(v+5,v+8,normal);}
    else{const auto* v=mesh+offset+gw_r16(mesh+indices+k*2)*stride;for(int i=0;i<3;++i)p[i]=gw_rf32(v+i*4);
      uv[0]=gw_rf32(v+12);uv[1]=gw_rf32(v+16);if(normals)for(int i=0;i<3;++i)normal[i]=gw_rf32(v+20+i*4);
      float world[3],n[3]={};
      for(int r=0;r<3;++r)world[r]=local[r*4]*p[0]+local[r*4+1]*p[1]+local[r*4+2]*p[2]+local[r*4+3];
      // Inverse transpose for this API's orthogonal scaled/rotated local columns.
      for(int c=0;c<3;++c){float l=local[c]*local[c]+local[4+c]*local[4+c]+local[8+c]*local[8+c];
        if(l>1e-12f)for(int r=0;r<3;++r)n[r]+=local[r*4+c]*normal[c]/l;}
      std::copy(world,world+3,p);std::copy(n,n+3,normal);
    }
    std::copy(p,p+3,o);o[3]=uv[0];o[4]=uv[1];std::copy(normal,normal+3,o+5);std::fill(o+8,o+12,1);
  }
  uint64_t id;{std::lock_guard<std::mutex> lock(work_mutex);id=++next_work;model_work.emplace(id,std::move(work));}
  GXAuroraCallback(model_record,&id,sizeof id);return 1;
}
