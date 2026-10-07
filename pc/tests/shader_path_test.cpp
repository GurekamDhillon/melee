// Pure Windows path fixture: no GPU, mod assets, or game.
#include "../platform/gw_shader_paths.hpp"
#include <filesystem>
#include <cstdio>
#include <cstdlib>
#include <string>
using namespace gwshader;

int main() {
#ifdef _WIN32
  const char* roots[] = {R"(C:\mods\sample)", R"(\\?\C:\mods\sample)"};
  const char* paths[] = {R"(C:\mods\sample\models\drive.material.json)", R"(\\?\C:\mods\sample\models\drive.material.json)"};
  for (auto root:roots) for (auto path:paths)
    if (relative_path(root,path)!="models/drive.material.json") return 1;
  if (relative_path(R"(\\server\share\sample)", R"(\\?\UNC\server\share\sample\models\drive.material.json)")!="models/drive.material.json") return 2;
  if (!relative_path(roots[0], R"(\\?\C:\mods\sample2\drive.material.json)").empty()) return 3;
  // A mod root that is a directory junction (a mods folder of links): the loader reports files by their
  // final (resolved) path, so the root must be matched through the link too.
  {
    namespace fs = std::filesystem;
    std::error_code ec;
    fs::path base = fs::temp_directory_path(ec) / ("gw_shader_path_" + std::to_string(std::filesystem::file_time_type::clock::now().time_since_epoch().count()));
    fs::path real = base / "real", link = base / "link", other = base / "other";
    fs::create_directories(real / "models", ec); fs::create_directories(other, ec);
    std::string cmd = "mklink /J \"" + link.string() + "\" \"" + real.string() + "\" >nul";
    if (std::system(cmd.c_str()) != 0) return 4;
    std::string file = (fs::canonical(real, ec) / "models" / "drive.material.json").string();
    if (relative_path(link.string(), file) != "models/drive.material.json") return 5;
    if (relative_path(link.string(), R"(\\?\)" + file) != "models/drive.material.json") return 6;
    if (!relative_path(link.string(), (fs::canonical(other, ec) / "stolen.wgsl").string()).empty()) return 7;
    fs::remove(link, ec); fs::remove_all(base, ec);
  }
#endif
  puts("shader_path: PASS");
}
