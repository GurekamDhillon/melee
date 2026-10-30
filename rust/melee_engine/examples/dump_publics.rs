fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut g = melee_engine::gcm::Gcm::open(&std::env::var("GW_ISO_VANILLA").unwrap()).unwrap();
    for f in &a[1..] {
        let d = melee_runtime::dat::DatArchive::parse(&g.read(f).unwrap()).unwrap();
        println!("== {f} data={:#x}", d.data.len());
        for s in &d.publics { println!("  {:#08x} {}", s.offset, s.name); }
        for s in &d.externs { println!("  extern {:#08x} {}", s.offset, s.name); }
    }
}
