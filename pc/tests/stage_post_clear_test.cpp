#include "gw_shader_contract.hpp"
#include <cassert>
#include <mutex>
#include <cstdio>
namespace shader_runtime {
struct Post { int shader=0; bool owns_shader=false,engine=false; };
gwshader::Handles<Post,32> posts;
gwshader::Handles<int,128> assets;
std::mutex state_mutex;
void refresh_stages() {}
}
#include "stage_post_clear_retail.inc"
int main() {
    using namespace shader_runtime;
    int sa=assets.add(1,1),sb=assets.add(2,2),sc=assets.add(1,3);
    int a=posts.add(1,{sa,true,false}),b=posts.add(2,{sb,true,false});
    int cover=posts.add(1,{sc,true,false});gw_Post_Protect(1,cover);
    gw_Post_Clear(0);assert(posts.get(a,1)&&posts.get(b,2)&&posts.get(cover,1));
    gw_Post_Clear(1);assert(!posts.get(a,1)&&!assets.get(sa,1));
    assert(posts.get(b,2)&&assets.get(sb,2)&&posts.get(cover,1)&&assets.get(sc,1));
    gw_Post_Clear(2);assert(!posts.get(b,2)&&posts.get(cover,1));
    puts("post_clear: caller only, no owner-zero wildcard, engine cover and shader survive passed");
}
