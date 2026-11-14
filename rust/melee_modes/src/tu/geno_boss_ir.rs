//! Boss IR -> Geno boss define, at load time (docs/specs/geno-v5.2-boss.md §6).
//!
//! The IR (`ports/ir/schema/boss.schema.json`) is a local file written by an analyst from the owner's own game files;
//! it is never in this repo. This module only knows the IR's *vocabulary*: op names, the engine-primitive names it
//! cites and its condition phrasing ("statusFrame >= T", "phase == 0 and status in {2,3}"). Every boss-specific
//! number, rule and table comes from the file. A phrase the lowering does not know is an error, so a boss never runs
//! with a silently dropped rule: the boss is then unavailable and the caller falls back.
//!
//! Where the file is: `MELEE_BOSS_IR_DIR`, else `<MELEE_BRAWL_LOCAL>/boss-ir`, else the first `_build/boss-ir` walking
//! up from the working directory. Paths are never printed.

use super::geno_boss::*;
use std::collections::BTreeMap;

// ------------------------------------------------------------------------------------------------- tiny JSON

#[derive(Clone, Debug, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    pub fn get(&self, k: &str) -> &J {
        static NULL: J = J::Null;
        match self {
            J::Obj(v) => v.iter().find(|(a, _)| a == k).map(|(_, b)| b).unwrap_or(&NULL),
            _ => &NULL,
        }
    }
    pub fn arr(&self) -> &[J] {
        match self {
            J::Arr(v) => v,
            _ => &[],
        }
    }
    pub fn str(&self) -> &str {
        match self {
            J::Str(s) => s,
            _ => "",
        }
    }
    pub fn num(&self) -> Option<f64> {
        match self {
            J::Num(n) => Some(*n),
            J::Bool(b) => Some(*b as i32 as f64),
            _ => None,
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, J::Null)
    }
}

pub fn parse_json(s: &str) -> Result<J, String> {
    let b = s.as_bytes();
    let mut i = 0;
    let v = value(b, &mut i)?;
    ws(b, &mut i);
    if i != b.len() {
        return Err(format!("json: trailing data at {i}"));
    }
    Ok(v)
}

fn ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && (b[*i] as char).is_ascii_whitespace() {
        *i += 1;
    }
}

fn value(b: &[u8], i: &mut usize) -> Result<J, String> {
    ws(b, i);
    let c = *b.get(*i).ok_or("json: eof")?;
    match c {
        b'{' => {
            *i += 1;
            let mut v = vec![];
            loop {
                ws(b, i);
                if b.get(*i) == Some(&b'}') {
                    *i += 1;
                    return Ok(J::Obj(v));
                }
                let k = match value(b, i)? {
                    J::Str(s) => s,
                    _ => return Err(format!("json: key at {i}")),
                };
                ws(b, i);
                if b.get(*i) != Some(&b':') {
                    return Err(format!("json: ':' at {i}"));
                }
                *i += 1;
                v.push((k, value(b, i)?));
                ws(b, i);
                match b.get(*i) {
                    Some(b',') => *i += 1,
                    Some(b'}') => {}
                    _ => return Err(format!("json: ',' at {i}")),
                }
            }
        }
        b'[' => {
            *i += 1;
            let mut v = vec![];
            loop {
                ws(b, i);
                if b.get(*i) == Some(&b']') {
                    *i += 1;
                    return Ok(J::Arr(v));
                }
                v.push(value(b, i)?);
                ws(b, i);
                match b.get(*i) {
                    Some(b',') => *i += 1,
                    Some(b']') => {}
                    _ => return Err(format!("json: ',' at {i}")),
                }
            }
        }
        b'"' => {
            *i += 1;
            let mut out = String::new();
            loop {
                let c = *b.get(*i).ok_or("json: eof in string")?;
                *i += 1;
                match c {
                    b'"' => return Ok(J::Str(out)),
                    b'\\' => {
                        let e = *b.get(*i).ok_or("json: eof")?;
                        *i += 1;
                        match e {
                            b'n' => out.push('\n'),
                            b't' => out.push('\t'),
                            b'r' => out.push('\r'),
                            b'b' | b'f' => {}
                            b'u' => {
                                let h = std::str::from_utf8(b.get(*i..*i + 4).ok_or("json: \\u")?).map_err(|e| e.to_string())?;
                                *i += 4;
                                out.push(char::from_u32(u32::from_str_radix(h, 16).map_err(|e| e.to_string())?).unwrap_or('?'));
                            }
                            x => out.push(x as char),
                        }
                    }
                    _ => {
                        // utf-8 passthrough
                        let start = *i - 1;
                        let mut end = *i;
                        while end < b.len() && (b[end] & 0xC0) == 0x80 {
                            end += 1;
                        }
                        out.push_str(std::str::from_utf8(&b[start..end]).unwrap_or("?"));
                        *i = end;
                    }
                }
            }
        }
        b't' if b[*i..].starts_with(b"true") => {
            *i += 4;
            Ok(J::Bool(true))
        }
        b'f' if b[*i..].starts_with(b"false") => {
            *i += 5;
            Ok(J::Bool(false))
        }
        b'n' if b[*i..].starts_with(b"null") => {
            *i += 4;
            Ok(J::Null)
        }
        _ => {
            let st = *i;
            while *i < b.len() && matches!(b[*i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                *i += 1;
            }
            std::str::from_utf8(&b[st..*i]).ok().and_then(|s| s.parse::<f64>().ok()).map(J::Num).ok_or(format!("json: value at {st}"))
        }
    }
}

// ------------------------------------------------------------------------------------------------- lowering

/// A motion the host has (name, length in frames, loops).
pub struct HostMotion<'a> {
    pub name: &'a str,
    pub len: i32,
    pub looping: bool,
}

