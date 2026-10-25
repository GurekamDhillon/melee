// Drives the shipping Aurora CARD API (aurora_card.lib: lib/card + lib/dolphin/card.cpp, GCI-folder mode)
// through a scenario script and prints every result. tests/card_oracle.rs runs the same script through
// melee_data::card. Built by oracle/card_oracle.py (MSVC x86, the flags and libs of Aurora's own CMake tree).
//
//   card_oracle <script> <root>
// Script lines (fields split on single spaces; hex = lowercase hex bytes, "-" = empty):
//   MD rel | MK rel hex      build the card folder tree under <root> before INIT
//   INIT                     CARDSetLoadType(GCIFOLDER), CARDSetBasePath(root, 0/1), CARDInit("GALE", "01")
//   GAME chan game maker     CARDSetGameAndMaker
//   PROBE c | PROBEEX c | CHECK c | FORMAT c | FREE c | ENCODING c | SECTOR c | SERIAL c | RESULT c | XFER c | MOUNT c | UNMOUNT c
//   CREATE c name size slot  | OPEN c name slot | FASTOPEN c idx slot | CLOSE slot
//   READ slot len off | WRITE slot len off seed
//   DELETE c name | FASTDELETE c idx | RENAME c old new
//   STATUS c idx | SETSTATUS c idx bannerfmt iconaddr iconfmt iconspeed commentaddr
//   REMOUNT c                aurora_card_remount
//   FILES                    every file under <root>: relative path, size, FNV (a .gci's modified time zeroed)
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cstdint>
#include <filesystem>
#include <string>
#include <vector>
#include <algorithm>

#include <dolphin/card.h>
#include <aurora/card.h>

namespace fs = std::filesystem;

static int hexval(char c) { return c <= '9' ? c - '0' : c - 'a' + 10; }
static std::vector<unsigned char> unhex(const char* h) {
  std::vector<unsigned char> v;
  if (!strcmp(h, "-")) return v;
  for (; h[0] && h[1]; h += 2) v.push_back((unsigned char)(hexval(h[0]) * 16 + hexval(h[1])));
  return v;
}
static uint64_t fnv(const unsigned char* p, size_t n) {
  uint64_t h = 0xcbf29ce484222325ull;
  while (n--) { h ^= *p++; h *= 0x100000001b3ull; }
  return h;
}
static void phex(const char* s, size_t n) {
  if (!n) { printf("-"); return; }
  for (size_t i = 0; i < n; ++i) printf("%02x", (unsigned char)s[i]);
}

static CARDFileInfo fis[16];

static void print_fi(const CARDFileInfo& f) {
  printf(" fi=%d,%d,%d,%d,%u", f.chan, f.fileNo, f.offset, f.length, (unsigned)f.iBlock);
}

