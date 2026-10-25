//! The agreed-match scene string (`np_build_scene`) and its parse.

/// `mode=vs;at=match;p1=<tok>/c<N>/hu;p2=...;stage=<tok>;match=stock;stocks=N;minutes=M;items=off;pause=0`
/// plus `;turbo=<hex>` / `;envoy=<hex>` when set.
#[allow(clippy::too_many_arguments)]
pub fn build_scene(host_tok: &str, host_c: i32, guest_tok: &str, guest_c: i32, stage_tok: &str, stocks: i32, minutes: i32, turbo: u32) -> String {
    let mut s = format!(
        "mode=vs;at=match;p1={host_tok}/c{host_c}/hu;p2={guest_tok}/c{guest_c}/hu;stage={stage_tok};match=stock;stocks={stocks};minutes={minutes};items=off;pause=0"
    );
    if turbo != 0 {
        s += &format!(";turbo={turbo:x}");
    }
    s
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParsedScene {
    pub host_tok: String,
    pub host_color: i32,
    pub guest_tok: String,
    pub guest_color: i32,
    pub stage_tok: String,
    pub stocks: i32,
    pub minutes: i32,
    pub turbo: u32,
    /// Keys the Rust sim does not implement (cpus, items=N, teams, envoy, run, skin...).
    pub extras: Vec<String>,
}

pub fn parse_scene(s: &str) -> Option<ParsedScene> {
    let mut p = ParsedScene::default();
    let mut ok = 0;
    for kv in s.split(';') {
        let kv = kv.trim();
        if kv.is_empty() {
            continue;
        }
        let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
        match k {
            "mode" | "at" | "match" | "pause" => {}
            "p1" | "p2" => {
                let mut it = v.split('/');
                let tok = it.next()?.to_string();
                let c = it.next().and_then(|c| c.strip_prefix('c')).and_then(|c| c.parse().ok()).unwrap_or(0);
                if k == "p1" {
                    p.host_tok = tok;
                    p.host_color = c;
                } else {
                    p.guest_tok = tok;
                    p.guest_color = c;
                }
                ok += 1;
            }
            "stage" => {
                p.stage_tok = v.to_string();
                ok += 1;
            }
            "stocks" => p.stocks = v.parse().ok()?,
            "minutes" => p.minutes = v.parse().ok()?,
            "items" if v == "off" => {}
            "turbo" => p.turbo = u32::from_str_radix(v, 16).ok()?,
            _ => p.extras.push(kv.to_string()),
        }
    }
    (ok == 3).then_some(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let s = build_scene("ck:2", 0, "ck:9", 1, "ext:31", 4, 8, 0);
        assert_eq!(s, "mode=vs;at=match;p1=ck:2/c0/hu;p2=ck:9/c1/hu;stage=ext:31;match=stock;stocks=4;minutes=8;items=off;pause=0");
        let p = parse_scene(&s).unwrap();
        assert_eq!((p.host_tok.as_str(), p.guest_color, p.stocks, p.minutes), ("ck:2", 1, 4, 8));
        assert!(p.extras.is_empty());
        assert!(parse_scene(&format!("{s};items=3;cpus=x")).unwrap().extras.len() == 2);
    }
}