struct Lx<'a> {
    ir: &'a J,
    vars: BTreeMap<String, u8>,
    tables: BTreeMap<String, u8>,
    kinetics: BTreeMap<String, u8>,
    parts: BTreeMap<String, u8>,
    motions: Vec<String>,
    attacks: BTreeMap<String, u8>,
    rule_slot: BTreeMap<i64, u8>,
    errors: Vec<String>,
    /// the last death window parsed ("same death check as msg 1" reuses it)
    death_window: Vec<f64>,
}

fn nums_in(t: &str) -> Vec<f64> {
    let mut out = vec![];
    let b = t.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let neg_ok = i + 1 < b.len() && (b[i] == b'-' || b[i] == b'+') && b[i + 1].is_ascii_digit() && (i == 0 || !b[i - 1].is_ascii_alphanumeric());
        if b[i].is_ascii_digit() || neg_ok {
            let st = i;
            i += 1;
            if b[st] == b'0' && i < b.len() && (b[i] == b'x' || b[i] == b'X') {
                i += 1;
                while i < b.len() && b[i].is_ascii_hexdigit() {
                    i += 1;
                }
                if let Ok(v) = i64::from_str_radix(&t[st + 2..i], 16) {
                    out.push(v as f64);
                }
                continue;
            }
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            if let Ok(v) = t[st..i].trim_end_matches('.').parse::<f64>() {
                out.push(v);
            }
            continue;
        }
        if b[i].is_ascii_alphabetic() {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    out
}

fn id_index(id: &str) -> Option<i64> {
    // "status:s11" -> 11, "rule:r5" -> 5, "kin:k16" -> 16, "stageev:e1000" -> 1000
    let tail = id.rsplit(':').next()?;
    tail.trim_start_matches(|c: char| c.is_ascii_alphabetic()).parse().ok()
}

fn msg_id(j: &J) -> Option<u32> {
    match j {
        J::Num(n) => Some(*n as u32),
        J::Str(s) => {
            let s = s.trim();
            if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                u32::from_str_radix(h, 16).ok()
            } else {
                s.parse().ok()
            }
        }
        _ => None,
    }
}

impl<'a> Lx<'a> {
    fn err(&mut self, what: &str, ctx: &str) {
        self.errors.push(format!("{ctx}: {what}"));
    }

    fn num(&self, j: &J) -> Num {
        match j {
            J::Num(n) => Num::K(*n as f32),
            J::Obj(_) => Num::Lin(j.get("base").num().unwrap_or(0.0) as f32, j.get("per_level").num().unwrap_or(0.0) as f32),
            _ => Num::K(0.0),
        }
    }

    fn var(&mut self, name: &str) -> Option<u8> {
        let n = name.trim().trim_start_matches("var:");
        self.vars.get(n).copied().or_else(|| self.vars.iter().find(|(k, _)| k.eq_ignore_ascii_case(n)).map(|(_, v)| *v))
    }

    fn status_ix(id: &str) -> Option<u8> {
        id_index(id).map(|v| v as u8)
    }

    fn rules_mask(&self, j: &J) -> u64 {
        let mut m = 0;
        for r in j.arr() {
            let ix = r.num().map(|v| v as i64).or_else(|| id_index(r.str()));
            if let Some(slot) = ix.and_then(|i| self.rule_slot.get(&i)) {
                m |= 1u64 << slot;
            }
        }
        m
    }

    fn weak_mask(&self, ir: &J) -> u8 {
        let mut m = 0;
        for (i, p) in ir.get("boss").get("parts").arr().iter().enumerate() {
            if p.get("role").str() == "weak_point" {
                m |= 1 << i;
            }
        }
        m
    }

    fn side_parts(&self) -> (u8, u8) {
        let mut l = 1;
        let mut r = 2;
        for (i, p) in self.ir.get("boss").get("parts").arr().iter().enumerate() {
            let n = p.get("name").str().to_ascii_lowercase();
            if p.get("role").str() == "weak_point" && n.contains("left") {
                l = i as u8;
            }
            if p.get("role").str() == "weak_point" && n.contains("right") {
                r = i as u8;
            }
        }
        (l, r)
    }

    fn death_pct(&self) -> f32 {
        for p in self.ir.get("boss").get("parameters").arr() {
            if p.get("meaning").str().contains("death threshold") {
                return p.get("value").num().unwrap_or(0.0) as f32;
            }
        }
        0.0
    }

