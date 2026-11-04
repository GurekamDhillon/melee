//! gd-ui-v1 A2: the host-owned menu node tree. Focus, wrap, key repeat and the kit look are the host's, so they are
//! the same in every mod. Layout is cached per shown root and recomputed only when a node of it changes; drawing
//! re-emits quads from the cached layout every present (no allocation: labels and choices are stored at setup).

use super::cmd::{Batch, Pen, ALIGN_CENTER, ALIGN_RIGHT};
use super::theme::*;
use melee_gx::ui::Atlas;

pub const K_PANEL: i32 = 0;
pub const K_LIST: i32 = 1;
pub const K_TABS: i32 = 2;
pub const K_BUTTON: i32 = 3;
pub const K_TOGGLE: i32 = 4;
pub const K_SLIDER: i32 = 5;
pub const K_CHOICE: i32 = 6;
pub const K_LABEL: i32 = 7;

pub const F_HIDDEN: u32 = 1;
pub const F_DISABLED: u32 = 2;
pub const F_NO_WRAP: u32 = 4;

pub const EV_ACTIVATE: i32 = 0;
pub const EV_CHANGE: i32 = 1;
pub const EV_BACK: i32 = 2;
pub const EV_FOCUS: i32 = 3;

/// GameCube PAD button bits.
pub const PAD_LEFT: u16 = 0x1;
pub const PAD_RIGHT: u16 = 0x2;
pub const PAD_DOWN: u16 = 0x4;
pub const PAD_UP: u16 = 0x8;
pub const PAD_Z: u16 = 0x10;
pub const PAD_R: u16 = 0x20;
pub const PAD_L: u16 = 0x40;
pub const PAD_A: u16 = 0x100;
pub const PAD_B: u16 = 0x200;
pub const PAD_START: u16 = 0x1000;

/// Key repeat: first repeat after this many presents, then every `REPEAT_EVERY`.
pub const REPEAT_DELAY: u32 = 15;
pub const REPEAT_EVERY: u32 = 4;

const ROW_H: f32 = 22.0;
const TABS_H: f32 = 24.0;
const PAD: f32 = 10.0;

#[derive(Default)]
pub struct Node {
    pub live: bool,
    pub module: u16,
    pub kind: i32,
    pub id: u32,
    pub parent: u32,
    pub children: Vec<u32>,
    pub label: String,
    pub value: i32,
    pub range: (i32, i32, i32),
    pub choices: Vec<String>,
    pub flags: u32,
    /// Placement asked by the mod (`gd_ui_node_rect`, roots only) and the cached layout rect.
    pub place: Option<[f32; 4]>,
    pub rect: [f32; 4],
}

impl Node {
    fn focusable(&self) -> bool {
        self.live && matches!(self.kind, K_BUTTON | K_TOGGLE | K_SLIDER | K_CHOICE) && self.flags & (F_HIDDEN | F_DISABLED) == 0
    }
}

#[derive(Clone, Copy)]
pub struct Shown {
    pub root: u32,
    pub layer: i32,
    pub z: i32,
    pub module: u16,
}

/// Navigation state of one mod (its focus and the held direction for repeat).
#[derive(Clone, Copy, Default)]
pub struct Nav {
    pub focus: u32,
    held: u16,
    dir: i32,
    age: u32,
}

#[derive(Default)]
pub struct Tree {
    pub nodes: Vec<Node>,
    free: Vec<u32>,
    pub shown: Vec<Shown>,
    pub nav: Vec<Nav>,
    pub dirty: bool,
    /// Events for the mods' `gd_ui_event` exports: (module, node, ev, value).
    pub events: Vec<(u16, u32, i32, i32)>,
}

impl Tree {
    pub fn get(&self, h: u32, module: u16) -> Option<&Node> {
        self.nodes.get(h.wrapping_sub(1) as usize).filter(|n| n.live && n.module == module)
    }
    pub fn get_mut(&mut self, h: u32, module: u16) -> Option<&mut Node> {
        self.dirty = true;
        self.nodes.get_mut(h.wrapping_sub(1) as usize).filter(|n| n.live && n.module == module)
    }

