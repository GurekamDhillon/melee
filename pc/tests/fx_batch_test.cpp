// Standalone fixture for the production helper; no renderer/GPU dependencies.
#include "../platform/gw_fx_batch.hpp"
#include <cassert>
struct Group { int pkg,em,first,count,mesh; bool safe; };
int main() {
    std::vector<Group> g={{1,0,0,1,-1,true},{2,0,1,1,-1,true},{1,0,2,1,-1,true},
                          {1,0,3,1,-1,false},{1,0,4,1,-1,true},{1,0,5,1,2,true}};
    std::vector<int> p={10,20,11,99,12,13};
    auto safe=[](const Group& x){return x.safe;};
    gw_fx_batch_additive(g,p,safe);
    assert(g.size()==5 && g[0].count==2 && g[1].pkg==2 && !g[2].safe);
    assert((p==std::vector<int>{10,11,20,99,12,13}));
    assert(g[3].count==1 && g[4].mesh==2); // barrier and mesh identity stay distinct
    g.clear();p.clear();
    gw_fx_batch_additive(g,p,safe);assert(g.empty() && p.empty());
    // Thirty drives, four steady additive kinds, one colour: four draw groups.
    for(int drive=0;drive<30;++drive)for(int kind=0;kind<4;++kind) {
        g.push_back({kind,0,int(p.size()),1,-1,true});p.push_back(drive*4+kind);
    }
    gw_fx_batch_additive(g,p,safe);
    assert(g.size()==4 && p.size()==120);
    for(int kind=0;kind<4;++kind) {
        assert(g[kind].count==30);
        for(int drive=0;drive<30;++drive) assert(p[g[kind].first+drive]==drive*4+kind);
    }
}
