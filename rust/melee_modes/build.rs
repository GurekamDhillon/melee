fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=TOU_ORACLE_DIR");
    if std::env::var("CARGO_FEATURE_C_ORACLE").is_ok() {
        let dir = std::env::var("TOU_ORACLE_DIR").unwrap_or_else(|_| {
            let m = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            format!("{m}/../target/tou-oracle")
        });
        println!("cargo:rustc-link-search=native={dir}");
    }
}
