/* Included after gw_Post_Add. Native source, independent of mod files/root. */
#include "gw_stage_cover.h"
extern "C" int gw_Post_StageCover(int owner,int flash,char* out,int cap) {
  using namespace shader_runtime;
  int shader;
  {
    std::lock_guard<std::mutex> state(state_mutex);std::string error;
    auto a=std::make_shared<Asset>();a->owner=owner;
    a->fragment="builtin/stage-cover";a->builtin=gw_stage_cover_wgsl;
    a->params_json=flash?"{\"progress\":0,\"flash\":1}":"{\"progress\":0,\"flash\":0}";
    if(!reload(*a,true,error)){error_out(error,out,cap);return 0;}
    shader=int(assets.add(owner,a));
    if(!shader){error_out("shader handle capacity exhausted",out,cap);return 0;}
  }
  return gw_Post_Add(owner,shader,10000,1,0,1,out,cap);
}
