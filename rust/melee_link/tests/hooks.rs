use std::collections::BTreeSet;

#[test]
fn install_runs_and_unlinked_todo_hooks_do_not_grow() {
    melee_link::install();
    let dir = env!("CARGO_MANIFEST_DIR");
    let mut reached = BTreeSet::new();
    for e in std::fs::read_dir(format!("{dir}/../../docs/rust-port/trace")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().map_or(false, |x| x == "csv") {
            for l in std::fs::read_to_string(&p).unwrap().lines().skip(1) {
                reached.insert(l.split(',').next().unwrap().to_string());
            }
        }
    }
    // hooks whose default is todo!()/panic and that nothing installs, among functions the C trace reaches
    let mut left: BTreeSet<String> = BTreeSet::new();
    for (name, path, linked, todo) in melee_link::HOOKS {
        if !linked && *todo && reached.contains(*name) { left.insert(format!("{name} {path}")); }
    }
    let base: BTreeSet<String> = std::fs::read_to_string(format!("{dir}/todo_reached_baseline.txt"))
        .unwrap_or_default().lines().filter(|l| !l.is_empty()).map(String::from).collect();
    println!("trace-reached hooks still at todo!(): {} (baseline {})", left.len(), base.len());
    if std::env::var("MELEE_LINK_WRITE_BASELINE").is_ok() {
        std::fs::write(format!("{dir}/todo_reached_baseline.txt"), left.iter().map(|s| format!("{s}\n")).collect::<String>()).unwrap();
        return;
    }
    let new: Vec<_> = left.difference(&base).collect();
    assert!(new.is_empty(), "new unlinked todo hooks: {new:?}");
}
