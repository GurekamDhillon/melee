// GPU-independent shader data contract. Author guide: workspace docs/shaders.md.
#pragma once
#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <map>
#include <string>
#include <vector>

namespace gwshader {
inline uint64_t content_hash(const std::string& s) {
  uint64_t h = 14695981039346656037ull;
  for (unsigned char c : s) { h ^= c; h *= 1099511628211ull; }
  return h;
}
inline bool valid_path(const std::string& s) {
  if (s.empty() || s.size() >= 240 || s.find("..") != s.npos) return false;
  size_t start = 0;
  for (size_t i = 0; i <= s.size(); ++i) {
    if (i == s.size() || s[i] == '/') {
      if (i == start || s[i-1] == '.' || s[i-1] == ' ') return false;
      start = i + 1;
    } else if (static_cast<unsigned char>(s[i]) < 32 || s[i] == 127 ||
               std::string("\\:*?\"<>|").find(s[i]) != std::string::npos) return false;
  }
  return true;
}
// Handle paths from GetFinalPathNameByHandle and caller paths identically.
// Extended UNC must retain its leading pair of separators after prefix removal.
inline std::string normalize_windows_path(std::string s) {
  for (char& c : s) {
    if (c == '/') c = '\\';
    if (c >= 'A' && c <= 'Z') c += 'a'-'A';
  }
  if (s.compare(0, 8, "\\\\?\\unc\\") == 0) s = "\\\\" + s.substr(8);
  else if (s.size() >= 7 && s.compare(0, 4, "\\\\?\\") == 0 &&
           s[5] == ':' && s[6] == '\\') s.erase(0, 4);
  return s;
}
inline bool inside(std::string root, std::string path) {
  root = normalize_windows_path(root); path = normalize_windows_path(path);
  while (!root.empty() && root.back() == '\\') root.pop_back();
  return path.size() > root.size() && path.compare(0, root.size(), root) == 0 && path[root.size()] == '\\';
}

inline bool identifier(const std::string& s) {
  if (s.empty() || s.size()>32) return false;
  auto alpha=[](char c) { return (c>='a'&&c<='z')||(c>='A'&&c<='Z')||c=='_'; };
  if (!alpha(s[0])) return false;
  for (char c:s) if (!alpha(c)&&!(c>='0'&&c<='9')) return false;
  return true;
}

// Small strict JSON reader: duplicate keys, non-finite numbers and unknown material fields fail.
struct Json {
  enum Kind { Object, Array, String, Number, Boolean, Null } kind = Null;
  std::map<std::string, Json> object;
  std::vector<Json> array;
  std::string string;
  double number = 0;
};
class Reader {
  const std::string& s; size_t at = 0;
  void ws() { while (at < s.size() && (s[at]==' ' || s[at]=='\n' || s[at]=='\r' || s[at]=='\t')) ++at; }
  bool take(char c) { ws(); if (at == s.size() || s[at]!=c) return false; ++at; return true; }
  bool str(std::string& out) {
    if (!take('"')) return false;
    while (at < s.size()) {
      unsigned char c = s[at++];
      if (c == '"') return true;
      if (c < 32) return false;
      if (c == '\\') {
        if (at == s.size()) return false;
        c = s[at++];
        if (c == 'n') c = '\n'; else if (c=='r') c='\r'; else if (c=='t') c='\t';
        else if (c!='"' && c!='\\' && c!='/') return false;
      }
      out.push_back(char(c));
    }
    return false;
  }
  bool value(Json& j, unsigned depth) {
    if (depth > 16) return false;
    ws(); if (at == s.size()) return false;
    if (s[at]=='{') {
      ++at; j.kind=Json::Object; if (take('}')) return true;
      do { std::string key; Json v; if (!str(key) || !take(':') || !value(v, depth+1) || !j.object.emplace(key, v).second) return false; } while (take(','));
      return take('}');
    }
    if (s[at]=='[') {
      ++at; j.kind=Json::Array; if (take(']')) return true;
      do { Json v; if (!value(v, depth+1) || j.array.size() >= 256) return false; j.array.push_back(v); } while (take(','));
      return take(']');
    }
    if (s[at]=='"') { j.kind=Json::String; return str(j.string); }
    for (auto literal : {"true", "false", "null"}) {
      size_t n = std::char_traits<char>::length(literal);
      if (s.compare(at,n,literal)==0) { at+=n; j.kind=literal[0]=='n'?Json::Null:Json::Boolean; j.number=literal[0]=='t'; return true; }
    }
    size_t begin = at;
    if (s[at]=='-') ++at;
    if (at == s.size()) return false;
    if (s[at]=='0') ++at;
    else { if (s[at]<'1'||s[at]>'9') return false; while (at<s.size()&&s[at]>='0'&&s[at]<='9') ++at; }
    if (at<s.size()&&s[at]=='.') {
      ++at; size_t digit=at; while (at<s.size()&&s[at]>='0'&&s[at]<='9') ++at; if (at==digit) return false;
    }
    if (at<s.size()&&(s[at]=='e'||s[at]=='E')) {
      ++at; if (at<s.size()&&(s[at]=='+'||s[at]=='-')) ++at;
      size_t digit=at; while (at<s.size()&&s[at]>='0'&&s[at]<='9') ++at; if (at==digit) return false;
    }
    j.kind=Json::Number; j.number=std::strtod(s.substr(begin,at-begin).c_str(),nullptr);
    return std::isfinite(j.number) && std::abs(j.number)<=100000;
  }
public:
  explicit Reader(const std::string& text): s(text) {}
  bool parse(Json& j) { if (s.size()>65536 || !value(j,0)) return false; ws(); return at==s.size(); }
};
struct Field { std::string name; unsigned offset, count; std::array<float,4> value{}; };
struct Schema {
  std::vector<Field> fields;
  unsigned bytes() const { return std::max(16u, unsigned(fields.size()*16)); }
  std::vector<float> defaults() const {
    std::vector<float> v(bytes()/4,0);
    for (const auto& f:fields) std::copy(f.value.begin(), f.value.end(),v.begin()+f.offset/4);
    return v;
  }
  std::string declarations() const {
    std::string s="struct Params {\n";
    if (fields.empty()) s+="  reserved: vec4f,\n";
    for (const auto& f:fields) s+="  "+f.name+": vec4f,\n";
    return s+="};\n@group(1) @binding(0) var<uniform> params: Params;\n";
  }
};
inline bool floats(const Json& j, std::vector<float>& v) {
  if (j.kind == Json::Number) { v.push_back(float(j.number)); return true; }
  if (j.kind != Json::Array || j.array.size()!=4) return false;
  for (const auto& a:j.array) { if (a.kind!=Json::Number) return false; v.push_back(float(a.number)); }
  return true;
}
inline bool schema_json(const Json& j, Schema& s, std::string& error) {
  Schema next;
  if (j.kind != Json::Object || j.object.size()>16) { error="params must be an object of at most 16 floats/vec4s"; return false; }
  for (const auto& pair:j.object) {
    std::vector<float> v;
    if (!identifier(pair.first) || !floats(pair.second,v)) { error="invalid parameter name or value: "+pair.first; return false; }
    Field f; f.name=pair.first; f.offset=unsigned(next.fields.size()*16); f.count=unsigned(v.size());
    std::copy(v.begin(),v.end(),f.value.begin()); next.fields.push_back(f);
  }
  s=next; return true;
}
inline bool parse_schema(const std::string& text, Schema& s, std::string& error) {
  Json j; if (!Reader(text).parse(j)) { error="invalid params JSON"; return false; } return schema_json(j,s,error);
}
inline bool set_param(const Schema& s, std::vector<float>& values, const std::string& key,
                      const std::vector<float>& v, std::string& error) {
  for (const auto& f:s.fields) if (f.name==key) {
    if (v.size()!=f.count || values.size()!=s.bytes()/4) break;
    for (float x:v) if (!std::isfinite(x) || std::abs(x)>100000) { error="parameter must be finite within +/-100000"; return false; }
    std::copy(v.begin(),v.end(),values.begin()+f.offset/4); return true;
  }
  error="unknown parameter or wrong float/vec4 type: "+key; return false;
}
struct Source {
  std::string text; unsigned fragment_line=0, vertex_line=0, vertex_end_line=0;
  unsigned line(unsigned generated, bool vertex) const {
    unsigned start=vertex?vertex_line:fragment_line;
    return generated>=start?generated-start+1:0;
  }
};
inline Source splice(const std::string& header, const Schema& schema, const std::string& fragment,
                     const std::string& vertex, const std::string& tail) {
  Source s; s.text=header+schema.declarations();
  s.text+="fn mod_vertex(position: vec3f, normal: vec3f, uv: vec2f) -> vec3f {\n";
  s.vertex_line=unsigned(std::count(s.text.begin(),s.text.end(),'\n')+1);
  s.text+=(vertex.empty()?"return position;":vertex)+"\n}\n";
  s.vertex_end_line=unsigned(std::count(s.text.begin(),s.text.end(),'\n')+1);
  // Optional module form contains helpers and mod_fragment, never new bindings
  // or entry points. Body form stays byte-for-byte compatible.
  bool module=fragment.rfind("// @module\n",0)==0 || fragment.rfind("// @module\r\n",0)==0;
  if (!module) s.text+="fn mod_fragment(in: Input) -> vec4f {\n";
  s.fragment_line=unsigned(std::count(s.text.begin(),s.text.end(),'\n')+1);
  if (module) {
    // Attributes are unnecessary for functions in this contract; rejecting them
    // prevents module bodies from defining resource bindings or entry points.
    auto code=fragment.substr(fragment.find('\n')+1);
    if (code.find('@')!=std::string::npos) s.text+="INVALID_MODULE_ATTRIBUTES\n";
    else s.text+=fragment+"\n";
  } else s.text+=fragment+"\n}\n";
  s.text+=tail; return s;
}
struct Material {
  bool enabled=false, glass=false, unlit=false;
  std::string fragment, vertex, albedo, normal, emissive;
  Schema schema;
  std::array<float,4> tint{1,1,1,1};
  float opacity=1, roughness=0.5f, emission=1;
};
inline bool parse_material(const std::string& text, Material& out, std::string& error) {
  Json j; Material m;
  if (!Reader(text).parse(j) || j.kind!=Json::Object) { error="invalid material JSON"; return false; }
  for (const auto& pair:j.object) {
    const auto& k=pair.first; const auto& v=pair.second;
    if (k=="builtin") {
      if (v.kind!=Json::String || (v.string!="lit"&&v.string!="glass"&&v.string!="unlit")) { error="unknown built-in material"; return false; }
      m.enabled=true; m.glass=v.string=="glass"; m.unlit=v.string=="unlit"; if (m.glass) m.opacity=0.32f;
    } else if (k=="fragment"||k=="vertex"||k=="albedo"||k=="normal"||k=="emissive") {
      if (v.kind!=Json::String || !valid_path(v.string)) { error="material path must be relative and contained"; return false; }
      if (k=="fragment") { m.fragment=v.string; m.enabled=true; }
      if (k=="vertex") m.vertex=v.string;
      if (k=="albedo") m.albedo=v.string;
      if (k=="normal") m.normal=v.string;
      if (k=="emissive") m.emissive=v.string;
    } else if (k=="params") { if (!schema_json(v,m.schema,error)) return false;
    } else if (k=="tint") {
      std::vector<float> a; if (!floats(v,a)||a.size()!=4) { error="tint must be vec4"; return false; }
      std::copy(a.begin(),a.end(),m.tint.begin());
    } else if (k=="opacity"||k=="roughness"||k=="emission") {
      if (v.kind!=Json::Number || v.number<0 || (k!="emission"&&v.number>1)) { error="invalid material scalar: "+k; return false; }
      if (k=="opacity") m.opacity=float(v.number); if (k=="roughness") m.roughness=float(v.number); if (k=="emission") m.emission=float(v.number);
    } else { error="unknown material field: "+k; return false; }
  }
  out=m; return true;
}
template<class T, size_t Capacity> class Handles {
  struct Slot { uint32_t id=0; int owner=0; T value{}; };
  std::array<Slot,Capacity> slots{}; uint32_t serial=0;
public:
  uint32_t add(int owner, const T& value) {
    if (serial==0x7fffffffu || owner<=0) return 0;
    for (auto& s:slots) if (!s.id) { s.value=value; s.owner=owner; return s.id=++serial; } return 0;
  }
  T* get(uint32_t h, int owner) { for (auto& s:slots) if (s.id==h && h && (s.owner==owner || owner==0)) return &s.value; return nullptr; }
  bool remove(uint32_t h, int owner) { for (auto& s:slots) if (s.id==h && h && s.owner==owner) { s=Slot{}; return true; } return false; }
  void clear(int owner) { for (auto& s:slots) if (!owner || s.owner==owner) s=Slot{}; }
  template<class F> void each(F fn) { for (auto& s:slots) if (s.id) fn(s.id,s.owner,s.value); }
};
} // namespace gwshader
