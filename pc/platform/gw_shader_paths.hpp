// Shared GPU-independent shader path conversion.
#pragma once
#include "gw_shader_contract.hpp"
#include <filesystem>
namespace gwshader {
inline std::string relative_path_lexical(const std::string& root,const std::string& path) {
  std::error_code ec;auto abs_root=std::filesystem::absolute(root,ec).lexically_normal();if(ec)return {};
  auto abs_path=std::filesystem::absolute(path,ec).lexically_normal();if(ec)return {};
#ifdef _WIN32
  abs_root=std::filesystem::path(normalize_windows_path(abs_root.string()));
  abs_path=std::filesystem::path(normalize_windows_path(abs_path.string()));
#endif
  auto rel=abs_path.lexically_relative(abs_root).generic_string();return valid_path(rel)?rel:std::string();
}
// The loaders report a file by its FINAL path (junctions and symlinks resolved), while a mod root may be the
// link itself (a mods folder of junctions). When the plain comparison finds no relation, compare both through
// their resolved forms. Containment is still enforced by read_contained on the opened handle, not by this.
inline std::string relative_path(const std::string& root,const std::string& path) {
  auto rel=relative_path_lexical(root,path);if(!rel.empty())return rel;
  std::error_code ec;auto real_root=std::filesystem::weakly_canonical(std::filesystem::path(root),ec);if(ec)return {};
  auto real_path=std::filesystem::weakly_canonical(std::filesystem::path(path),ec);if(ec)return {};
  return relative_path_lexical(real_root.string(),real_path.string());
}
}