    /// `gd_ui_node`: 0 when the parent is invalid or the kind unknown.
    pub fn create(&mut self, module: u16, parent: u32, kind: i32, id: u32) -> u32 {
        if !(K_PANEL..=K_LABEL).contains(&kind) || (parent != 0 && self.get(parent, module).is_none()) || self.nodes.len() >= 4096 && self.free.is_empty() {
            return 0;
        }
        let n = Node { live: true, module, kind, id, parent, range: (0, 100, 1), ..Default::default() };
        let h = match self.free.pop() {
            Some(h) => {
                self.nodes[h as usize - 1] = n;
                h
            }
            None => {
                self.nodes.push(n);
                self.nodes.len() as u32
            }
        };
        if parent != 0 {
            self.nodes[parent as usize - 1].children.push(h);
        }
        self.dirty = true;
        h
    }

    pub fn free_subtree(&mut self, h: u32, module: u16) {
        let Some(n) = self.get(h, module) else { return };
        let parent = n.parent;
        if parent != 0 {
            self.nodes[parent as usize - 1].children.retain(|&c| c != h);
        }
        let mut stack = vec![h];
        while let Some(x) = stack.pop() {
            let n = &mut self.nodes[x as usize - 1];
            stack.extend(n.children.drain(..));
            n.live = false;
            self.free.push(x);
        }
        self.shown.retain(|s| self.nodes[s.root as usize - 1].live);
        for nav in &mut self.nav {
            if nav.focus != 0 && !self.nodes[nav.focus as usize - 1].live {
                nav.focus = 0;
            }
        }
        self.dirty = true;
    }

    /// Every node of a mod (unload, trap).
    pub fn drop_module(&mut self, module: u16) {
        let roots: Vec<u32> = (1..=self.nodes.len() as u32).filter(|&h| self.nodes[h as usize - 1].live && self.nodes[h as usize - 1].module == module && self.nodes[h as usize - 1].parent == 0).collect();
        for r in roots {
            self.free_subtree(r, module);
        }
        self.shown.retain(|s| s.module != module);
    }

    pub fn show(&mut self, module: u16, root: u32, layer: i32, z: i32) -> bool {
        if self.get(root, module).is_none() {
            return false;
        }
        self.shown.retain(|s| s.root != root);
        self.shown.push(Shown { root, layer, z, module });
        let nav = self.nav_mut(module);
        if nav.focus == 0 {
            let f = self.first_focusable(root);
            self.nav_mut(module).focus = f;
        }
        self.dirty = true;
        true
    }

    pub fn hide(&mut self, module: u16, root: u32) {
        self.shown.retain(|s| !(s.root == root && s.module == module));
    }

    pub fn nav_mut(&mut self, module: u16) -> &mut Nav {
        let m = module as usize;
        if self.nav.len() <= m {
            self.nav.resize(m + 1, Nav::default());
        }
        &mut self.nav[m]
    }

    fn first_focusable(&self, h: u32) -> u32 {
        let n = &self.nodes[h as usize - 1];
        if n.focusable() {
            return h;
        }
        for &c in &n.children {
            if self.nodes[c as usize - 1].flags & F_HIDDEN != 0 {
                continue;
            }
            let f = self.first_focusable(c);
            if f != 0 {
                return f;
            }
        }
        0
    }

    fn find_kind(&self, h: u32, kind: i32) -> u32 {
        let n = &self.nodes[h as usize - 1];
        if n.kind == kind && n.flags & F_HIDDEN == 0 {
            return h;
        }
        n.children.iter().map(|&c| self.find_kind(c, kind)).find(|&x| x != 0).unwrap_or(0)
    }