    /// One clause: `name op number`, `status in {a,b}`, `status != n`, `flag`, `flag clear`, `a >= b`.
    fn clause(&mut self, c: &str, ctx: &str) -> Option<Cond> {
        let c = c.trim();
        let low = c.to_ascii_lowercase();
        if low == "always" {
            return Some(Cond::Always);
        }
        if low == "empty" || low == "reaction ended" {
            return Some(Cond::Never);
        }
        if low == "motion ended" {
            return Some(Cond::MotionEnd);
        }
        if low.starts_with("status in {") || low.starts_with("status is ") {
            let m = nums_in(c).iter().fold(0u64, |m, &v| m | 1u64 << (v as u64));
            return Some(Cond::StatusIn(m));
        }
        for (pat, op) in [(" >= ", Cmp::Ge), (" <= ", Cmp::Le), (" == ", Cmp::Eq), (" != ", Cmp::Ne), (" < ", Cmp::Lt), (" > ", Cmp::Gt)] {
            if let Some((a, b)) = c.split_once(pat) {
                let (a, b) = (a.trim(), b.trim());
                if a.eq_ignore_ascii_case("statusFrame") {
                    if let Some(v) = self.var(b) {
                        return Some(Cond::TimerDone(v));
                    }
                }
                if a.eq_ignore_ascii_case("status") {
                    let m = nums_in(b).iter().fold(0u64, |m, &v| m | 1u64 << (v as u64));
                    return Some(if op == Cmp::Ne { Cond::Not(Box::new(Cond::StatusIn(m))) } else { Cond::StatusIn(m) });
                }
                if a.eq_ignore_ascii_case("prevStatus") && op == Cmp::Eq {
                    let m = nums_in(b).iter().fold(0u64, |m, &v| m | 1u64 << (v as u64));
                    return Some(Cond::PrevStatusIn(m));
                }
                let av = self.var(a);
                if let Some(av) = av {
                    if let Ok(k) = b.parse::<f32>() {
                        return Some(Cond::Var(av, op, k));
                    }
                    if let Some(bv) = self.var(b) {
                        return Some(Cond::VarVar(av, op, bv));
                    }
                }
            }
        }
        let (name, clear) = match low.strip_suffix(" clear") {
            Some(_) => (&c[..c.len() - 6], true),
            None => (c, false),
        };
        if let Some(v) = self.var(name) {
            return Some(Cond::Var(v, if clear { Cmp::Eq } else { Cmp::Ne }, 0.0));
        }
        self.err(&format!("unknown condition clause '{c}'"), ctx);
        None
    }

