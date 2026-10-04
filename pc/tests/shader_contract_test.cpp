/* Standalone actual-contract test; also included by the in-engine registry. */
#include "../platform/gw_shader_contract.hpp"
#include <cstdio>
#define CHECK(x) do { if (!(x)) { std::fprintf(stderr, "shader contract line %d: %s\n", __LINE__, #x); return 1; } } while (0)
int shader_contract_test() {
  using namespace gwshader;
  CHECK(valid_path("shaders/grade.wgsl"));
  CHECK(!valid_path("../grade.wgsl") && !valid_path("shaders/../grade.wgsl"));
  CHECK(!valid_path("shaders/x:stream") && !valid_path("shaders/x. /y"));
  CHECK(!valid_path(std::string("shaders/a\0b", 11)));
  CHECK(inside("C:\\mods\\sample", "c:\\mods\\sample\\shaders\\x"));
  CHECK(!inside("C:\\mods\\sample", "C:\\mods\\sample2\\x"));
  CHECK(normalize_windows_path(R"(\\?\C:\mods\sample\models\drive.material.json)") ==
        normalize_windows_path(R"(C:\mods\sample\models\drive.material.json)"));
  CHECK(normalize_windows_path(R"(\\?\UNC\server\share\mods\sample)") ==
        normalize_windows_path(R"(\\server\share\mods\sample)"));
  CHECK(inside(R"(C:\mods\sample)", R"(\\?\C:\mods\sample\models\drive.material.json)"));
  CHECK(inside(R"(\\?\C:\mods\sample)", R"(C:\mods\sample\models\drive.material.json)"));
  CHECK(!inside(R"(C:\mods\sample)", R"(\\?\C:\mods\sample2\models\drive.material.json)"));
  // Explicit coverage for the identifier helper restored during path normalization.
  CHECK(identifier("strength") && identifier("_tint2"));
  CHECK(identifier(std::string(32, 'a')) && !identifier(std::string(33, 'a')));
  CHECK(!identifier("") && !identifier("2tint") && !identifier("bad-name"));
  CHECK(!identifier(std::string("a\0b", 3)) && !identifier("a b"));
  Schema schema; std::string error;
  CHECK(parse_schema(R"({"strength":1,"tint":[1,0.5,0.25,1]})", schema, error));
  CHECK(schema.fields.size() == 2 && schema.fields[1].offset == 16 && schema.bytes() == 32);
  CHECK(schema.declarations().find("strength: vec4f") != std::string::npos);
  CHECK(!parse_schema(R"({"strength":1,"strength":2})", schema, error));
  CHECK(!parse_schema(R"({"bad-name":1})", schema, error));
  CHECK(!parse_schema(R"({"x":[1,2,3]})", schema, error));
  CHECK(!parse_schema(R"({"x":1e100})", schema, error));
  CHECK(parse_schema(R"({"x":1,"color":[1,1,1,1]})", schema, error));
  auto values = schema.defaults(); auto original = values;
  CHECK(!set_param(schema, values, "missing", {1}, error) && values == original);
  CHECK(!set_param(schema, values, "color", {1}, error) && values == original);
  CHECK(set_param(schema, values, "x", {2}, error) && values[4] == 2);
  auto source = splice("HEADER\n", schema, "return vec4f(1.0);\n", "return position;\n", "TAIL\n");
  CHECK(source.fragment_line > source.vertex_line);
  CHECK(source.text.find("fn mod_fragment") != std::string::npos);
  CHECK(source.line(source.fragment_line, false) == 1);
  CHECK(source.line(source.vertex_line, true) == 1);
  CHECK(content_hash(source.text) != content_hash(source.text + " "));
  auto module = splice("HEADER\n", schema,
    "// @module\nfn twice(x:f32)->f32 { return x*2; }\nfn mod_fragment(in:Input)->vec4f { return vec4f(twice(1)); }\n", "", "TAIL\n");
  CHECK(module.text.find("fn mod_fragment(in: Input) -> vec4f {\n// @module") == std::string::npos);
  CHECK(module.line(module.fragment_line + 1, false) == 2);
  Material material;
  CHECK(parse_material("{}", material, error) && !material.enabled);
  CHECK(parse_material(R"({"builtin":"glass","opacity":0.32,"tint":[0.7,0.9,1,1]})", material, error));
  CHECK(material.enabled && material.glass && material.opacity == 0.32f);
  CHECK(!parse_material(R"({"builtin":"glass","opacity":2})", material, error));
  CHECK(!parse_material(R"({"builtin":"unknown"})", material, error));
  CHECK(!parse_material(R"({"fragment":"../bad.wgsl"})", material, error));
  Handles<int, 2> handles;
  auto a = handles.add(7, 42); auto b = handles.add(8, 43);
  CHECK(a && b && !handles.add(9, 44));
  CHECK(!handles.get(a, 8) && handles.get(a, 7) && *handles.get(a, 7) == 42);
  CHECK(handles.remove(a, 7) && !handles.get(a, 7));
  auto c = handles.add(7, 45); CHECK(c > b && !handles.get(a, 7));
  handles.clear(7); CHECK(!handles.get(c, 7) && handles.get(b, 8));
  handles.clear(0); CHECK(!handles.get(b, 8));
  return 0;
}
#ifndef GW_SHADER_ENGINE_TEST
int main() { int rc = shader_contract_test(); if (!rc) puts("shader_contract: PASS"); return rc; }
#endif
