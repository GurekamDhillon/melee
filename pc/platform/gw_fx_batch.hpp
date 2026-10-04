#ifndef GW_FX_BATCH_HPP
#define GW_FX_BATCH_HPP
#include <cstddef>
#include <vector>

/* Only caller-approved additive runs may reorder. Every other group is an
 * ordering barrier, including custom shaders that rely on local instance ids. */
template<class Group, class Particle, class Safe>
void gw_fx_batch_additive(std::vector<Group>& groups, std::vector<Particle>& particles, Safe safe) {
    std::vector<Group> output;
    std::vector<Particle> packed;
    output.reserve(groups.size()); packed.reserve(particles.size());
    for (size_t begin=0; begin<groups.size();) {
        size_t end=begin+1;
        if (safe(groups[begin])) while(end<groups.size() && safe(groups[end])) ++end;
        std::vector<size_t> keys;
        for(size_t i=begin;i<end;++i) {
            bool found=false;
            for(size_t k:keys) if(groups[k].pkg==groups[i].pkg && groups[k].em==groups[i].em &&
                                  groups[k].mesh==groups[i].mesh) {found=true;break;}
            if(!found) keys.push_back(i);
        }
        for(size_t key:keys) {
            Group merged=groups[key]; merged.first=int(packed.size()); merged.count=0;
            for(size_t i=begin;i<end;++i) if(groups[key].pkg==groups[i].pkg && groups[key].em==groups[i].em &&
                                            groups[key].mesh==groups[i].mesh) {
                const Group& g=groups[i];
                packed.insert(packed.end(),particles.begin()+g.first,particles.begin()+g.first+g.count);
                merged.count+=g.count;
            }
            output.push_back(merged);
        }
        begin=end;
    }
    groups.swap(output); particles.swap(packed);
}
#endif