    /// Lay out every shown root (only when something changed).
    pub fn layout(&mut self) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        for i in 0..self.shown.len() {
            let r = self.shown[i].root;
            let p = self.nodes[r as usize - 1].place.unwrap_or([40.0, 40.0, 300.0, 0.0]);
            let h = self.lay(r, p[0], p[1], p[2]);
            let n = &mut self.nodes[r as usize - 1];
            n.rect = [p[0], p[1], p[2], if p[3] > 0.0 { p[3] } else { h }];
        }
    }

    /// Lay out `h` at (x, y) with width w; returns its height.
    fn lay(&mut self, h: u32, x: f32, y: f32, w: f32) -> f32 {
        let (kind, hidden) = { let n = &self.nodes[h as usize - 1]; (n.kind, n.flags & F_HIDDEN != 0) };
        if hidden {
            self.nodes[h as usize - 1].rect = [x, y, w, 0.0];
            return 0.0;
        }
        let ht = match kind {
            K_PANEL | K_LIST => {
                let inset = if kind == K_PANEL { PAD } else { 0.0 };
                let mut cy = y + inset + if kind == K_PANEL && !self.nodes[h as usize - 1].label.is_empty() { 20.0 } else { 0.0 };
                let kids = std::mem::take(&mut self.nodes[h as usize - 1].children);
                for &c in &kids {
                    cy += self.lay(c, x + inset, cy, w - 2.0 * inset);
                }
                self.nodes[h as usize - 1].children = kids;
                cy - y + inset
            }
            K_TABS => TABS_H + 6.0,
            K_LABEL => 18.0,
            _ => ROW_H,
        };
        self.nodes[h as usize - 1].rect = [x, y, w, ht];
        ht
    }

    /// Draw every shown tree of `module` (or all) into the batch, with the theme's tokens.
    pub fn draw(&self, b: &mut Batch, atlas: &Atlas, theme: i32) {
        for s in &self.shown {
            let mut pen = Pen::new(s.module);
            pen.layer = s.layer;
            pen.z = s.z;
            let focus = self.nav.get(s.module as usize).map_or(0, |n| n.focus);
            self.draw_node(b, atlas, &pen, theme, s.root, focus);
        }
    }

    fn draw_node(&self, b: &mut Batch, a: &Atlas, pen: &Pen, th: i32, h: u32, focus: u32) {
        let n = &self.nodes[h as usize - 1];
        if n.flags & F_HIDDEN != 0 {
            return;
        }
        let [x, y, w, ht] = n.rect;
        let t = |k| token(th, k);
        let dim = if n.flags & F_DISABLED != 0 { 0xFFFF_FF60u32 } else { 0xFFFF_FFFF };
        let fade = |c: u32| (c & 0xFFFF_FF00) | (((c & 0xFF) * (dim & 0xFF)) / 255);
        match n.kind {
            K_PANEL => {
                panel(b, a, pen, th, x, y, w, ht, false);
                if !n.label.is_empty() {
                    b.text(pen, a, x + PAD, y + 8.0, 0, t(TOK_MUTED), 0, w - 2.0 * PAD, &n.label);
                }
            }
            K_TABS => {
                // L and R key caps around the tab labels; the active tab on the accent
                key_cap(b, a, pen, th, x, y + 3.0, "L");
                key_cap(b, a, pen, th, x + w - 20.0, y + 3.0, "R");
                let k = n.choices.len().max(1) as f32;
                let tw = (w - 48.0) / k;
                for (i, c) in n.choices.iter().enumerate() {
                    let tx = x + 24.0 + i as f32 * tw;
                    let on = i as i32 == n.value;
                    if on {
                        fill(b, a, pen, th, tx, y + 2.0, tw - 2.0, TABS_H - 2.0, t(TOK_ACCENT));
                    }
                    b.text(pen, a, tx + tw * 0.5, y + 7.0, 1, if on { t(TOK_ACCENT_INK) } else { t(TOK_MUTED) }, ALIGN_CENTER, tw - 4.0, c);
                }
            }
            K_LABEL => b.text(pen, a, x, y + 3.0, 0, fade(t(TOK_MUTED)), 0, w, &n.label),
            K_BUTTON | K_TOGGLE | K_SLIDER | K_CHOICE => {
                let on = h == focus;
                if on {
                    b.rect(pen, a, x, y, w, ht, (t(TOK_ACCENT) & 0xFFFF_FF00) | 0x24, 0.0);
                    b.rect(pen, a, x, y, 4.0, ht, t(TOK_ACCENT), 0.0);
                }
                b.rect(pen, a, x, y + ht - 1.0, w, 1.0, t(TOK_DIVIDER), 0.0);
                b.text(pen, a, x + 10.0, y + 5.0, 1, fade(t(TOK_TEXT)), 0, w * 0.55, &n.label);
                let rx = x + w - 8.0;
                match n.kind {
                    K_TOGGLE => {
                        let v = n.value != 0;
                        b.text(pen, a, rx, y + 5.0, 1, if v { t(TOK_ACCENT) } else { t(TOK_MUTED) }, ALIGN_RIGHT, 0.0, if v { "ON" } else { "OFF" });
                    }
                    K_CHOICE => {
                        let c = n.choices.get(n.value.max(0) as usize).map_or("", |s| s.as_str());
                        b.text(pen, a, rx, y + 5.0, 1, t(TOK_MUTED), ALIGN_RIGHT, 0.0, "▶");
                        b.text(pen, a, rx - 14.0, y + 5.0, 1, fade(t(TOK_TEXT)), ALIGN_RIGHT, w * 0.35, c);
                        let cw = melee_gx::ui::measure(c, super::cmd::ROLES[1].0).min(w * 0.35);
                        b.text(pen, a, rx - 22.0 - cw, y + 5.0, 1, t(TOK_MUTED), ALIGN_RIGHT, 0.0, "◀");
                    }
                    K_SLIDER => {
                        let (lo, hi, _) = n.range;
                        let f = if hi > lo { (n.value - lo) as f32 / (hi - lo) as f32 } else { 0.0 };
                        let tx = rx - 40.0 - 80.0;
                        b.rect(pen, a, tx, y + 9.0, 80.0, 4.0, t(TOK_CELL_IDLE), 0.0);
                        b.rect(pen, a, tx, y + 9.0, 80.0 * f.clamp(0.0, 1.0), 4.0, t(TOK_ACCENT), 0.0);
                        let mut buf = [0u8; 12];
                        b.text(pen, a, rx, y + 5.0, 1, fade(t(TOK_TEXT)), ALIGN_RIGHT, 0.0, itoa(n.value, &mut buf));
                    }
                    _ => b.text(pen, a, rx, y + 5.0, 1, t(TOK_ACCENT), ALIGN_RIGHT, 0.0, "A ›"),
                }
            }
            _ => {}
        }
        for &c in &n.children {
            self.draw_node(b, a, pen, th, c, focus);
        }
    }

    /// One present of navigation for `module` from its captured pad: buttons (held) and the stick.
    pub fn navigate(&mut self, module: u16, buttons: u16, pressed: u16, sx: i8, sy: i8) {
        let Some(root) = self.shown.iter().rev().find(|s| s.module == module).map(|s| s.root) else { return };
        // direction with the kit's repeat: -1 up, 1 down, -2 left, 2 right
        let mut dir = 0;
        if buttons & PAD_UP != 0 || sy > 48 { dir = -1 } else if buttons & PAD_DOWN != 0 || sy < -48 { dir = 1 }
        if buttons & PAD_LEFT != 0 || sx < -48 { dir = -2 } else if buttons & PAD_RIGHT != 0 || sx > 48 { dir = 2 }
        let nav = *self.nav_mut(module);
        let (fire, age) = if dir == 0 {
            (false, 0)
        } else if dir != nav.dir {
            (true, 0)
        } else {
            let age = nav.age + 1;
            (age >= REPEAT_DELAY && (age - REPEAT_DELAY) % REPEAT_EVERY == 0, age)
        };
        {
            let nv = self.nav_mut(module);
            nv.dir = dir;
            nv.age = age;
            nv.held = buttons;
        }
        let mut focus = nav.focus;
        if focus == 0 || self.get(focus, module).is_none() {
            focus = self.first_focusable(root);
            self.nav_mut(module).focus = focus;
        }
        if fire && focus != 0 {
            match dir {
                -1 | 1 => self.move_focus(module, focus, dir),
                _ => self.step_value(module, focus, if dir < 0 { -1 } else { 1 }),
            }
        }
        let focus = self.nav_mut(module).focus;
        if pressed & PAD_A != 0 && focus != 0 {
            let n = &mut self.nodes[focus as usize - 1];
            if n.kind == K_TOGGLE {
                n.value = (n.value == 0) as i32;
                let v = n.value;
                self.events.push((module, focus, EV_CHANGE, v));
            }
            let v = self.nodes[focus as usize - 1].value;
            self.events.push((module, focus, EV_ACTIVATE, v));
        }
        if pressed & PAD_B != 0 {
            self.events.push((module, focus, EV_BACK, 0));
        }
        if pressed & (PAD_L | PAD_R) != 0 {
            let tabs = self.find_kind(root, K_TABS);
            if tabs != 0 {
                let n = &mut self.nodes[tabs as usize - 1];
                let k = n.choices.len().max(1) as i32;
                n.value = (n.value + if pressed & PAD_R != 0 { 1 } else { -1 }).rem_euclid(k);
                let v = n.value;
                self.events.push((module, tabs, EV_CHANGE, v));
            }
        }
    }

    fn move_focus(&mut self, module: u16, focus: u32, d: i32) {
        let parent = self.nodes[focus as usize - 1].parent;
        if parent == 0 {
            return;
        }
        let p = &self.nodes[parent as usize - 1];
        let sib: &Vec<u32> = &p.children;
        let Some(i) = sib.iter().position(|&c| c == focus) else { return };
        let wrap = p.flags & F_NO_WRAP == 0;
        let n = sib.len() as i32;
        let mut j = i as i32;
        for _ in 0..n {
            j += d;
            if j < 0 || j >= n {
                if !wrap {
                    return;
                }
                j = j.rem_euclid(n);
            }
            let c = sib[j as usize];
            if self.nodes[c as usize - 1].focusable() {
                self.nav_mut(module).focus = c;
                self.events.push((module, c, EV_FOCUS, 0));
                self.dirty = true;
                return;
            }
        }
    }

    fn step_value(&mut self, module: u16, focus: u32, d: i32) {
        let n = &mut self.nodes[focus as usize - 1];
        let v0 = n.value;
        match n.kind {
            K_SLIDER => {
                let (lo, hi, st) = n.range;
                n.value = (n.value + d * st.max(1)).clamp(lo, hi);
            }
            K_CHOICE => {
                let k = n.choices.len().max(1) as i32;
                n.value = if n.flags & F_NO_WRAP != 0 { (n.value + d).clamp(0, k - 1) } else { (n.value + d).rem_euclid(k) };
            }
            K_TOGGLE => n.value = (n.value == 0) as i32,
            _ => return,
        }
        if n.value != v0 {
            let v = n.value;
            self.events.push((module, focus, EV_CHANGE, v));
        }
    }
}

