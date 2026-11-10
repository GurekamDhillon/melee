// pool.toml -> envoy_lint -> $OUT_DIR/pool_gen.rs. A lint failure fails the build of every Envoy mod.
#[path = "src/codes.rs"]
#[allow(dead_code)]
mod codes;
#[path = "src/spec.rs"]
mod spec;

fn main() {
    println!("cargo:rerun-if-changed=pool.toml");
    println!("cargo:rerun-if-changed=src/spec.rs");
    println!("cargo:rerun-if-changed=src/codes.rs");
    let src = std::fs::read_to_string("pool.toml").expect("envoy_data: pool.toml");
    let pool = spec::parse(&src).unwrap_or_else(|e| panic!("envoy_lint: pool.toml: {e}"));
    let errs = spec::lint(&pool);
    if !errs.is_empty() {
        for e in &errs {
            println!("cargo:warning=envoy_lint: {e}");
        }
        panic!("envoy_lint: {} problem(s) in pool.toml:\n{}", errs.len(), errs.join("\n"));
    }
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("pool_gen.rs");
    std::fs::write(out, spec::generate(&pool)).unwrap();
}
