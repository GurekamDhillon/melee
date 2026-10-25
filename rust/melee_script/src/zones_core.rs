//! Zone simulation core (`pc/gameworld/script_zones_core.h`): pure state, no host, allocation of game memory or clock.
//! The game half keeps one [`SzState`] in its snapshotted state and answers the `ScriptGame::zone_*` scalar accessors from it.

pub const SZ_CAP: usize = 64;
pub const SZ_VERTICES: usize = 16;
pub const SZ_ENTITIES: usize = 12; // primary/sub for each of six owners
pub const SZ_EVENTS: usize = SZ_ENTITIES * (2 * SZ_CAP + 2);
pub const SZ_ENTER: i32 = 1;
pub const SZ_EXIT: i32 = 2;
pub const SZ_NONE: i32 = 3;
pub const SZ_SOME: i32 = 4;

/// `SZF_*` field numbers of `ScriptGame_ZoneField`
pub const SZF_HANDLE: i32 = 0;
pub const SZF_OWNER: i32 = 1;
pub const SZF_LIVE: i32 = 2;
pub const SZF_MODEL: i32 = 3;
pub const SZF_N: i32 = 4;
pub const SZF_TAGS: i32 = 5;
pub const SZF_AREA: i32 = 6;
pub const SZF_DX: i32 = 7;
pub const SZF_DY: i32 = 8;
pub const SZF_X: i32 = 16;
pub const SZF_Y: i32 = 32;
/// `SZT_*` text numbers
pub const SZT_NAME: i32 = 0;
pub const SZT_LABEL: i32 = 1;
pub const SZT_KIND: i32 = 2;
pub const SZT_TAG: i32 = 3;
/// `SZE_*` event field numbers
pub const SZE_WHAT: i32 = 0;
pub const SZE_ENTITY: i32 = 1;
pub const SZE_PORT: i32 = 2;
pub const SZE_SUB: i32 = 3;
pub const SZE_X: i32 = 4;
pub const SZE_Y: i32 = 5;
pub const SZE_ZONE: i32 = 6;
pub const SZE_FROM: i32 = 16;

fn finite(f: f32) -> bool {
    f.to_bits() & 0x7f80_0000 != 0x7f80_0000
}

