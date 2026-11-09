//! Fighter-controls preferences on disk (melee_engine::fighter_controls::STORE): `fighter_controls.cfg` in the
//! YOUGAME_DATA folder when there is one, else beside settings.cfg, in settings.cfg's own `key=value` format and with the
//! C build's keys (gw_fighter_controls_runtime.inc wrote them into settings.cfg itself). `MELEE_CONTROLS_CFG` names
//! another file. Read when a match is frozen and written when a menu saves a choice; never read by the simulation.
use melee_data::settings::Settings;
use std::path::PathBuf;

pub fn path() -> PathBuf {
    if let Some(p) = std::env::var_os("MELEE_CONTROLS_CFG").filter(|p| !p.is_empty()) {
        return PathBuf::from(p);
    }
    if let Some(d) = melee_platform::yougame::data_dir() {
        return PathBuf::from(d).join("fighter_controls.cfg");
    }
    melee_data::settings::default_path().with_file_name("fighter_controls.cfg")
}

fn read(key: &str) -> Option<String> {
    let mut s = Settings::new(path());
    let (found, v) = s.get_str(key, 64, None);
    found.then(|| String::from_utf8_lossy(&v).into_owned())
}

fn write(key: &str, value: u32) -> bool {
    let mut s = Settings::new(path());
    s.set_str(key, &value.to_string());
    s.last_save_ok()
}

/// Points the engine's preference store at the file (once, at startup).
pub fn install() {
    unsafe { melee_engine::fighter_controls::STORE = Some((read, write)) };
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_trip_through_the_file() {
        let p = std::env::temp_dir().join(format!("fc_store_{}.cfg", std::process::id()));
        let _ = std::fs::remove_file(&p);
        std::env::set_var("MELEE_CONTROLS_CFG", &p);
        assert_eq!(super::read("fc_f0_p0_s0_tap"), None);
        assert!(super::write("fc_f0_p0_s0_tap", 1));
        assert!(super::write("fc_f0_p0_s0_retail08", 4));
        assert_eq!((super::read("fc_f0_p0_s0_tap").as_deref(), super::read("fc_f0_p0_s0_retail08").as_deref()), (Some("1"), Some("4")));
        assert!(std::fs::read_to_string(&p).unwrap().contains("fc_f0_p0_s0_tap=1"));
        std::env::remove_var("MELEE_CONTROLS_CFG");
        let _ = std::fs::remove_file(&p);
    }
}