    /// A condition text: engine-primitive names first, then `and`-joined clauses (parenthesised notes dropped).
    fn cond(&mut self, text: &str, ctx: &str) -> Cond {
        let low = text.to_ascii_lowercase();
        if low.contains("commonaicheckdamageair") {
            return Cond::Hit(HitKind::Air);
        }
        if low.contains("commonaicheckdamageground") {
            return Cond::Hit(HitKind::Ground);
        }
        if low.contains("commonaicheckdamageturn") {
            return Cond::Hit(HitKind::Behind);
        }
        if low.contains("commonaicheckdamagenoreaction") {
            return Cond::Hit(HitKind::Any);
        }
        if low.starts_with("hitbyplayer") && low.contains("min(") {
            let pct = nums_in(text.split("<=").nth(1).unwrap_or("")).first().copied().unwrap_or(0.0) as f32;
            return Cond::And(Box::new(Cond::Hit(HitKind::Any)), Box::new(Cond::MinWeakPctLe(pct)));
        }
        if low.starts_with("death check") {
            let (l, r) = self.side_parts();
            let p = if low.contains("left") { l } else { r };
            return Cond::PartPctLe(p, self.death_pct());
        }
        if low.contains("target exists and none is locked") {
            return Cond::Target(true);
        }
        if low.contains("target is present") {
            return Cond::Target(false);
        }
        // drop "(...)" notes
        let mut s = String::new();
        let mut depth = 0;
        for ch in text.chars() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ if depth == 0 => s.push(ch),
                _ => {}
            }
        }
        let mut out: Option<Cond> = None;
        for part in s.split(" and ") {
            if let Some(c) = self.clause(part, ctx) {
                out = Some(match out {
                    None => c,
                    Some(o) => Cond::And(Box::new(o), Box::new(c)),
                });
            }
        }
        out.unwrap_or(Cond::Never)
    }

    fn ops(&mut self, list: &J, ctx: &str, per_frame: bool) -> Vec<Op> {
        let mut out = vec![];
        let arr = list.arr();
        for (k, o) in arr.iter().enumerate() {
            let next_store = arr.get(k + 1).filter(|n| n.get("op").str() == "set_var" && n.get("text").str() == "index").and_then(|n| n.get("args").get("var").str().to_string().into());
            if o.get("op").str() == "set_var" && o.get("text").str() == "index" {
                continue; // folded into the pick before it
            }
            let op = self.op(o, ctx, per_frame, next_store);
            out.extend(op);
        }
        out
    }

    fn op(&mut self, o: &J, ctx: &str, per_frame: bool, store: Option<String>) -> Vec<Op> {
        let name = o.get("op").str();
        let a = o.get("args");
        let text = o.get("text").str();
        let tl = text.to_ascii_lowercase();
        let one = |x: Op| vec![x];
        match name {
            "disable_rules" => one(Op::DisableRules),
            "reset_rules" => one(Op::ResetRules),
            "enable_rules" => one(Op::EnableRules(self.rules_mask(a.get("rules")))),
            "reset_target" => one(Op::ResetTarget),
            "lock_target" => one(Op::LockTarget),
            "stop_movement" => one(Op::StopMove),
            "despawn" => one(Op::Despawn),
            "effect" => one(Op::Nop), // presentation only
            "intangible" => {
                let on = a.get("on").num().map(|v| v != 0.0).unwrap_or(true);
                let p = a.get("part").str();
                let mask = if p.is_empty() {
                    0xFF
                } else if let Some(&ix) = self.parts.get(p) {
                    1 << ix
                } else if p.contains("cages") || p.contains("both") {
                    self.weak_mask(self.ir)
                } else {
                    self.err(&format!("unknown part '{p}'"), ctx);
                    0
                };
                one(Op::Intangible(mask, on))
            }
            "stage_event" => {
                if let Some(e) = id_index(a.get("event").str()) {
                    return one(Op::StageEvent(e as i32));
                }
                let n = nums_in(text);
                if n.len() >= 2 && tl.contains("lower") && tl.contains(" if ") {
                    if let Some(v) = self.var("lowerSideIsLeft") {
                        return one(Op::StageEventIf(v, n[0] as i32, n[1] as i32));
                    }
                }
                if n.len() >= 2 && tl.contains("left") && tl.contains("right") {
                    let (l, r) = self.side_parts();
                    let mut t = [0; MAX_PARTS];
                    t[l as usize] = n[0] as i32;
                    t[r as usize] = n[1] as i32;
                    return one(Op::StageEventOfSender(t));
                }
                self.err(&format!("stage_event '{text}'"), ctx);
                vec![]
            }
            "set_motion" => match self.motions.iter().position(|m| m == a.get("motion").str()) {
                Some(i) => one(Op::Motion(i as u8)),
                None => {
                    self.err(&format!("unknown motion '{}'", a.get("motion").str()), ctx);
                    vec![]
                }
            },
            "set_movement" => {
                let k = a.get("kind").str();
                if k == "animation_root" {
                    return one(Op::Nop); // root motion is the host's (the motion's own root track)
                }
                match self.kinetics.get(k) {
                    Some(&i) => one(Op::Kinetic(i, a.get("along").str() == "aim")),
                    None => {
                        self.err(&format!("unknown movement kind '{k}'"), ctx);
                        vec![]
                    }
                }
            }
            "set_timer" => {
                let Some(v) = self.var(a.get("var").str()) else {
                    self.err("set_timer var", ctx);
                    return vec![];
                };
                if !a.get("range").is_null() {
                    one(Op::SetTimer(v, self.num(a.get("range").get("lo")), self.num(a.get("range").get("hi"))))
                } else {
                    let n = self.num(a.get("value"));
                    one(Op::SetTimer(v, n, n))
                }
            }
            "add_var" => match self.var(a.get("var").str()) {
                Some(v) => one(Op::AddVar(v, a.get("value").num().unwrap_or(1.0) as f32)),
                None => {
                    self.err("add_var var", ctx);
                    vec![]
                }
            },
            "set_var" => {
                let vn = a.get("var").str();
                let val = a.get("value");
                if vn == "strength" {
                    let n = val.num().or_else(|| nums_in(val.str().split('=').nth(1).unwrap_or(val.str())).first().copied());
                    return one(Op::SetStrength(n.unwrap_or(1.0) as u8));
                }
                if vn == "hp" {
                    return one(Op::Nop); // the core pool is not read by any rule; the defeat status ends the fight
                }
                let Some(v) = self.var(vn) else {
                    self.err(&format!("unknown var '{vn}'"), ctx);
                    return vec![];
                };
                if tl.contains("left% < right%") {
                    let (l, r) = self.side_parts();
                    return one(Op::SetLess(v, l, r));
                }
                match val {
                    J::Num(_) | J::Bool(_) => one(Op::SetVar(v, Num::K(val.num().unwrap_or(0.0) as f32))),
                    _ if !text.is_empty() && val.is_null() => {
                        // e.g. "2 from the next menu jump": informational (the table entry sets it)
                        one(Op::Nop)
                    }
                    _ => {
                        self.err(&format!("set_var value for '{vn}'"), ctx);
                        vec![]
                    }
                }
            }
            "send" => {
                let m = a.get("msg").str();
                let vt = a.get("value").str().to_ascii_lowercase();
                match m {
                    _ if m.ends_with("5105") => {
                        let unless = if vt.contains("only while") { vt.split("only while").nth(1).and_then(|s| s.split_whitespace().next()).and_then(|w| self.var(w)) } else { None };
                        one(Op::Forward(unless))
                    }
                    _ if m.ends_with("5112") => one(Op::Nop), // parts derive their (integer) level from the boss's L
                    _ if m.ends_with("5106") => one(Op::SetCounter(a.get("value").num().unwrap_or(0.0) as i32)),
                    _ if m.contains("clear hitboxes") => one(Op::ClearHitboxes(self.weak_mask(self.ir))),
                    _ => {
                        self.err(&format!("send '{m}'"), ctx);
                        vec![]
                    }
                }
            }
            "pick" => {
                let t = a.get("table").str();
                let Some(&ti) = self.tables.get(t) else {
                    self.err(&format!("unknown table '{t}'"), ctx);
                    return vec![];
                };
                let motion_table = self.ir.get("boss").get("selection_tables").arr()[ti as usize].get("entries").arr().iter().all(|e| e.get("leads_to").is_null());
                if motion_table {
                    return one(if per_frame { Op::MotionPickOnEnd(ti) } else { Op::MotionPick(ti) });
                }
                let store = store.and_then(|s| self.var(&s)).or_else(|| if tl.contains("side=") { self.var("side") } else { None });
                one(Op::Pick(ti, store))
            }
            "if" => {
                let c = self.cond(text, ctx);
                let t = self.ops(a.get("then"), ctx, per_frame);
                let e = self.ops(a.get("else"), ctx, per_frame);
                one(Op::If(c, t, e))
            }
            "change_status" => {
                if let Some(s) = Self::status_ix(a.get("to").str()) {
                    return one(Op::ChangeStatus(s));
                }
                let n = nums_in(text);
                if tl.contains("target.x < boss.x") && n.len() >= 2 {
                    return one(Op::StatusBySide(n[0] as u8, n[1] as u8));
                }
                if tl.contains("(left)") && tl.contains("(right)") && n.len() >= 2 {
                    let (l, r) = self.side_parts();
                    let mut t = [0u8; MAX_PARTS];
                    t[l as usize] = n[0] as u8;
                    t[r as usize] = n[1] as u8;
                    return one(Op::StatusOfSender(t));
                }
                self.err(&format!("change_status '{text}'"), ctx);
                vec![]
            }
            "set_goal_x" => {
                let n = nums_in(text.split("->").skip(1).collect::<Vec<_>>().join(" ").as_str());
                // "-> -68 (44006) else +68 (44007)": first number after each arrow / else
                let lt = text.split("->").nth(1).map(nums_in).and_then(|v| v.first().copied());
                let ge = text.split("else").nth(1).map(nums_in).and_then(|v| v.first().copied());
                match (lt, ge) {
                    (Some(a), Some(b)) => one(Op::GoalBySide(a as f32, b as f32)),
                    _ => {
                        self.err(&format!("set_goal_x '{text}' {n:?}"), ctx);
                        vec![]
                    }
                }
            }
            "sound" => one(Op::Sound(a.get("id").num().map(|v| v as i32).unwrap_or(-1))),
            "shake" => one(Op::Shake(a.get("strength").num().unwrap_or(0.0) as u8)),
            "clear_hitboxes" => {
                let p = a.get("part").str();
                let mask = self.parts.get(p).map(|i| 1u8 << i).or(if p == "body" { Some(1) } else { None }).unwrap_or(0);
                one(Op::ClearHitboxes(mask))
            }
            "hitbox" => {
                let at = a.get("attack").str();
                match self.attacks.get(at) {
                    Some(&i) => one(Op::Hitbox(i, 0)),
                    None => one(Op::Nop), // a box of the current attack (by motion), host side
                }
            }
            "other" => {
                if tl.contains("commonnoneflydamageproc") || tl.contains("noro") || tl.contains("ground snap") || tl.contains("reply handled") || tl.contains("copy cage motion") || tl.contains("trophy") {
                    one(Op::Nop)
                } else if tl.starts_with("aim = angle") {
                    one(Op::Aim)
                } else if tl.contains("death check") {
                    // "-198 left / -197 right": the one-priority window per sending part; "same death check as msg 1"
                    let n: Vec<f64> = nums_in(text).into_iter().filter(|v| *v < 0.0).collect();
                    let n = if n.len() >= 2 { n } else { self.death_window.clone() };
                    if n.len() >= 2 {
                        self.death_window = n.clone();
                        let (l, r) = self.side_parts();
                        let mut t = [i32::MAX; MAX_PARTS];
                        t[l as usize] = n[0] as i32;
                        t[r as usize] = n[1] as i32;
                        one(Op::EvalWindowOfSender(t))
                    } else {
                        self.err(&format!("death window '{text}'"), ctx);
                        vec![]
                    }
                } else {
                    self.err(&format!("op other '{text}'"), ctx);
                    vec![]
                }
            }
            _ => {
                self.err(&format!("unknown op '{name}'"), ctx);
                vec![]
            }
        }
    }
}

