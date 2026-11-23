//! The launcher's `port-settings.ini` as a document we can edit without losing anything (the settings window,
//! settings_window.rs).
//!
//! The file is `key value` per line (launcher_compat.rs reads it the same way). Writing touches only the lines of the
//! keys we change: every other line (keys the Rust engine does not use, the controls tables, comments, blank lines,
//! odd spacing) comes back byte for byte, in its place. A key we set that the file lacks is appended.
//!
//! Mapping (our settings.cfg key / value -> his key / value), the reverse of `launcher_compat::map_settings`:
//! `widescreen` 0/1 -> `widescreen` (and `truewidescreen 0` when turned off), `fullscreen` off|borderless|exclusive ->
//! `fullscreen` + `exclusivefullscreen`, `vsync` 0/1, `render_scale` auto|N -> `scale` 0|N, `msaa` N, `volume`,
//! `music`. Our other rows (overlays, game, customize) live in settings.cfg only.
use std::path::Path;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PortIni {
    lines: Vec<String>,
    /// The file ended with a newline (kept as it was).
    trailing_newline: bool,
    /// CRLF line ends (his Windows build writes LF, a hand edit may not).
    crlf: bool,
}

fn key_of(line: &str) -> Option<&str> {
    let l = line.trim_start();
    if l.is_empty() || l.starts_with('#') || l.starts_with(';') || l.starts_with('[') {
        return None;
    }
    Some(l.split(char::is_whitespace).next().unwrap_or(l))
}

impl PortIni {
    pub fn parse(text: &str) -> PortIni {
        let crlf = text.contains("\r\n");
        let trailing_newline = text.is_empty() || text.ends_with('\n');
        let body = text.strip_suffix('\n').map(|t| t.strip_suffix('\r').unwrap_or(t)).unwrap_or(text);
        let lines = if text.is_empty() { Vec::new() } else { body.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l).to_string()).collect() };
        PortIni { lines, trailing_newline, crlf }
    }

    pub fn load(path: &Path) -> PortIni {
        std::fs::read_to_string(path).map(|t| PortIni::parse(&t)).unwrap_or_default()
    }

    pub fn render(&self) -> String {
        let nl = if self.crlf { "\r\n" } else { "\n" };
        let mut s = self.lines.join(nl);
        if self.trailing_newline && !self.lines.is_empty() {
            s.push_str(nl);
        }
        s
    }

    /// Write through a temporary file beside it, so a crash mid-write never leaves half a file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let tmp = path.with_extension("ini.tmp");
        std::fs::write(&tmp, self.render())?;
        std::fs::rename(&tmp, path)
    }

    /// The last value of `key` (the reader takes the last one too).
    pub fn get(&self, key: &str) -> Option<String> {
        self.lines.iter().rev().find(|l| key_of(l) == Some(key)).map(|l| {
            let l = l.trim_start();
            l[key.len()..].trim().to_string()
        })
    }

    /// Set `key`: its last line is rewritten (indent and separator kept), else a line is appended. Returns whether
    /// the text changed.
    pub fn set(&mut self, key: &str, value: &str) -> bool {
        if let Some(i) = self.lines.iter().rposition(|l| key_of(l) == Some(key)) {
            let l = &self.lines[i];
            let indent = &l[..l.len() - l.trim_start().len()];
            let after = &l.trim_start()[key.len()..];
            let sep_len = after.len() - after.trim_start().len();
            let sep = if sep_len == 0 { " " } else { &after[..sep_len] };
            let new = format!("{indent}{key}{sep}{value}");
            if new == self.lines[i] {
                return false;
            }
            self.lines[i] = new;
        } else {
            self.lines.push(format!("{key} {value}"));
            self.trailing_newline = true;
        }
        true
    }

    pub fn pairs(&self) -> Vec<(String, String)> {
        crate::launcher_compat::parse_settings(&self.render())
    }
}

/// One change of ours (settings.cfg key, stored value) as his keys and values. Empty: not one of his.
pub fn to_port(key: &str, value: &str, ini: &PortIni) -> Vec<(&'static str, String)> {
    let on = |v: &str| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "on" | "yes" | "true");
    match key {
        "widescreen" => {
            let mut v = vec![("widescreen", (on(value) as u8).to_string())];
            if !on(value) && ini.get("truewidescreen").is_some() {
                v.push(("truewidescreen", "0".into()));
            }
            v
        }
        "fullscreen" => {
            let (fs, ex) = match value {
                "borderless" => ("1", "0"),
                "exclusive" => ("1", "1"),
                _ => ("0", "0"),
            };
            let mut v = vec![("fullscreen", fs.to_string())];
            // exclusivefullscreen only means something with fullscreen on: leave it alone when turning it off
            if fs == "1" || ini.get("exclusivefullscreen").is_some() {
                v.push(("exclusivefullscreen", ex.to_string()));
            }
            v
        }
        "vsync" => vec![("vsync", (on(value) as u8).to_string())],
        "render_scale" => vec![("scale", if value == "auto" { "0".into() } else { value.to_string() })],
        "msaa" => vec![("msaa", value.to_string())],
        "volume" => vec![("volume", value.to_string())],
        "music" => vec![("music", value.to_string())],
        _ => vec![],
    }
}

