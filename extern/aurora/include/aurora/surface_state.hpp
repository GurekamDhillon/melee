#pragma once
// GPU-independent surface state. Render-only; never read by simulation.
#include <array>
#include <cmath>
#include <cstdint>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

namespace gw_surface {
inline std::string splice(std::string generated, const std::string& body, const std::string& call) {
    if (body.empty()) return generated;
    const auto entry = generated.find("@fragment\nfn fs_main(");
    auto finish = generated.find("\n    return prev;", entry);
    if (finish == std::string::npos) finish = generated.find("\n    out.color = prev;", entry);
    if (entry == std::string::npos || finish == std::string::npos) return {};
    generated.insert(finish, "\n    " + call);
    generated.insert(entry, body + "\n");
    return generated;
}
struct Params { std::array<float, 20> data{}; };
static_assert(sizeof(Params) == 80);
struct Program { std::string source, label; };
class Registry {
    std::mutex mutex;
    std::vector<std::shared_ptr<const Program>> programs{nullptr};
public:
    uint32_t find(const std::string& source) {
        std::lock_guard lock(mutex);
        for (uint32_t i = 1; i < programs.size(); ++i)
            if (programs[i]->source == source) return i;
        return 0;
    }
    uint32_t add(std::string source, std::string label) {
        if (source.empty() || source.size() > 65536) return 0;
        std::lock_guard lock(mutex);
        for (uint32_t i = 1; i < programs.size(); ++i)
            if (programs[i]->source == source) return i;
        if (programs.size() >= 257) return 0;
        programs.push_back(std::make_shared<Program>(Program{std::move(source), std::move(label)}));
        return static_cast<uint32_t>(programs.size() - 1);
    }
    std::shared_ptr<const Program> get(uint32_t id) {
        std::lock_guard lock(mutex);
        return id < programs.size() ? programs[id] : nullptr;
    }
};
struct Binding { uint32_t program = 0, owner = 0; Params params{}; };
class Selection {
    // Index 0 = unrelated, 1..6 = fighter slots, 7 = original stage.
    std::array<Binding, 8> bindings{};
    std::array<Binding, 32> stack{};
    Binding current{};
    unsigned depth = 0, overflow = 0;
public:
    bool set(unsigned slot, uint32_t program, uint32_t owner, const Params& params) {
        if (!slot || slot >= bindings.size() || !owner) return false;
        for (float f : params.data) if (!std::isfinite(f)) return false;
        auto& b = bindings[slot];
        if (b.owner && b.owner != owner) return false;
        b = program ? Binding{program, owner, params} : Binding{};
        return true;
    }
    Binding enter(unsigned slot) {
        if (depth == stack.size()) { ++overflow; return current = {}; }
        stack[depth++] = current;
        return current = slot < bindings.size() ? bindings[slot] : Binding{};
    }
    Binding leave() {
        if (overflow) { --overflow; return current = {}; }
        return current = depth ? stack[--depth] : Binding{};
    }
    void release(uint32_t owner) {
        for (auto& b : bindings) if (!owner || b.owner == owner) b = {};
    }
};
} // namespace gw_surface