/// Lower a parsed IR into a define. `motions` is what the host's model provides (names, lengths, loops).
pub fn lower(ir: &J, motions: &[HostMotion]) -> Result<Define, String> {
    let boss = ir.get("boss");
    if boss.is_null() {
        return Err("no boss block".into());
    }
    let mut lx = Lx {
        ir,
        vars: BTreeMap::new(),
        tables: BTreeMap::new(),
        kinetics: BTreeMap::new(),
        parts: BTreeMap::new(),
        motions: motions.iter().map(|m| m.name.to_string()).collect(),
        attacks: BTreeMap::new(),
        rule_slot: BTreeMap::new(),
        errors: vec![],
        death_window: vec![],
    };
    // vars
    let mut vars = vec![];
    for v in boss.get("variables").arr() {
        let id = v.get("id").str().trim_start_matches("var:").to_string();
        if v.get("engine_maintained").num() == Some(1.0) {
            continue;
        }
        let init = v.get("initial").num().unwrap_or(0.0) as f32;
        lx.vars.insert(id.clone(), vars.len() as u8);
        vars.push((id, init));
    }
    if vars.len() > MAX_VARS {
        return Err("too many variables".into());
    }
    for (i, t) in boss.get("selection_tables").arr().iter().enumerate() {
        lx.tables.insert(t.get("id").str().to_string(), i as u8);
    }
    let mut kinetics = vec![];
    for (i, k) in boss.get("movement_kinds").arr().iter().enumerate() {
        lx.kinetics.insert(k.get("id").str().to_string(), i as u8);
        kinetics.push(match k.get("mode").str() {
            "accelerate_along_angle" => Kinetic::Accel { start: k.get("start_speed").num().unwrap_or(0.0) as f32, acc: lx.num(k.get("acceleration")), max: lx.num(k.get("max_speed")) },
            "grounded" | "decay" => Kinetic::Decay(lx.num(k.get("decay"))),
            _ => Kinetic::Decay(Num::K(0.0)),
        });
    }
    for (i, a) in boss.get("attacks").arr().iter().enumerate() {
        lx.attacks.insert(a.get("id").str().to_string(), i as u8);
    }
    let rules_j = boss.get("rules").arr();
    let mut sorted: Vec<&J> = rules_j.iter().collect();
    sorted.sort_by_key(|r| r.get("index").num().unwrap_or(0.0) as i64);
    if sorted.len() > MAX_RULES {
        return Err("too many rules".into());
    }
    for (slot, r) in sorted.iter().enumerate() {
        lx.rule_slot.insert(r.get("index").num().unwrap_or(0.0) as i64, slot as u8);
    }
    // parts
    let pools = boss.get("hp_pools");
    let mut parts = vec![];
    for (i, p) in boss.get("parts").arr().iter().enumerate() {
        lx.parts.insert(p.get("id").str().to_string(), i as u8);
        let role = match p.get("role").str() {
            "core" => Role::Core,
            "weak_point" => Role::WeakPoint,
            _ => Role::Limb,
        };
        let pool = pools.arr().iter().find(|h| h.get("id").str() == p.get("hp_pool").str());
        let hp = pool.map(|h| lx.num(h.get("max"))).unwrap_or(Num::K(0.0));
        let mut part = Part { name: p.get("name").str().to_string(), role, hp, on_damaged: None, thresholds: vec![], on_zero: None, intangible_in: 0, int_level: p.get("own_level").str().contains("trunc") };
        for h in p.get("hurtboxes").arr() {
            for s in h.get("intangible_in").arr() {
                if let Some(ix) = Lx::status_ix(s.str()) {
                    part.intangible_in |= 1u64 << ix;
                }
            }
        }
        for pr in p.get("procs").arr() {
            let b = pr.get("behaviour").str();
            let bl = b.to_ascii_lowercase();
            match pr.get("name").str() {
                "monitor" => {
                    if let Some(x) = bl.split("send msg").nth(1).map(nums_in).and_then(|v| v.first().copied()) {
                        part.on_damaged = Some(x as u32);
                    }
                    for seg in b.split(';').flat_map(|s| s.split("Else")) {
                        if let (Some(c), Some(rest)) = (seg.split("counter").nth(1), seg.split("->").nth(1)) {
                            let cn = nums_in(c);
                            let m = nums_in(rest);
                            if cn.len() >= 2 && !m.is_empty() {
                                part.thresholds.push((cn[0] as i32, cn[1] as f32, m[0] as u32));
                            }
                        }
                    }
                }
                "broken" => {
                    if let Some(x) = b.split("send").nth(1).map(nums_in).and_then(|v| v.first().copied()) {
                        part.on_zero = Some(x as u32);
                    }
                }
                _ => {}
            }
        }
        parts.push(part);
    }
    if parts.len() > MAX_PARTS {
        return Err("too many parts".into());
    }
    // motions + timelines
    let mut mlist = vec![];
    for m in motions {
        let mut tl = vec![];
        if let Some(t) = boss.get("timelines").arr().iter().find(|t| t.get("motion").str() == m.name) {
            for e in t.get("events").arr() {
                let ops = lx.op(e.get("op"), &format!("timeline {}", m.name), false, None);
                let part = e.get("part").str();
                for mut op in ops {
                    if let (Op::Hitbox(a, _), Some(&p)) = (&op, lx.parts.get(part)) {
                        op = Op::Hitbox(*a, p);
                    }
                    if let (Op::ClearHitboxes(0), Some(&p)) = (&op, lx.parts.get(part)) {
                        op = Op::ClearHitboxes(1 << p);
                    }
                    tl.push(TimelineEvent {
                        frame: e.get("frame").num().unwrap_or(0.0) as i32,
                        count: e.get("repeat").get("count").num().unwrap_or(1.0) as i32,
                        every: e.get("repeat").get("every").num().unwrap_or(0.0) as i32,
                        op,
                    });
                }
            }
        }
        mlist.push(Motion { name: m.name.to_string(), len: m.len, looping: m.looping, timeline: tl });
    }
    // tables
    let mut tables = vec![];
    for t in boss.get("selection_tables").arr() {
        let ctx = format!("table {}", t.get("id").str());
        let mut entries = vec![];
        for e in t.get("entries").arr() {
            let label = e.get("label").str();
            let mut ops = vec![];
            let to = Lx::status_ix(e.get("leads_to").str());
            // "name: a=0, b = cond ? x : y" sets vars before the change
            if let Some((_, sets)) = label.split_once(':') {
                if !sets.contains("instead") {
                    for s in sets.split(',') {
                        if let Some((v, rhs)) = s.split_once('=') {
                            let Some(vi) = lx.var(v) else { continue };
                            if let Some((c, ab)) = rhs.split_once('?') {
                                let (x, y) = ab.split_once(':').unwrap_or((ab, ab));
                                let cond = lx.cond(c.trim().replace(">=", " >= ").as_str(), &ctx);
                                ops.push(Op::If(cond, vec![Op::SetVar(vi, Num::K(x.trim().parse().unwrap_or(0.0)))], vec![Op::SetVar(vi, Num::K(y.trim().parse().unwrap_or(0.0)))]));
                            } else if let Ok(k) = rhs.trim().parse::<f32>() {
                                ops.push(Op::SetVar(vi, Num::K(k)));
                            }
                        }
                    }
                }
            }
            // "(phase>=1: enable rule 24 instead)"
            let alt = label.split_once('(').and_then(|(_, r)| r.split_once(')')).map(|(x, _)| x).filter(|x| x.contains("instead"));
            let change = to.map(Op::ChangeStatus);
            match (alt, change) {
                (Some(x), Some(ch)) => {
                    let (c, act) = x.split_once(':').unwrap_or((x, ""));
                    let cond = lx.cond(c.replace(">=", " >= ").as_str(), &ctx);
                    let rules: u64 = nums_in(act).iter().filter_map(|r| lx.rule_slot.get(&(*r as i64))).fold(0, |m, s| m | 1u64 << s);
                    ops.push(Op::If(cond, vec![Op::EnableRules(rules)], vec![ch]));
                }
                (None, Some(ch)) => ops.push(ch),
                _ => {}
            }
            let motion = if to.is_none() { lx.motions.iter().position(|m| label.split_whitespace().next() == Some(m.as_str())).map(|i| i as u8) } else { None };
            entries.push(TableEntry { weight: lx.num(e.get("weight")), ops, motion });
        }
        tables.push(Table { name: t.get("name").str().to_string(), repeat: t.get("repeat_factor").num().unwrap_or(1.0) as f32, entries });
    }
    // fix the label conditions written without spaces ("phase>=1")
    // statuses
    let mut statuses = vec![];
    for s in boss.get("statuses").arr() {
        let ix = Lx::status_ix(s.get("id").str()).unwrap_or(0);
        let ctx = format!("status {ix}");
        let enter = lx.ops(s.get("on_enter"), &ctx, false);
        let frame = lx.ops(s.get("per_frame"), &ctx, true);
        let once = frame.iter().any(|o| matches!(o, Op::Despawn));
        statuses.push(Status { index: ix, name: s.get("name").str().to_string(), enter, frame, frame_once: once, intangible: s.get("intangible").num() == Some(1.0) });
    }
    // rules
    let mut rules = vec![];
    for r in &sorted {
        let ix = r.get("index").num().unwrap_or(0.0) as u16;
        let ctx = format!("rule {ix}");
        let used = r.get("used").num() != Some(0.0);
        let cond = if used { lx.cond(r.get("condition").str(), &ctx) } else { Cond::Never };
        let ops = if used { lx.ops(r.get("action"), &ctx, false) } else { vec![] };
        rules.push(Rule {
            index: ix,
            priority: r.get("priority").num().unwrap_or(0.0) as i32,
            default_on: r.get("default_enabled").num() == Some(1.0),
            cond,
            ops,
            to: Lx::status_ix(r.get("state").str()),
            per_target: r.get("per_target").num() == Some(1.0),
        });
    }
    // messages
    let mut messages = vec![];
    for m in boss.get("messages").arr() {
        if m.get("handling").arr().is_empty() {
            continue;
        }
        let Some(id) = msg_id(m.get("raw_id")) else { continue };
        let ops = lx.ops(m.get("handling"), &format!("message {id:#x}"), false);
        messages.push((id, ops));
    }
    // level
    let lv = boss.get("level");
    let tab = lv.get("tables").arr().iter().find(|t| t.get("selected_when").str() == "default").or(lv.get("tables").arr().first());
    let level_rows: Vec<Vec<f32>> = tab.map(|t| t.get("rows").arr().iter().map(|r| r.arr().iter().map(|x| x.num().unwrap_or(0.0) as f32).collect()).collect()).unwrap_or_default();
    // target selection
    let ts = boss.get("target_selection");
    let mut slot_w = [0f32; 4];
    for (i, w) in ts.get("slot_weights").arr().iter().take(4).enumerate() {
        slot_w[i] = w.num().unwrap_or(0.0) as f32;
    }
    // routing: from the core part
    let (mut route_mask, mut route_factor) = (0u8, 1.0f32);
    for r in boss.get("damage_routing").arr() {
        let from = lx.parts.get(r.get("from").str()).copied();
        if from.is_some_and(|f| parts[f as usize].role == Role::Core) {
            for t in r.get("to").arr() {
                if let Some(&p) = lx.parts.get(t.str()) {
                    route_mask |= 1 << p;
                }
            }
            route_factor = r.get("factor").num().unwrap_or(1.0) as f32;
        }
    }
    let mut defeat_status = [None, None];
    for d in boss.get("defeat").arr() {
        let st = Lx::status_ix(d.get("state").str());
        match d.get("kind").str() {
            "threshold_then_hit" => defeat_status[0] = st,
            "part_break" => defeat_status[1] = st,
            _ => {}
        }
    }
    let start_status = boss.get("spawns").arr().iter().filter_map(|s| s.get("spawned_by").arr().first()).filter_map(|s| Lx::status_ix(s.str())).next().unwrap_or(0);
    let cam = boss.get("camera");
    let camera = if cam.is_null() {
        None
    } else {
        let l = cam.get("limits");
        Some(Camera {
            distance: cam.get("distance").num().unwrap_or(0.0) as f32,
            ease: cam.get("ease").num().unwrap_or(0.05) as f32,
            tilt: cam.get("tilt_degrees").num().unwrap_or(0.0) as f32,
            fov: cam.get("fov_degrees").num().unwrap_or(30.0) as f32,
            limits: [l.get("left").num().unwrap_or(-180.0) as f32, l.get("right").num().unwrap_or(180.0) as f32, l.get("bottom").num().unwrap_or(-70.0) as f32, l.get("top").num().unwrap_or(120.0) as f32],
        })
    };
    let win = boss.get("rule_engine").get("default_window").arr();
    let window = (win.first().and_then(|v| v.num()).unwrap_or(-50.0) as i32, win.get(1).and_then(|v| v.num()).unwrap_or(250.0) as i32);
    let attacks = boss.get("attacks").arr().iter().map(|a| Attack { name: a.get("name").str().to_string(), boxes: vec![] }).collect();
    if !lx.errors.is_empty() {
        return Err(lx.errors.join("; "));
    }
    Ok(Define {
        name: ir.get("subject").get("character").str().to_string(),
        parts,
        vars,
        statuses,
        rules,
        tables,
        kinetics,
        motions: mlist,
        attacks,
        messages,
        level_rows,
        strength0: lv.get("initial").get("strength").num().unwrap_or(1.0) as u8,
        start_status,
        slot_w,
        slot_repeat: ts.get("repeat_factor").num().unwrap_or(1.0) as f32,
        route_mask,
        route_factor,
        defeat_status,
        goal_snap: true,
        spawn_x: boss.get("arena").get("boss_spawn").arr().first().and_then(|v| v.num()).unwrap_or(0.0) as f32,
        camera,
        window,
        // the shared enemy engine's passes (enemy-engine IR): -100, the damage window on a hit frame, the default window
        passes: vec![(-100, -100, false), (-250, -150, true), (window.0, window.1, false)],
    })
}

