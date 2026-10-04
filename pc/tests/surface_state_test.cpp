#include "../platform/gw_surface_state.hpp"
#include "../platform/gw_surface_path.h"
#include <cassert>
#include <cmath>
#include <iostream>
int main() {
    using namespace gw_surface;
    static_assert(sizeof(Params) == 80);
    assert(gw_surface_path_valid("shaders/a.wgsl", 14));
    assert(gw_surface_path_valid("shaders/nested/a.wgsl", 21));
    for (const char* p : {"shaders/../a.wgsl", "shaders//a.wgsl", "shaders/x /a.wgsl",
                         "shaders/x./a.wgsl", "shaders/x:a.wgsl", "C:/a.wgsl", "shaders/a.txt"})
        assert(!gw_surface_path_valid(p, strlen(p)));
    assert(!gw_surface_path_valid("shaders/a.wgsl\0evil", 19));
    Registry r;
    auto a = r.add("fn gd_surface() {}", "a");
    assert(a && a == r.add("fn gd_surface() {}", "b"));
    auto b = r.add("fn gd_surface() { }", "b");
    assert(b != a && r.get(a)->source != r.get(b)->source);
    assert(!r.add("", "empty"));
    assert(!r.get(999));
    for (unsigned i = 0; i < 254; ++i) assert(r.add("source " + std::to_string(i), "capacity"));
    assert(!r.add("one too many", "capacity"));
    assert(r.add("fn gd_surface() {}", "still cached") == a);
    assert(splice("@fragment\nfn fs_main() {\n    return prev;\n}", "", "CALL") ==
           "@fragment\nfn fs_main() {\n    return prev;\n}");
    auto sp = splice("@fragment\nfn fs_main() {\n    return prev;\n}", "BODY", "CALL");
    assert(sp.find("BODY\n@fragment") != std::string::npos);
    assert(sp.find("CALL\n    return prev;") != std::string::npos);
    assert(splice("unexpected template", "BODY", "CALL").empty());
    Selection s;
    Params p{}; p.data[4] = 0.3f;
    assert(s.set(1, a, 7, p));
    assert(!s.set(1, b, 8, p)); // another script cannot steal a selection
    assert(s.enter(1).program == a);
    assert(s.enter(0).program == 0); // nested unrelated draw must be vanilla
    assert(s.leave().program == a);
    assert(s.leave().program == 0);
    assert(s.leave().program == 0);
    s.release(8); assert(s.enter(1).program == a); s.leave();
    s.release(7); assert(s.enter(1).program == 0); s.leave();
    p.data[4] = INFINITY; assert(!s.set(1, a, 7, p));
    assert(!s.set(99, a, 7, Params{}));
    for (int i = 0; i < 40; ++i) s.enter(0);
    for (int i = 0; i < 40; ++i) assert(s.leave().program == 0);
    std::cout << "surface registry, ownership, nested scopes, finite params: PASS\n";
}