/// i32 to decimal into a stack buffer (no allocation).
pub fn itoa(v: i32, buf: &mut [u8; 12]) -> &str {
    let mut i = buf.len();
    let neg = v < 0;
    let mut u = v.unsigned_abs();
    loop {
        i -= 1;
        buf[i] = b'0' + (u % 10) as u8;
        u /= 10;
        if u == 0 {
            break;
        }
    }
    if neg {
        i -= 1;
        buf[i] = b'-';
    }
    core::str::from_utf8(&buf[i..]).unwrap()
}

/// A theme fill at the theme's radius (square themes use a RECT).
pub fn fill(b: &mut Batch, a: &Atlas, pen: &Pen, th: i32, x: f32, y: f32, w: f32, h: f32, c: u32) {
    match token(th, TOK_SLICE_FILL) {
        0 => b.rect(pen, a, x, y, w, h, c, 0.0),
        s => b.slice(pen, a, s as usize, x, y, w, h, c),
    }
}

/// A themed panel: shadow or glow, fill, border (part 1).
pub fn panel(b: &mut Batch, a: &Atlas, pen: &Pen, th: i32, x: f32, y: f32, w: f32, h: f32, hud: bool) {
    let t = |k| token(th, k);
    if t(TOK_GLOW) != 0 {
        b.slice(pen, a, t(TOK_GLOW) as usize, x, y, w, h, 0xFF3E_A573);
    }
    if t(TOK_SHADOW) & 0xFF != 0 {
        fill(b, a, pen, th, x + t(TOK_SHADOW_X) as f32, y + t(TOK_SHADOW_Y) as f32, w, h, t(TOK_SHADOW));
    }
    fill(b, a, pen, th, x, y, w, h, t(if hud { TOK_PANEL_BG_HUD } else { TOK_PANEL_BG }));
    match t(TOK_SLICE_BORDER) {
        0 => {
            let bw = t(TOK_BORDER_W) as f32;
            let c = t(TOK_BORDER);
            b.rect(pen, a, x, y, w, bw, c, 0.0);
            b.rect(pen, a, x, y + h - bw, w, bw, c, 0.0);
            b.rect(pen, a, x, y, bw, h, c, 0.0);
            b.rect(pen, a, x + w - bw, y, bw, h, c, 0.0);
        }
        s => b.slice(pen, a, s as usize, x, y, w, h, t(TOK_BORDER)),
    }
}

/// A key cap (part 7): 18x18 bordered square with the key letter.
pub fn key_cap(b: &mut Batch, a: &Atlas, pen: &Pen, th: i32, x: f32, y: f32, k: &str) {
    let t = |k| token(th, k);
    let w = 12.0 + melee_gx::ui::measure(k, 11.0);
    fill(b, a, pen, th, x, y, w.max(18.0), 18.0, t(TOK_CELL_IDLE));
    b.slice(pen, a, 4, x, y, w.max(18.0), 18.0, t(TOK_KEY_BORDER));
    b.text(pen, a, x + w.max(18.0) * 0.5, y + 3.0, 1, t(TOK_KEY_TEXT), ALIGN_CENTER, 0.0, k);
}