/// The IR directory (never printed).
pub fn ir_dir() -> Option<std::path::PathBuf> {
    if let Some(p) = std::env::var_os("MELEE_BOSS_IR_DIR") {
        return Some(p.into());
    }
    if let Some(p) = std::env::var_os("MELEE_BRAWL_LOCAL") {
        let d = std::path::PathBuf::from(p).join("boss-ir");
        if d.is_dir() {
            return Some(d);
        }
    }
    let mut d = std::env::current_dir().ok()?;
    loop {
        let c = d.join("_build").join("boss-ir");
        if c.is_dir() {
            return Some(c);
        }
        if !d.pop() {
            return None;
        }
    }
}

/// Load `<ir dir>/<file>` and lower it. Errors name the problem, never the path.
pub fn load(file: &str, motions: &[HostMotion]) -> Result<Define, String> {
    let dir = ir_dir().ok_or("no boss IR directory (MELEE_BOSS_IR_DIR / MELEE_BRAWL_LOCAL)")?;
    let text = std::fs::read_to_string(dir.join(file)).map_err(|_| format!("{file}: not readable"))?;
    let j = parse_json(&text)?;
    lower(&j, motions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip() {
        let j = parse_json(r#"{"a": [1, -2.5, "x\"y", true, null, {"b": "0x13FF"}]}"#).unwrap();
        assert_eq!(j.get("a").arr()[1].num(), Some(-2.5));
        assert_eq!(j.get("a").arr()[2].str(), "x\"y");
        assert_eq!(msg_id(j.get("a").arr()[5].get("b")), Some(0x13FF));
        assert_eq!(nums_in("counter 0 and pct <= 50 -> send 0x1400"), vec![0.0, 50.0, 5120.0]);
        assert_eq!(nums_in("-> -68 (44006) else +68"), vec![-68.0, 44006.0, 68.0]);
    }

    /// The real IR, when the owner's local file is present (skipped otherwise).
    #[test]
    fn lowers_local_petey_ir() {
        let ms = crate::tu::petey::host_motions();
        match load("petey.brawl.ir.json", &ms) {
            Ok(d) => {
                assert_eq!(d.parts.len(), 4);
                assert!(d.rules.len() >= 30);
                assert_eq!(d.start_status, 2);
                eprintln!("lowered: {} statuses, {} rules, {} tables, {} messages", d.statuses.len(), d.rules.len(), d.tables.len(), d.messages.len());
            }
            Err(e) if e.contains("no boss IR") || e.contains("not readable") => eprintln!("skipped: {e}"),
            Err(e) => panic!("{e}"),
        }
    }

    /// Headless fight on the lowered IR: one player, scripted hits (MELEE_GB_SIM_HITS "frame:part:dmg,..."), the
    /// status trail printed. Skipped without the local IR.
    #[test]
    fn simulate_local_petey() {
        let ms = crate::tu::petey::host_motions();
        let Ok(d) = load("petey.brawl.ir.json", &ms) else { return };
        let w = World { slots: [Some(30.0), None, None, None], airborne: false, out_of_area: false };
        begin(&d, w, 5, 1000, 0x5E7E);
        let hits = std::env::var("MELEE_GB_SIM_HITS").unwrap_or("50:1:16,400:0:66,900:0:39,1500:0:22,1700:0:3".into());
        let mut st = unsafe { S.status };
        eprintln!("sim: hp {:?} L {}", unsafe { S.hp }, level(&d));
        for f in 1..2400u32 {
            for h in hits.split(',') {
                let v: std::vec::Vec<i32> = h.split(':').filter_map(|x| x.parse().ok()).collect();
                if v.len() == 3 && v[0] as u32 == f {
                    damage(&d, v[1] as usize, v[2]);
                    eprintln!("sim: f{f} hit part {} {}", v[1], v[2]);
                }
            }
            tick(&d, w);
            let s = unsafe { S };
            for &(k, v) in s.ev.iter().take(s.ev_n as usize) {
                if k == EV_STAGE {
                    eprintln!("sim: f{f} stage {v}");
                }
            }
            if s.status != st {
                eprintln!("sim: f{f} {st} -> {} x {:.1} hp {:?} L {:.1}", s.status, s.x, &s.hp[..3], level(&d));
                st = s.status;
            }
        }
    }
}
