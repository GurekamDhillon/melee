#include "src/tint/lang/wgsl/reader/reader.h"
#include "src/tint/api/tint.h"
#include <fstream>
#include <iostream>
#include <sstream>
int main(int argc, char** argv) {
    tint::Initialize();
    tint::wgsl::reader::Options options;
    options.allowed_features = tint::wgsl::AllowedFeatures::Everything();
    for (int i=1; i<argc; ++i) {
        std::ifstream f(argv[i]);
        std::ostringstream text; text << f.rdbuf();
        if (!f || text.str().empty()) return 2;
        tint::Source::File file(argv[i], text.str());
        auto program = tint::wgsl::reader::Parse(&file, options);
        if (!program.IsValid()) {
            std::cerr << program.Diagnostics().Str() << "\n";
            return 1;
        }
        std::cout << "WGSL validated: " << argv[i] << "\n";
    }
    tint::Shutdown();
}
