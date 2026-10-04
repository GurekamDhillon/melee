#pragma once
#include <array>
#include <cstdint>
#include <vector>
#include <algorithm>

namespace aurora::gfx {
// Host-only declaration handles. Never snapshotted or used by simulation.
class PipelineWarmRegistry {
  struct Entry { uint32_t token = 0; bool sealed = false; std::vector<uint64_t> refs; };
  std::array<Entry, 64> entries{};
  uint32_t serial = 0;
  uint32_t active = 0;
public:
  uint32_t begin() {
    if (active || serial == UINT32_MAX) return 0;
    for (auto& e : entries) if (!e.token) {
      e.token = ++serial; e.sealed = false; e.refs.clear(); active = e.token; return active;
    }
    return 0;
  }
  bool capturing() const { return active != 0; }
  void record(uint64_t ref) {
    for (auto& e : entries) if (e.token == active && active) {
      if (std::find(e.refs.begin(), e.refs.end(), ref) == e.refs.end()) e.refs.push_back(ref);
      return;
    }
  }
  void end() { for (auto& e : entries) if (e.token == active) e.sealed = true; active = 0; }
  template<class Ready> int pending(uint32_t token, Ready ready) const {
    for (const auto& e : entries) if (e.token == token && token) {
      if (!e.sealed) return -2;
      int n = 0; for (auto ref : e.refs) n += !ready(ref); return n;
    }
    return -1;
  }
  void release(uint32_t token) {
    for (auto& e : entries) if (e.token == token) { e = {}; if (active == token) active = 0; }
  }
  void clear() { for (auto& e : entries) e = {}; active = 0; }
};
}
