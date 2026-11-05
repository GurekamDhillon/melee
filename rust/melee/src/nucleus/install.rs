//! Installing a downloaded Nucleus post (ports of gw_nucleus_install.h, gw_nucleus_loose.h and gw_nucleus_bundle.h).
//! Pure file work over what the worker downloaded; no network, no game state. Where each kind goes:
//!
//! * COSTUME: the Rust build's skin catalog (cosmetics.rs, `Cosmetics/skins/<slot>/<name>.dat` + `.csp.png` /
//!   `.stock.png`), through `cosmetics_import::install_skin`: the slot is typed by CONTENT (the archive's
//!   `Ply<Fighter>5K<Colour>_Share_joint`), as the C installer does; the C build's `mods/nucleus-<post>-<fighter>/`
//!   skin-mod folder is not read by the Rust engine, the Cosmetics catalog is (the Skins section picks it).
//! * STAGE SKIN / EFFECT / MENU / AUDIO (loose overrides): `mods/nucleus-<post>-<group>/{mod.json, files/<disc path>}`,
//!   each file identified by its CONTENT against the game's own copy (HSD public symbols, >= 55% alike, no structural
//!   symbol missing), written through a `.staging-*` folder swapped in whole; other mods replacing the same file are
//!   recorded in `conflicts` and turned off (the newest install wins); enabled for the next boot. The same layout and
//!   mod.json as the C, which `melee_data::modreg`'s overlay mounts.
//! * GD MOD ZIP (a post whose archive carries `mods/<id>/mod.json`): each folder to `mods/<id>/` (never the exe, never
//!   .exe/.py/.sha256/.zip, never a path outside the folder), a `.nucleus-bundle` marker so only this post's own
//!   earlier copy is ever replaced (the player's own folder of the same name is kept).
//! * Custom stages and MexManager fighter zips need the m-ex runtime (parity.md: not ported): refused with a reason.
use super::catalog::{self, has_ext, basename, stage_name, q, FIGHTERS, FC_AUDIO, FC_EFFECT, FC_MENU, FC_STAGE};
use std::path::{Path, PathBuf};

/// The game's own files, read off the game thread (the worker's own disc handle).
pub struct Env<'a> {
    pub reference: &'a mut dyn FnMut(&str) -> Option<Vec<u8>>,
    pub can_commit: &'a dyn Fn() -> bool,
}

pub struct Post<'a> {
    pub id: i64,
    pub title: &'a str,
    pub author: &'a str,
    pub page: &'a str,
    pub updated: &'a str,
    pub installed_at: String,
}