#[derive(Clone, Debug, PartialEq)]
pub struct SzDef {
    pub handle: i32,
    pub owner: i32,
    pub live: i32,
    pub model: i32,
    pub n: usize,
    pub ntags: usize,
    pub name: String,
    pub label: String,
    pub kind: String,
    pub tags: [String; 8],
    pub x: [f32; SZ_VERTICES],
    pub y: [f32; SZ_VERTICES],
    pub dx: f32,
    pub dy: f32,
    pub area: f32,
}
impl Default for SzDef {
    fn default() -> Self {
        SzDef {
            handle: 0,
            owner: 0,
            live: 0,
            model: 0,
            n: 0,
            ntags: 0,
            name: String::new(),
            label: String::new(),
            kind: String::new(),
            tags: Default::default(),
            x: [0.0; SZ_VERTICES],
            y: [0.0; SZ_VERTICES],
            dx: 0.0,
            dy: 0.0,
            area: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SzInput {
    pub token: i32,
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SzEntity {
    pub token: i32,
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub handles: [i32; SZ_CAP],
    pub frames: [i32; SZ_CAP],
}
impl Default for SzEntity {
    fn default() -> Self {
        SzEntity { token: 0, id: 0, x: 0.0, y: 0.0, handles: [0; SZ_CAP], frames: [0; SZ_CAP] }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct SzEvent {
    pub what: i32,
    pub entity: i32,
    pub port: i32,
    pub sub: i32,
    pub x: f32,
    pub y: f32,
    pub zone_handle: i32,
    pub zone_name: String,
    pub zone_label: String,
    pub zone_kind: String,
    pub from: [i32; SZ_CAP],
}
#[derive(Clone, Debug, PartialEq)]
pub struct SzState {
    pub z: Vec<SzDef>,
    pub e: Vec<SzEntity>,
    pub ev: Vec<SzEvent>,
    pub serial: i32,
    pub entity_serial: i32,
    pub count: i32,
    pub pending: i32,
}
impl Default for SzState {
    fn default() -> Self {
        SzState {
            z: vec![SzDef::default(); SZ_CAP],
            e: vec![SzEntity::default(); SZ_ENTITIES],
            ev: Vec::new(),
            serial: 0,
            entity_serial: 0,
            count: 0,
            pending: 0,
        }
    }
}

/// `sz_validate`: normalises the winding to counter-clockwise and fills `area`
pub fn validate(z: &mut SzDef) -> bool {
    if z.n < 3 || z.n > SZ_VERTICES || z.name.is_empty() || z.kind.is_empty() || z.ntags > 8 {
        return false;
    }
    let mut area = 0f32;
    for i in 0..z.n {
        let j = (i + 1) % z.n;
        if !finite(z.x[i]) || !finite(z.y[i]) || z.x[i].abs() > 49000.0 || z.y[i].abs() > 49000.0 {
            return false;
        }
        if z.x[i] == z.x[j] && z.y[i] == z.y[j] {
            return false;
        }
        area += z.x[i] * z.y[j] - z.x[j] * z.y[i];
    }
    if !finite(area) || area == 0.0 {
        return false;
    }
    if area < 0.0 {
        for i in 0..z.n / 2 {
            z.x.swap(i, z.n - 1 - i);
            z.y.swap(i, z.n - 1 - i);
        }
    }
    // every other vertex must be strictly inside each directed edge: refuses collinear, repeated and self-intersecting input
    for i in 0..z.n {
        for j in 0..z.n {
            if j != i && j != (i + 1) % z.n {
                let k = (i + 1) % z.n;
                let cross = (z.x[k] - z.x[i]) * (z.y[j] - z.y[i]) - (z.y[k] - z.y[i]) * (z.x[j] - z.x[i]);
                if !(cross > 0.0) {
                    return false;
                }
            }
        }
    }
    z.area = area.abs() * 0.5;
    true
}

/// `sz_contains`: bottom and left edges included, top and right excluded
pub fn contains(z: &SzDef, x: f32, y: f32) -> bool {
    let (x, y) = (x - z.dx, y - z.dy);
    if z.live != 1 || !finite(x) || !finite(y) {
        return false;
    }
    for i in 0..z.n {
        let j = (i + 1) % z.n;
        let (dx, dy) = (z.x[j] - z.x[i], z.y[j] - z.y[i]);
        let c = dx * (y - z.y[i]) - dy * (x - z.x[i]);
        if c < 0.0 || (c == 0.0 && !(dy < 0.0 || (dy == 0.0 && dx > 0.0))) {
            return false;
        }
    }
    true
}

impl SzState {
    pub fn slot(&self, handle: i32) -> Option<usize> {
        (0..SZ_CAP).find(|&i| self.z[i].live != 0 && self.z[i].handle == handle && handle > 0)
    }
    /// `sz_put`: add (handle 0) or replace a zone; the handle, or <0: -1 invalid, -2 not yours, -3 duplicate name, -4 capacity
    pub fn put(&mut self, value: &SzDef, handle: i32) -> i32 {
        let mut z = value.clone();
        let mut at = if handle != 0 { self.slot(handle).map(|v| v as i32).unwrap_or(-1) } else { -1 };
        if !validate(&mut z) || z.owner <= 0 {
            return -1;
        }
        if handle != 0 && (at < 0 || self.z[at as usize].live != 1 || self.z[at as usize].owner != z.owner) {
            return -2;
        }
        for i in 0..SZ_CAP {
            if self.z[i].live == 1 && i as i32 != at && self.z[i].name == z.name {
                return -3;
            }
        }
        let mut handle = handle;
        if handle == 0 {
            if self.serial == i32::MAX {
                return -4;
            }
            match (0..SZ_CAP).find(|&i| self.z[i].live == 0) {
                Some(i) => at = i as i32,
                None => return -4,
            }
            self.serial += 1;
            handle = self.serial;
            self.count += 1;
        }
        z.handle = handle;
        z.live = 1;
        self.z[at as usize] = z;
        handle
    }
    /// `sz_remove`: the payload stays until the terminal pass
    pub fn remove(&mut self, handle: i32, owner: i32) -> bool {
        let Some(at) = self.slot(handle) else { return false };
        if self.z[at].live != 1 || self.z[at].owner != owner {
            return false;
        }
        self.z[at].live = 2;
        self.count -= 1;
        self.pending += 1;
        true
    }
    fn emit(&mut self, what: i32, e: usize, z: Option<usize>, entity: &SzEntity, from: &[i32; SZ_CAP], mask: i32) {
        if mask & (1 << (what - 1)) == 0 {
            return;
        }
        let mut v = SzEvent {
            what,
            entity: entity.id,
            port: e as i32 / 2 + 1,
            sub: e as i32 % 2,
            x: entity.x,
            y: entity.y,
            zone_handle: 0,
            zone_name: String::new(),
            zone_label: String::new(),
            zone_kind: String::new(),
            from: *from,
        };
        if let Some(z) = z {
            let z = &self.z[z];
            v.zone_handle = z.handle;
            v.zone_name = z.name.clone();
            v.zone_label = z.label.clone();
            v.zone_kind = z.kind.clone();
        }
        self.ev.push(v);
    }
    /// `sz_step`: one frame of enter/exit/none/some bookkeeping for the twelve fighter entities
    pub fn step(&mut self, input: &[SzInput; SZ_ENTITIES], mask: i32, advance: bool) {
        self.ev.clear();
        // zones in handle order (stable insertion sort, as the C does)
        let mut order: Vec<usize> = (0..SZ_CAP).collect();
        for i in 0..SZ_CAP {
            let mut k = i;
            while k > 0 && self.z[order[k - 1]].handle > self.z[order[k]].handle {
                order.swap(k - 1, k);
                k -= 1;
            }
        }
        for e in 0..SZ_ENTITIES {
            let old = self.e[e].clone();
            let mut next = old.clone();
            let from = old.handles;
            let (mut before, mut after) = (0, 0);
            let newlife = old.token != input[e].token;
            if newlife {
                next = SzEntity { token: input[e].token, ..Default::default() };
                if next.token != 0 && self.entity_serial < i32::MAX {
                    self.entity_serial += 1;
                    next.id = self.entity_serial;
                }
            }
            if input[e].token != 0 {
                next.x = input[e].x;
                next.y = input[e].y;
            }
            for i in 0..SZ_CAP {
                let inside = input[e].token != 0 && next.id != 0 && contains(&self.z[i], next.x, next.y);
                if old.handles[i] != 0 {
                    before += 1;
                }
                next.handles[i] = if inside { self.z[i].handle } else { 0 };
                next.frames[i] =
                    if inside && !newlife && old.handles[i] == next.handles[i] { old.frames[i] } else { 0 };
                if inside {
                    after += 1;
                    if advance && next.frames[i] < i32::MAX {
                        next.frames[i] += 1;
                    }
                }
            }
            for &i in &order {
                if old.handles[i] != 0 && (newlife || old.handles[i] != next.handles[i]) {
                    let who = if newlife { &old } else { &next };
                    self.emit(SZ_EXIT, e, Some(i), who, &from, mask);
                }
            }
            if before != 0 && (newlife || after == 0) {
                self.emit(SZ_NONE, e, None, if newlife { &old } else { &next }, &from, mask);
            }
            for &i in &order {
                if next.handles[i] != 0 && (newlife || old.handles[i] != next.handles[i]) {
                    self.emit(SZ_ENTER, e, Some(i), &next, &from, mask);
                }
            }
            if after != 0 && (newlife || before == 0) {
                self.emit(SZ_SOME, e, None, &next, &from, mask);
            }
            self.e[e] = next;
        }
        for i in 0..SZ_CAP {
            if self.z[i].live == 2 {
                self.z[i] = SzDef::default();
            }
        }
        self.pending = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(name: &str, x0: f32, x1: f32) -> SzDef {
        let mut z = SzDef { name: name.into(), label: name.into(), kind: "room".into(), owner: 1, n: 4, ..Default::default() };
        z.x[0] = x0;
        z.x[3] = x0;
        z.x[1] = x1;
        z.x[2] = x1;
        z.y[0] = -10.0;
        z.y[1] = -10.0;
        z.y[2] = 10.0;
        z.y[3] = 10.0;
        z
    }

    /// Port of pc/tests/zones_test.c
    #[test]
    fn zones_core_fixture() {
        let mut s = SzState::default();
        let mut inp = [SzInput::default(); SZ_ENTITIES];
        let (a, b) = (rect("left", -10.0, 0.0), rect("right", 0.0, 10.0));
        assert_eq!(s.put(&a, 0), 1);
        assert_eq!(s.put(&b, 0), 2);
        assert!(contains(&s.z[0], -10.0, -10.0));
        assert!(!contains(&s.z[0], 0.0, 0.0));
        assert!(contains(&s.z[1], 0.0, 0.0));
        assert!(!contains(&s.z[1], 10.0, 10.0));
        assert!(!contains(&s.z[0], -10.0, 10.0));
        assert!(!contains(&s.z[0], 0.0, -10.0));
        inp[0] = SzInput { token: 7, x: -1.0, y: 0.0 };
        s.step(&inp, 15, true);
        assert!(s.ev.len() == 2 && s.ev[0].what == SZ_ENTER && s.ev[1].what == SZ_SOME);
        for _ in 0..20 {
            s.step(&inp, 15, true);
            assert!(s.ev.is_empty());
        }
        assert_eq!(s.e[0].frames[0], 21);
        inp[0].x = -10.0;
        for _ in 0..20 {
            s.step(&inp, 15, true);
            assert!(s.ev.is_empty());
        }
        inp[0].x = 0.0;
        s.step(&inp, 15, true);
        assert_eq!(s.ev.len(), 2);
        assert!(s.ev[0].what == SZ_EXIT && s.ev[1].what == SZ_ENTER);
        for _ in 0..10 {
            inp[0].x = -1.0;
            s.step(&inp, 15, true);
            assert_eq!(s.ev.len(), 2);
            inp[0].x = 1.0;
            s.step(&inp, 15, true);
            assert_eq!(s.ev.len(), 2);
        }
        inp[0].x = 30.0;
        s.step(&inp, 15, true);
        assert!(s.ev.len() == 2 && s.ev[1].what == SZ_NONE);
        inp[0].x = 1.0;
        s.step(&inp, 15, true);
        assert!(s.ev.len() == 2 && s.ev[1].what == SZ_SOME);
        inp[0].token = 0;
        s.step(&inp, 15, true);
        assert_eq!(s.ev.len(), 2);
        inp[0].token = 8;
        s.step(&inp, 0, true);
        assert!(s.ev.is_empty() && s.e[0].frames[1] == 1);
        let saved = s.clone();
        s.step(&inp, 15, true);
        let expected = s.clone();
        s = saved;
        s.step(&inp, 15, true);
        assert_eq!(s, expected); // all state, including events
        assert!(!s.remove(2, 99));
        s.remove(2, 1);
        s.step(&inp, 15, true);
        assert!(s.ev.len() == 2 && s.ev[0].zone_handle == 2);
        s = SzState::default();
        for i in 0..SZ_CAP {
            assert!(s.put(&rect(&format!("z{i}"), -10.0, 10.0), 0) > 0);
        }
        assert!(s.put(&rect("overflow", -10.0, 10.0), 0) < 0);
        let saved = s.clone();
        let mut a = s.z[0].clone();
        a.x[0] = f32::NAN;
        assert!(s.put(&a, a.handle) < 0);
        assert_eq!(s, saved);
        s = SzState::default();
        let h = s.put(&rect("moving", -1.0, 1.0), 0);
        s.z[0].dx = 10.0;
        assert!(contains(&s.z[0], 10.0, 0.0));
        assert!(!contains(&s.z[0], 0.0, 0.0));
        for (i, v) in inp.iter_mut().enumerate() {
            *v = SzInput { token: i as i32 + 1, x: 10.0, y: 0.0 };
        }
        s.step(&inp, 15, true);
        assert_eq!(s.ev.len(), SZ_ENTITIES * 2);
        s.remove(h, 1);
        s.step(&inp, 15, false);
        assert_eq!(s.ev.len(), SZ_ENTITIES * 2);
        s = SzState::default();
        let mut a = rect("poly", -1.0, 1.0);
        a.n = 3;
        a.x[..3].copy_from_slice(&[0.0, 2.0, 0.0]);
        a.y[..3].copy_from_slice(&[0.0, 0.0, 2.0]);
        assert!(s.put(&a, 0) > 0);
        assert!(contains(&s.z[0], 0.5, 0.5));
        assert!(!contains(&s.z[0], 2.0, 2.0));
        assert!(contains(&s.z[0], 0.0, 0.0));
        assert!(!contains(&s.z[0], 1.0, 1.0));
        let mut b = a.clone();
        for i in 0..3 {
            b.x[i] = a.x[2 - i];
            b.y[i] = a.y[2 - i];
        }
        b.name = "reverse".into();
        assert!(s.put(&b, 0) > 0);
        assert!(contains(&s.z[1], 0.5, 0.5));
        let saved = s.clone();
        let mut a = s.z[0].clone();
        a.x[1] = a.x[0];
        a.y[1] = a.y[0];
        assert!(s.put(&a, a.handle) < 0);
        assert_eq!(s, saved);
        s = SzState::default();
        inp = [SzInput::default(); SZ_ENTITIES];
        for i in 0..SZ_CAP {
            assert!(s.put(&rect(&format!("full{i}"), -10.0, 10.0), 0) > 0);
        }
        for (i, v) in inp.iter_mut().enumerate() {
            *v = SzInput { token: i as i32 + 1, x: 0.0, y: 0.0 };
        }
        s.step(&inp, 15, true);
        assert_eq!(s.ev.len(), SZ_ENTITIES * (SZ_CAP + 1));
        for v in inp.iter_mut() {
            v.token += 100;
        }
        s.step(&inp, 15, true);
        assert_eq!(s.ev.len(), SZ_EVENTS); // no drop at worst-case lifetime replacement
        for v in inp.iter_mut() {
            v.x = 100.0;
        }
        s.step(&inp, 1, true);
        assert_eq!(s.ev.len(), 0); // exit/none hooks unarmed
        for v in inp.iter_mut() {
            v.x = 0.0;
        }
        s.step(&inp, 1, true);
        assert_eq!(s.ev.len(), SZ_ENTITIES * SZ_CAP); // enter only
    }
}