int main(int argc, char** argv) {
  if (argc < 3) return 2;
  FILE* sf = fopen(argv[1], "r");
  std::string root = argv[2];
  static char line[1 << 17];
  while (sf && fgets(line, sizeof line, sf)) {
    char* f[9] = {0};
    int nf = 0;
    char* p = line;
    line[strcspn(line, "\r\n")] = 0;
    while (nf < 9) { f[nf++] = p; p = strchr(p, ' '); if (!p) break; *p++ = 0; }
    const std::string op = f[0];
    struct Flusher { ~Flusher() { fflush(stdout); } } flusher;
    auto arg = [&](int i) { return i < nf ? f[i] : (char*)"0"; };
    for (char* q = f[1]; nf > 1 && *q; ++q) if (*q == '~' && (op == "MD" || op == "MK")) *q = ' ';
    if (op == "MD") {
      fs::create_directories(root + "/" + f[1]);
    } else if (op == "MK") {
      fs::path path = root + "/" + f[1];
      fs::create_directories(path.parent_path());
      auto d = unhex(f[2]);
      FILE* o = fopen(path.string().c_str(), "wb");
      fwrite(d.data(), 1, d.size(), o);
      fclose(o);
    } else if (op == "INIT") {
      CARDSetLoadType(CARD_GCIFOLDER);
      CARDSetBasePath(root.c_str(), 0);
      CARDSetBasePath(root.c_str(), 1);
      CARDInit("GALE", "01");
      printf("INIT\n");
    } else if (op == "GAME") {
      CARDSetGameAndMaker(atoi(f[1]), f[2], f[3]);
      printf("GAME\n");
    } else if (op == "PROBE") {
      printf("PROBE %d\n", (int)CARDProbe(atoi(f[1])));
    } else if (op == "PROBEEX") {
      s32 m = -7, s = -7;
      s32 r = CARDProbeEx(atoi(f[1]), &m, &s);
      printf("PROBEEX %d %d %d\n", r, m, s);
    } else if (op == "CHECK") {
      printf("CHECK %d\n", CARDCheck(atoi(f[1])));
    } else if (op == "FORMAT") {
      printf("FORMAT %d\n", CARDFormat(atoi(f[1])));
    } else if (op == "FREE") {
      s32 b = -7, n = -7;
      s32 r = CARDFreeBlocks(atoi(f[1]), &b, &n);
      printf("FREE %d %d %d\n", r, b, n);
    } else if (op == "ENCODING") {
      u16 e = 99;
      s32 r = CARDGetEncoding(atoi(f[1]), &e);
      printf("ENCODING %d %u\n", r, (unsigned)e);
    } else if (op == "SECTOR") {
      u32 s = 99;
      s32 r = CARDGetSectorSize(atoi(f[1]), &s);
      printf("SECTOR %d %u\n", r, s);
    } else if (op == "SERIAL") {
      u64 s = 99;
      s32 r = CARDGetSerialNo(atoi(f[1]), &s);
      printf("SERIAL %d %llu\n", r, (unsigned long long)s);
    } else if (op == "RESULT") {
      printf("RESULT %d\n", CARDGetResultCode(atoi(f[1])));
    } else if (op == "XFER") {
      printf("XFER %d\n", CARDGetXferredBytes(atoi(f[1])));
    } else if (op == "MOUNT") {
      printf("MOUNT %d\n", CARDMount(atoi(f[1]), nullptr, nullptr));
    } else if (op == "UNMOUNT") {
      printf("UNMOUNT %d\n", CARDUnmount(atoi(f[1])));
    } else if (op == "CREATE") {
      int slot = atoi(f[4]);
      memset(&fis[slot], 0, sizeof fis[slot]);
      s32 r = CARDCreate(atoi(f[1]), f[2], (u32)strtoul(f[3], nullptr, 10), &fis[slot]);
      printf("CREATE %d", r);
      print_fi(fis[slot]);
      printf("\n");
    } else if (op == "OPEN") {
      int slot = atoi(f[3]);
      memset(&fis[slot], 0, sizeof fis[slot]);
      s32 r = CARDOpen(atoi(f[1]), f[2], &fis[slot]);
      printf("OPEN %d", r);
      print_fi(fis[slot]);
      printf("\n");
    } else if (op == "FASTOPEN") {
      int slot = atoi(f[3]);
      memset(&fis[slot], 0, sizeof fis[slot]);
      s32 r = CARDFastOpen(atoi(f[1]), atoi(f[2]), &fis[slot]);
      printf("FASTOPEN %d", r);
      print_fi(fis[slot]);
      printf("\n");
    } else if (op == "CLOSE") {
      int slot = atoi(f[1]);
      printf("CLOSE %d\n", CARDClose(&fis[slot]));
    } else if (op == "READ") {
      int slot = atoi(f[1]);
      s32 len = (s32)strtol(f[2], nullptr, 10), off = (s32)strtol(f[3], nullptr, 10);
      std::vector<unsigned char> buf((size_t)(len > 0 ? len : 1) + 8, 0xEE);
      s32 r = CARDRead(&fis[slot], buf.data(), len, off);
      printf("READ %d ", r);
      if (r == 0 && len > 0) printf("%016llx ", (unsigned long long)fnv(buf.data(), (size_t)len)), phex((char*)buf.data(), std::min<size_t>((size_t)len, 16));
      else printf("-");
      printf("\n");
    } else if (op == "WRITE") {
      int slot = atoi(f[1]);
      s32 len = (s32)strtol(f[2], nullptr, 10), off = (s32)strtol(f[3], nullptr, 10);
      unsigned seed = (unsigned)strtoul(f[4], nullptr, 10);
      std::vector<unsigned char> buf((size_t)(len > 0 ? len : 1));
      for (size_t i = 0; i < buf.size(); ++i) buf[i] = (unsigned char)((seed + i * 7) & 255);
      s32 r = CARDWrite(&fis[slot], buf.data(), len, off);
      printf("WRITE %d\n", r);
    } else if (op == "DELETE") {
      printf("DELETE %d\n", CARDDelete(atoi(f[1]), f[2]));
    } else if (op == "FASTDELETE") {
      printf("FASTDELETE %d\n", CARDFastDelete(atoi(f[1]), atoi(f[2])));
    } else if (op == "RENAME") {
      printf("RENAME %d\n", CARDRename(atoi(f[1]), f[2], f[3]));
    } else if (op == "STATUS") {
      CARDStat st;
      memset(&st, 0xAB, sizeof st);
      s32 r = CARDGetStatus(atoi(f[1]), atoi(f[2]), &st);
      printf("STATUS %d", r);
      if (r == 0) {
        printf(" name=");
        phex(st.fileName, strnlen(st.fileName, 32));
        printf(" len=%u t game=", st.length);
        phex((char*)st.gameName, 4);
        printf(" co=");
        phex((char*)st.company, 2);
        printf(" bf=%u ia=%u if=%u is=%u ca=%u ob=%u obt=%u oi=", st.bannerFormat, st.iconAddr, st.iconFormat, st.iconSpeed, st.commentAddr,
               st.offsetBanner, st.offsetBannerTlut);
        for (int i = 0; i < 8; ++i) printf("%u%s", st.offsetIcon[i], i < 7 ? "," : "");
        printf(" oit=%u od=%u", st.offsetIconTlut, st.offsetData);
      }
      printf("\n");
    } else if (op == "SETSTATUS") {
      CARDStat st;
      memset(&st, 0, sizeof st);
      st.bannerFormat = (u8)atoi(f[3]);
      st.iconAddr = (u32)strtoul(f[4], nullptr, 10);
      st.iconFormat = (u16)atoi(f[5]);
      st.iconSpeed = (u16)atoi(f[6]);
      st.commentAddr = (u32)strtoul(arg(7), nullptr, 10);
      printf("SETSTATUS %d\n", CARDSetStatus(atoi(f[1]), atoi(f[2]), &st));
    } else if (op == "REMOUNT") {
      printf("REMOUNT %d\n", (int)aurora_card_remount(atoi(f[1])));
    } else if (op == "FILES") {
      std::vector<std::string> names;
      std::error_code ec;
      for (auto it = fs::recursive_directory_iterator(root, ec); !ec && it != fs::recursive_directory_iterator(); it.increment(ec)) {
        if (it->is_regular_file()) names.push_back(fs::relative(it->path(), root).generic_string());
      }
      std::sort(names.begin(), names.end());
      for (auto& n : names) {
        FILE* fp = fopen((root + "/" + n).c_str(), "rb");
        std::vector<unsigned char> d;
        unsigned char b[4096];
        size_t g;
        while ((g = fread(b, 1, sizeof b, fp)) > 0) d.insert(d.end(), b, b + g);
        fclose(fp);
        if (n.size() > 4 && n.substr(n.size() - 4) == ".gci" && d.size() >= 0x2C) memset(d.data() + 0x28, 0, 4);
        printf("F %s %zu %016llx\n", n.c_str(), d.size(), (unsigned long long)fnv(d.data(), d.size()));
      }
    }
  }
  return 0;
}
