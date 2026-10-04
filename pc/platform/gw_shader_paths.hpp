// Shared GPU-independent shader path conversion.
#pragma once
#include "gw_shader_contract.hpp"
#include <filesystem>
namespace gwshader {
inline std::string relative_path(const std::string& root,const std::string& path) {
  std::error_code ec;auto abs_root=std::filesystem::absolute(root,ec).lexically_normal();if(ec)return {};
  auto abs_path=std::filesystem::absolute(path,ec).lexically_normal();if(ec)return {};
#ifdef _WIN32
  abs_root=std::filesystem::path(normalize_windows_path(abs_root.string()));
  abs_path=std::filesystem::path(normalize_windows_path(abs_path.string()));
#endif
  auto rel=abs_path.lexically_relative(abs_root).generic_string();return valid_path(rel)?rel:std::string();
}
}
