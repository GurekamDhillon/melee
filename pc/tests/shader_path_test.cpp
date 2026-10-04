// Pure Windows path fixture: no GPU, mod assets, or game.
#include "../platform/gw_shader_paths.hpp"
#include <filesystem>
#include <cstdio>
using namespace gwshader;

int main() {
#ifdef _WIN32
  const char* roots[] = {R"(C:\mods\sample)", R"(\\?\C:\mods\sample)"};
  const char* paths[] = {R"(C:\mods\sample\models\drive.material.json)", R"(\\?\C:\mods\sample\models\drive.material.json)"};
  for (auto root:roots) for (auto path:paths)
    if (relative_path(root,path)!="models/drive.material.json") return 1;
  if (relative_path(R"(\\server\share\sample)", R"(\\?\UNC\server\share\sample\models\drive.material.json)")!="models/drive.material.json") return 2;
  if (!relative_path(roots[0], R"(\\?\C:\mods\sample2\drive.material.json)").empty()) return 3;
#endif
  puts("shader_path: PASS");
}
