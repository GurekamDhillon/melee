#define main shader_base_main
#include "shader_runtime_test.cpp"
#undef main
int main(int argc,char** argv) {
  CHECK(argc==4);
  int rc=shader_base_main(3,argv);if(rc)return rc;
  using namespace shader_runtime;
  Schema s;std::string error,text;uint64_t stamp=0;
  CHECK(read_contained(argv[3],"shaders/impact.wgsl",text,stamp,error));
  CHECK(parse_schema(R"({"center":[0.5,0.5,0,0],"progress":0,"elapsed":0,"intensity":1,"flash_limit":0,"limiter_strength":0})",s,error));
  auto p=compile(splice(header,s,text,"",tail),s,"shaders/impact.wgsl","",error);
  if(!p->valid){std::cerr<<error<<"\n";return 1;}
  CHECK(get_pipeline(p,aurora::gfx::scene_render_target_layout(),0,0));
  auto invalid=compile(splice(header,s,"// @module\nfn helper()->f32 { return missing_helper; }\nfn mod_fragment(in:Input)->vec4f { return vec4f(helper()); }","",tail),s,"module.wgsl","",error);
  CHECK(!invalid->valid && error.find("module.wgsl:2:")!=std::string::npos);
  auto attr=compile(splice(header,s,"// @module\n@fragment fn mod_fragment(in:Input)->vec4f { return vec4f(1); }","",tail),s,"attribute.wgsl","",error);
  CHECK(!attr->valid);
  auto bad_vertex=compile(splice(header,s,text,"\nreturn missing_vertex;",tail),s,"module.wgsl","vertex.wgsl",error);
  CHECK(!bad_vertex->valid && error.find("vertex.wgsl:2:")!=std::string::npos);
  puts("owner v3 WGSL compile/pipeline: PASS");return 0;
}
