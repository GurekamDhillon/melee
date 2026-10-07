#include "../platform/gw_motion_history.hpp"
#include <cassert>
#include <iostream>
int main() {
    using namespace gw_motion;
    History<int, 8> h;
    assert(h.push(1, 10));
    assert(!h.push(1, 99)); // pause/interpolated presents cannot duplicate logic samples
    assert(h.push(2, 20));
    assert(h.age(2, 1) && *h.age(2, 1) == 10);
    assert(!h.age(2, 0)); // exclude live pose
    assert(h.push(0, 30)); // rewind invalidates the old future
    assert(!h.age(2, 1));
    for (int i=1;i<=20;++i) h.push(i,i);
    assert(h.size()==8 && !h.age(20,8) && *h.age(20,7)==13);
    History<int,8> copy=h; h.clear();
    assert(h.size()==0 && *copy.age(20,1)==19);
    assert(fade(0,10,1)==1 && fade(10,10,1)==0);
    assert(fade(5,10,2)==0.25f);
    Point a{0,0,0}, b{1,1,0}, c{2,0,0}, d{3,1,0};
    assert(distance2(catmull(a,b,c,d,0),b)<1e-6f);
    assert(distance2(catmull(a,b,c,d,1),c)<1e-6f);
    auto arc=catmull(a,b,c,d,0.25f);
    assert(arc.y>0.75f); // a smooth curve, not linear interpolation
    Budget budget{100,3};
    assert(budget.take(40,1) && !budget.take(61,1));
    assert(budget.take(60,2) && !budget.take(1,1));
    // Colour variety helpers: hue rotation, gradient stops, pulse ripple, width profile.
    {
        float red[3]={1,0,0};hue_rotate(red,120);
        assert(std::abs(red[0])<1e-5f && std::abs(red[1]-1)<1e-5f && std::abs(red[2])<1e-5f); // red + 120 deg = green
        float grey[3]={0.4f,0.4f,0.4f};hue_rotate(grey,77);
        assert(std::abs(grey[0]-0.4f)<1e-5f && std::abs(grey[1]-0.4f)<1e-5f && std::abs(grey[2]-0.4f)<1e-5f); // grey is hue-free
        float round_trip[3]={0.8f,0.3f,0.1f};hue_rotate(round_trip,360);
        assert(std::abs(round_trip[0]-0.8f)<1e-5f && std::abs(round_trip[2]-0.1f)<1e-5f);
        const float stops[3][4]={{1,0,0,1},{0,1,0,0.5f},{0,0,1,0}};float out[4];
        gradient_sample(stops,3,0,out);assert(out[0]==1 && out[3]==1);
        gradient_sample(stops,3,0.5f,out);assert(out[1]==1 && out[3]==0.5f);
        gradient_sample(stops,3,1,out);assert(out[2]==1 && out[3]==0);
        gradient_sample(stops,3,0.25f,out);assert(std::abs(out[0]-0.5f)<1e-6f && std::abs(out[1]-0.5f)<1e-6f);
        gradient_sample(stops,3,9,out);assert(out[2]==1); // clamped
        assert(pulse_factor(4,0,1.3f,0.2f)==1); // zero depth: untouched
        for(int i=0;i<100;++i){float f=pulse_factor(3,0.6f,i*0.07f,i*0.01f);assert(f>=0.4f-1e-5f && f<=1.f+1e-5f);}
        assert(width_profile(0.5f,1,0)==0.5f);                 // plain taper unchanged
        assert(width_profile(0.5f,0,2)>width_profile(0.5f,0,0)); // swell bulges mid-trail
        assert(width_profile(0.f,0,3)==1 && std::abs(width_profile(1.f,0,3)-1)<1e-5f); // ends stay put
        assert(width_profile(0.5f,0,-1)<1e-5f);                // full pinch closes the middle
    }
    std::cout << "motion history: duplicate frames, rewind, ring, lifetime, arc, budget, hue/gradient/pulse/width PASS\n";
}