/// His file's values as our settings.cfg keys and values (from `launcher_compat::map_settings`). `volume` is ours in
/// audio.cfg, returned here as `volume` too; the caller saves it there.
pub fn from_port(ini: &PortIni) -> Vec<(String, String)> {
    crate::launcher_compat::map_settings(&ini.pairs())
        .into_iter()
        .filter_map(|(k, v)| {
            let key = match k.as_str() {
                "MELEE_FULLSCREEN" => "fullscreen",
                "MELEE_WIDESCREEN" => "widescreen",
                "MELEE_RENDER_SCALE" => "render_scale",
                "MELEE_MSAA" => "msaa",
                "MELEE_VSYNC" => return Some(("vsync".into(), if v == "on" { "1".into() } else { "0".into() })),
                "MELEE_VOLUME" => "volume",
                "MELEE_MUSIC" => "music",
                "MELEE_WINDOW" => "window",
                _ => return None,
            };
            Some((key.to_string(), v))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIS: &str = "# Melee Unlocked port settings\nfullscreen 0\nwindow 1280x960\nscale 2\nvsync 1\nvolume 70\nmusic 80\n\
dlss_mode quality\n; hand note\nkey_A 88\npad1_A 4096\ngc1_A 256\nds41_Cross 1\n  backgroundinput\t1\nwidescreen 0\n";

    #[test]
    fn round_trip_is_byte_exact() {
        for t in [HIS, "", "volume 5", "a 1\r\nb 2\r\n", "\n\n# only comments\n"] {
            assert_eq!(PortIni::parse(t).render(), t, "{t:?}");
        }
    }

    #[test]
    fn set_keeps_unknown_keys_comments_and_order() {
        let mut d = PortIni::parse(HIS);
        assert!(d.set("volume", "35"));
        assert!(!d.set("volume", "35"), "same value: no change");
        assert!(d.set("backgroundinput", "0"));
        assert!(d.set("ssaa", "2"), "a missing key is appended");
        let out = d.render();
        let want = HIS.replace("volume 70", "volume 35").replace("  backgroundinput\t1", "  backgroundinput\t0") + "ssaa 2\n";
        assert_eq!(out, want);
        for keep in ["# Melee Unlocked port settings", "dlss_mode quality", "; hand note", "key_A 88", "pad1_A 4096", "gc1_A 256", "ds41_Cross 1"] {
            assert!(out.lines().any(|l| l == keep), "{keep} kept");
        }
        assert_eq!(d.get("volume").as_deref(), Some("35"));
        assert_eq!(d.get("backgroundinput").as_deref(), Some("0"));
    }

    #[test]
    fn crlf_and_no_trailing_newline_kept() {
        let mut d = PortIni::parse("vsync 1\r\nfps 144");
        d.set("vsync", "0");
        assert_eq!(d.render(), "vsync 0\r\nfps 144");
    }

    #[test]
    fn our_keys_map_to_his_and_back() {
        let mut d = PortIni::parse(HIS);
        for (k, v) in [("fullscreen", "exclusive"), ("widescreen", "1"), ("render_scale", "auto"), ("msaa", "4"), ("vsync", "0"), ("music", "40")] {
            for (hk, hv) in to_port(k, v, &d) {
                d.set(hk, &hv);
            }
        }
        assert!(to_port("input_display", "1", &d).is_empty());
        assert_eq!(d.get("fullscreen").as_deref(), Some("1"));
        assert_eq!(d.get("exclusivefullscreen").as_deref(), Some("1"));
        assert_eq!(d.get("scale").as_deref(), Some("0"));
        let back = from_port(&d);
        let get = |k: &str| back.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone());
        assert_eq!(get("fullscreen").as_deref(), Some("exclusive"));
        assert_eq!(get("widescreen").as_deref(), Some("1"));
        assert_eq!(get("render_scale").as_deref(), Some("auto"));
        assert_eq!(get("msaa").as_deref(), Some("4"));
        assert_eq!(get("vsync").as_deref(), Some("0"));
        assert_eq!(get("music").as_deref(), Some("40"));
        assert_eq!(get("volume").as_deref(), Some("70"));
        // widescreen off also clears true widescreen when the file has it
        let d2 = PortIni::parse("truewidescreen 1\n");
        assert_eq!(to_port("widescreen", "0", &d2), vec![("widescreen", "0".to_string()), ("truewidescreen", "0".to_string())]);
    }
}
