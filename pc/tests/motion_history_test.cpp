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
    std::cout << "motion history: duplicate frames, rewind, ring, lifetime, arc, budget PASS\n";
}