fn be32(b: &[u8], o: usize) -> u32 {
    b.get(o..o + 4).map_or(0, |s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

/// The public + external symbol names of an HSD archive (`nc_dat_symbols`).
pub fn dat_symbols(raw: &[u8]) -> Result<Vec<String>, String> {
    if raw.len() < 0x20 {
        return Err("too small to be a DAT".into());
    }
    let (fsize, dsize, nreloc, npub, next) = (be32(raw, 0) as usize, be32(raw, 4) as u64, be32(raw, 8) as u64, be32(raw, 12) as u64, be32(raw, 16) as u64);
    if fsize < 0x20 || fsize > raw.len() || npub > 8192 || next > 8192 || nreloc > 4_000_000 {
        return Err("not an HSD archive".into());
    }
    let o_pub = 0x20 + dsize + nreloc * 4;
    let o_sym = o_pub + (npub + next) * 8;
    if o_sym > fsize as u64 {
        return Err("archive tables run past the end".into());
    }
    let mut v = Vec::new();
    for i in 0..(npub + next) {
        let at = (o_sym + be32(raw, (o_pub + i * 8 + 4) as usize) as u64) as usize;
        if at >= fsize {
            continue;
        }
        let s = &raw[at..fsize];
        let Some(e) = s.iter().take(199).position(|&c| c == 0) else { continue };
        v.push(String::from_utf8_lossy(&s[..e]).into_owned());
    }
    Ok(v)
}

fn is_tex(s: &str) -> bool {
    s.ends_with("_image") || s.ends_with("_tlut") || s.ends_with("_tlut_desc") || s.ends_with("_image_desc")
}

/// |common| * 100 / |either|, and how many of the reference's non-texture names the candidate lacks.
fn similarity(cand: &[String], reference: &[String]) -> (u32, u32) {
    let cs: std::collections::HashSet<&str> = cand.iter().map(|s| s.as_str()).collect();
    let (mut common, mut miss) = (0u32, 0u32);
    for r in reference {
        if cs.contains(r.as_str()) {
            common += 1;
        } else if !is_tex(r) {
            miss += 1;
        }
    }
    let uni = cand.len() as u32 + reference.len() as u32 - common;
    (if uni > 0 { common * 100 / uni } else { 0 }, miss)
}

/// (fighter index, colour code) from a costume archive's `Ply<Token>5K<Colour>_Share_joint` (`nc_dat_costume`).
pub fn dat_costume(raw: &[u8]) -> Result<(usize, String, String), String> {
    let syms = dat_symbols(raw)?;
    for s in &syms {
        let Some(mid) = s.strip_prefix("Ply").and_then(|m| m.strip_suffix("_Share_joint")) else { continue };
        if s.len() < 20 {
            continue;
        }
        let Some(k) = mid.get(1..).and_then(|t| t.find("5K")).map(|k| k + 1) else { continue };
        let (tok, col) = (&mid[..k], &mid[k + 2..]);
        if let Some(f) = catalog::fighter_by_token(tok) {
            return Ok((f, tok.to_string(), col.to_string()));
        }
        return Err(format!("a {tok} costume (not a retail fighter)"));
    }
    Err("no costume symbol (Ply<Fighter>5K<Colour>_Share_joint) in it".into())
}

// ---------------------------------------------------------------- folders, enabled.txt

pub fn rmtree(p: &Path) {
    let _ = std::fs::remove_dir_all(p);
}

/// Move a finished staging folder over `to` in one rename (the old copy aside first, put back on failure).
fn swap_dir(from: &Path, to: &Path) -> bool {
    let had = to.is_dir();
    let old = to.with_file_name(format!(".staging-old-{}", to.file_name().unwrap_or_default().to_string_lossy()));
    if had {
        rmtree(&old);
        if std::fs::rename(to, &old).is_err() {
            return false;
        }
    }
    if std::fs::rename(from, to).is_err() {
        if had {
            let _ = std::fs::rename(&old, to);
        }
        return false;
    }
    if had {
        rmtree(&old);
    }
    true
}

/// The mod folders (no dot folders, not `_incoming`).
fn subdirs(mods: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(mods)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| !n.starts_with('.') && n.len() <= 63 && !n.eq_ignore_ascii_case("_incoming"))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn enabled_lines(mods: &Path) -> Option<Vec<String>> {
    std::fs::read_to_string(mods.join("enabled.txt")).ok().map(|t| t.lines().map(|l| l.trim_end_matches([' ', '\r']).to_string()).collect())
}

#[allow(dead_code)]
/// 1 when the mod is in the next-boot set (no enabled.txt: every mod is).
pub fn enabled_query(mods: &Path, id: &str) -> bool {
    enabled_lines(mods).is_none_or(|l| l.iter().any(|x| x.eq_ignore_ascii_case(id)))
}

/// Add / remove `id` in enabled.txt; with no enabled.txt everything is on: adding is a no-op, removing writes the
/// file with every other folder (`nc_enabled_edit` / `nc_enabled_materialize`).
pub fn enabled_edit(mods: &Path, id: &str, on: bool, also_off: &[String]) -> bool {
    let mut lines = match enabled_lines(mods) {
        Some(l) => l,
        None if on && also_off.is_empty() => return true,
        None => subdirs(mods).into_iter().filter(|n| !n.eq_ignore_ascii_case("targettest")).collect(),
    };
    lines.retain(|l| !l.is_empty() && !also_off.iter().any(|o| o.eq_ignore_ascii_case(l)) && (on || !l.eq_ignore_ascii_case(id)));
    if on && !lines.iter().any(|l| l.eq_ignore_ascii_case(id)) {
        lines.push(id.to_string());
    }
    let mut t = lines.join("\n");
    t.push('\n');
    super::queue::write_atomic(&mods.join("enabled.txt"), t.as_bytes()).is_ok()
}

fn json_targets(mods: &Path, id: &str) -> (Vec<String>, Vec<String>) {
    let Ok(t) = std::fs::read_to_string(mods.join(id).join("mod.json")) else { return (vec![], vec![]) };
    let Ok(j) = crate::cosmetics_import::Json::parse(&t) else { return (vec![], vec![]) };
    let arr = |v: Option<&crate::cosmetics_import::Json>| -> Vec<String> {
        match v {
            Some(crate::cosmetics_import::Json::Arr(a)) => a.iter().filter_map(|s| s.str().map(|s| s.to_string())).collect(),
            _ => vec![],
        }
    };
    let nu = j.get("nucleus").or_else(|| j.get("local"));
    (arr(nu.and_then(|n| n.get("targets"))), arr(j.get("conflicts")))
}

/// Other mod folders that replace any of `targets` (their mod.json targets, or a files/<target> of their own) or
/// that list `id` as a conflict.
fn find_conflicts(mods: &Path, id: &str, targets: &[String]) -> Vec<String> {
    subdirs(mods)
        .into_iter()
        .filter(|n| !n.eq_ignore_ascii_case(id))
        .filter(|n| {
            let (theirs, conf) = json_targets(mods, n);
            targets.iter().any(|t| theirs.iter().any(|x| x.eq_ignore_ascii_case(t)) || mods.join(n).join("files").join(t).is_file()) || conf.iter().any(|c| c.eq_ignore_ascii_case(id))
        })
        .take(16)
        .collect()
}

fn write(p: &Path, b: &[u8]) -> bool {
    p.parent().is_none_or(|d| std::fs::create_dir_all(d).is_ok()) && std::fs::write(p, b).is_ok()
}

fn fnv64(b: &[u8]) -> u64 {
    b.iter().fold(1469598103934665603u64, |h, &c| (h ^ c as u64).wrapping_mul(1099511628211))
}

// ---------------------------------------------------------------- costumes

pub struct CostumeIn {
    pub file_id: i64,
    pub filename: String,
    pub color: String,
    pub data: Vec<u8>,
    pub csp: Option<Vec<u8>>,
    pub stock: Option<Vec<u8>>,
}

/// The skin name (`nc_costume_name`): the post title, plus the colour when the post has several costumes.
fn costume_name(title: &str, colour: &str, several: bool) -> String {
    let t = if several && !colour.is_empty() { format!("{title} {colour}") } else { title.to_string() };
    let s: String = t.chars().filter(|c| (' '..'\x7f').contains(c)).take(23).collect();
    let s = s.trim_end().to_string();
    if s.is_empty() { "Nucleus".into() } else { s }
}

pub fn install_costumes(cosmetics: &Path, post: &Post, files: Vec<CostumeIn>, log: &mut dyn FnMut(String)) -> Result<String, String> {
    let several = files.len() > 1;
    let mut done = Vec::new();
    let mut first_err = String::new();
    for f in files {
        let r = (|| -> Result<String, String> {
            let (fighter, tok, col) = dat_costume(&f.data).map_err(|e| format!("Not a plain costume file: {e}."))?;
            if !FIGHTERS[fighter].installable {
                return Err(format!("{} costumes come with another fighter (not yet).", FIGHTERS[fighter].label));
            }
            // the default costume's symbol carries no colour (PlyPeach5K_Share_joint): it is Nr
            let col = if col.is_empty() { "Nr".to_string() } else { col };
            let slot = format!("pl{}{}", FIGHTERS[fighter].pl, col).to_ascii_lowercase();
            if crate::cosmetics::slot_of(&format!("{slot}.dat")).is_none() || catalog::colour_index(fighter, &col).is_none() {
                return Err(format!("{tok} colour {col} is not a retail costume slot."));
            }
            let mut pics = Vec::new();
            for (k, p) in [("csp", &f.csp), ("stock", &f.stock)] {
                if let Some(p) = p {
                    if crate::cosmetics_import::decode_png(p).is_ok() {
                        pics.push((k, p.clone()));
                    } else {
                        log(format!("install {}: file {}: the {k} picture does not decode - default art", post.id, f.file_id));
                    }
                }
            }
            crate::cosmetics_import::install_skin(cosmetics, &slot, &costume_name(post.title, &f.color, several), f.data.clone(), pics)
        })();
        match r {
            Ok(w) => {
                log(format!("install {}: file {} ({}): {w}", post.id, f.file_id, f.filename));
                done.push(w);
            }
            Err(e) => {
                log(format!("install {}: file {} ({}): {e}", post.id, f.file_id, f.filename));
                if first_err.is_empty() {
                    first_err = e;
                }
            }
        }
    }
    if done.is_empty() {
        return Err(if first_err.is_empty() { "Nothing in this post could be installed.".into() } else { first_err });
    }
    let n = done.len();
    Ok(format!("Installed {n} skin{} ({}). Pick {} in the Skins section.", if n == 1 { "" } else { "s" }, done.join(", "), if n == 1 { "it" } else { "them" }))
}

// ---------------------------------------------------------------- loose overrides

pub struct LooseIn {
    pub file_id: i64,
    pub cat: usize,
    pub filename: String,
    pub stem: String,
    pub data: Vec<u8>,
}

struct Plan {
    input: usize,
    group: String,
    kind: &'static str,
    targets: Vec<String>,
    what: String,
    note: String,
}

const SIM_MIN: u32 = 55;
const STAGE_PROBE: [&str; 6] = ["GrNBa", "GrNLa", "GrSt", "GrOp", "GrPs", "GrIz"];

/// Identify one file by content (`nc_loose_plan_file`).
fn plan_file(f: &LooseIn, env: &mut Env) -> Result<Plan, String> {
    let bn = basename(&f.filename).to_string();
    if f.cat == FC_AUDIO {
        if has_ext(&bn, ".hps") {
            if f.data.len() < 0x20 || &f.data[..7] != b" HALPST" {
                return Err(format!("{bn} is not an HPS music file"));
            }
        } else if f.data.len() < 0x20 {
            return Err(format!("{bn} is too small to be a sound bank"));
        }
        let path = ["audio/", "audio/us/"].iter().map(|d| format!("{d}{bn}")).find(|p| (env.reference)(p).is_some()).ok_or(format!("{bn} is not one of the game's audio files"))?;
        return Ok(Plan { input: 0, group: "audio".into(), kind: "misc", targets: vec![path], what: bn.chars().take(40).collect(), note: String::new() });
    }
    if f.cat != FC_STAGE && f.cat != FC_EFFECT && f.cat != FC_MENU {
        return Err(catalog::cat_why_not(f.cat, false).into());
    }
    let cs = dat_symbols(&f.data).map_err(|e| format!("{bn}: {e}"))?;
    if f.cat == FC_STAGE {
        if !["map_head", "coll_data", "grGroundParam"].iter().all(|s| cs.iter().any(|c| c == s)) {
            return Err(format!("{bn} is not a stage file (no map_head / coll_data / grGroundParam)"));
        }
        let stem = f.stem.split('.').next().unwrap_or("").to_string();
        let ext = if has_ext(&f.stem, ".usd") { ".usd" } else { ".dat" };
        let (mut best, mut bsim, mut hint) = (String::new(), -1i64, -1i64);
        let cands: Vec<&str> = std::iter::once(stem.as_str()).chain(STAGE_PROBE.iter().copied().filter(|c| *c != stem)).filter(|c| !c.is_empty()).collect();
        for (i, cand) in cands.iter().enumerate() {
            let Some(rb) = (env.reference)(&format!("{cand}.dat")) else { continue };
            let Ok(rs) = dat_symbols(&rb) else { continue };
            let (sim, miss) = similarity(&cs, &rs);
            let first = i == 0 && !stem.is_empty();
            if first {
                hint = if miss == 0 { sim as i64 } else { 0 };
            }
            if miss == 0 && sim as i64 > bsim {
                bsim = sim as i64;
                best = cand.to_string();
            }
            if first && miss == 0 && sim >= SIM_MIN {
                break; // the name told the truth
            }
        }
        if bsim < SIM_MIN as i64 {
            return Err(if stem.is_empty() { format!("{bn}: could not tell which stage this is") } else { format!("{bn} does not look like {} (it matches {}% of the game's own file)", stage_name(&stem), hint.max(0)) });
        }
        let note = if !stem.is_empty() && stem != best { format!("{bn} is named {stem} but is really {} - installed as {}", stage_name(&best), stage_name(&best)) } else { String::new() };
        let fam = if best.starts_with("GrPs") && best.len() == 5 { "GrPs".to_string() } else { best.clone() };
        return Ok(Plan { input: 0, group: format!("stage-{}", fam.to_ascii_lowercase()), kind: "stage", targets: vec![format!("{best}{ext}")], what: stage_name(&best).chars().take(47).collect(), note });
    }
    // an effect or a menu file: the game's own copy of the same name exists and has this file's structure
    let target = f.stem.clone();
    if target.is_empty() {
        return Err(format!("{bn}: not a game file name"));
    }
    let rb = (env.reference)(&target).ok_or(format!("this game has no {target} to replace"))?;
    let rs = dat_symbols(&rb).map_err(|_| format!("the game's {target} could not be read"))?;
    let (sim, miss) = similarity(&cs, &rs);
    if miss > 0 {
        return Err(format!("{bn} lacks {miss} part{} the game's own {target} has", if miss == 1 { "" } else { "s" }));
    }
    if sim < SIM_MIN {
        return Err(format!("{bn} does not look like the game's {target} ({sim}% alike)"));
    }
    let note = if f.cat == FC_MENU { format!("menu files are global: only one mod can replace {target}") } else { String::new() };
    Ok(Plan { input: 0, group: if f.cat == FC_EFFECT { "fx" } else { "menu" }.into(), kind: "misc", targets: vec![target.clone()], what: target, note })
}

fn source_json(post: &Post) -> String {
    format!(
        ",\n  \"source\": \"nucleus\",\n  \"nucleus\": {{\n    \"id\": {},\n    \"author\": {},\n    \"page_url\": {},\n    \"updated_at\": {},\n    \"installed_at\": {}",
        post.id,
        q(post.author),
        q(post.page),
        q(post.updated),
        q(&post.installed_at)
    )
}

pub fn install_loose(mods: &Path, post: &Post, input: &[LooseIn], env: &mut Env, log: &mut dyn FnMut(String)) -> Result<String, String> {
    let mut plans: Vec<Plan> = Vec::new();
    let mut first_err = String::new();
    for (i, f) in input.iter().enumerate() {
        match plan_file(f, env) {
            Ok(mut p) => {
                p.input = i;
                if !p.note.is_empty() {
                    log(format!("install {}: {}", post.id, p.note));
                }
                plans.push(p);
            }
            Err(e) => {
                log(format!("install {}: file {} ({}) refused: {e}", post.id, f.file_id, f.filename));
                if first_err.is_empty() {
                    first_err = e;
                }
            }
        }
    }
    let mut groups: Vec<String> = Vec::new();
    for p in &plans {
        if !groups.contains(&p.group) {
            groups.push(p.group.clone());
        }
    }
    let (mut installed, mut turned_off) = (0, Vec::<String>::new());
    for g in groups {
        let mut sub: Vec<&Plan> = plans.iter().filter(|p| p.group == g).take(64).collect();
        // prefix-named stages (GrPs ...) read the .usd on an American disc: the .dat is mirrored there unless the post has one
        let mut extra: Vec<(usize, String)> = Vec::new();
        for (k, p) in sub.iter().enumerate() {
            let t = &p.targets[0];
            if p.kind == "stage" && t.ends_with(".dat") {
                let stem = &t[..t.len() - 4];
                let want = format!("{stem}.usd");
                if catalog::STAGE_USD_STEMS.contains(&stem) && !sub.iter().any(|o| o.targets[0] == want) {
                    extra.push((k, want));
                }
            }
        }
        let cat = if g.starts_with("stage") { "stage" } else { g.as_str() };
        let mod_id = format!("nucleus-{}-{g}", post.id);
        let grp_name = sub[0].what.clone();
        let mut title: String = if post.title.is_empty() { if cat == "stage" { "Stage skin" } else { "Nucleus mod" }.into() } else { post.title.chars().take(60).collect() };
        if cat == "stage" && !title.contains(&grp_name) {
            title += &format!(" ({})", grp_name.chars().take(28).collect::<String>());
        }
        let stage_dir = mods.join(format!(".staging-{mod_id}"));
        let final_dir = mods.join(&mod_id);
        rmtree(&stage_dir);
        let mut targets: Vec<String> = Vec::new();
        let mut ok = true;
        let mut nf = 0;
        for (k, p) in sub.iter().enumerate() {
            let data = &input[p.input].data;
            let mut ts = p.targets.clone();
            ts.extend(extra.iter().filter(|e| e.0 == k).map(|e| e.1.clone()));
            for t in ts {
                if !write(&stage_dir.join("files").join(&t), data) {
                    ok = false;
                }
                nf += 1;
                targets.push(t);
            }
        }
        if !ok {
            rmtree(&stage_dir);
            if first_err.is_empty() {
                first_err = "Cannot write to the mods folder.".into();
            }
            continue;
        }
        let conf = find_conflicts(mods, &mod_id, &targets);
        let desc = format!(
            "By {}. From SSBM Nucleus: {}/post/{}.{}",
            if post.author.is_empty() { "unknown" } else { post.author },
            catalog::SITE,
            post.id,
            match cat {
                "stage" => " Online, both players need the same skin for the stage to be offered.",
                "menu" => " Menu files are global: one active per file.",
                _ => "",
            }
        );
        let files_json: Vec<String> = sub.iter().map(|p| format!("{{\"id\": {}, \"size\": {}, \"fnv64\": \"{:016x}\"}}", input[p.input].file_id, input[p.input].data.len(), fnv64(&input[p.input].data))).collect();
        let json = format!(
            "{{\n  \"id\": {},\n  \"name\": {},\n  \"version\": {},\n  \"kind\": {},\n  \"authors\": {},\n  \"description\": {},\n  \"conflicts\": [{}]{},\n    \"category\": {},\n    \"stage\": {},\n    \"targets\": [{}],\n    \"files\": [{}]\n  }}\n}}\n",
            q(&mod_id),
            q(&title),
            q(post.updated),
            q(sub[0].kind),
            q(post.author),
            q(&desc),
            conf.iter().map(|c| q(c)).collect::<Vec<_>>().join(", "),
            source_json(post),
            q(cat),
            q(if cat == "stage" { &grp_name } else { "" }),
            targets.iter().map(|t| q(t)).collect::<Vec<_>>().join(", "),
            files_json.join(", ")
        );
        sub.clear();
        if !write(&stage_dir.join("mod.json"), json.as_bytes()) {
            rmtree(&stage_dir);
            first_err = "Cannot write mod.json.".into();
            continue;
        }
        if !(env.can_commit)() {
            rmtree(&stage_dir);
            first_err = "Install stopped: you went online or the game is closing. Try again offline.".into();
            break;
        }
        if !swap_dir(&stage_dir, &final_dir) {
            rmtree(&stage_dir);
            first_err = format!("Could not replace {mod_id} (the game may be using it). Restart and try again.");
            continue;
        }
        // on for the next boot; whatever else replaces the same files goes off (the newest install wins)
        if !enabled_edit(mods, &mod_id, true, &conf) {
            log(format!("install {}: could not update enabled.txt", post.id));
        }
        turned_off.extend(conf);
        installed += 1;
        log(format!("install {}: {mod_id}: {nf} file(s) for {grp_name}", post.id));
    }
    if installed == 0 {
        return Err(if first_err.is_empty() { "Nothing in this post could be installed.".into() } else { first_err });
    }
    let extra = turned_off.first().map(|t| format!(" Turned off {t} (it replaces the same file).")).unwrap_or_default();
    Ok(format!("Installed {installed} mod{}. Restart the game to use {}.{extra}", if installed == 1 { "" } else { "s" }, if installed == 1 { "it" } else { "them" }))
}

/// Remove every folder of post `id` (nucleus-<id>-*) and its enabled.txt lines.
#[allow(dead_code)]
pub fn remove_post(mods: &Path, id: i64) -> usize {
    let pre = format!("nucleus-{id}-");
    let mut n = 0;
    for d in subdirs(mods).into_iter().filter(|d| d.starts_with(&pre)) {
        rmtree(&mods.join(&d));
        enabled_edit(mods, &d, false, &[]);
        n += 1;
    }
    n
}

// ---------------------------------------------------------------- GD mod zips

/// A path inside a mod folder that may be written: relative, no `..`, no drive / stream / absolute part.
fn rel_ok(rel: &str) -> bool {
    !rel.is_empty() && !rel.starts_with(['/', '\\']) && !rel.contains(':') && rel.split(['/', '\\']).all(|c| !c.is_empty() && c != ".." && c != ".")
}
fn bundle_writable(rel: &str) -> bool {
    ![".exe", ".py", ".sha256", ".zip"].iter().any(|e| has_ext(rel, e))
}

pub fn install_bundle(mods: &Path, post: &Post, zip: crate::cosmetics_import::Zip, env: &mut Env, log: &mut dyn FnMut(String)) -> Result<String, String> {
    let mut ids: Vec<String> = Vec::new();
    for e in &zip.entries {
        if let Some((id, rel)) = e.name.strip_prefix("mods/").and_then(|r| r.split_once('/')) {
            if rel == "mod.json" && catalog::bundle_id_ok(id) && !ids.iter().any(|x| x.eq_ignore_ascii_case(id)) {
                ids.push(id.to_string());
            }
        }
    }
    if ids.is_empty() {
        return Err("This zip has no mods/<name>/mod.json folder, so there is nothing to install.".into());
    }
    if ids.len() > 8 {
        return Err("This zip carries more than 8 mods. Install them as separate packages.".into());
    }
    let tag = format!("post:{}", post.id);
    let (mut done, mut kept) = (Vec::new(), Vec::new());
    let mut total = 0usize;
    for id in &ids {
        let dest = mods.join(id);
        if dest.is_dir() {
            let old = std::fs::read_to_string(dest.join(".nucleus-bundle")).unwrap_or_default();
            if old.lines().next() != Some(tag.as_str()) {
                log(format!("bundle: mods/{id} is already installed and was not installed from this post; kept"));
                kept.push(id.clone());
                continue;
            }
        }
        let stage = mods.join(format!(".staging-{id}"));
        rmtree(&stage);
        let mut n = 0;
        let pre = format!("mods/{id}/");
        for e in zip.entries.iter().filter(|e| e.name.len() > pre.len() && e.name[..pre.len()].eq_ignore_ascii_case(&pre)) {
            let rel = &e.name[pre.len()..];
            if rel.ends_with('/') || !bundle_writable(rel) {
                continue;
            }
            if !rel_ok(rel) {
                rmtree(&stage);
                return Err(format!("The zip has an unsafe path ({}).", e.name));
            }
            let data = zip.read(e).map_err(|err| {
                rmtree(&stage);
                format!("Could not unpack {}: {err}.", e.name)
            })?;
            total += data.len();
            if total > 512 << 20 {
                rmtree(&stage);
                return Err("This zip is too big to install at once.".into());
            }
            if !write(&stage.join(rel), &data) {
                rmtree(&stage);
                return Err(format!("Cannot write {rel}."));
            }
            n += 1;
        }
        if !write(&stage.join(".nucleus-bundle"), tag.as_bytes()) || !(env.can_commit)() {
            rmtree(&stage);
            return Err("Install stopped: you went online or the game is closing. Try again offline.".into());
        }
        if !swap_dir(&stage, &dest) {
            rmtree(&stage);
            return Err(format!("Cannot put {id} in the mods folder."));
        }
        if !enabled_edit(mods, id, true, &[]) {
            return Err(format!("Installed {id} but could not enable it in enabled.txt."));
        }
        log(format!("bundle: mods/{id}: {n} files"));
        done.push(id.clone());
    }
    if done.is_empty() {
        return Err(format!("Nothing new to install: {} {} already in your mods folder from somewhere else.", kept[0], if kept.len() > 1 { "and the others are" } else { "is" }));
    }
    let mut msg = format!("Installed {} mod{} ({}). Restart the game to use {}.", done.len(), if done.len() == 1 { "" } else { "s" }, done.join(", "), if done.len() == 1 { "it" } else { "them" });
    for k in kept {
        msg += &format!(" Kept your own {k}.");
    }
    Ok(msg)
}

/// A crash mid-install leaves only `.staging-*` folders: removed at the next start.
pub fn clean_staging(mods: &Path) -> usize {
    let mut n = 0;
    if let Ok(rd) = std::fs::read_dir(mods) {
        for e in rd.flatten() {
            if e.file_name().to_string_lossy().starts_with(".staging-") && e.path().is_dir() {
                rmtree(&e.path());
                n += 1;
            }
        }
    }
    n
}

#[allow(dead_code)]
pub fn mods_path(p: &str) -> PathBuf {
    PathBuf::from(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dat(syms: &[&str]) -> Vec<u8> {
        // header, no data, no relocs, n public roots, string table
        let mut strings = Vec::new();
        let mut roots = Vec::new();
        for s in syms {
            roots.extend_from_slice(&0u32.to_be_bytes());
            roots.extend_from_slice(&(strings.len() as u32).to_be_bytes());
            strings.extend_from_slice(s.as_bytes());
            strings.push(0);
        }
        let size = 0x20 + roots.len() + strings.len();
        let mut b = Vec::new();
        for v in [size as u32, 0, 0, syms.len() as u32, 0, 0, 0, 0] {
            b.extend_from_slice(&v.to_be_bytes());
        }
        b.extend(roots);
        b.extend(strings);
        b
    }
    #[test]
    fn costume_symbol_types_the_slot() {
        let (f, tok, col) = dat_costume(&dat(&["PlyFox5KGr_Share_joint", "PlyFox5KGr_Share_matanim_joint"])).unwrap();
        assert_eq!((FIGHTERS[f].pl, tok.as_str(), col.as_str()), ("Fx", "Fox", "Gr"));
        assert!(dat_costume(&dat(&["map_head"])).is_err());
    }
    #[test]
    fn loose_stage_is_typed_by_content_and_enabled() {
        let dir = std::env::temp_dir().join(format!("nc-loose-{}", std::process::id()));
        rmtree(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let game = dat(&["map_head", "coll_data", "grGroundParam", "a_image", "x"]);
        let skin = dat(&["map_head", "coll_data", "grGroundParam", "b_image", "x"]);
        let post = Post { id: 7, title: "Pumpkin", author: "a", page: "p", updated: "u", installed_at: "i".into() };
        let input = [LooseIn { file_id: 1, cat: FC_STAGE, filename: "Pumpkin - GrOp.dat".into(), stem: "GrOp.dat".into(), data: skin }];
        let mut reference = |p: &str| (p == "GrOp.dat").then(|| game.clone());
        let mut env = Env { reference: &mut reference, can_commit: &|| true };
        let mut logs = Vec::new();
        let r = install_loose(&dir, &post, &input, &mut env, &mut |l| logs.push(l));
        assert!(r.is_ok(), "{r:?} {logs:?}");
        assert!(dir.join("nucleus-7-stage-grop/files/GrOp.dat").is_file());
        assert!(dir.join("nucleus-7-stage-grop/mod.json").is_file());
        rmtree(&dir);
    }
    #[test]
    fn paths_are_checked() {
        assert!(rel_ok("files/GrOp.dat"));
        assert!(!rel_ok("../x"));
        assert!(!rel_ok("C:/x"));
        assert!(!bundle_writable("melee-pc.exe"));
    }
}
