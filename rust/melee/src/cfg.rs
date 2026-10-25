//! The port's settings.cfg through `melee_data::settings` (the port of gw_settings.c): the
//! environment (`MELEE_<KEY>`) wins, then `settings.cfg` beside the executable (or
//! `MELEE_SETTINGS_CFG`), then the default. Only launcher preferences are read here; nothing in it
//! reaches the game logic.

use melee_data::settings::Settings;

pub fn open() -> Settings {
    Settings::open_default()
}

/// A 0/1-style preference: `MELEE_<KEY>` / `key=` in settings.cfg, else `dflt`.
pub fn flag(key: &str, dflt: bool) -> bool {
    open().get_int(key, dflt as i32) != 0
}

/// A string preference, `None` when neither the environment nor the file has a nonempty value.
pub fn text(key: &str) -> Option<String> {
    let v = open().str_or(key, "");
    (!v.is_empty()).then_some(v)
}

#[cfg(test)]
mod tests {
    use melee_data::settings::Settings;

    #[test]
    fn file_then_default_env_wins() {
        let p = std::env::temp_dir().join(format!("cfg_{}.cfg", std::process::id()));
        std::fs::write(&p, "audio=0\nmods_dir=D:/m\n").unwrap();
        let mut s = Settings::with_env(&p, Box::new(|k| (k == "MELEE_SKIP_INTRO").then(|| b"1".to_vec())));
        assert_eq!(s.get_int("audio", 1), 0);
        assert_eq!(s.get_int("skip_intro", 0), 1);
        assert_eq!(s.get_int("mods", 1), 1);
        assert_eq!(s.str_or("mods_dir", ""), "D:/m");
        let _ = std::fs::remove_file(&p);
    }
}
